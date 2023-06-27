#[cfg(hrtim_v1)]
pub mod advanced_pwm;
pub mod complementary_pwm;
pub mod simple_pwm;

#[cfg(hrtim_v1)]
use core::ops;

use stm32_metapac::timer::vals::Ckd;

#[cfg(hrtim_v1)]
use crate::time::Hertz;

#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
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

#[cfg(hrtim_v1)]
#[derive(Clone, Copy)]
pub(crate) enum HighResolutionControlPrescaler {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}

#[cfg(hrtim_v1)]
impl ops::Div<HighResolutionControlPrescaler> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: HighResolutionControlPrescaler) -> Self::Output {
        let divisor = match rhs {
            HighResolutionControlPrescaler::Div1 => 1,
            HighResolutionControlPrescaler::Div2 => 2,
            HighResolutionControlPrescaler::Div4 => 4,
            HighResolutionControlPrescaler::Div8 => 8,
            HighResolutionControlPrescaler::Div16 => 16,
            HighResolutionControlPrescaler::Div32 => 32,
            HighResolutionControlPrescaler::Div64 => 64,
            HighResolutionControlPrescaler::Div128 => 128,
        };

        Hertz(self.0 / divisor)
    }
}

#[cfg(hrtim_v1)]
impl From<HighResolutionControlPrescaler> for u8 {
    fn from(val: HighResolutionControlPrescaler) -> Self {
        match val {
            HighResolutionControlPrescaler::Div1 => 0b000,
            HighResolutionControlPrescaler::Div2 => 0b001,
            HighResolutionControlPrescaler::Div4 => 0b010,
            HighResolutionControlPrescaler::Div8 => 0b011,
            HighResolutionControlPrescaler::Div16 => 0b100,
            HighResolutionControlPrescaler::Div32 => 0b101,
            HighResolutionControlPrescaler::Div64 => 0b110,
            HighResolutionControlPrescaler::Div128 => 0b111,
        }
    }
}

#[cfg(hrtim_v1)]
impl HighResolutionControlPrescaler {
    pub fn compute_min(base_f: Hertz, frequency: Hertz) -> Self {
        *[
            HighResolutionControlPrescaler::Div1,
            HighResolutionControlPrescaler::Div2,
            HighResolutionControlPrescaler::Div4,
            HighResolutionControlPrescaler::Div8,
            HighResolutionControlPrescaler::Div16,
            HighResolutionControlPrescaler::Div32,
            HighResolutionControlPrescaler::Div64,
            HighResolutionControlPrescaler::Div128,
        ]
        .iter()
        .skip_while(|psc| frequency <= base_f / **psc)
        .next()
        .unwrap()
    }
}

pub(crate) mod sealed {
    use super::*;

    #[cfg(hrtim_v1)]
    pub trait AdvancedCaptureCompare16bitInstance: crate::timer::sealed::HighResolutionControlInstance {
        fn set_master_frequency(frequency: Hertz);

        fn set_channel_frequency(channnel: usize, frequency: Hertz);

        //        fn enable_outputs(enable: bool);
        //
        //        fn enable_channel(&mut self, channel: usize, enable: bool);
    }

    pub trait CaptureCompare16bitInstance: crate::timer::sealed::GeneralPurpose16bitInstance {
        /// Global output enable. Does not do anything on non-advanced timers.
        fn enable_outputs(&mut self, enable: bool);

        fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode);

        fn enable_channel(&mut self, channel: Channel, enable: bool);

        fn set_compare_value(&mut self, channel: Channel, value: u16);

        fn get_max_compare_value(&self) -> u16;
    }

    pub trait ComplementaryCaptureCompare16bitInstance: CaptureCompare16bitInstance {
        fn set_dead_time_clock_division(&mut self, value: Ckd);

        fn set_dead_time_value(&mut self, value: u8);

        fn enable_complementary_channel(&mut self, channel: Channel, enable: bool);
    }

    pub trait CaptureCompare32bitInstance: crate::timer::sealed::GeneralPurpose32bitInstance {
        fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode);

        fn enable_channel(&mut self, channel: Channel, enable: bool);

        fn set_compare_value(&mut self, channel: Channel, value: u32);

        fn get_max_compare_value(&self) -> u32;
    }
}

#[cfg(hrtim_v1)]
pub trait AdvancedCaptureCompare16bitInstance: sealed::AdvancedCaptureCompare16bitInstance + 'static {}

pub trait CaptureCompare16bitInstance:
    sealed::CaptureCompare16bitInstance + crate::timer::GeneralPurpose16bitInstance + 'static
{
}

pub trait ComplementaryCaptureCompare16bitInstance:
    sealed::ComplementaryCaptureCompare16bitInstance + crate::timer::AdvancedControlInstance + 'static
{
}

pub trait CaptureCompare32bitInstance:
    sealed::CaptureCompare32bitInstance + CaptureCompare16bitInstance + crate::timer::GeneralPurpose32bitInstance + 'static
{
}

#[allow(unused)]
macro_rules! impl_compare_capable_16bit {
    ($inst:ident) => {
        impl crate::pwm::sealed::CaptureCompare16bitInstance for crate::peripherals::$inst {
            fn enable_outputs(&mut self, _enable: bool) {}

            fn set_output_compare_mode(&mut self, channel: crate::pwm::Channel, mode: OutputCompareMode) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                let r = Self::regs_gp16();
                let raw_channel: usize = channel.raw();
                r.ccmr_output(raw_channel / 2)
                    .modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
            }

            fn enable_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                Self::regs_gp16()
                    .ccer()
                    .modify(|w| w.set_cce(channel.raw(), enable));
            }

            fn set_compare_value(&mut self, channel: Channel, value: u16) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                Self::regs_gp16().ccr(channel.raw()).modify(|w| w.set_ccr(value));
            }

            fn get_max_compare_value(&self) -> u16 {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                Self::regs_gp16().arr().read().arr()
            }
        }
    };
}

foreach_interrupt! {
    ($inst:ident, timer, TIM_GP16, UP, $irq:ident) => {
        impl_compare_capable_16bit!($inst);

        impl CaptureCompare16bitInstance for crate::peripherals::$inst {

        }
    };

    ($inst:ident, timer, TIM_GP32, UP, $irq:ident) => {
        impl_compare_capable_16bit!($inst);
        impl crate::pwm::sealed::CaptureCompare32bitInstance for crate::peripherals::$inst {
            fn set_output_compare_mode(
                &mut self,
                channel: crate::pwm::Channel,
                mode: OutputCompareMode,
            ) {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                let raw_channel = channel.raw();
                Self::regs_gp32().ccmr_output(raw_channel / 2).modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
            }

            fn enable_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                Self::regs_gp32().ccer().modify(|w| w.set_cce(channel.raw(), enable));
            }

            fn set_compare_value(&mut self, channel: Channel, value: u32) {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                Self::regs_gp32().ccr(channel.raw()).modify(|w| w.set_ccr(value));
            }

            fn get_max_compare_value(&self) -> u32 {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                Self::regs_gp32().arr().read().arr() as u32
            }
        }
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {

        }
        impl CaptureCompare32bitInstance for crate::peripherals::$inst {

        }
    };

    ($inst:ident, timer, TIM_ADV, UP, $irq:ident) => {
        impl crate::pwm::sealed::CaptureCompare16bitInstance for crate::peripherals::$inst {
            fn enable_outputs(&mut self, enable: bool) {
                use crate::timer::sealed::AdvancedControlInstance;
                let r = Self::regs_advanced();
                r.bdtr().modify(|w| w.set_moe(enable));
            }

            fn set_output_compare_mode(
                &mut self,
                channel: crate::pwm::Channel,
                mode: OutputCompareMode,
            ) {
                use crate::timer::sealed::AdvancedControlInstance;
                let r = Self::regs_advanced();
                let raw_channel: usize = channel.raw();
                r.ccmr_output(raw_channel / 2)
                    .modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
            }

            fn enable_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced()
                    .ccer()
                    .modify(|w| w.set_cce(channel.raw(), enable));
            }

            fn set_compare_value(&mut self, channel: Channel, value: u16) {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced()
                    .ccr(channel.raw())
                    .modify(|w| w.set_ccr(value));
            }

            fn get_max_compare_value(&self) -> u16 {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced().arr().read().arr()
            }
        }

        impl CaptureCompare16bitInstance for crate::peripherals::$inst {

        }

        impl crate::pwm::sealed::ComplementaryCaptureCompare16bitInstance for crate::peripherals::$inst {
            fn set_dead_time_clock_division(&mut self, value: Ckd) {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced().cr1().modify(|w| w.set_ckd(value));
            }

            fn set_dead_time_value(&mut self, value: u8) {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced().bdtr().modify(|w| w.set_dtg(value));
            }

            fn enable_complementary_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced()
                    .ccer()
                    .modify(|w| w.set_ccne(channel.raw(), enable));
            }
        }

        impl ComplementaryCaptureCompare16bitInstance for crate::peripherals::$inst {

        }
    };

    ($inst:ident, hrtim, HRTIM, MASTER, $irq:ident) => {
        impl crate::pwm::sealed::AdvancedCaptureCompare16bitInstance for crate::peripherals::$inst {
            fn set_master_frequency(frequency: Hertz) {
                use crate::rcc::sealed::RccPeripheral;
                use crate::timer::sealed::HighResolutionControlInstance;

                let f = frequency.0;
                // TODO: fix frequency source
                // let timer_f = Self::frequency().0;
                let timer_f = Hertz(144_000_000).0;
                let base_f = Hertz((32 * timer_f as u64 / u16::MAX as u64) as u32);
                let psc = HighResolutionControlPrescaler::compute_min(base_f, frequency);

                let psc_timer_f = Hertz(timer_f) / psc;
                let per: u16 = (psc_timer_f / f).0 as u16;

                let regs = Self::regs();

                regs.mcr().modify(|w| w.set_ckpsc(psc.into()));
                regs.mper().modify(|w| w.set_mper(per));
            }

            fn set_channel_frequency(channel: usize, frequency: Hertz) {
                use crate::rcc::sealed::RccPeripheral;
                use crate::timer::sealed::HighResolutionControlInstance;

                let f = frequency.0;
                // TODO: fix frequency source
                // let timer_f = Self::frequency().0;
                let timer_f = Hertz(144_000_000).0;
                let base_f = Hertz((32 * timer_f as u64 / u16::MAX as u64) as u32);
                let psc = HighResolutionControlPrescaler::compute_min(base_f, frequency);

                let psc_timer_f = Hertz(timer_f) / psc;
                let per: u16 = (psc_timer_f / f).0 as u16;

                let regs = Self::regs();

                regs.tim(channel).cr().modify(|w| w.set_ckpsc(psc.into()));
                regs.tim(channel).per().modify(|w| w.set_per(per));
            }
        }

        impl AdvancedCaptureCompare16bitInstance for crate::peripherals::$inst {

        }
    };
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

#[cfg(hrtim_v1)]
mod hrtim_pins {
    use super::*;

    pin_trait!(ChannelAPin, AdvancedCaptureCompare16bitInstance);
    pin_trait!(ChannelAComplementaryPin, AdvancedCaptureCompare16bitInstance);
    pin_trait!(ChannelBPin, AdvancedCaptureCompare16bitInstance);
    pin_trait!(ChannelBComplementaryPin, AdvancedCaptureCompare16bitInstance);
    pin_trait!(ChannelCPin, AdvancedCaptureCompare16bitInstance);
    pin_trait!(ChannelCComplementaryPin, AdvancedCaptureCompare16bitInstance);
    pin_trait!(ChannelDPin, AdvancedCaptureCompare16bitInstance);
    pin_trait!(ChannelDComplementaryPin, AdvancedCaptureCompare16bitInstance);
    pin_trait!(ChannelEPin, AdvancedCaptureCompare16bitInstance);
    pin_trait!(ChannelEComplementaryPin, AdvancedCaptureCompare16bitInstance);
}

#[cfg(hrtim_v1)]
pub use hrtim_pins::*;
