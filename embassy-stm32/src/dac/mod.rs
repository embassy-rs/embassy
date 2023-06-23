#![macro_use]

use embassy_hal_common::{into_ref, PeripheralRef};

use crate::dma::{Transfer, TransferOptions};
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
    const fn index(&self) -> usize {
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
/// Single 8 or 12 bit value that can be output by the DAC
pub enum Value {
    // 8 bit value
    Bit8(u8),
    // 12 bit value stored in a u16, left-aligned
    Bit12Left(u16),
    // 12 bit value stored in a u16, right-aligned
    Bit12Right(u16),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Array variant of [`Value`]
pub enum ValueArray<'a> {
    // 8 bit values
    Bit8(&'a [u8]),
    // 12 bit value stored in a u16, left-aligned
    Bit12Left(&'a [u16]),
    // 12 bit values stored in a u16, right-aligned
    Bit12Right(&'a [u16]),
}

pub struct Dac<'d, T: Instance, Tx> {
    ch1: bool,
    ch2: bool,
    txdma: PeripheralRef<'d, Tx>,
    _peri: PeripheralRef<'d, T>,
}

impl<'d, T: Instance, Tx> Dac<'d, T, Tx> {
    pub fn new_ch1(
        peri: impl Peripheral<P = T> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        _ch1: impl Peripheral<P = impl DacPin<T, 1>> + 'd,
    ) -> Self {
        into_ref!(peri);
        Self::new_inner(peri, true, false, txdma)
    }

    pub fn new_ch2(
        peri: impl Peripheral<P = T> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        _ch2: impl Peripheral<P = impl DacPin<T, 2>> + 'd,
    ) -> Self {
        into_ref!(peri);
        Self::new_inner(peri, false, true, txdma)
    }

    pub fn new_ch1_and_ch2(
        peri: impl Peripheral<P = T> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        _ch1: impl Peripheral<P = impl DacPin<T, 1>> + 'd,
        _ch2: impl Peripheral<P = impl DacPin<T, 2>> + 'd,
    ) -> Self {
        into_ref!(peri);
        Self::new_inner(peri, true, true, txdma)
    }

    /// Perform initialisation steps for the DAC
    fn new_inner(peri: PeripheralRef<'d, T>, ch1: bool, ch2: bool, txdma: impl Peripheral<P = Tx> + 'd) -> Self {
        into_ref!(txdma);
        T::enable();
        T::reset();

        let mut dac = Self {
            ch1,
            ch2,
            txdma,
            _peri: peri,
        };

        // Configure each activated channel. All results can be `unwrap`ed since they
        // will only error if the channel is not configured (i.e. ch1, ch2 are false)
        if ch1 {
            dac.set_channel_mode(Channel::Ch1, 0).unwrap();
            dac.enable_channel(Channel::Ch1).unwrap();
            dac.set_trigger_enable(Channel::Ch1, true).unwrap();
        }
        if ch2 {
            dac.set_channel_mode(Channel::Ch2, 0).unwrap();
            dac.enable_channel(Channel::Ch2).unwrap();
            dac.set_trigger_enable(Channel::Ch2, true).unwrap();
        }

        dac
    }

    /// Check the channel is configured
    fn check_channel_configured(&self, ch: Channel) -> Result<(), Error> {
        if (ch == Channel::Ch1 && !self.ch1) || (ch == Channel::Ch2 && !self.ch2) {
            Err(Error::UnconfiguredChannel)
        } else {
            Ok(())
        }
    }

    /// Enable trigger of the given channel
    fn set_trigger_enable(&mut self, ch: Channel, on: bool) -> Result<(), Error> {
        self.check_channel_configured(ch)?;
        T::regs().cr().modify(|reg| {
            reg.set_ten(ch.index(), on);
        });
        Ok(())
    }

    /// Set mode register of the given channel
    fn set_channel_mode(&mut self, ch: Channel, val: u8) -> Result<(), Error> {
        self.check_channel_configured(ch)?;
        T::regs().mcr().modify(|reg| {
            reg.set_mode(ch.index(), val);
        });
        Ok(())
    }

    /// Set enable register of the given channel
    fn set_channel_enable(&mut self, ch: Channel, on: bool) -> Result<(), Error> {
        self.check_channel_configured(ch)?;
        T::regs().cr().modify(|reg| {
            reg.set_en(ch.index(), on);
        });
        Ok(())
    }

    /// Enable the DAC channel `ch`
    pub fn enable_channel(&mut self, ch: Channel) -> Result<(), Error> {
        self.set_channel_enable(ch, true)
    }

    /// Disable the DAC channel `ch`
    pub fn disable_channel(&mut self, ch: Channel) -> Result<(), Error> {
        self.set_channel_enable(ch, false)
    }

    /// Select a new trigger for CH1 (disables the channel)
    pub fn select_trigger_ch1(&mut self, trigger: Ch1Trigger) -> Result<(), Error> {
        self.check_channel_configured(Channel::Ch1)?;
        unwrap!(self.disable_channel(Channel::Ch1));
        T::regs().cr().modify(|reg| {
            reg.set_tsel1(trigger.tsel());
        });
        Ok(())
    }

    /// Select a new trigger for CH2 (disables the channel)  
    pub fn select_trigger_ch2(&mut self, trigger: Ch2Trigger) -> Result<(), Error> {
        self.check_channel_configured(Channel::Ch2)?;
        unwrap!(self.disable_channel(Channel::Ch2));
        T::regs().cr().modify(|reg| {
            reg.set_tsel2(trigger.tsel());
        });
        Ok(())
    }

    /// Perform a software trigger on `ch`
    pub fn trigger(&mut self, ch: Channel) -> Result<(), Error> {
        self.check_channel_configured(ch)?;
        T::regs().swtrigr().write(|reg| {
            reg.set_swtrig(ch.index(), true);
        });
        Ok(())
    }

    /// Perform a software trigger on all channels
    pub fn trigger_all(&mut self) {
        T::regs().swtrigr().write(|reg| {
            reg.set_swtrig(Channel::Ch1.index(), true);
            reg.set_swtrig(Channel::Ch2.index(), true);
        });
    }

    /// Set a value to be output by the DAC on trigger.
    ///
    /// The `value` is written to the corresponding "data holding register"
    pub fn set(&mut self, ch: Channel, value: Value) -> Result<(), Error> {
        self.check_channel_configured(ch)?;
        match value {
            Value::Bit8(v) => T::regs().dhr8r(ch.index()).write(|reg| reg.set_dhr(v)),
            Value::Bit12Left(v) => T::regs().dhr12l(ch.index()).write(|reg| reg.set_dhr(v)),
            Value::Bit12Right(v) => T::regs().dhr12r(ch.index()).write(|reg| reg.set_dhr(v)),
        }
        Ok(())
    }

    /// Write `data` to the DAC CH1 via DMA.
    ///
    /// To prevent delays/glitches when outputting a periodic waveform, the `circular` flag can be set.
    /// This will configure a circular DMA transfer that periodically outputs the `data`.
    /// Note that for performance reasons in circular mode the transfer complete interrupt is disabled.
    ///
    /// **Important:** Channel 1 has to be configured for the DAC instance!
    pub async fn write_ch1(&mut self, data: ValueArray<'_>, circular: bool) -> Result<(), Error>
    where
        Tx: Dma<T>,
    {
        self.check_channel_configured(Channel::Ch1)?;
        self.write_inner(data, circular, Channel::Ch1).await
    }

    /// Write `data` to the DAC CH2 via DMA.
    ///
    /// To prevent delays/glitches when outputting a periodic waveform, the `circular` flag can be set.
    /// This will configure a circular DMA transfer that periodically outputs the `data`.
    /// Note that for performance reasons in circular mode the transfer complete interrupt is disabled.
    ///
    /// **Important:** Channel 2 has to be configured for the DAC instance!
    pub async fn write_ch2(&mut self, data: ValueArray<'_>, circular: bool) -> Result<(), Error>
    where
        Tx: Dma<T>,
    {
        self.check_channel_configured(Channel::Ch2)?;
        self.write_inner(data, circular, Channel::Ch2).await
    }

    /// Performs the dma write for the given channel.
    /// TODO: Should self be &mut?
    async fn write_inner(&self, data_ch1: ValueArray<'_>, circular: bool, channel: Channel) -> Result<(), Error>
    where
        Tx: Dma<T>,
    {
        let channel = channel.index();

        // Enable DAC and DMA
        T::regs().cr().modify(|w| {
            w.set_en(channel, true);
            w.set_dmaen(channel, true);
        });

        let tx_request = self.txdma.request();
        let dma_channel = &self.txdma;

        // Initiate the correct type of DMA transfer depending on what data is passed
        let tx_f = match data_ch1 {
            ValueArray::Bit8(buf) => unsafe {
                Transfer::new_write(
                    dma_channel,
                    tx_request,
                    buf,
                    T::regs().dhr8r(channel).as_ptr() as *mut u8,
                    TransferOptions {
                        circular,
                        half_transfer_ir: false,
                        complete_transfer_ir: !circular,
                    },
                )
            },
            ValueArray::Bit12Left(buf) => unsafe {
                Transfer::new_write(
                    dma_channel,
                    tx_request,
                    buf,
                    T::regs().dhr12l(channel).as_ptr() as *mut u16,
                    TransferOptions {
                        circular,
                        half_transfer_ir: false,
                        complete_transfer_ir: !circular,
                    },
                )
            },
            ValueArray::Bit12Right(buf) => unsafe {
                Transfer::new_write(
                    dma_channel,
                    tx_request,
                    buf,
                    T::regs().dhr12r(channel).as_ptr() as *mut u16,
                    TransferOptions {
                        circular,
                        half_transfer_ir: false,
                        complete_transfer_ir: !circular,
                    },
                )
            },
        };

        tx_f.await;

        // finish dma
        // TODO: Do we need to check any status registers here?
        T::regs().cr().modify(|w| {
            // Disable the DAC peripheral
            w.set_en(channel, false);
            // Disable the DMA. TODO: Is this necessary?
            w.set_dmaen(channel, false);
        });

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
