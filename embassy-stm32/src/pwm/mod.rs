pub mod complementary_pwm;
pub mod simple_pwm;

use stm32_metapac::timer::vals::Ckd;

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

pub(crate) mod sealed {
    use super::*;

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
