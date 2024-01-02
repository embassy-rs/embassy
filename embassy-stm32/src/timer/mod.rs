//! Timers, PWM, quadrature decoder.

pub mod complementary_pwm;
pub mod qei;
pub mod simple_pwm;

use stm32_metapac::timer::vals;

use crate::interrupt;
use crate::rcc::RccPeripheral;
use crate::time::Hertz;

/// Low-level timer access.
#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

pub(crate) mod sealed {
    use super::*;

    /// Basic 16-bit timer instance.
    pub trait Basic16bitInstance: RccPeripheral {
        /// Interrupt for this timer.
        type Interrupt: interrupt::typelevel::Interrupt;

        /// Get access to the basic 16bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs() -> crate::pac::timer::TimBasic;

        /// Start the timer.
        fn start(&mut self) {
            Self::regs().cr1().modify(|r| r.set_cen(true));
        }

        /// Stop the timer.
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
            let divide_by = pclk_ticks_per_timer_period / (u32::from(psc) + 1);

            // the timer counts `0..=arr`, we want it to count `0..divide_by`
            let arr = unwrap!(u16::try_from(divide_by - 1));

            let regs = Self::regs();
            regs.psc().write(|r| r.set_psc(psc));
            regs.arr().write(|r| r.set_arr(arr));

            regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTERONLY));
            regs.egr().write(|r| r.set_ug(true));
            regs.cr1().modify(|r| r.set_urs(vals::Urs::ANYEVENT));
        }

        /// Clear update interrupt.
        ///
        /// Returns whether the update interrupt flag was set.
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

        /// Enable/disable the update interrupt.
        fn enable_update_interrupt(&mut self, enable: bool) {
            Self::regs().dier().modify(|r| r.set_uie(enable));
        }

        /// Enable/disable the update dma.
        fn enable_update_dma(&mut self, enable: bool) {
            Self::regs().dier().modify(|r| r.set_ude(enable));
        }

        /// Get the update dma enable/disable state.
        fn get_update_dma_state(&self) -> bool {
            Self::regs().dier().read().ude()
        }

        /// Enable/disable autoreload preload.
        fn set_autoreload_preload(&mut self, enable: bool) {
            Self::regs().cr1().modify(|r| r.set_arpe(enable));
        }

        /// Get the timer frequency.
        fn get_frequency(&self) -> Hertz {
            let timer_f = Self::frequency();

            let regs = Self::regs();
            let arr = regs.arr().read().arr();
            let psc = regs.psc().read().psc();

            timer_f / arr / (psc + 1)
        }
    }

    /// Gneral-purpose 16-bit timer instance.
    pub trait GeneralPurpose16bitInstance: Basic16bitInstance {
        /// Get access to the general purpose 16bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs_gp16() -> crate::pac::timer::TimGp16;

        /// Set counting mode.
        fn set_counting_mode(&mut self, mode: CountingMode) {
            let (cms, dir) = mode.into();

            let timer_enabled = Self::regs().cr1().read().cen();
            // Changing from edge aligned to center aligned (and vice versa) is not allowed while the timer is running.
            // Changing direction is discouraged while the timer is running.
            assert!(!timer_enabled);

            Self::regs_gp16().cr1().modify(|r| r.set_dir(dir));
            Self::regs_gp16().cr1().modify(|r| r.set_cms(cms))
        }

        /// Get counting mode.
        fn get_counting_mode(&self) -> CountingMode {
            let cr1 = Self::regs_gp16().cr1().read();
            (cr1.cms(), cr1.dir()).into()
        }

        /// Set clock divider.
        fn set_clock_division(&mut self, ckd: vals::Ckd) {
            Self::regs_gp16().cr1().modify(|r| r.set_ckd(ckd));
        }
    }

    /// Gneral-purpose 32-bit timer instance.
    pub trait GeneralPurpose32bitInstance: GeneralPurpose16bitInstance {
        /// Get access to the general purpose 32bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs_gp32() -> crate::pac::timer::TimGp32;

        /// Set timer frequency.
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

        /// Get timer frequency.
        fn get_frequency(&self) -> Hertz {
            let timer_f = Self::frequency();

            let regs = Self::regs_gp32();
            let arr = regs.arr().read().arr();
            let psc = regs.psc().read().psc();

            timer_f / arr / (psc + 1)
        }
    }

    /// Advanced control timer instance.
    pub trait AdvancedControlInstance: GeneralPurpose16bitInstance {
        /// Get access to the advanced timer registers.
        fn regs_advanced() -> crate::pac::timer::TimAdv;
    }

    /// Capture/Compare 16-bit timer instance.
    pub trait CaptureCompare16bitInstance: GeneralPurpose16bitInstance {
        /// Set input capture filter.
        fn set_input_capture_filter(&mut self, channel: Channel, icf: vals::Icf) {
            let raw_channel = channel.index();
            Self::regs_gp16()
                .ccmr_input(raw_channel / 2)
                .modify(|r| r.set_icf(raw_channel % 2, icf));
        }

        /// Clear input interrupt.
        fn clear_input_interrupt(&mut self, channel: Channel) {
            Self::regs_gp16().sr().modify(|r| r.set_ccif(channel.index(), false));
        }

        /// Enable input interrupt.
        fn enable_input_interrupt(&mut self, channel: Channel, enable: bool) {
            Self::regs_gp16().dier().modify(|r| r.set_ccie(channel.index(), enable));
        }

        /// Set input capture prescaler.
        fn set_input_capture_prescaler(&mut self, channel: Channel, factor: u8) {
            let raw_channel = channel.index();
            Self::regs_gp16()
                .ccmr_input(raw_channel / 2)
                .modify(|r| r.set_icpsc(raw_channel % 2, factor));
        }

        /// Set input TI selection.
        fn set_input_ti_selection(&mut self, channel: Channel, tisel: InputTISelection) {
            let raw_channel = channel.index();
            Self::regs_gp16()
                .ccmr_input(raw_channel / 2)
                .modify(|r| r.set_ccs(raw_channel % 2, tisel.into()));
        }

        /// Set input capture mode.
        fn set_input_capture_mode(&mut self, channel: Channel, mode: InputCaptureMode) {
            Self::regs_gp16().ccer().modify(|r| match mode {
                InputCaptureMode::Rising => {
                    r.set_ccnp(channel.index(), false);
                    r.set_ccp(channel.index(), false);
                }
                InputCaptureMode::Falling => {
                    r.set_ccnp(channel.index(), false);
                    r.set_ccp(channel.index(), true);
                }
                InputCaptureMode::BothEdges => {
                    r.set_ccnp(channel.index(), true);
                    r.set_ccp(channel.index(), true);
                }
            });
        }

        /// Enable timer outputs.
        fn enable_outputs(&mut self);

        /// Set output compare mode.
        fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode) {
            let r = Self::regs_gp16();
            let raw_channel: usize = channel.index();
            r.ccmr_output(raw_channel / 2)
                .modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
        }

        /// Set output polarity.
        fn set_output_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
            Self::regs_gp16()
                .ccer()
                .modify(|w| w.set_ccp(channel.index(), polarity.into()));
        }

        /// Enable/disable a channel.
        fn enable_channel(&mut self, channel: Channel, enable: bool) {
            Self::regs_gp16().ccer().modify(|w| w.set_cce(channel.index(), enable));
        }

        /// Get enable/disable state of a channel
        fn get_channel_enable_state(&self, channel: Channel) -> bool {
            Self::regs_gp16().ccer().read().cce(channel.index())
        }

        /// Set compare value for a channel.
        fn set_compare_value(&mut self, channel: Channel, value: u16) {
            Self::regs_gp16().ccr(channel.index()).modify(|w| w.set_ccr(value));
        }

        /// Get capture value for a channel.
        fn get_capture_value(&mut self, channel: Channel) -> u16 {
            Self::regs_gp16().ccr(channel.index()).read().ccr()
        }

        /// Get max compare value. This depends on the timer frequency and the clock frequency from RCC.
        fn get_max_compare_value(&self) -> u16 {
            Self::regs_gp16().arr().read().arr()
        }

        /// Get compare value for a channel.
        fn get_compare_value(&self, channel: Channel) -> u16 {
            Self::regs_gp16().ccr(channel.index()).read().ccr()
        }

        /// Set output compare preload.
        fn set_output_compare_preload(&mut self, channel: Channel, preload: bool) {
            let channel_index = channel.index();
            Self::regs_gp16()
                .ccmr_output(channel_index / 2)
                .modify(|w| w.set_ocpe(channel_index % 2, preload));
        }
    }

    /// Capture/Compare 16-bit timer instance with complementary pin support.
    pub trait ComplementaryCaptureCompare16bitInstance: CaptureCompare16bitInstance + AdvancedControlInstance {
        /// Set complementary output polarity.
        fn set_complementary_output_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
            Self::regs_advanced()
                .ccer()
                .modify(|w| w.set_ccnp(channel.index(), polarity.into()));
        }

        /// Set clock divider for the dead time.
        fn set_dead_time_clock_division(&mut self, value: vals::Ckd) {
            Self::regs_advanced().cr1().modify(|w| w.set_ckd(value));
        }

        /// Set dead time, as a fraction of the max duty value.
        fn set_dead_time_value(&mut self, value: u8) {
            Self::regs_advanced().bdtr().modify(|w| w.set_dtg(value));
        }

        /// Enable/disable a complementary channel.
        fn enable_complementary_channel(&mut self, channel: Channel, enable: bool) {
            Self::regs_advanced()
                .ccer()
                .modify(|w| w.set_ccne(channel.index(), enable));
        }
    }

    /// Capture/Compare 32-bit timer instance.
    pub trait CaptureCompare32bitInstance: GeneralPurpose32bitInstance + CaptureCompare16bitInstance {
        /// Set comapre value for a channel.
        fn set_compare_value(&mut self, channel: Channel, value: u32) {
            Self::regs_gp32().ccr(channel.index()).modify(|w| w.set_ccr(value));
        }

        /// Get capture value for a channel.
        fn get_capture_value(&mut self, channel: Channel) -> u32 {
            Self::regs_gp32().ccr(channel.index()).read().ccr()
        }

        /// Get max compare value. This depends on the timer frequency and the clock frequency from RCC.
        fn get_max_compare_value(&self) -> u32 {
            Self::regs_gp32().arr().read().arr()
        }

        /// Get compare value for a channel.
        fn get_compare_value(&self, channel: Channel) -> u32 {
            Self::regs_gp32().ccr(channel.index()).read().ccr()
        }
    }
}

/// Timer channel.
#[derive(Clone, Copy)]
pub enum Channel {
    /// Channel 1.
    Ch1,
    /// Channel 2.
    Ch2,
    /// Channel 3.
    Ch3,
    /// Channel 4.
    Ch4,
}

impl Channel {
    /// Get the channel index (0..3)
    pub fn index(&self) -> usize {
        match self {
            Channel::Ch1 => 0,
            Channel::Ch2 => 1,
            Channel::Ch3 => 2,
            Channel::Ch4 => 3,
        }
    }
}

/// Input capture mode.
#[derive(Clone, Copy)]
pub enum InputCaptureMode {
    /// Rising edge only.
    Rising,
    /// Falling edge only.
    Falling,
    /// Both rising or falling edges.
    BothEdges,
}

/// Input TI selection.
#[derive(Clone, Copy)]
pub enum InputTISelection {
    /// Normal
    Normal,
    /// Alternate
    Alternate,
    /// TRC
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

/// Timer counting mode.
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
    /// Return whether this mode is edge-aligned (up or down).
    pub fn is_edge_aligned(&self) -> bool {
        match self {
            CountingMode::EdgeAlignedUp | CountingMode::EdgeAlignedDown => true,
            _ => false,
        }
    }

    /// Return whether this mode is center-aligned.
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

/// Output compare mode.
#[derive(Clone, Copy)]
pub enum OutputCompareMode {
    /// The comparison between the output compare register TIMx_CCRx and
    /// the counter TIMx_CNT has no effect on the outputs.
    /// (this mode is used to generate a timing base).
    Frozen,
    /// Set channel to active level on match. OCxREF signal is forced high when the
    /// counter TIMx_CNT matches the capture/compare register x (TIMx_CCRx).
    ActiveOnMatch,
    /// Set channel to inactive level on match. OCxREF signal is forced low when the
    /// counter TIMx_CNT matches the capture/compare register x (TIMx_CCRx).
    InactiveOnMatch,
    /// Toggle - OCxREF toggles when TIMx_CNT=TIMx_CCRx.
    Toggle,
    /// Force inactive level - OCxREF is forced low.
    ForceInactive,
    /// Force active level - OCxREF is forced high.
    ForceActive,
    /// PWM mode 1 - In upcounting, channel is active as long as TIMx_CNT<TIMx_CCRx
    /// else inactive. In downcounting, channel is inactive (OCxREF=0) as long as
    /// TIMx_CNT>TIMx_CCRx else active (OCxREF=1).
    PwmMode1,
    /// PWM mode 2 - In upcounting, channel is inactive as long as
    /// TIMx_CNT<TIMx_CCRx else active. In downcounting, channel is active as long as
    /// TIMx_CNT>TIMx_CCRx else inactive.
    PwmMode2,
    // TODO: there's more modes here depending on the chip family.
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

/// Timer output pin polarity.
#[derive(Clone, Copy)]
pub enum OutputPolarity {
    /// Active high (higher duty value makes the pin spend more time high).
    ActiveHigh,
    /// Active low (higher duty value makes the pin spend more time low).
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

/// Basic 16-bit timer instance.
pub trait Basic16bitInstance: sealed::Basic16bitInstance + 'static {}

/// Gneral-purpose 16-bit timer instance.
pub trait GeneralPurpose16bitInstance: sealed::GeneralPurpose16bitInstance + Basic16bitInstance + 'static {}

/// Gneral-purpose 32-bit timer instance.
pub trait GeneralPurpose32bitInstance:
    sealed::GeneralPurpose32bitInstance + GeneralPurpose16bitInstance + 'static
{
}

/// Advanced control timer instance.
pub trait AdvancedControlInstance: sealed::AdvancedControlInstance + GeneralPurpose16bitInstance + 'static {}

/// Capture/Compare 16-bit timer instance.
pub trait CaptureCompare16bitInstance:
    sealed::CaptureCompare16bitInstance + GeneralPurpose16bitInstance + 'static
{
}

/// Capture/Compare 16-bit timer instance with complementary pin support.
pub trait ComplementaryCaptureCompare16bitInstance:
    sealed::ComplementaryCaptureCompare16bitInstance + CaptureCompare16bitInstance + AdvancedControlInstance + 'static
{
}

/// Capture/Compare 32-bit timer instance.
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

// Update Event trigger DMA for every timer
dma_trait!(UpDma, Basic16bitInstance);
