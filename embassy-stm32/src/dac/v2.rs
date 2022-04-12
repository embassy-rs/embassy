use crate::dac::{DacPin, Instance};
use crate::pac::dac;
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    UnconfiguredChannel,
    InvalidValue,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Channel {
    Ch1,
    Ch2,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Ch1Trigger {
    Tim6,
    Tim3,
    Tim7,
    Tim15,
    Tim2,
    Exti9,
    Software,
}

impl Ch1Trigger {
    fn tsel(&self) -> dac::vals::Tsel1 {
        match self {
            Ch1Trigger::Tim6 => dac::vals::Tsel1::TIM6_TRGO,
            Ch1Trigger::Tim3 => dac::vals::Tsel1::TIM3_TRGO,
            Ch1Trigger::Tim7 => dac::vals::Tsel1::TIM7_TRGO,
            Ch1Trigger::Tim15 => dac::vals::Tsel1::TIM15_TRGO,
            Ch1Trigger::Tim2 => dac::vals::Tsel1::TIM2_TRGO,
            Ch1Trigger::Exti9 => dac::vals::Tsel1::EXTI9,
            Ch1Trigger::Software => dac::vals::Tsel1::SOFTWARE,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Ch2Trigger {
    Tim6,
    Tim8,
    Tim7,
    Tim5,
    Tim2,
    Tim4,
    Exti9,
    Software,
}

impl Ch2Trigger {
    fn tsel(&self) -> dac::vals::Tsel2 {
        match self {
            Ch2Trigger::Tim6 => dac::vals::Tsel2::TIM6_TRGO,
            Ch2Trigger::Tim8 => dac::vals::Tsel2::TIM8_TRGO,
            Ch2Trigger::Tim7 => dac::vals::Tsel2::TIM7_TRGO,
            Ch2Trigger::Tim5 => dac::vals::Tsel2::TIM5_TRGO,
            Ch2Trigger::Tim2 => dac::vals::Tsel2::TIM2_TRGO,
            Ch2Trigger::Tim4 => dac::vals::Tsel2::TIM4_TRGO,
            Ch2Trigger::Exti9 => dac::vals::Tsel2::EXTI9,
            Ch2Trigger::Software => dac::vals::Tsel2::SOFTWARE,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Alignment {
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Value {
    Bit8(u8),
    Bit12(u16, Alignment),
}

pub struct Dac<'d, T: Instance> {
    channels: u8,
    phantom: PhantomData<&'d mut T>,
}

macro_rules! enable {
    ($enable_reg:ident, $enable_field:ident, $reset_reg:ident, $reset_field:ident) => {
        crate::pac::RCC
            .$enable_reg()
            .modify(|w| w.$enable_field(true));
        crate::pac::RCC
            .$reset_reg()
            .modify(|w| w.$reset_field(true));
        crate::pac::RCC
            .$reset_reg()
            .modify(|w| w.$reset_field(false));
    };
}

impl<'d, T: Instance> Dac<'d, T> {
    pub fn new_1ch(
        peri: impl Unborrow<Target = T> + 'd,
        _ch1: impl Unborrow<Target = impl DacPin<T, 1>> + 'd,
    ) -> Self {
        unborrow!(peri);
        Self::new_inner(peri, 1)
    }

    pub fn new_2ch(
        peri: impl Unborrow<Target = T> + 'd,
        _ch1: impl Unborrow<Target = impl DacPin<T, 1>> + 'd,
        _ch2: impl Unborrow<Target = impl DacPin<T, 2>> + 'd,
    ) -> Self {
        unborrow!(peri);
        Self::new_inner(peri, 2)
    }

    fn new_inner(_peri: T, channels: u8) -> Self {
        unsafe {
            // Sadly we cannot use `RccPeripheral::enable` since devices are quite inconsistent DAC clock
            // configuration.
            critical_section::with(|_| {
                #[cfg(rcc_h7)]
                enable!(apb1lenr, set_dac12en, apb1lrstr, set_dac12rst);
                #[cfg(rcc_h7ab)]
                enable!(apb1lenr, set_dac1en, apb1lrstr, set_dac1rst);
                #[cfg(stm32g0)]
                enable!(apbenr1, set_dac1en, apbrstr1, set_dac1rst);
                #[cfg(any(stm32l4, stm32l5))]
                enable!(apb1enr1, set_dac1en, apb1rstr1, set_dac1rst);
            });

            if channels >= 1 {
                T::regs().cr().modify(|reg| {
                    reg.set_en1(true);
                });
            }

            if channels >= 2 {
                T::regs().cr().modify(|reg| {
                    reg.set_en2(true);
                });
            }
        }

        Self {
            channels,
            phantom: PhantomData,
        }
    }

    /// Check the channel is configured
    fn check_channel_exists(&self, ch: Channel) -> Result<(), Error> {
        if ch == Channel::Ch2 && self.channels < 2 {
            Err(Error::UnconfiguredChannel)
        } else {
            Ok(())
        }
    }

    fn set_channel_enable(&mut self, ch: Channel, on: bool) -> Result<(), Error> {
        self.check_channel_exists(ch)?;
        match ch {
            Channel::Ch1 => unsafe {
                T::regs().cr().modify(|reg| {
                    reg.set_en1(on);
                })
            },
            Channel::Ch2 => unsafe {
                T::regs().cr().modify(|reg| {
                    reg.set_en2(on);
                });
            },
        }
        Ok(())
    }

    pub fn enable_channel(&mut self, ch: Channel) -> Result<(), Error> {
        self.set_channel_enable(ch, true)
    }

    pub fn disable_channel(&mut self, ch: Channel) -> Result<(), Error> {
        self.set_channel_enable(ch, false)
    }

    pub fn select_trigger_ch1(&mut self, trigger: Ch1Trigger) -> Result<(), Error> {
        self.check_channel_exists(Channel::Ch1)?;
        unwrap!(self.disable_channel(Channel::Ch1));
        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_tsel1(trigger.tsel());
            })
        }
        Ok(())
    }

    pub fn select_trigger_ch2(&mut self, trigger: Ch2Trigger) -> Result<(), Error> {
        self.check_channel_exists(Channel::Ch2)?;
        unwrap!(self.disable_channel(Channel::Ch2));
        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_tsel2(trigger.tsel());
            })
        }
        Ok(())
    }

    pub fn trigger(&mut self, ch: Channel) -> Result<(), Error> {
        self.check_channel_exists(ch)?;
        match ch {
            Channel::Ch1 => unsafe {
                T::regs().swtrigr().write(|reg| {
                    reg.set_swtrig1(true);
                });
            },
            Channel::Ch2 => unsafe {
                T::regs().swtrigr().write(|reg| {
                    reg.set_swtrig2(true);
                })
            },
        }
        Ok(())
    }

    pub fn trigger_all(&mut self) {
        unsafe {
            T::regs().swtrigr().write(|reg| {
                reg.set_swtrig1(true);
                reg.set_swtrig2(true);
            })
        }
    }

    pub fn set(&mut self, ch: Channel, value: Value) -> Result<(), Error> {
        self.check_channel_exists(ch)?;
        match ch {
            Channel::Ch1 => match value {
                Value::Bit8(v) => unsafe {
                    T::regs().dhr8r1().write(|reg| reg.set_dacc1dhr(v));
                },
                Value::Bit12(v, Alignment::Left) => unsafe {
                    T::regs().dhr12l1().write(|reg| reg.set_dacc1dhr(v));
                },
                Value::Bit12(v, Alignment::Right) => unsafe {
                    T::regs().dhr12r1().write(|reg| reg.set_dacc1dhr(v));
                },
            },
            Channel::Ch2 => match value {
                Value::Bit8(v) => unsafe {
                    T::regs().dhr8r2().write(|reg| reg.set_dacc2dhr(v));
                },
                Value::Bit12(v, Alignment::Left) => unsafe {
                    T::regs().dhr12l2().write(|reg| reg.set_dacc2dhr(v));
                },
                Value::Bit12(v, Alignment::Right) => unsafe {
                    T::regs().dhr12r2().write(|reg| reg.set_dacc2dhr(v));
                },
            },
        }
        Ok(())
    }
}
