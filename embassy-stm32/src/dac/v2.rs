use crate::dac::{DacPin, Instance};
use crate::gpio::AnyPin;
use crate::pac::dac;
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

/// Sadly we cannot use `RccPeripheral::enable` since devices are quite inconsistent DAC clock
/// configuration.
unsafe fn enable() {
    #[cfg(rcc_h7)]
    crate::pac::RCC.apb1lenr().modify(|w| w.set_dac12en(true));
    #[cfg(rcc_g0)]
    crate::pac::RCC.apbenr1().modify(|w| w.set_dac1en(true));
    #[cfg(rcc_l4)]
    crate::pac::RCC.apb1enr1().modify(|w| w.set_dac1en(true));
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    UnconfiguredChannel,
    InvalidValue,
}

pub enum Channel {
    Ch1,
    Ch2,
}

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

pub enum Alignment {
    Left,
    Right,
}

pub enum Value {
    Bit8(u8),
    Bit12(u16, Alignment),
}

pub struct Dac<'d, T: Instance> {
    ch1: Option<AnyPin>,
    ch2: Option<AnyPin>,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Dac<'d, T> {
    pub fn new(
        _peri: impl Unborrow<Target = T> + 'd,
        ch1: impl Unborrow<Target = impl DacPin<T, 1>>,
        ch2: impl Unborrow<Target = impl DacPin<T, 2>>,
    ) -> Self {
        unborrow!(ch1, ch2);

        unsafe {
            enable();
        }

        let ch1 = ch1.degrade_optional();
        if ch1.is_some() {
            unsafe {
                T::regs().cr().modify(|reg| {
                    reg.set_en1(true);
                });
            }
        }

        let ch2 = ch2.degrade_optional();
        if ch2.is_some() {
            unsafe {
                T::regs().cr().modify(|reg| {
                    reg.set_en2(true);
                });
            }
        }

        Self {
            ch1,
            ch2,
            phantom: PhantomData,
        }
    }

    fn set_channel_enable(&mut self, ch: Channel, on: bool) -> Result<(), Error> {
        match ch {
            Channel::Ch1 => {
                if self.ch1.is_none() {
                    Err(Error::UnconfiguredChannel)
                } else {
                    unsafe {
                        T::regs().cr().modify(|reg| {
                            reg.set_en1(on);
                        });
                    }
                    Ok(())
                }
            }
            Channel::Ch2 => {
                if self.ch2.is_none() {
                    Err(Error::UnconfiguredChannel)
                } else {
                    unsafe {
                        T::regs().cr().modify(|reg| {
                            reg.set_en2(on);
                        });
                    }
                    Ok(())
                }
            }
        }
    }

    pub fn enable_channel(&mut self, ch: Channel) -> Result<(), Error> {
        self.set_channel_enable(ch, true)
    }

    pub fn disable_channel(&mut self, ch: Channel) -> Result<(), Error> {
        self.set_channel_enable(ch, false)
    }

    pub fn select_trigger_ch1(&mut self, trigger: Ch1Trigger) -> Result<(), Error> {
        if self.ch1.is_none() {
            return Err(Error::UnconfiguredChannel);
        }
        unwrap!(self.disable_channel(Channel::Ch1));
        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_tsel1(trigger.tsel());
            })
        }
        Ok(())
    }

    pub fn select_trigger_ch2(&mut self, trigger: Ch2Trigger) -> Result<(), Error> {
        if self.ch2.is_none() {
            return Err(Error::UnconfiguredChannel);
        }
        unwrap!(self.disable_channel(Channel::Ch2));
        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_tsel2(trigger.tsel());
            })
        }
        Ok(())
    }

    pub fn trigger(&mut self, ch: Channel) -> Result<(), Error> {
        match ch {
            Channel::Ch1 => {
                if self.ch1.is_none() {
                    Err(Error::UnconfiguredChannel)
                } else {
                    unsafe {
                        T::regs().swtrigr().write(|reg| {
                            reg.set_swtrig1(true);
                        });
                    }
                    Ok(())
                }
            }
            Channel::Ch2 => {
                if self.ch2.is_none() {
                    Err(Error::UnconfiguredChannel)
                } else {
                    unsafe {
                        T::regs().swtrigr().write(|reg| {
                            reg.set_swtrig2(true);
                        });
                    }
                    Ok(())
                }
            }
        }
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
        match ch {
            Channel::Ch1 => {
                if self.ch1.is_none() {
                    Err(Error::UnconfiguredChannel)
                } else {
                    match value {
                        Value::Bit8(v) => unsafe {
                            T::regs().dhr8r1().write(|reg| reg.set_dacc1dhr(v));
                        },
                        Value::Bit12(v, Alignment::Left) => unsafe {
                            T::regs().dhr12l1().write(|reg| reg.set_dacc1dhr(v));
                        },
                        Value::Bit12(v, Alignment::Right) => unsafe {
                            T::regs().dhr12r1().write(|reg| reg.set_dacc1dhr(v));
                        },
                    }
                    Ok(())
                }
            }
            Channel::Ch2 => {
                if self.ch2.is_none() {
                    Err(Error::UnconfiguredChannel)
                } else {
                    match value {
                        Value::Bit8(v) => unsafe {
                            T::regs().dhr8r2().write(|reg| reg.set_dacc2dhr(v));
                        },
                        Value::Bit12(v, Alignment::Left) => unsafe {
                            T::regs().dhr12l2().write(|reg| reg.set_dacc2dhr(v));
                        },
                        Value::Bit12(v, Alignment::Right) => unsafe {
                            T::regs().dhr12r2().write(|reg| reg.set_dacc2dhr(v));
                        },
                    }
                    Ok(())
                }
            }
        }
    }
}
