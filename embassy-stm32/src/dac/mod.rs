//! Digital to Analog Converter (DAC)
#![macro_use]

use core::marker::PhantomData;

use crate::dma::ChannelAndRequest;
use crate::mode::{Async, Blocking, Mode as PeriMode};
#[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
use crate::pac::dac;
use crate::pac::dac::Dac as Regs;
use crate::rcc::{self, RccInfo, RccPeripheral, SealedRccPeripheral};
use crate::time::Hertz;
use crate::{Peri, peripherals};

mod tsel;
use embassy_hal_internal::PeripheralType;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
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

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum ChannelEvent {
    Enable,
    Disable,
}

struct InnerState {
    channel_count: usize,
}

type SharedState = embassy_sync::blocking_mutex::Mutex<CriticalSectionRawMutex, core::cell::RefCell<InnerState>>;
struct State {
    state: SharedState,
}

impl State {
    /// Adjusts the channel count in response to a `ChannelEvent`, returning the updated value.
    pub fn adjust_channel_count(&self, event: ChannelEvent) -> usize {
        self.state.lock(|state| {
            {
                let mut mut_state = state.borrow_mut();
                match event {
                    ChannelEvent::Enable => {
                        mut_state.channel_count += 1;
                    }
                    ChannelEvent::Disable => {
                        mut_state.channel_count -= 1;
                    }
                };
            }
            state.borrow().channel_count
        })
    }
}
/// Driver for a single DAC channel.
///
/// If you want to use both channels, either together or independently,
/// create a [`Dac`] first and use it to access each channel.
pub struct DacChannel<'d, M: PeriMode> {
    phantom: PhantomData<&'d mut M>,
    #[allow(unused)]
    dma: Option<ChannelAndRequest<'d>>,
    info: &'static Info,
    state: &'static State,
    _ker_clk: Hertz,
    idx: usize,
}

impl<'d> DacChannel<'d, Async> {
    /// Create a new `DacChannel` instance, consuming the underlying DAC peripheral.
    ///
    /// The channel is enabled on creation and begin to drive the output pin.
    /// Note that some methods, such as `set_trigger()` and `set_mode()`, will
    /// disable the channel; you must re-enable it with `enable()`.
    ///
    /// By default, triggering is disabled, but it can be enabled using
    /// [`DacChannel::set_trigger()`].
    pub fn new<T: Instance, C: Channel, D: Dma<T, C>>(
        peri: Peri<'d, T>,
        dma: Peri<'d, D>,
        _irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'd,
        pin: Peri<'d, impl DacPin<T, C>>,
    ) -> Self {
        pin.set_as_analog();
        Self::new_inner::<T, C>(
            peri,
            new_dma!(dma, _irq),
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
    pub fn new_internal<T: Instance, C: Channel, D: Dma<T, C>>(
        peri: Peri<'d, T>,
        dma: Peri<'d, D>,
        _irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'd,
    ) -> Self {
        Self::new_inner::<T, C>(peri, new_dma!(dma, _irq), Mode::NormalInternalUnbuffered)
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
        self.info.regs.cr().modify(|w| {
            w.set_en(self.idx, true);
            w.set_dmaen(self.idx, true);
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
            ValueArray::Bit8(buf) => unsafe {
                dma.write(buf, self.info.regs.dhr8r(self.idx).as_ptr() as *mut u8, tx_options)
            },
            ValueArray::Bit12Left(buf) => unsafe {
                dma.write(buf, self.info.regs.dhr12l(self.idx).as_ptr() as *mut u16, tx_options)
            },
            ValueArray::Bit12Right(buf) => unsafe {
                dma.write(buf, self.info.regs.dhr12r(self.idx).as_ptr() as *mut u16, tx_options)
            },
        };

        tx_f.await;

        self.info.regs.cr().modify(|w| {
            w.set_en(self.idx, false);
            w.set_dmaen(self.idx, false);
        });
    }
}

impl<'d> DacChannel<'d, Blocking> {
    /// Create a new `DacChannel` instance, consuming the underlying DAC peripheral.
    ///
    /// The channel is enabled on creation and begin to drive the output pin.
    /// Note that some methods, such as `set_trigger()` and `set_mode()`, will
    /// disable the channel; you must re-enable it with `enable()`.
    ///
    /// By default, triggering is disabled, but it can be enabled using
    /// [`DacChannel::set_trigger()`].
    pub fn new_blocking<T: Instance, C: Channel>(peri: Peri<'d, T>, pin: Peri<'d, impl DacPin<T, C>>) -> Self {
        pin.set_as_analog();
        Self::new_inner::<T, C>(
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
    pub fn new_internal_blocking<T: Instance, C: Channel>(peri: Peri<'d, T>) -> Self {
        Self::new_inner::<T, C>(peri, None, Mode::NormalInternalUnbuffered)
    }
}

impl<'d, M: PeriMode> DacChannel<'d, M> {
    fn new_inner<T: Instance, C: Channel>(
        _peri: Peri<'d, T>,
        dma: Option<ChannelAndRequest<'d>>,
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))] mode: Mode,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        let mut dac = Self {
            phantom: PhantomData,
            info: T::info(),
            state: T::state(),
            _ker_clk: T::frequency(),
            idx: C::IDX,
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
            self.info.regs.cr().modify(|reg| {
                reg.set_en(self.idx, on);
            });
        });
        let event = if on {
            ChannelEvent::Enable
        } else {
            ChannelEvent::Disable
        };
        let channel_count = self.state.adjust_channel_count(event);
        // Disable the DAC only if no more channels are using it.
        if channel_count == 0 {
            self.info.rcc.disable();
        }
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
            self.info.regs.cr().modify(|reg| {
                reg.set_en(self.idx, false);
                reg.set_tsel(self.idx, source as u8);
            });
        });
    }

    /// Enable or disable triggering for this channel.
    pub fn set_triggering(&mut self, on: bool) {
        critical_section::with(|_| {
            self.info.regs.cr().modify(|reg| {
                reg.set_ten(self.idx, on);
            });
        });
    }

    /// Software trigger this channel.
    pub fn trigger(&mut self) {
        self.info.regs.swtrigr().write(|reg| {
            reg.set_swtrig(self.idx, true);
        });
    }

    /// Set mode of this channel.
    ///
    /// This method disables the channel, so you may need to re-enable afterwards.
    #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
    pub fn set_mode(&mut self, mode: Mode) {
        critical_section::with(|_| {
            self.info.regs.cr().modify(|reg| {
                reg.set_en(self.idx, false);
            });
            self.info.regs.mcr().modify(|reg| {
                reg.set_mode(self.idx, mode.mode());
            });
        });
    }

    /// Write a new value to this channel.
    ///
    /// If triggering is not enabled, the new value is immediately output; otherwise,
    /// it will be output after the next trigger.
    pub fn set(&mut self, value: Value) {
        match value {
            Value::Bit8(v) => self.info.regs.dhr8r(self.idx).write(|reg| reg.set_dhr(v)),
            Value::Bit12Left(v) => self.info.regs.dhr12l(self.idx).write(|reg| reg.set_dhr(v)),
            Value::Bit12Right(v) => self.info.regs.dhr12r(self.idx).write(|reg| reg.set_dhr(v)),
        }
    }

    /// Read the current output value of the DAC.
    pub fn read(&self) -> u16 {
        self.info.regs.dor(self.idx).read().dor()
    }

    /// Set HFSEL as appropriate for the current peripheral clock frequency.
    #[cfg(dac_v5)]
    fn set_hfsel(&mut self) {
        if self._ker_clk >= crate::time::mhz(80) {
            critical_section::with(|_| {
                self.info.regs.cr().modify(|reg| {
                    reg.set_hfsel(true);
                });
            });
        }
    }

    /// Set HFSEL as appropriate for the current peripheral clock frequency.
    #[cfg(any(dac_v6, dac_v7))]
    fn set_hfsel(&mut self) {
        if self._ker_clk >= crate::time::mhz(160) {
            critical_section::with(|_| {
                self.info.regs.mcr().modify(|reg| {
                    reg.set_hfsel(0b10);
                });
            });
        } else if self._ker_clk >= crate::time::mhz(80) {
            critical_section::with(|_| {
                self.info.regs.mcr().modify(|reg| {
                    reg.set_hfsel(0b01);
                });
            });
        }
    }
}

impl<'d, M: PeriMode> Drop for DacChannel<'d, M> {
    fn drop(&mut self) {
        self.disable();
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
pub struct Dac<'d, M: PeriMode> {
    info: &'static Info,
    ch1: DacChannel<'d, M>,
    ch2: DacChannel<'d, M>,
}

impl<'d> Dac<'d, Async> {
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
    pub fn new<T: Instance, D1: Dma<T, Ch1>, D2: Dma<T, Ch2>>(
        peri: Peri<'d, T>,
        dma_ch1: Peri<'d, D1>,
        dma_ch2: Peri<'d, D2>,
        _irq: impl crate::interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + crate::interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + 'd,
        pin_ch1: Peri<'d, impl DacPin<T, Ch1> + crate::gpio::Pin>,
        pin_ch2: Peri<'d, impl DacPin<T, Ch2> + crate::gpio::Pin>,
    ) -> Self {
        pin_ch1.set_as_analog();
        pin_ch2.set_as_analog();
        Self::new_inner(
            peri,
            new_dma!(dma_ch1, _irq),
            new_dma!(dma_ch2, _irq),
            #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
            Mode::NormalExternalBuffered,
        )
    }
    /// Create a new `Dac` instance with external output pins and unbuffered mode.
    ///
    /// This function consumes the underlying DAC peripheral and allows access to both channels.
    /// The channels are configured for external output with the buffer disabled.
    ///
    /// The channels are enabled on creation and begin to drive their output pins.
    /// Note that some methods, such as `set_trigger()` and `set_mode()`, will
    /// disable the channel; you must re-enable them with `enable()`.
    ///
    /// By default, triggering is disabled, but it can be enabled using the `set_trigger()`
    /// method on the underlying channels.
    ///
    /// # Arguments
    ///
    /// * `peri` - The DAC peripheral instance.
    /// * `dma_ch1` - The DMA channel for DAC channel 1.
    /// * `dma_ch2` - The DMA channel for DAC channel 2.
    /// * `_irq` - The interrupt binding for DMA channels 1 and 2.
    /// * `pin_ch1` - The GPIO pin for DAC channel 1 output.
    /// * `pin_ch2` - The GPIO pin for DAC channel 2 output.
    ///
    /// # Returns
    ///
    /// A new `Dac` instance in unbuffered mode.
    pub fn new_unbuffered<T: Instance, D1: Dma<T, Ch1>, D2: Dma<T, Ch2>>(
        peri: Peri<'d, T>,
        dma_ch1: Peri<'d, D1>,
        dma_ch2: Peri<'d, D2>,
        _irq: impl crate::interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + crate::interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + 'd,
        pin_ch1: Peri<'d, impl DacPin<T, Ch1> + crate::gpio::Pin>,
        pin_ch2: Peri<'d, impl DacPin<T, Ch2> + crate::gpio::Pin>,
    ) -> Self {
        pin_ch1.set_as_analog();
        pin_ch2.set_as_analog();
        Self::new_inner(
            peri,
            new_dma!(dma_ch1, _irq),
            new_dma!(dma_ch2, _irq),
            #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
            Mode::NormalExternalUnbuffered,
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
    pub fn new_internal<T: Instance, D1: Dma<T, Ch1>, D2: Dma<T, Ch2>>(
        peri: Peri<'d, T>,
        dma_ch1: Peri<'d, D1>,
        dma_ch2: Peri<'d, D2>,
        _irq: impl crate::interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + crate::interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + 'd,
    ) -> Self {
        Self::new_inner(
            peri,
            new_dma!(dma_ch1, _irq),
            new_dma!(dma_ch2, _irq),
            Mode::NormalInternalUnbuffered,
        )
    }
}

impl<'d> Dac<'d, Blocking> {
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
    pub fn new_blocking<T: Instance>(
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
    pub fn new_internal<T: Instance>(peri: Peri<'d, T>) -> Self {
        Self::new_inner(peri, None, None, Mode::NormalInternalUnbuffered)
    }
}

impl<'d, M: PeriMode> Dac<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        dma_ch1: Option<ChannelAndRequest<'d>>,
        dma_ch2: Option<ChannelAndRequest<'d>>,
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))] mode: Mode,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        let mut ch1 = DacChannel {
            phantom: PhantomData,
            info: T::info(),
            state: T::state(),
            _ker_clk: T::frequency(),
            idx: Ch1::IDX,
            dma: dma_ch1,
        };
        #[cfg(any(dac_v5, dac_v6, dac_v7))]
        ch1.set_hfsel();
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
        ch1.set_mode(mode);
        ch1.enable();

        let mut ch2 = DacChannel {
            phantom: PhantomData,
            info: T::info(),
            state: T::state(),
            _ker_clk: T::frequency(),
            idx: Ch2::IDX,
            dma: dma_ch2,
        };
        #[cfg(any(dac_v5, dac_v6, dac_v7))]
        ch2.set_hfsel();
        #[cfg(any(dac_v3, dac_v4, dac_v5, dac_v6, dac_v7))]
        ch2.set_mode(mode);
        ch2.enable();

        Self {
            info: T::info(),
            ch1,
            ch2,
        }
    }

    /// Split this `Dac` into separate channels.
    ///
    /// You can access and move the channels around separately after splitting.
    pub fn split(self) -> (DacChannel<'d, M>, DacChannel<'d, M>) {
        (self.ch1, self.ch2)
    }

    /// Temporarily access channel 1.
    pub fn ch1(&mut self) -> &mut DacChannel<'d, M> {
        &mut self.ch1
    }

    /// Temporarily access channel 2.
    pub fn ch2(&mut self) -> &mut DacChannel<'d, M> {
        &mut self.ch2
    }

    /// Simultaneously update channels 1 and 2 with a new value.
    ///
    /// If triggering is not enabled, the new values are immediately output;
    /// otherwise, they will be output after the next trigger.
    pub fn set(&mut self, values: DualValue) {
        match values {
            DualValue::Bit8(v1, v2) => self.info.regs.dhr8rd().write(|reg| {
                reg.set_dhr(0, v1);
                reg.set_dhr(1, v2);
            }),
            DualValue::Bit12Left(v1, v2) => self.info.regs.dhr12ld().write(|reg| {
                reg.set_dhr(0, v1);
                reg.set_dhr(1, v2);
            }),
            DualValue::Bit12Right(v1, v2) => self.info.regs.dhr12rd().write(|reg| {
                reg.set_dhr(0, v1);
                reg.set_dhr(1, v2);
            }),
        }
    }
}

trait SealedInstance {
    fn info() -> &'static Info;

    fn state() -> &'static State {
        static STATE: State = State {
            state: embassy_sync::blocking_mutex::Mutex::new(core::cell::RefCell::new(InnerState { channel_count: 0 })),
        };
        &STATE
    }
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

struct Info {
    regs: Regs,
    rcc: RccInfo,
}

foreach_peripheral!(
    (dac, $inst:ident) => {
        impl crate::dac::SealedInstance for peripherals::$inst {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: unsafe { Regs::from_ptr(crate::pac::$inst.as_ptr()) },
                    rcc: crate::peripherals::$inst::RCC_INFO,
                };
                &INFO
            }
        }

        impl crate::dac::Instance for peripherals::$inst {}
    };
);
