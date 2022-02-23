pub mod simple_pwm;

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

    pub trait CaptureCompare16bitInstance: crate::timer::sealed::Basic16bitInstance {
        unsafe fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode);

        unsafe fn enable_channel(&mut self, channel: Channel, enable: bool);

        unsafe fn set_compare_value(&mut self, channel: Channel, value: u16);

        unsafe fn get_max_compare_value(&self) -> u16;
    }

    pub trait CaptureCompare32bitInstance:
        crate::timer::sealed::GeneralPurpose32bitInstance
    {
        unsafe fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode);

        unsafe fn enable_channel(&mut self, channel: Channel, enable: bool);

        unsafe fn set_compare_value(&mut self, channel: Channel, value: u32);

        unsafe fn get_max_compare_value(&self) -> u32;
    }
}

pub trait CaptureCompare16bitInstance:
    sealed::CaptureCompare16bitInstance + crate::timer::Basic16bitInstance + 'static
{
}
pub trait CaptureCompare32bitInstance:
    sealed::CaptureCompare32bitInstance
    + CaptureCompare16bitInstance
    + crate::timer::GeneralPurpose32bitInstance
    + 'static
{
}

#[allow(unused)]
macro_rules! impl_compare_capable_16bit {
    ($inst:ident) => {
        impl crate::pwm::sealed::CaptureCompare16bitInstance for crate::peripherals::$inst {
            unsafe fn set_output_compare_mode(
                &mut self,
                channel: crate::pwm::Channel,
                mode: OutputCompareMode,
            ) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                let r = self.regs_gp16();
                let raw_channel: usize = channel.raw();
                r.ccmr_output(raw_channel / 2)
                    .modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
            }

            unsafe fn enable_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                self.regs_gp16()
                    .ccer()
                    .modify(|w| w.set_cce(channel.raw(), enable));
            }

            unsafe fn set_compare_value(&mut self, channel: Channel, value: u16) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                self.regs_gp16()
                    .ccr(channel.raw())
                    .modify(|w| w.set_ccr(value));
            }

            unsafe fn get_max_compare_value(&self) -> u16 {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                self.regs_gp16().arr().read().arr()
            }
        }
    };
}

crate::pac::interrupts! {
    ($inst:ident, timer, TIM_GP16, UP, $irq:ident) => {
        impl crate::pwm::sealed::CaptureCompare16bitInstance for crate::peripherals::$inst {
            unsafe fn set_output_compare_mode(
                &mut self,
                channel: crate::pwm::Channel,
                mode: OutputCompareMode,
            ) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                let r = self.regs_gp16();
                let raw_channel: usize = channel.raw();
                r.ccmr_output(raw_channel / 2)
                    .modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
            }

            unsafe fn enable_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                self.regs_gp16()
                    .ccer()
                    .modify(|w| w.set_cce(channel.raw(), enable));
            }

            unsafe fn set_compare_value(&mut self, channel: Channel, value: u16) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                self.regs_gp16()
                    .ccr(channel.raw())
                    .modify(|w| w.set_ccr(value));
            }

            unsafe fn get_max_compare_value(&self) -> u16 {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                self.regs_gp16().arr().read().arr()
            }
        }

        impl CaptureCompare16bitInstance for crate::peripherals::$inst {

        }
    };

    ($inst:ident, timer, TIM_GP32, UP, $irq:ident) => {
        impl_compare_capable_16bit!($inst);
        impl crate::pwm::sealed::CaptureCompare32bitInstance for crate::peripherals::$inst {
            unsafe fn set_output_compare_mode(
                &mut self,
                channel: crate::pwm::Channel,
                mode: OutputCompareMode,
            ) {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                let raw_channel = channel.raw();
                self.regs_gp32().ccmr_output(raw_channel / 2).modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
            }

            unsafe fn enable_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                self.regs_gp32().ccer().modify(|w| w.set_cce(channel.raw(), enable));
            }

            unsafe fn set_compare_value(&mut self, channel: Channel, value: u32) {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                self.regs_gp32().ccr(channel.raw()).modify(|w| w.set_ccr(value));
            }

            unsafe fn get_max_compare_value(&self) -> u32 {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                self.regs_gp32().arr().read().arr() as u32
            }
        }
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {

        }
        impl CaptureCompare32bitInstance for crate::peripherals::$inst {

        }
    };

    ($inst:ident, timer, TIM_ADV, UP, $irq:ident) => {
        impl crate::pwm::sealed::CaptureCompare16bitInstance for crate::peripherals::$inst {
            unsafe fn set_output_compare_mode(
                &mut self,
                channel: crate::pwm::Channel,
                mode: OutputCompareMode,
            ) {
                use crate::timer::sealed::AdvancedControlInstance;
                let r = self.regs_advanced();
                let raw_channel: usize = channel.raw();
                r.ccmr_output(raw_channel / 2)
                    .modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
            }

            unsafe fn enable_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::AdvancedControlInstance;
                self.regs_advanced()
                    .ccer()
                    .modify(|w| w.set_cce(channel.raw(), enable));
            }

            unsafe fn set_compare_value(&mut self, channel: Channel, value: u16) {
                use crate::timer::sealed::AdvancedControlInstance;
                self.regs_advanced()
                    .ccr(channel.raw())
                    .modify(|w| w.set_ccr(value));
            }

            unsafe fn get_max_compare_value(&self) -> u16 {
                use crate::timer::sealed::AdvancedControlInstance;
                self.regs_advanced().arr().read().arr()
            }
        }

        impl CaptureCompare16bitInstance for crate::peripherals::$inst {

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
