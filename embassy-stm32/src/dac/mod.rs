#![macro_use]

use embassy_hal_common::{into_ref, PeripheralRef};

use crate::dma::Transfer;
use crate::pac::dac;
use crate::rcc::RccPeripheral;
use crate::{peripherals, Peripheral};

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

impl Channel {
    fn index(&self) -> usize {
        match self {
            Channel::Ch1 => 0,
            Channel::Ch2 => 1,
        }
    }
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

pub struct Dac<'d, T: Instance, Tx> {
    channels: u8,
    txdma: PeripheralRef<'d, Tx>,
    _peri: PeripheralRef<'d, T>,
}

impl<'d, T: Instance, Tx> Dac<'d, T, Tx> {
    pub fn new_1ch(
        peri: impl Peripheral<P = T> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        _ch1: impl Peripheral<P = impl DacPin<T, 1>> + 'd,
    ) -> Self {
        into_ref!(peri);
        Self::new_inner(peri, 1, txdma)
    }

    pub fn new_2ch(
        peri: impl Peripheral<P = T> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        _ch1: impl Peripheral<P = impl DacPin<T, 1>> + 'd,
        _ch2: impl Peripheral<P = impl DacPin<T, 2>> + 'd,
    ) -> Self {
        into_ref!(peri);
        Self::new_inner(peri, 2, txdma)
    }

    fn new_inner(peri: PeripheralRef<'d, T>, channels: u8, txdma: impl Peripheral<P = Tx> + 'd) -> Self {
        into_ref!(txdma);
        T::enable();
        T::reset();

        unsafe {
            T::regs().mcr().modify(|reg| {
                for ch in 0..channels {
                    reg.set_mode(ch as usize, 0);
                    reg.set_mode(ch as usize, 0);
                }
            });

            T::regs().cr().modify(|reg| {
                for ch in 0..channels {
                    reg.set_en(ch as usize, true);
                    reg.set_ten(ch as usize, true);
                }
            });
        }

        Self {
            channels,
            txdma,
            _peri: peri,
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
        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_en(ch.index(), on);
            })
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
        unsafe {
            T::regs().swtrigr().write(|reg| {
                reg.set_swtrig(ch.index(), true);
            });
        }
        Ok(())
    }

    pub fn trigger_all(&mut self) {
        unsafe {
            T::regs().swtrigr().write(|reg| {
                reg.set_swtrig(Channel::Ch1.index(), true);
                reg.set_swtrig(Channel::Ch2.index(), true);
            })
        }
    }

    pub fn set(&mut self, ch: Channel, value: Value) -> Result<(), Error> {
        self.check_channel_exists(ch)?;
        match value {
            Value::Bit8(v) => unsafe {
                T::regs().dhr8r(ch.index()).write(|reg| reg.set_dhr(v));
            },
            Value::Bit12(v, Alignment::Left) => unsafe {
                T::regs().dhr12l(ch.index()).write(|reg| reg.set_dhr(v));
            },
            Value::Bit12(v, Alignment::Right) => unsafe {
                T::regs().dhr12r(ch.index()).write(|reg| reg.set_dhr(v));
            },
        }
        Ok(())
    }

    /// TODO: Allow an array of Value instead of only u16, right-aligned
    pub async fn write(&mut self, data: &[u16]) -> Result<(), Error>
    where
        Tx: Dma<T>,
    {
        // TODO: Make this a parameter or get it from the struct or so...
        const CHANNEL: usize = 0;

        //debug!("Starting DAC");
        unsafe {
            T::regs().cr().modify(|w| {
                w.set_en(CHANNEL, true);
                w.set_dmaen(CHANNEL, true);
            });
        }

        let tx_request = self.txdma.request();

        // Use the 12 bit right-aligned register for now. TODO: distinguish values
        let tx_dst = T::regs().dhr12r(CHANNEL).ptr() as *mut u16;

        let tx_f = unsafe { Transfer::new_write(&mut self.txdma, tx_request, data, tx_dst, Default::default()) };

        //debug!("Awaiting tx_f");

        tx_f.await;

        // finish dma
        unsafe {
            // TODO: Do we need to check any status registers here?

            T::regs().cr().modify(|w| {
                // Disable the dac peripheral
                w.set_en(CHANNEL, false);
                // Disable the DMA. TODO: Is this necessary?
                w.set_dmaen(CHANNEL, false);
            });
        }
        Ok(())
    }
}

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> &'static crate::pac::dac::Dac;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral + 'static {}
dma_trait!(Dma, Instance);

pub trait DacPin<T: Instance, const C: u8>: crate::gpio::Pin + 'static {}

foreach_peripheral!(
    (dac, $inst:ident) => {
        // H7 uses single bit for both DAC1 and DAC2, this is a hack until a proper fix is implemented
        #[cfg(rcc_h7)]
        impl crate::rcc::sealed::RccPeripheral for peripherals::$inst {
            fn frequency() -> crate::time::Hertz {
                critical_section::with(|_| unsafe {
                    crate::rcc::get_freqs().apb1
                })
            }

            fn reset() {
                critical_section::with(|_| unsafe {
                    crate::pac::RCC.apb1lrstr().modify(|w| w.set_dac12rst(true));
                    crate::pac::RCC.apb1lrstr().modify(|w| w.set_dac12rst(false));
                })
            }

            fn enable() {
                critical_section::with(|_| unsafe {
                    crate::pac::RCC.apb1lenr().modify(|w| w.set_dac12en(true));
                })
            }

            fn disable() {
                critical_section::with(|_| unsafe {
                    crate::pac::RCC.apb1lenr().modify(|w| w.set_dac12en(false));
                })
            }
        }

        #[cfg(rcc_h7)]
        impl crate::rcc::RccPeripheral for peripherals::$inst {}

        impl crate::dac::sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::dac::Dac {
                &crate::pac::$inst
            }
        }

        impl crate::dac::Instance for peripherals::$inst {}
    };
);

macro_rules! impl_dac_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::dac::DacPin<peripherals::$inst, $ch> for crate::peripherals::$pin {}
    };
}
