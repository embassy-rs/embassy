#[cfg(feature = "unstable-pac")]
#[macro_use]
pub mod pins;

#[cfg(not(feature = "unstable-pac"))]
#[macro_use]
pub(crate) mod pins;

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

    pub trait CaptureCompareCapable16bitInstance:
        crate::timer::sealed::GeneralPurpose16bitInstance
    {
        unsafe fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode);

        unsafe fn enable_channel(&mut self, channel: Channel, enable: bool);

        unsafe fn set_compare_value(&mut self, channel: Channel, value: u16);

        unsafe fn get_max_compare_value(&self) -> u16;
    }

    pub trait CaptureCompareCapable32bitInstance:
        crate::timer::sealed::GeneralPurpose32bitInstance
    {
        unsafe fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode);

        unsafe fn enable_channel(&mut self, channel: Channel, enable: bool);

        unsafe fn set_compare_value(&mut self, channel: Channel, value: u32);

        unsafe fn get_max_compare_value(&self) -> u32;
    }
}

pub trait CaptureCompareCapable16bitInstance:
    sealed::CaptureCompareCapable16bitInstance + crate::timer::GeneralPurpose16bitInstance + 'static
{
}
pub trait CaptureCompareCapable32bitInstance:
    sealed::CaptureCompareCapable32bitInstance + crate::timer::GeneralPurpose32bitInstance + 'static
{
}

#[allow(unused)]
macro_rules! impl_compare_capable_16bit {
    ($inst:ident) => {
        impl crate::pwm::sealed::CaptureCompareCapable16bitInstance for crate::peripherals::$inst {
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
        impl_compare_capable_16bit!($inst);

        impl CaptureCompareCapable16bitInstance for crate::peripherals::$inst {

        }
    };

    ($inst:ident, timer, TIM_GP32, UP, $irq:ident) => {
        impl_compare_capable_16bit!($inst);
        impl crate::pwm::sealed::CaptureCompareCapable32bitInstance for crate::peripherals::$inst {
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
        impl CaptureCompareCapable16bitInstance for crate::peripherals::$inst {

        }
        impl CaptureCompareCapable32bitInstance for crate::peripherals::$inst {

        }
    };

    ($inst:ident, timer, TIM_ADV, UP, $irq:ident) => {
        impl_compare_capable_16bit!($inst);
        impl CaptureCompareCapable16bitInstance for crate::peripherals::$inst {

        }
    };
}

#[allow(unused)]
macro_rules! impl_pwm_nopin {
    ($inst:ident) => {
        impl_no_pin!($inst, Channel1Pin);
        impl_no_pin!($inst, Channel1ComplementaryPin);
        impl_no_pin!($inst, Channel2Pin);
        impl_no_pin!($inst, Channel2ComplementaryPin);
        impl_no_pin!($inst, Channel3Pin);
        impl_no_pin!($inst, Channel3ComplementaryPin);
        impl_no_pin!($inst, Channel4Pin);
        impl_no_pin!($inst, Channel4ComplementaryPin);
        impl_no_pin!($inst, ExternalTriggerPin);
        impl_no_pin!($inst, BreakInputPin);
        impl_no_pin!($inst, BreakInputComparator1Pin);
        impl_no_pin!($inst, BreakInputComparator2Pin);
        impl_no_pin!($inst, BreakInput2Pin);
        impl_no_pin!($inst, BreakInput2Comparator1Pin);
        impl_no_pin!($inst, BreakInput2Comparator2Pin);
    };
}

crate::pac::peripherals!(
    (timer, $inst:ident) => { impl_pwm_nopin!($inst); };
);

crate::pac::peripheral_pins!(
    ($inst:ident, timer, $block:ident, $pin:ident, CH1, $af:expr) => {
        impl_pin!($inst, Channel1Pin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, CH1N, $af:expr) => {
        impl_pin!($inst, Channel1ComplementaryPin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, CH2, $af:expr) => {
        impl_pin!($inst, Channel2Pin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, CH2N, $af:expr) => {
        impl_pin!($inst, Channel2ComplementaryPin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, CH3, $af:expr) => {
        impl_pin!($inst, Channel3Pin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, CH3N, $af:expr) => {
        impl_pin!($inst, Channel3ComplementaryPin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, CH4, $af:expr) => {
        impl_pin!($inst, Channel4Pin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, CH4N, $af:expr) => {
        impl_pin!($inst, Channel4ComplementaryPin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, ETR, $af:expr) => {
        impl_pin!($inst, ExternalTriggerPin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, BKIN, $af:expr) => {
        impl_pin!($inst, BreakInputPin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, BKIN_COMP1, $af:expr) => {
        impl_pin!($inst, BreakInputComparator1Pin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, BKIN_COMP2, $af:expr) => {
        impl_pin!($inst, BreakInputComparator2Pin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, BKIN2, $af:expr) => {
        impl_pin!($inst, BreakInput2Pin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, BKIN2_COMP1, $af:expr) => {
        impl_pin!($inst, BreakInput2Comparator1Pin, $pin, $af);
    };
    ($inst:ident, timer, $block:ident, $pin:ident, BKIN2_COMP2, $af:expr) => {
        impl_pin!($inst, BreakInput2Comparator2Pin, $pin, $af);
    };
);
