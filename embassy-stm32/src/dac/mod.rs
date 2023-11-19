#![macro_use]

//! Provide access to the STM32 digital-to-analog converter (DAC).
use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};

#[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
use crate::pac::dac;
use crate::rcc::RccPeripheral;
use crate::{peripherals, Peripheral};

mod tsel;
pub use tsel::TriggerSel;

#[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Mode {
    /// Normal mode, channel is connected to external pin with buffer enabled.
    NormalExternalBuffered,
    /// Normal mode, channel is connected to external pin and internal peripherals
    /// with buffer enabled.
    NormalBothBuffered,
    /// Normal mode, channel is connected to external pin with buffer disabled.
    NormalExternalUnbuffered,
    /// Normal mode, channel is connected to internal peripherals with buffer disabled.
    NormalInternalUnbuffered,
    /// Sample-and-hold mode, channel is connected to external pin with buffer enabled.
    SampleHoldExternalBuffered,
    /// Sample-and-hold mode, channel is connected to external pin and internal peripherals
    /// with buffer enabled.
    SampleHoldBothBuffered,
    /// Sample-and-hold mode, channel is connected to external pin and internal peripherals
    /// with buffer disabled.
    SampleHoldBothUnbuffered,
    /// Sample-and-hold mode, channel is connected to internal peripherals with buffer disabled.
    SampleHoldInternalUnbuffered,
}

#[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
impl Mode {
    fn mode(&self) -> dac::vals::Mode {
        match self {
            Mode::NormalExternalBuffered => dac::vals::Mode::NORMAL_EXT_BUFEN,
            Mode::NormalBothBuffered => dac::vals::Mode::NORMAL_EXT_INT_BUFEN,
            Mode::NormalExternalUnbuffered => dac::vals::Mode::NORMAL_EXT_BUFDIS,
            Mode::NormalInternalUnbuffered => dac::vals::Mode::NORMAL_INT_BUFDIS,
            Mode::SampleHoldExternalBuffered => dac::vals::Mode::SAMPHOLD_EXT_BUFEN,
            Mode::SampleHoldBothBuffered => dac::vals::Mode::SAMPHOLD_EXT_INT_BUFEN,
            Mode::SampleHoldBothUnbuffered => dac::vals::Mode::SAMPHOLD_EXT_INT_BUFDIS,
            Mode::SampleHoldInternalUnbuffered => dac::vals::Mode::SAMPHOLD_INT_BUFDIS,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Custom Errors
pub enum Error {
    UnconfiguredChannel,
    InvalidValue,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// DAC Channels
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
/// Provide common functions for DAC channels
pub trait DacChannel<T: Instance, Tx> {
    const CHANNEL: Channel;

    /// Enable trigger of the given channel
    fn set_trigger_enable(&mut self, on: bool) -> Result<(), Error> {
        T::regs().cr().modify(|reg| {
            reg.set_ten(Self::CHANNEL.index(), on);
        });
        Ok(())
    }

    /// Set mode register of the given channel
    #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
    fn set_channel_mode(&mut self, mode: Mode) -> Result<(), Error> {
        T::regs().mcr().modify(|reg| {
            reg.set_mode(Self::CHANNEL.index(), mode.mode());
        });
        Ok(())
    }

    /// Set enable register of the given channel
    fn set_channel_enable(&mut self, on: bool) -> Result<(), Error> {
        T::regs().cr().modify(|reg| {
            reg.set_en(Self::CHANNEL.index(), on);
        });
        Ok(())
    }

    /// Enable the DAC channel `ch`
    fn enable_channel(&mut self) -> Result<(), Error> {
        self.set_channel_enable(true)
    }

    /// Disable the DAC channel `ch`
    fn disable_channel(&mut self) -> Result<(), Error> {
        self.set_channel_enable(false)
    }

    /// Perform a software trigger on `ch`
    fn trigger(&mut self) {
        T::regs().swtrigr().write(|reg| {
            reg.set_swtrig(Self::CHANNEL.index(), true);
        });
    }

    /// Set a value to be output by the DAC on trigger.
    ///
    /// The `value` is written to the corresponding "data holding register".
    fn set(&mut self, value: Value) -> Result<(), Error> {
        match value {
            Value::Bit8(v) => T::regs().dhr8r(Self::CHANNEL.index()).write(|reg| reg.set_dhr(v)),
            Value::Bit12Left(v) => T::regs().dhr12l(Self::CHANNEL.index()).write(|reg| reg.set_dhr(v)),
            Value::Bit12Right(v) => T::regs().dhr12r(Self::CHANNEL.index()).write(|reg| reg.set_dhr(v)),
        }
        Ok(())
    }
}

/// Hold two DAC channels
///
/// Note: This consumes the DAC `Instance` only once, allowing to get both channels simultaneously.
///
/// # Example for obtaining both DAC channels
///
/// ```ignore
/// // DMA channels and pins may need to be changed for your controller
/// let (dac_ch1, dac_ch2) =
///     embassy_stm32::dac::Dac::new(p.DAC1, p.DMA1_CH3, p.DMA1_CH4, p.PA4, p.PA5).split();
/// ```
pub struct Dac<'d, T: Instance, TxCh1, TxCh2> {
    ch1: DacCh1<'d, T, TxCh1>,
    ch2: DacCh2<'d, T, TxCh2>,
}

/// DAC CH1
///
/// Note: This consumes the DAC `Instance`. Use [`Dac::new`] to get both channels simultaneously.
pub struct DacCh1<'d, T: Instance, Tx> {
    /// To consume T
    _peri: PeripheralRef<'d, T>,
    #[allow(unused)] // For chips whose DMA is not (yet) supported
    dma: PeripheralRef<'d, Tx>,
}

/// DAC CH2
///
/// Note: This consumes the DAC `Instance`. Use [`Dac::new`] to get both channels simultaneously.
pub struct DacCh2<'d, T: Instance, Tx> {
    /// Instead of PeripheralRef to consume T
    phantom: PhantomData<&'d mut T>,
    #[allow(unused)] // For chips whose DMA is not (yet) supported
    dma: PeripheralRef<'d, Tx>,
}

impl<'d, T: Instance, Tx> DacCh1<'d, T, Tx> {
    /// Obtain DAC CH1
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        dma: impl Peripheral<P = Tx> + 'd,
        pin: impl Peripheral<P = impl DacPin<T, 1>> + crate::gpio::sealed::Pin + 'd,
    ) -> Self {
        pin.set_as_analog();
        into_ref!(peri, dma);
        T::enable_and_reset();

        let mut dac = Self { _peri: peri, dma };

        // Configure each activated channel. All results can be `unwrap`ed since they
        // will only error if the channel is not configured (i.e. ch1, ch2 are false)
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
        dac.set_channel_mode(Mode::NormalExternalBuffered).unwrap();
        dac.enable_channel().unwrap();
        dac.set_trigger_enable(true).unwrap();

        dac
    }

    /// Select a new trigger for this channel
    ///
    /// **Important**: This disables the channel!
    pub fn select_trigger(&mut self, trigger: TriggerSel) -> Result<(), Error> {
        unwrap!(self.disable_channel());
        T::regs().cr().modify(|reg| {
            reg.set_tsel(0, trigger.tsel());
        });
        Ok(())
    }

    /// Write `data` to the DAC CH1 via DMA.
    ///
    /// To prevent delays/glitches when outputting a periodic waveform, the `circular` flag can be set.
    /// This will configure a circular DMA transfer that periodically outputs the `data`.
    /// Note that for performance reasons in circular mode the transfer complete interrupt is disabled.
    ///
    /// **Important:** Channel 1 has to be configured for the DAC instance!
    pub async fn write(&mut self, data: ValueArray<'_>, circular: bool) -> Result<(), Error>
    where
        Tx: DmaCh1<T>,
    {
        let channel = Channel::Ch1.index();
        debug!("Writing to channel {}", channel);

        // Enable DAC and DMA
        T::regs().cr().modify(|w| {
            w.set_en(channel, true);
            w.set_dmaen(channel, true);
        });

        let tx_request = self.dma.request();
        let dma_channel = &mut self.dma;

        let tx_options = crate::dma::TransferOptions {
            circular,
            half_transfer_ir: false,
            complete_transfer_ir: !circular,
            ..Default::default()
        };

        // Initiate the correct type of DMA transfer depending on what data is passed
        let tx_f = match data {
            ValueArray::Bit8(buf) => unsafe {
                crate::dma::Transfer::new_write(
                    dma_channel,
                    tx_request,
                    buf,
                    T::regs().dhr8r(channel).as_ptr() as *mut u8,
                    tx_options,
                )
            },
            ValueArray::Bit12Left(buf) => unsafe {
                crate::dma::Transfer::new_write(
                    dma_channel,
                    tx_request,
                    buf,
                    T::regs().dhr12l(channel).as_ptr() as *mut u16,
                    tx_options,
                )
            },
            ValueArray::Bit12Right(buf) => unsafe {
                crate::dma::Transfer::new_write(
                    dma_channel,
                    tx_request,
                    buf,
                    T::regs().dhr12r(channel).as_ptr() as *mut u16,
                    tx_options,
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

impl<'d, T: Instance, Tx> DacCh2<'d, T, Tx> {
    /// Obtain DAC CH2
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        dma: impl Peripheral<P = Tx> + 'd,
        pin: impl Peripheral<P = impl DacPin<T, 2>> + crate::gpio::sealed::Pin + 'd,
    ) -> Self {
        pin.set_as_analog();
        into_ref!(_peri, dma);
        T::enable_and_reset();

        let mut dac = Self {
            phantom: PhantomData,
            dma,
        };

        // Configure each activated channel. All results can be `unwrap`ed since they
        // will only error if the channel is not configured (i.e. ch1, ch2 are false)
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
        dac.set_channel_mode(Mode::NormalExternalBuffered).unwrap();
        dac.enable_channel().unwrap();
        dac.set_trigger_enable(true).unwrap();

        dac
    }

    /// Select a new trigger for this channel
    pub fn select_trigger(&mut self, trigger: TriggerSel) -> Result<(), Error> {
        unwrap!(self.disable_channel());
        T::regs().cr().modify(|reg| {
            reg.set_tsel(1, trigger.tsel());
        });
        Ok(())
    }

    /// Write `data` to the DAC CH2 via DMA.
    ///
    /// To prevent delays/glitches when outputting a periodic waveform, the `circular` flag can be set.
    /// This will configure a circular DMA transfer that periodically outputs the `data`.
    /// Note that for performance reasons in circular mode the transfer complete interrupt is disabled.
    ///
    /// **Important:** Channel 2 has to be configured for the DAC instance!
    pub async fn write(&mut self, data: ValueArray<'_>, circular: bool) -> Result<(), Error>
    where
        Tx: DmaCh2<T>,
    {
        let channel = Channel::Ch2.index();
        debug!("Writing to channel {}", channel);

        // Enable DAC and DMA
        T::regs().cr().modify(|w| {
            w.set_en(channel, true);
            w.set_dmaen(channel, true);
        });

        let tx_request = self.dma.request();
        let dma_channel = &mut self.dma;

        let tx_options = crate::dma::TransferOptions {
            circular,
            half_transfer_ir: false,
            complete_transfer_ir: !circular,
            ..Default::default()
        };

        // Initiate the correct type of DMA transfer depending on what data is passed
        let tx_f = match data {
            ValueArray::Bit8(buf) => unsafe {
                crate::dma::Transfer::new_write(
                    dma_channel,
                    tx_request,
                    buf,
                    T::regs().dhr8r(channel).as_ptr() as *mut u8,
                    tx_options,
                )
            },
            ValueArray::Bit12Left(buf) => unsafe {
                crate::dma::Transfer::new_write(
                    dma_channel,
                    tx_request,
                    buf,
                    T::regs().dhr12l(channel).as_ptr() as *mut u16,
                    tx_options,
                )
            },
            ValueArray::Bit12Right(buf) => unsafe {
                crate::dma::Transfer::new_write(
                    dma_channel,
                    tx_request,
                    buf,
                    T::regs().dhr12r(channel).as_ptr() as *mut u16,
                    tx_options,
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

impl<'d, T: Instance, TxCh1, TxCh2> Dac<'d, T, TxCh1, TxCh2> {
    /// Create a new DAC instance with both channels.
    ///
    /// This is used to obtain two independent channels via `split()` for use e.g. with DMA.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        dma_ch1: impl Peripheral<P = TxCh1> + 'd,
        dma_ch2: impl Peripheral<P = TxCh2> + 'd,
        pin_ch1: impl Peripheral<P = impl DacPin<T, 1>> + crate::gpio::sealed::Pin + 'd,
        pin_ch2: impl Peripheral<P = impl DacPin<T, 2>> + crate::gpio::sealed::Pin + 'd,
    ) -> Self {
        pin_ch1.set_as_analog();
        pin_ch2.set_as_analog();
        into_ref!(peri, dma_ch1, dma_ch2);
        T::enable_and_reset();

        let mut dac_ch1 = DacCh1 {
            _peri: peri,
            dma: dma_ch1,
        };

        let mut dac_ch2 = DacCh2 {
            phantom: PhantomData,
            dma: dma_ch2,
        };

        // Configure each activated channel. All results can be `unwrap`ed since they
        // will only error if the channel is not configured (i.e. ch1, ch2 are false)
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v73))]
        dac_ch1.set_channel_mode(Mode::NormalExternalBuffered).unwrap();
        dac_ch1.enable_channel().unwrap();
        dac_ch1.set_trigger_enable(true).unwrap();

        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
        dac_ch2.set_channel_mode(Mode::NormalExternalBuffered).unwrap();
        dac_ch2.enable_channel().unwrap();
        dac_ch2.set_trigger_enable(true).unwrap();

        Self {
            ch1: dac_ch1,
            ch2: dac_ch2,
        }
    }

    /// Split the DAC into CH1 and CH2 for independent use.
    pub fn split(self) -> (DacCh1<'d, T, TxCh1>, DacCh2<'d, T, TxCh2>) {
        (self.ch1, self.ch2)
    }

    /// Get mutable reference to CH1
    pub fn ch1_mut(&mut self) -> &mut DacCh1<'d, T, TxCh1> {
        &mut self.ch1
    }

    /// Get mutable reference to CH2
    pub fn ch2_mut(&mut self) -> &mut DacCh2<'d, T, TxCh2> {
        &mut self.ch2
    }

    /// Get reference to CH1
    pub fn ch1(&mut self) -> &DacCh1<'d, T, TxCh1> {
        &self.ch1
    }

    /// Get reference to CH2
    pub fn ch2(&mut self) -> &DacCh2<'d, T, TxCh2> {
        &self.ch2
    }
}

impl<'d, T: Instance, Tx> DacChannel<T, Tx> for DacCh1<'d, T, Tx> {
    const CHANNEL: Channel = Channel::Ch1;
}

impl<'d, T: Instance, Tx> DacChannel<T, Tx> for DacCh2<'d, T, Tx> {
    const CHANNEL: Channel = Channel::Ch2;
}

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> &'static crate::pac::dac::Dac;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral + 'static {}
dma_trait!(DmaCh1, Instance);
dma_trait!(DmaCh2, Instance);

/// Marks a pin that can be used with the DAC
pub trait DacPin<T: Instance, const C: u8>: crate::gpio::Pin + 'static {}

foreach_peripheral!(
    (dac, $inst:ident) => {
        // H7 uses single bit for both DAC1 and DAC2, this is a hack until a proper fix is implemented
        #[cfg(any(rcc_h7, rcc_h7rm0433))]
        impl crate::rcc::sealed::RccPeripheral for peripherals::$inst {
            fn frequency() -> crate::time::Hertz {
                critical_section::with(|_| unsafe { crate::rcc::get_freqs().pclk1 })
            }

            fn enable_and_reset_with_cs(_cs: critical_section::CriticalSection) {
                crate::pac::RCC.apb1lrstr().modify(|w| w.set_dac12rst(true));
                crate::pac::RCC.apb1lrstr().modify(|w| w.set_dac12rst(false));
                crate::pac::RCC.apb1lenr().modify(|w| w.set_dac12en(true));
            }

            fn disable_with_cs(_cs: critical_section::CriticalSection) {
                crate::pac::RCC.apb1lenr().modify(|w| w.set_dac12en(false))
            }
        }

        #[cfg(any(rcc_h7, rcc_h7rm0433))]
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
