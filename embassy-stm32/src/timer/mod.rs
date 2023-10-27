pub mod complementary_pwm;
pub mod qei;
pub mod simple_pwm;

use stm32_metapac::timer::vals;

use crate::interrupt;
use crate::rcc::RccPeripheral;
use crate::time::Hertz;

#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

pub(crate) mod sealed {

    use super::*;
    pub trait Basic16bitInstance: RccPeripheral {
        type Interrupt: interrupt::typelevel::Interrupt;

        fn regs() -> crate::pac::timer::TimBasic;

        fn start(&mut self) {
            Self::regs().cr1().modify(|r| r.set_cen(true));
        }

        fn stop(&mut self) {
            Self::regs().cr1().modify(|r| r.set_cen(false));
        }

        /// Reset the counter value to 0
        fn reset(&mut self) {
            Self::regs().cnt().write(|r| r.set_cnt(0));
        }

        /// Set the frequency of how many times per second the timer counts up to the max value or down to 0.
        ///
        /// This means that in the default edge-aligned mode,
        /// the timer counter will wrap around at the same frequency as is being set.
        /// In center-aligned mode (which not all timers support), the wrap-around frequency is effectively halved
        /// because it needs to count up and down.
        fn set_frequency(&mut self, frequency: Hertz) {
            let f = frequency.0;
            let timer_f = Self::frequency().0;
            assert!(f > 0);
            let pclk_ticks_per_timer_period = timer_f / f;
            let psc: u16 = unwrap!(((pclk_ticks_per_timer_period - 1) / (1 << 16)).try_into());
            let arr: u16 = unwrap!((pclk_ticks_per_timer_period / (u32::from(psc) + 1)).try_into());

            let regs = Self::regs();
            regs.psc().write(|r| r.set_psc(psc));
            regs.arr().write(|r| r.set_arr(arr));

            regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTERONLY));
            regs.egr().write(|r| r.set_ug(true));
            regs.cr1().modify(|r| r.set_urs(vals::Urs::ANYEVENT));
        }

        fn clear_update_interrupt(&mut self) -> bool {
            let regs = Self::regs();
            let sr = regs.sr().read();
            if sr.uif() {
                regs.sr().modify(|r| {
                    r.set_uif(false);
                });
                true
            } else {
                false
            }
        }

        fn enable_update_interrupt(&mut self, enable: bool) {
            Self::regs().dier().write(|r| r.set_uie(enable));
        }

        fn set_autoreload_preload(&mut self, enable: vals::Arpe) {
            Self::regs().cr1().modify(|r| r.set_arpe(enable));
        }

        fn get_frequency(&self) -> Hertz {
            let timer_f = Self::frequency();

            let regs = Self::regs();
            let arr = regs.arr().read().arr();
            let psc = regs.psc().read().psc();

            timer_f / arr / (psc + 1)
        }
    }

    pub trait GeneralPurpose16bitInstance: Basic16bitInstance {
        fn regs_gp16() -> crate::pac::timer::TimGp16;

        fn set_counting_mode(&mut self, mode: CountingMode) {
            let (cms, dir) = mode.into();

            let timer_enabled = Self::regs().cr1().read().cen();
            // Changing from edge aligned to center aligned (and vice versa) is not allowed while the timer is running.
            // Changing direction is discouraged while the timer is running.
            assert!(!timer_enabled);

            Self::regs_gp16().cr1().modify(|r| r.set_dir(dir));
            Self::regs_gp16().cr1().modify(|r| r.set_cms(cms))
        }

        fn get_counting_mode(&self) -> CountingMode {
            let cr1 = Self::regs_gp16().cr1().read();
            (cr1.cms(), cr1.dir()).into()
        }

        fn set_clock_division(&mut self, ckd: vals::Ckd) {
            Self::regs_gp16().cr1().modify(|r| r.set_ckd(ckd));
        }
    }

    pub trait GeneralPurpose32bitInstance: GeneralPurpose16bitInstance {
        fn regs_gp32() -> crate::pac::timer::TimGp32;

        fn set_frequency(&mut self, frequency: Hertz) {
            let f = frequency.0;
            assert!(f > 0);
            let timer_f = Self::frequency().0;
            let pclk_ticks_per_timer_period = (timer_f / f) as u64;
            let psc: u16 = unwrap!(((pclk_ticks_per_timer_period - 1) / (1 << 32)).try_into());
            let arr: u32 = unwrap!((pclk_ticks_per_timer_period / (psc as u64 + 1)).try_into());

            let regs = Self::regs_gp32();
            regs.psc().write(|r| r.set_psc(psc));
            regs.arr().write(|r| r.set_arr(arr));

            regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTERONLY));
            regs.egr().write(|r| r.set_ug(true));
            regs.cr1().modify(|r| r.set_urs(vals::Urs::ANYEVENT));
        }

        fn get_frequency(&self) -> Hertz {
            let timer_f = Self::frequency();

            let regs = Self::regs_gp32();
            let arr = regs.arr().read().arr();
            let psc = regs.psc().read().psc();

            timer_f / arr / (psc + 1)
        }
    }

    pub trait AdvancedControlInstance: GeneralPurpose16bitInstance {
        fn regs_advanced() -> crate::pac::timer::TimAdv;
    }

    pub trait CaptureCompare16bitInstance: GeneralPurpose16bitInstance {
        fn set_input_capture_filter(&mut self, channel: Channel, icf: vals::Icf) {
            let raw_channel = channel.raw();
            Self::regs_gp16()
                .ccmr_input(raw_channel / 2)
                .modify(|r| r.set_icf(raw_channel % 2, icf));
        }

        fn clear_input_interrupt(&mut self, channel: Channel) {
            Self::regs_gp16().sr().modify(|r| r.set_ccif(channel.raw(), false));
        }

        fn enable_input_interrupt(&mut self, channel: Channel, enable: bool) {
            Self::regs_gp16().dier().modify(|r| r.set_ccie(channel.raw(), enable));
        }
        fn set_input_capture_prescaler(&mut self, channel: Channel, factor: u8) {
            let raw_channel = channel.raw();
            Self::regs_gp16()
                .ccmr_input(raw_channel / 2)
                .modify(|r| r.set_icpsc(raw_channel % 2, factor));
        }

        fn set_input_ti_selection(&mut self, channel: Channel, tisel: InputTISelection) {
            let raw_channel = channel.raw();
            Self::regs_gp16()
                .ccmr_input(raw_channel / 2)
                .modify(|r| r.set_ccs(raw_channel % 2, tisel.into()));
        }
        fn set_input_capture_mode(&mut self, channel: Channel, mode: InputCaptureMode) {
            Self::regs_gp16().ccer().modify(|r| match mode {
                InputCaptureMode::Rising => {
                    r.set_ccnp(channel.raw(), false);
                    r.set_ccp(channel.raw(), false);
                }
                InputCaptureMode::Falling => {
                    r.set_ccnp(channel.raw(), false);
                    r.set_ccp(channel.raw(), true);
                }
                InputCaptureMode::BothEdges => {
                    r.set_ccnp(channel.raw(), true);
                    r.set_ccp(channel.raw(), true);
                }
            });
        }
        fn enable_outputs(&mut self);

        fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode) {
            let r = Self::regs_gp16();
            let raw_channel: usize = channel.raw();
            r.ccmr_output(raw_channel / 2)
                .modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
        }

        fn set_output_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
            Self::regs_gp16()
                .ccer()
                .modify(|w| w.set_ccp(channel.raw(), polarity.into()));
        }

        fn enable_channel(&mut self, channel: Channel, enable: bool) {
            Self::regs_gp16().ccer().modify(|w| w.set_cce(channel.raw(), enable));
        }

        fn set_compare_value(&mut self, channel: Channel, value: u16) {
            Self::regs_gp16().ccr(channel.raw()).modify(|w| w.set_ccr(value));
        }

        fn get_capture_value(&mut self, channel: Channel) -> u16 {
            Self::regs_gp16().ccr(channel.raw()).read().ccr()
        }

        fn get_max_compare_value(&self) -> u16 {
            Self::regs_gp16().arr().read().arr()
        }

        fn get_compare_value(&self, channel: Channel) -> u16 {
            Self::regs_gp16().ccr(channel.raw()).read().ccr()
        }
    }

    pub trait ComplementaryCaptureCompare16bitInstance: CaptureCompare16bitInstance + AdvancedControlInstance {
        fn set_complementary_output_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
            Self::regs_advanced()
                .ccer()
                .modify(|w| w.set_ccnp(channel.raw(), polarity.into()));
        }

        fn set_dead_time_clock_division(&mut self, value: vals::Ckd) {
            Self::regs_advanced().cr1().modify(|w| w.set_ckd(value));
        }

        fn set_dead_time_value(&mut self, value: u8) {
            Self::regs_advanced().bdtr().modify(|w| w.set_dtg(value));
        }

        fn enable_complementary_channel(&mut self, channel: Channel, enable: bool) {
            Self::regs_advanced()
                .ccer()
                .modify(|w| w.set_ccne(channel.raw(), enable));
        }
    }

    pub trait CaptureCompare32bitInstance: GeneralPurpose32bitInstance + CaptureCompare16bitInstance {
        fn set_compare_value(&mut self, channel: Channel, value: u32) {
            Self::regs_gp32().ccr(channel.raw()).modify(|w| w.set_ccr(value));
        }

        fn get_capture_value(&mut self, channel: Channel) -> u32 {
            Self::regs_gp32().ccr(channel.raw()).read().ccr()
        }

        fn get_max_compare_value(&self) -> u32 {
            Self::regs_gp32().arr().read().arr()
        }

        fn get_compare_value(&self, channel: Channel) -> u32 {
            Self::regs_gp32().ccr(channel.raw()).read().ccr()
        }
    }
}

#[derive(Clone, Copy)]
pub enum Channel {
    Ch1,
    Ch2,
    Ch3,
    Ch4,
}

impl Channel {
    pub fn raw(&self) -> usize {
        match self {
            Channel::Ch1 => 0,
            Channel::Ch2 => 1,
            Channel::Ch3 => 2,
            Channel::Ch4 => 3,
        }
    }
}

#[derive(Clone, Copy)]
pub enum InputCaptureMode {
    Rising,
    Falling,
    BothEdges,
}

#[derive(Clone, Copy)]
pub enum InputTISelection {
    Normal,
    Alternate,
    TRC,
}

impl From<InputTISelection> for stm32_metapac::timer::vals::CcmrInputCcs {
    fn from(tisel: InputTISelection) -> Self {
        match tisel {
            InputTISelection::Normal => stm32_metapac::timer::vals::CcmrInputCcs::TI4,
            InputTISelection::Alternate => stm32_metapac::timer::vals::CcmrInputCcs::TI3,
            InputTISelection::TRC => stm32_metapac::timer::vals::CcmrInputCcs::TRC,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CountingMode {
    #[default]
    /// The timer counts up to the reload value and then resets back to 0.
    EdgeAlignedUp,
    /// The timer counts down to 0 and then resets back to the reload value.
    EdgeAlignedDown,
    /// The timer counts up to the reload value and then counts back to 0.
    ///
    /// The output compare interrupt flags of channels configured in output are
    /// set when the counter is counting down.
    CenterAlignedDownInterrupts,
    /// The timer counts up to the reload value and then counts back to 0.
    ///
    /// The output compare interrupt flags of channels configured in output are
    /// set when the counter is counting up.
    CenterAlignedUpInterrupts,
    /// The timer counts up to the reload value and then counts back to 0.
    ///
    /// The output compare interrupt flags of channels configured in output are
    /// set when the counter is counting both up or down.
    CenterAlignedBothInterrupts,
}

impl CountingMode {
    pub fn is_edge_aligned(&self) -> bool {
        match self {
            CountingMode::EdgeAlignedUp | CountingMode::EdgeAlignedDown => true,
            _ => false,
        }
    }

    pub fn is_center_aligned(&self) -> bool {
        match self {
            CountingMode::CenterAlignedDownInterrupts
            | CountingMode::CenterAlignedUpInterrupts
            | CountingMode::CenterAlignedBothInterrupts => true,
            _ => false,
        }
    }
}

impl From<CountingMode> for (vals::Cms, vals::Dir) {
    fn from(value: CountingMode) -> Self {
        match value {
            CountingMode::EdgeAlignedUp => (vals::Cms::EDGEALIGNED, vals::Dir::UP),
            CountingMode::EdgeAlignedDown => (vals::Cms::EDGEALIGNED, vals::Dir::DOWN),
            CountingMode::CenterAlignedDownInterrupts => (vals::Cms::CENTERALIGNED1, vals::Dir::UP),
            CountingMode::CenterAlignedUpInterrupts => (vals::Cms::CENTERALIGNED2, vals::Dir::UP),
            CountingMode::CenterAlignedBothInterrupts => (vals::Cms::CENTERALIGNED3, vals::Dir::UP),
        }
    }
}

impl From<(vals::Cms, vals::Dir)> for CountingMode {
    fn from(value: (vals::Cms, vals::Dir)) -> Self {
        match value {
            (vals::Cms::EDGEALIGNED, vals::Dir::UP) => CountingMode::EdgeAlignedUp,
            (vals::Cms::EDGEALIGNED, vals::Dir::DOWN) => CountingMode::EdgeAlignedDown,
            (vals::Cms::CENTERALIGNED1, _) => CountingMode::CenterAlignedDownInterrupts,
            (vals::Cms::CENTERALIGNED2, _) => CountingMode::CenterAlignedUpInterrupts,
            (vals::Cms::CENTERALIGNED3, _) => CountingMode::CenterAlignedBothInterrupts,
        }
    }
}

#[derive(Clone, Copy)]
pub enum OutputCompareMode {
    Frozen,
    ActiveOnMatch,
    InactiveOnMatch,
    Toggle,
    ForceInactive,
    ForceActive,
    PwmMode1,
    PwmMode2,
}

impl From<OutputCompareMode> for stm32_metapac::timer::vals::Ocm {
    fn from(mode: OutputCompareMode) -> Self {
        match mode {
            OutputCompareMode::Frozen => stm32_metapac::timer::vals::Ocm::FROZEN,
            OutputCompareMode::ActiveOnMatch => stm32_metapac::timer::vals::Ocm::ACTIVEONMATCH,
            OutputCompareMode::InactiveOnMatch => stm32_metapac::timer::vals::Ocm::INACTIVEONMATCH,
            OutputCompareMode::Toggle => stm32_metapac::timer::vals::Ocm::TOGGLE,
            OutputCompareMode::ForceInactive => stm32_metapac::timer::vals::Ocm::FORCEINACTIVE,
            OutputCompareMode::ForceActive => stm32_metapac::timer::vals::Ocm::FORCEACTIVE,
            OutputCompareMode::PwmMode1 => stm32_metapac::timer::vals::Ocm::PWMMODE1,
            OutputCompareMode::PwmMode2 => stm32_metapac::timer::vals::Ocm::PWMMODE2,
        }
    }
}

#[derive(Clone, Copy)]
pub enum OutputPolarity {
    ActiveHigh,
    ActiveLow,
}

impl From<OutputPolarity> for bool {
    fn from(mode: OutputPolarity) -> Self {
        match mode {
            OutputPolarity::ActiveHigh => false,
            OutputPolarity::ActiveLow => true,
        }
    }
}

pub trait Basic16bitInstance: sealed::Basic16bitInstance + 'static {}

pub trait GeneralPurpose16bitInstance: sealed::GeneralPurpose16bitInstance + 'static {}

pub trait GeneralPurpose32bitInstance: sealed::GeneralPurpose32bitInstance + 'static {}

pub trait AdvancedControlInstance: sealed::AdvancedControlInstance + 'static {}

pub trait CaptureCompare16bitInstance:
    sealed::CaptureCompare16bitInstance + GeneralPurpose16bitInstance + 'static
{
}

pub trait ComplementaryCaptureCompare16bitInstance:
    sealed::ComplementaryCaptureCompare16bitInstance + AdvancedControlInstance + 'static
{
}

pub trait CaptureCompare32bitInstance:
    sealed::CaptureCompare32bitInstance + CaptureCompare16bitInstance + GeneralPurpose32bitInstance + 'static
{
}

pin_trait!(Channel1Pin, CaptureCompare16bitInstance);
pin_trait!(Channel1ComplementaryPin, CaptureCompare16bitInstance);
pin_trait!(Channel2Pin, CaptureCompare16bitInstance);
pin_trait!(Channel2ComplementaryPin, CaptureCompare16bitInstance);
pin_trait!(Channel3Pin, CaptureCompare16bitInstance);
pin_trait!(Channel3ComplementaryPin, CaptureCompare16bitInstance);
pin_trait!(Channel4Pin, CaptureCompare16bitInstance);
pin_trait!(Channel4ComplementaryPin, CaptureCompare16bitInstance);
pin_trait!(ExternalTriggerPin, CaptureCompare16bitInstance);
pin_trait!(BreakInputPin, CaptureCompare16bitInstance);
pin_trait!(BreakInputComparator1Pin, CaptureCompare16bitInstance);
pin_trait!(BreakInputComparator2Pin, CaptureCompare16bitInstance);
pin_trait!(BreakInput2Pin, CaptureCompare16bitInstance);
pin_trait!(BreakInput2Comparator1Pin, CaptureCompare16bitInstance);
pin_trait!(BreakInput2Comparator2Pin, CaptureCompare16bitInstance);

#[allow(unused)]
macro_rules! impl_basic_16bit_timer {
    ($inst:ident, $irq:ident) => {
        impl sealed::Basic16bitInstance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;

            fn regs() -> crate::pac::timer::TimBasic {
                unsafe { crate::pac::timer::TimBasic::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_32bit_timer {
    ($inst:ident) => {
        impl sealed::GeneralPurpose32bitInstance for crate::peripherals::$inst {
            fn regs_gp32() -> crate::pac::timer::TimGp32 {
                crate::pac::$inst
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_compare_capable_16bit {
    ($inst:ident) => {
        impl sealed::CaptureCompare16bitInstance for crate::peripherals::$inst {
            fn enable_outputs(&mut self) {}
        }
    };
}

foreach_interrupt! {
    ($inst:ident, timer, TIM_BASIC, UP, $irq:ident) => {
        impl_basic_16bit_timer!($inst, $irq);
        impl Basic16bitInstance for crate::peripherals::$inst {}
    };
    ($inst:ident, timer, TIM_GP16, UP, $irq:ident) => {
        impl_basic_16bit_timer!($inst, $irq);
        impl_compare_capable_16bit!($inst);
        impl Basic16bitInstance for crate::peripherals::$inst {}
        impl GeneralPurpose16bitInstance for crate::peripherals::$inst {}
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {}

        impl sealed::GeneralPurpose16bitInstance for crate::peripherals::$inst {
            fn regs_gp16() -> crate::pac::timer::TimGp16 {
                crate::pac::$inst
            }
        }
    };

    ($inst:ident, timer, TIM_GP32, UP, $irq:ident) => {
        impl_basic_16bit_timer!($inst, $irq);
        impl_32bit_timer!($inst);
        impl_compare_capable_16bit!($inst);
        impl Basic16bitInstance for crate::peripherals::$inst {}
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {}
        impl CaptureCompare32bitInstance for crate::peripherals::$inst {}
        impl GeneralPurpose16bitInstance for crate::peripherals::$inst {}
        impl GeneralPurpose32bitInstance for crate::peripherals::$inst {}
        impl sealed::CaptureCompare32bitInstance for crate::peripherals::$inst {}

        impl sealed::GeneralPurpose16bitInstance for crate::peripherals::$inst {
            fn regs_gp16() -> crate::pac::timer::TimGp16 {
                unsafe { crate::pac::timer::TimGp16::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }
    };

    ($inst:ident, timer, TIM_ADV, UP, $irq:ident) => {
        impl_basic_16bit_timer!($inst, $irq);

        impl Basic16bitInstance for crate::peripherals::$inst {}
        impl GeneralPurpose16bitInstance for crate::peripherals::$inst {}
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {}
        impl ComplementaryCaptureCompare16bitInstance for crate::peripherals::$inst {}
        impl AdvancedControlInstance for crate::peripherals::$inst {}
        impl sealed::CaptureCompare16bitInstance for crate::peripherals::$inst {
            fn enable_outputs(&mut self) {
                use crate::timer::sealed::AdvancedControlInstance;
                let r = Self::regs_advanced();
                r.bdtr().modify(|w| w.set_moe(true));
            }
        }
        impl sealed::ComplementaryCaptureCompare16bitInstance for crate::peripherals::$inst {}
        impl sealed::GeneralPurpose16bitInstance for crate::peripherals::$inst {
            fn regs_gp16() -> crate::pac::timer::TimGp16 {
                unsafe { crate::pac::timer::TimGp16::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }

        impl sealed::AdvancedControlInstance for crate::peripherals::$inst {
            fn regs_advanced() -> crate::pac::timer::TimAdv {
                crate::pac::$inst
            }
        }
    };
}
