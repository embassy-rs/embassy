//! Digital to Analog Converter (DAC)
#![macro_use]

use core::marker::PhantomData;

use crate::dma::ChannelAndRequest;
use crate::mode::{Async, Blocking, Mode as PeriMode};
#[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
use crate::pac::dac;
use crate::rcc::{self, RccPeripheral};
use crate::{peripherals, Peri};

mod tsel;
use embassy_hal_internal::PeripheralType;
pub use tsel::TriggerSel;

/// Operating mode for DAC channel
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
/// Single 8 or 12 bit value that can be output by the DAC.
///
/// 12-bit values outside the permitted range are silently truncated.
pub enum Value {
    /// 8 bit value
    Bit8(u8),
    /// 12 bit value stored in a u16, left-aligned
    Bit12Left(u16),
    /// 12 bit value stored in a u16, right-aligned
    Bit12Right(u16),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Dual 8 or 12 bit values that can be output by the DAC channels 1 and 2 simultaneously.
///
/// 12-bit values outside the permitted range are silently truncated.
pub enum DualValue {
    /// 8 bit value
    Bit8(u8, u8),
    /// 12 bit value stored in a u16, left-aligned
    Bit12Left(u16, u16),
    /// 12 bit value stored in a u16, right-aligned
    Bit12Right(u16, u16),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Array variant of [`Value`].
pub enum ValueArray<'a> {
    /// 8 bit values
    Bit8(&'a [u8]),
    /// 12 bit value stored in a u16, left-aligned
    Bit12Left(&'a [u16]),
    /// 12 bit values stored in a u16, right-aligned
    Bit12Right(&'a [u16]),
}

/// Driver for a single DAC channel.
///
/// If you want to use both channels, either together or independently,
/// create a [`Dac`] first and use it to access each channel.
pub struct DacChannel<'d, T: Instance, C: Channel, M: PeriMode> {
    phantom: PhantomData<&'d mut (T, C, M)>,
    #[allow(unused)]
    dma: Option<ChannelAndRequest<'d>>,
}

/// DAC channel 1 type alias.
pub type DacCh1<'d, T, M> = DacChannel<'d, T, Ch1, M>;
/// DAC channel 2 type alias.
pub type DacCh2<'d, T, M> = DacChannel<'d, T, Ch2, M>;

impl<'d, T: Instance, C: Channel> DacChannel<'d, T, C, Async> {
    /// Create a new `DacChannel` instance, consuming the underlying DAC peripheral.
    ///
    /// The channel is enabled on creation and begin to drive the output pin.
    /// Note that some methods, such as `set_trigger()` and `set_mode()`, will
    /// disable the channel; you must re-enable it with `enable()`.
    ///
    /// By default, triggering is disabled, but it can be enabled using
    /// [`DacChannel::set_trigger()`].
    pub fn new(peri: Peri<'d, T>, dma: Peri<'d, impl Dma<T, C>>, pin: Peri<'d, impl DacPin<T, C>>) -> Self {
        pin.set_as_analog();
        Self::new_inner(
            peri,
            new_dma!(dma),
            #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
            Mode::NormalExternalBuffered,
        )
    }

    /// Create a new `DacChannel` instance where the external output pin is not used,
    /// so the DAC can only be used to generate internal signals.
    /// The GPIO pin is therefore available to be used for other functions.
    ///
    /// The channel is set to [`Mode::NormalInternalUnbuffered`] and enabled on creation.
    /// Note that some methods, such as `set_trigger()` and `set_mode()`, will disable the
    /// channel; you must re-enable it with `enable()`.
    ///
    /// By default, triggering is disabled, but it can be enabled using
    /// [`DacChannel::set_trigger()`].
    #[cfg(all(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7), not(any(stm32h56x, stm32h57x))))]
    pub fn new_internal(peri: Peri<'d, T>, dma: Peri<'d, impl Dma<T, C>>) -> Self {
        Self::new_inner(peri, new_dma!(dma), Mode::NormalInternalUnbuffered)
    }

    /// Write `data` to this channel via DMA.
    ///
    /// To prevent delays or glitches when outputing a periodic waveform, the `circular`
    /// flag can be set. This configures a circular DMA transfer that continually outputs
    /// `data`. Note that for performance reasons in circular mode the transfer-complete
    /// interrupt is disabled.
    #[cfg(not(gpdma))]
    pub async fn write(&mut self, data: ValueArray<'_>, circular: bool) {
        // Enable DAC and DMA
        T::regs().cr().modify(|w| {
            w.set_en(C::IDX, true);
            w.set_dmaen(C::IDX, true);
        });

        let dma = self.dma.as_mut().unwrap();

        let tx_options = crate::dma::TransferOptions {
            circular,
            half_transfer_ir: false,
            complete_transfer_ir: !circular,
            ..Default::default()
        };

        // Initiate the correct type of DMA transfer depending on what data is passed
        let tx_f = match data {
            ValueArray::Bit8(buf) => unsafe { dma.write(buf, T::regs().dhr8r(C::IDX).as_ptr() as *mut u8, tx_options) },
            ValueArray::Bit12Left(buf) => unsafe {
                dma.write(buf, T::regs().dhr12l(C::IDX).as_ptr() as *mut u16, tx_options)
            },
            ValueArray::Bit12Right(buf) => unsafe {
                dma.write(buf, T::regs().dhr12r(C::IDX).as_ptr() as *mut u16, tx_options)
            },
        };

        tx_f.await;

        T::regs().cr().modify(|w| {
            w.set_en(C::IDX, false);
            w.set_dmaen(C::IDX, false);
        });
    }
}

impl<'d, T: Instance, C: Channel> DacChannel<'d, T, C, Blocking> {
    /// Create a new `DacChannel` instance, consuming the underlying DAC peripheral.
    ///
    /// The channel is enabled on creation and begin to drive the output pin.
    /// Note that some methods, such as `set_trigger()` and `set_mode()`, will
    /// disable the channel; you must re-enable it with `enable()`.
    ///
    /// By default, triggering is disabled, but it can be enabled using
    /// [`DacChannel::set_trigger()`].
    pub fn new_blocking(peri: Peri<'d, T>, pin: Peri<'d, impl DacPin<T, C>>) -> Self {
        pin.set_as_analog();
        Self::new_inner(
            peri,
            None,
            #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
            Mode::NormalExternalBuffered,
        )
    }

    /// Create a new `DacChannel` instance where the external output pin is not used,
    /// so the DAC can only be used to generate internal signals.
    /// The GPIO pin is therefore available to be used for other functions.
    ///
    /// The channel is set to [`Mode::NormalInternalUnbuffered`] and enabled on creation.
    /// Note that some methods, such as `set_trigger()` and `set_mode()`, will disable the
    /// channel; you must re-enable it with `enable()`.
    ///
    /// By default, triggering is disabled, but it can be enabled using
    /// [`DacChannel::set_trigger()`].
    #[cfg(all(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7), not(any(stm32h56x, stm32h57x))))]
    pub fn new_internal_blocking(peri: Peri<'d, T>) -> Self {
        Self::new_inner(peri, None, Mode::NormalInternalUnbuffered)
    }
}

impl<'d, T: Instance, C: Channel, M: PeriMode> DacChannel<'d, T, C, M> {
    fn new_inner(
        _peri: Peri<'d, T>,
        dma: Option<ChannelAndRequest<'d>>,
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))] mode: Mode,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        let mut dac = Self {
            phantom: PhantomData,
            dma,
        };
        #[cfg(any(dac_v5, dac_v6, dac_v7))]
        dac.set_hfsel();
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
        dac.set_mode(mode);
        dac.enable();
        dac
    }

    /// Enable or disable this channel.
    pub fn set_enable(&mut self, on: bool) {
        critical_section::with(|_| {
            T::regs().cr().modify(|reg| {
                reg.set_en(C::IDX, on);
            });
        });
    }

    /// Enable this channel.
    pub fn enable(&mut self) {
        self.set_enable(true)
    }

    /// Disable this channel.
    pub fn disable(&mut self) {
        self.set_enable(false)
    }

    /// Set the trigger source for this channel.
    ///
    /// This method disables the channel, so you may need to re-enable afterwards.
    pub fn set_trigger(&mut self, source: TriggerSel) {
        critical_section::with(|_| {
            T::regs().cr().modify(|reg| {
                reg.set_en(C::IDX, false);
                reg.set_tsel(C::IDX, source as u8);
            });
        });
    }

    /// Enable or disable triggering for this channel.
    pub fn set_triggering(&mut self, on: bool) {
        critical_section::with(|_| {
            T::regs().cr().modify(|reg| {
                reg.set_ten(C::IDX, on);
            });
        });
    }

    /// Software trigger this channel.
    pub fn trigger(&mut self) {
        T::regs().swtrigr().write(|reg| {
            reg.set_swtrig(C::IDX, true);
        });
    }

    /// Set mode of this channel.
    ///
    /// This method disables the channel, so you may need to re-enable afterwards.
    #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
    pub fn set_mode(&mut self, mode: Mode) {
        critical_section::with(|_| {
            T::regs().cr().modify(|reg| {
                reg.set_en(C::IDX, false);
            });
            T::regs().mcr().modify(|reg| {
                reg.set_mode(C::IDX, mode.mode());
            });
        });
    }

    /// Write a new value to this channel.
    ///
    /// If triggering is not enabled, the new value is immediately output; otherwise,
    /// it will be output after the next trigger.
    pub fn set(&mut self, value: Value) {
        match value {
            Value::Bit8(v) => T::regs().dhr8r(C::IDX).write(|reg| reg.set_dhr(v)),
            Value::Bit12Left(v) => T::regs().dhr12l(C::IDX).write(|reg| reg.set_dhr(v)),
            Value::Bit12Right(v) => T::regs().dhr12r(C::IDX).write(|reg| reg.set_dhr(v)),
        }
    }

    /// Read the current output value of the DAC.
    pub fn read(&self) -> u16 {
        T::regs().dor(C::IDX).read().dor()
    }

    /// Set HFSEL as appropriate for the current peripheral clock frequency.
    #[cfg(dac_v5)]
    fn set_hfsel(&mut self) {
        if T::frequency() >= crate::time::mhz(80) {
            critical_section::with(|_| {
                T::regs().cr().modify(|reg| {
                    reg.set_hfsel(true);
                });
            });
        }
    }

    /// Set HFSEL as appropriate for the current peripheral clock frequency.
    #[cfg(any(dac_v6, dac_v7))]
    fn set_hfsel(&mut self) {
        if T::frequency() >= crate::time::mhz(160) {
            critical_section::with(|_| {
                T::regs().mcr().modify(|reg| {
                    reg.set_hfsel(0b10);
                });
            });
        } else if T::frequency() >= crate::time::mhz(80) {
            critical_section::with(|_| {
                T::regs().mcr().modify(|reg| {
                    reg.set_hfsel(0b01);
                });
            });
        }
    }
}

impl<'d, T: Instance, C: Channel, M: PeriMode> Drop for DacChannel<'d, T, C, M> {
    fn drop(&mut self) {
        rcc::disable::<T>();
    }
}

/// DAC driver.
///
/// Use this struct when you want to use both channels, either together or independently.
///
/// # Example
///
/// ```ignore
/// // Pins may need to be changed for your specific device.
/// let (dac_ch1, dac_ch2) = embassy_stm32::dac::Dac::new_blocking(p.DAC1, p.PA4, p.PA5).split();
/// ```
pub struct Dac<'d, T: Instance, M: PeriMode> {
    ch1: DacChannel<'d, T, Ch1, M>,
    ch2: DacChannel<'d, T, Ch2, M>,
}

impl<'d, T: Instance> Dac<'d, T, Async> {
    /// Create a new `Dac` instance, consuming the underlying DAC peripheral.
    ///
    /// This struct allows you to access both channels of the DAC, where available. You can either
    /// call `split()` to obtain separate `DacChannel`s, or use methods on `Dac` to use
    /// the two channels together.
    ///
    /// The channels are enabled on creation and begin to drive their output pins.
    /// Note that some methods, such as `set_trigger()` and `set_mode()`, will
    /// disable the channel; you must re-enable them with `enable()`.
    ///
    /// By default, triggering is disabled, but it can be enabled using the `set_trigger()`
    /// method on the underlying channels.
    pub fn new(
        peri: Peri<'d, T>,
        dma_ch1: Peri<'d, impl Dma<T, Ch1>>,
        dma_ch2: Peri<'d, impl Dma<T, Ch2>>,
        pin_ch1: Peri<'d, impl DacPin<T, Ch1> + crate::gpio::Pin>,
        pin_ch2: Peri<'d, impl DacPin<T, Ch2> + crate::gpio::Pin>,
    ) -> Self {
        pin_ch1.set_as_analog();
        pin_ch2.set_as_analog();
        Self::new_inner(
            peri,
            new_dma!(dma_ch1),
            new_dma!(dma_ch2),
            #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
            Mode::NormalExternalBuffered,
        )
    }

    /// Create a new `Dac` instance where the external output pins are not used,
    /// so the DAC can only be used to generate internal signals but the GPIO
    /// pins remain available for other functions.
    ///
    /// This struct allows you to access both channels of the DAC, where available. You can either
    /// call `split()` to obtain separate `DacChannel`s, or use methods on `Dac` to use the two
    /// channels together.
    ///
    /// The channels are set to [`Mode::NormalInternalUnbuffered`] and enabled on creation.
    /// Note that some methods, such as `set_trigger()` and `set_mode()`, will disable the
    /// channel; you must re-enable them with `enable()`.
    ///
    /// By default, triggering is disabled, but it can be enabled using the `set_trigger()`
    /// method on the underlying channels.
    #[cfg(all(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7), not(any(stm32h56x, stm32h57x))))]
    pub fn new_internal(
        peri: Peri<'d, T>,
        dma_ch1: Peri<'d, impl Dma<T, Ch1>>,
        dma_ch2: Peri<'d, impl Dma<T, Ch2>>,
    ) -> Self {
        Self::new_inner(
            peri,
            new_dma!(dma_ch1),
            new_dma!(dma_ch2),
            Mode::NormalInternalUnbuffered,
        )
    }
}

impl<'d, T: Instance> Dac<'d, T, Blocking> {
    /// Create a new `Dac` instance, consuming the underlying DAC peripheral.
    ///
    /// This struct allows you to access both channels of the DAC, where available. You can either
    /// call `split()` to obtain separate `DacChannel`s, or use methods on `Dac` to use
    /// the two channels together.
    ///
    /// The channels are enabled on creation and begin to drive their output pins.
    /// Note that some methods, such as `set_trigger()` and `set_mode()`, will
    /// disable the channel; you must re-enable them with `enable()`.
    ///
    /// By default, triggering is disabled, but it can be enabled using the `set_trigger()`
    /// method on the underlying channels.
    pub fn new_blocking(
        peri: Peri<'d, T>,
        pin_ch1: Peri<'d, impl DacPin<T, Ch1> + crate::gpio::Pin>,
        pin_ch2: Peri<'d, impl DacPin<T, Ch2> + crate::gpio::Pin>,
    ) -> Self {
        pin_ch1.set_as_analog();
        pin_ch2.set_as_analog();
        Self::new_inner(
            peri,
            None,
            None,
            #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
            Mode::NormalExternalBuffered,
        )
    }

    /// Create a new `Dac` instance where the external output pins are not used,
    /// so the DAC can only be used to generate internal signals but the GPIO
    /// pins remain available for other functions.
    ///
    /// This struct allows you to access both channels of the DAC, where available. You can either
    /// call `split()` to obtain separate `DacChannel`s, or use methods on `Dac` to use the two
    /// channels together.
    ///
    /// The channels are set to [`Mode::NormalInternalUnbuffered`] and enabled on creation.
    /// Note that some methods, such as `set_trigger()` and `set_mode()`, will disable the
    /// channel; you must re-enable them with `enable()`.
    ///
    /// By default, triggering is disabled, but it can be enabled using the `set_trigger()`
    /// method on the underlying channels.
    #[cfg(all(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7), not(any(stm32h56x, stm32h57x))))]
    pub fn new_internal(peri: Peri<'d, T>) -> Self {
        Self::new_inner(peri, None, None, Mode::NormalInternalUnbuffered)
    }
}

impl<'d, T: Instance, M: PeriMode> Dac<'d, T, M> {
    fn new_inner(
        _peri: Peri<'d, T>,
        dma_ch1: Option<ChannelAndRequest<'d>>,
        dma_ch2: Option<ChannelAndRequest<'d>>,
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))] mode: Mode,
    ) -> Self {
        // Enable twice to increment the DAC refcount for each channel.
        rcc::enable_and_reset::<T>();
        rcc::enable_and_reset::<T>();

        let mut ch1 = DacCh1 {
            phantom: PhantomData,
            dma: dma_ch1,
        };
        #[cfg(any(dac_v5, dac_v6, dac_v7))]
        ch1.set_hfsel();
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
        ch1.set_mode(mode);
        ch1.enable();

        let mut ch2 = DacCh2 {
            phantom: PhantomData,
            dma: dma_ch2,
        };
        #[cfg(any(dac_v5, dac_v6, dac_v7))]
        ch2.set_hfsel();
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
        ch2.set_mode(mode);
        ch2.enable();

        Self { ch1, ch2 }
    }

    /// Split this `Dac` into separate channels.
    ///
    /// You can access and move the channels around separately after splitting.
    pub fn split(self) -> (DacCh1<'d, T, M>, DacCh2<'d, T, M>) {
        (self.ch1, self.ch2)
    }

    /// Temporarily access channel 1.
    pub fn ch1(&mut self) -> &mut DacCh1<'d, T, M> {
        &mut self.ch1
    }

    /// Temporarily access channel 2.
    pub fn ch2(&mut self) -> &mut DacCh2<'d, T, M> {
        &mut self.ch2
    }

    /// Simultaneously update channels 1 and 2 with a new value.
    ///
    /// If triggering is not enabled, the new values are immediately output;
    /// otherwise, they will be output after the next trigger.
    pub fn set(&mut self, values: DualValue) {
        match values {
            DualValue::Bit8(v1, v2) => T::regs().dhr8rd().write(|reg| {
                reg.set_dhr(0, v1);
                reg.set_dhr(1, v2);
            }),
            DualValue::Bit12Left(v1, v2) => T::regs().dhr12ld().write(|reg| {
                reg.set_dhr(0, v1);
                reg.set_dhr(1, v2);
            }),
            DualValue::Bit12Right(v1, v2) => T::regs().dhr12rd().write(|reg| {
                reg.set_dhr(0, v1);
                reg.set_dhr(1, v2);
            }),
        }
    }
}

trait SealedInstance {
    fn regs() -> crate::pac::dac::Dac;
}

/// DAC instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral + 'static {}

/// Channel 1 marker type.
pub enum Ch1 {}
/// Channel 2 marker type.
pub enum Ch2 {}

trait SealedChannel {
    const IDX: usize;
}
/// DAC channel trait.
#[allow(private_bounds)]
pub trait Channel: SealedChannel {}

impl SealedChannel for Ch1 {
    const IDX: usize = 0;
}
impl SealedChannel for Ch2 {
    const IDX: usize = 1;
}
impl Channel for Ch1 {}
impl Channel for Ch2 {}

dma_trait!(Dma, Instance, Channel);
pin_trait!(DacPin, Instance, Channel);

foreach_peripheral!(
    (dac, $inst:ident) => {
        impl crate::dac::SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::dac::Dac {
                crate::pac::$inst
            }
        }

        impl crate::dac::Instance for peripherals::$inst {}
    };
);
