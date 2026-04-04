//! Analog to Digital Converter (ADC)

#![macro_use]
#![allow(missing_docs)] // TODO
#![cfg_attr(adc_f3v3, allow(unused))]

#[cfg(not(any(adc_f3v3, adc_wba)))]
#[cfg_attr(adc_f1, path = "f1.rs")]
#[cfg_attr(adc_f3v1, path = "f3.rs")]
#[cfg_attr(adc_f3v2, path = "f3_v1_1.rs")]
#[cfg_attr(adc_v1, path = "v1.rs")]
#[cfg_attr(adc_l0, path = "v1.rs")]
#[cfg_attr(adc_v2, path = "v2.rs")]
#[cfg_attr(any(adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0), path = "v3.rs")]
#[cfg_attr(any(adc_v4, adc_u5, adc_u3), path = "v4.rs")]
#[cfg_attr(adc_g4, path = "g4.rs")]
#[cfg_attr(adc_c0, path = "c0.rs")]
mod _version;

#[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0, adc_wba, adc_c0))]
mod ringbuffered;

#[cfg(any(
    adc_g4, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u5, adc_u3, adc_wba, adc_c0, adc_v2
))]
mod configured_sequence;

use core::marker::PhantomData;

#[allow(unused)]
#[cfg(not(any(adc_f3v3, adc_wba)))]
pub use _version::*;
#[cfg(any(
    adc_g4, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u5, adc_u3, adc_wba, adc_c0, adc_v2
))]
pub use configured_sequence::ConfiguredSequence;
#[allow(unused)]
use embassy_hal_internal::PeripheralType;
#[cfg(any(adc_f1, adc_f3v1, adc_v1, adc_l0, adc_f3v2))]
use embassy_sync::waitqueue::AtomicWaker;
#[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0, adc_wba, adc_c0))]
pub use ringbuffered::RingBufferedAdc;

#[cfg(adc_u5)]
use crate::pac::adc::vals::Adc4SampleTime;
#[cfg(adc_wba)]
use crate::pac::adc::vals::SampleTime as Adc4SampleTime;

#[cfg(any(adc_u5, adc_wba))]
#[path = "adc4.rs"]
pub mod adc4;

#[allow(unused)]
pub(self) use crate::block_for_us as blocking_delay_us;
pub use crate::pac::adc::vals;
#[cfg(any(adc_v2, adc_g4, adc_g0, adc_c0))]
pub use crate::pac::adc::vals::Exten;
#[cfg(not(any(adc_f1, adc_f3v3)))]
pub use crate::pac::adc::vals::Res as Resolution;
pub use crate::pac::adc::vals::SampleTime;
use crate::peripherals;

dma_trait!(RxDma, Instance);

#[cfg(not(any(adc_v2, adc_g4, adc_g0, adc_c0)))]
/// Trigger edge stub.
pub struct Exten;

pub struct RegularAdcTrigger<T: Instance> {
    _trigger: u8,
    _edge: Exten,
    _typ: PhantomData<T>,
}

impl<T: Instance> RegularAdcTrigger<T> {
    pub fn from(trigger: impl RegularTrigger<T>, edge: Exten) -> Option<Self> {
        Some(Self {
            _trigger: trigger.signal(),
            _edge: edge,
            _typ: PhantomData,
        })
    }
}

pub struct InjectedAdcTrigger<T: Instance> {
    _trigger: u8,
    _edge: Exten,
    _typ: PhantomData<T>,
}

impl<T: Instance> InjectedAdcTrigger<T> {
    pub fn from(trigger: impl InjectedTrigger<T>, edge: Exten) -> Self {
        Self {
            _trigger: trigger.signal(),
            _edge: edge,
            _typ: PhantomData,
        }
    }
}

/// Analog to Digital driver.
pub struct Adc<'d, T: Instance> {
    #[allow(unused)]
    adc: crate::Peri<'d, T>,
}

#[cfg(any(adc_f1, adc_f3v1, adc_v1, adc_l0, adc_f3v2))]
pub struct State {
    pub waker: AtomicWaker,
}

#[cfg(any(adc_f1, adc_f3v1, adc_v1, adc_l0, adc_f3v2))]
impl State {
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

#[cfg(any(adc_f1, adc_f3v1, adc_f3v2, adc_v1, adc_l0))]
trait_set::trait_set! {
    pub trait DefaultInstance = Instance;
}

#[cfg(any(
    adc_v2, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u3, adc_u5, adc_g4, adc_c0
))]
trait_set::trait_set! {
    pub trait DefaultInstance = Instance<Regs = crate::pac::adc::Adc>;
}

#[cfg(adc_wba)]
trait_set::trait_set! {
    pub trait DefaultInstance = Instance<Regs = crate::pac::adc::Adc4>;
}

pub trait BasicAdcRegs {
    type SampleTime: Copy;
}

#[cfg(any(
    adc_v2, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u3, adc_u5, adc_wba, adc_g4, adc_c0
))]
trait AdcRegs: BasicAdcRegs {
    fn enable(&self);
    fn start(&self);
    fn stop(&self);
    fn convert(&self);
    fn configure_dma(&self, conversion_mode: ConversionMode);
    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), Self::SampleTime)>);
    fn data(&self) -> *mut u16;
}

#[cfg(any(adc_v2, adc_g4))]
trait SealedInjectedAdcRegs: AdcRegs {
    fn configure_injected_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), Self::SampleTime)>);
    fn configure_injected_trigger(&self, trigger: (u8, Exten), interrupt: bool);
    fn start_injected(&self);
    fn stop_injected(&self);
    fn read_injected(&self, data: &mut [u16]);
}

#[cfg(any(adc_v2, adc_g4))]
#[allow(private_bounds)]
pub trait InjectedAdcRegs: SealedInjectedAdcRegs {}

#[cfg(any(adc_v2, adc_g4))]
impl<T: SealedInjectedAdcRegs> InjectedAdcRegs for T {}

#[allow(private_bounds)]
pub trait BasicInstance {
    #[cfg(any(
        adc_v2, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u3, adc_u5, adc_wba, adc_g4, adc_c0
    ))]
    type Regs: AdcRegs;
}

trait SealedInstance: BasicInstance {
    #[cfg(any(adc_f1, adc_f3v1, adc_f3v2, adc_v1, adc_l0))]
    fn regs() -> crate::pac::adc::Adc;
    #[cfg(any(
        adc_v2, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u3, adc_u5, adc_wba, adc_g4, adc_c0
    ))]
    fn regs() -> Self::Regs;
    #[cfg(not(any(adc_f1, adc_v1, adc_l0, adc_f3v3, adc_f3v2, adc_g0)))]
    #[allow(unused)]
    fn common_regs() -> crate::pac::adccommon::AdcCommon;
    #[cfg(any(adc_f1, adc_f3v1, adc_v1, adc_l0, adc_f3v2))]
    fn state() -> &'static State;
}

pub(crate) trait SealedAdcChannel<T> {
    #[cfg(any(adc_v1, adc_c0, adc_l0, adc_v2, adc_g4, adc_v3, adc_v4, adc_u5, adc_u3, adc_wba))]
    fn setup(&mut self) {}

    #[allow(unused)]
    fn channel(&self) -> u8;

    #[allow(unused)]
    fn is_differential(&self) -> bool {
        false
    }
}

#[cfg(any(adc_c0, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u5, adc_u3))]
/// Number of samples used for averaging.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Averaging {
    Disabled,
    Samples2,
    Samples4,
    Samples8,
    Samples16,
    Samples32,
    Samples64,
    Samples128,
    Samples256,
    #[cfg(any(adc_c0, adc_v4, adc_u5, adc_u3))]
    Samples512,
    #[cfg(any(adc_c0, adc_v4, adc_u5, adc_u3))]
    Samples1024,
}

#[cfg(any(
    adc_v2, adc_g4, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u5, adc_u3, adc_wba, adc_c0
))]
pub(crate) enum ConversionMode {
    // Should match the cfg on "read" below
    #[cfg(any(
        adc_g4, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u5, adc_u3, adc_wba, adc_c0, adc_v2
    ))]
    Singular,
    // Should match the cfg on "into_ring_buffered" below
    #[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0, adc_wba, adc_c0))]
    Repeated(Option<(u8, Exten)>),
    // Should match the cfg on "configured_sequence" below
    #[cfg(any(
        adc_g4, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u5, adc_u3, adc_wba, adc_c0, adc_v2
    ))]
    ConfiguredSequence,
}

impl<'d, T: Instance> Adc<'d, T> {
    #[cfg(any(
        adc_v2, adc_g4, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_u3, adc_u5, adc_v3, adc_v4, adc_wba, adc_c0
    ))]
    /// Read an ADC pin.
    pub fn blocking_read(
        &mut self,
        channel: &mut impl AdcChannel<T>,
        sample_time: <T::Regs as BasicAdcRegs>::SampleTime,
    ) -> u16 {
        #[cfg(any(adc_v1, adc_c0, adc_l0, adc_v2, adc_g4, adc_v3, adc_v4, adc_u3, adc_u5, adc_wba))]
        channel.setup();

        T::regs().stop();
        T::regs().configure_sequence([((channel.channel(), channel.is_differential()), sample_time)].into_iter());

        T::regs().enable();
        T::regs().convert();

        unsafe { core::ptr::read_volatile(T::regs().data()) }
    }

    #[cfg(any(
        adc_g4, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u5, adc_u3, adc_wba, adc_c0, adc_v2
    ))]
    /// Read one or multiple ADC regular channels using DMA.
    ///
    /// `readings` must have a length that is a multiple of the length of the `sequence` iterator.
    ///
    /// Example
    /// ```rust,ignore
    /// use embassy_stm32::adc::{Adc, AdcChannel}
    ///
    /// let mut adc = Adc::new(p.ADC1);
    /// let mut adc_pin0 = p.PA0.into();
    /// let mut adc_pin1 = p.PA1.into();
    /// let mut measurements = [0u16; 2];
    ///
    /// adc.read(
    ///     p.DMA1_CH2.reborrow(),
    ///     Irqs,
    ///     [
    ///         (&mut *adc_pin0, SampleTime::CYCLES160_5),
    ///         (&mut *adc_pin1, SampleTime::CYCLES160_5),
    ///     ]
    ///     .into_iter(),
    ///     &mut measurements,
    /// )
    /// .await;
    /// defmt::info!("measurements: {}", measurements);
    /// ```
    ///
    /// Note: This is not very efficient as the ADC needs to be reconfigured for each read. Use
    /// `into_ring_buffered`, `into_ring_buffered_and_injected`
    ///
    /// Note: Depending on hardware limitations, this method may require channels to be passed
    /// in order or require the sequence to have the same sample time for all channnels, depending
    /// on the number and properties of the channels in the sequence. This method will panic if
    /// the hardware cannot deliver the requested configuration.
    pub async fn read<'a, 'b: 'a, D: RxDma<T>>(
        &mut self,
        rx_dma: embassy_hal_internal::Peri<'a, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
        sequence: impl ExactSizeIterator<Item = (&'a mut AnyAdcChannel<'b, T>, <T::Regs as BasicAdcRegs>::SampleTime)>,
        readings: &mut [u16],
    ) {
        let _scoped_wake_guard = <T as crate::rcc::SealedRccPeripheral>::RCC_INFO.wake_guard();

        assert!(sequence.len() != 0, "Asynchronous read sequence cannot be empty");
        assert!(
            readings.len() % sequence.len() == 0,
            "Readings length must be a multiple of sequence length"
        );
        assert!(
            sequence.len() <= 16,
            "Asynchronous read sequence cannot be more than 16 in length"
        );

        // Ensure no conversions are ongoing
        T::regs().stop();
        T::regs().configure_sequence(
            sequence.map(|(channel, sample_time)| ((channel.channel, channel.is_differential), sample_time)),
        );

        T::regs().enable();
        T::regs().configure_dma(ConversionMode::Singular);

        let request = rx_dma.request();
        let mut dma_channel = crate::dma::Channel::new(rx_dma, irq);
        let transfer = unsafe { dma_channel.read(request, T::regs().data(), readings, Default::default()) };

        T::regs().start();

        // Wait for conversion sequence to finish.
        transfer.await;

        // Ensure conversions are finished.
        T::regs().stop();
    }

    #[cfg(any(
        adc_g4, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u5, adc_u3, adc_wba, adc_c0, adc_v2
    ))]
    /// Configure an ADC channel sequence once and return a [`ConfiguredSequence`] for repeated
    /// DMA reads without reprogramming the sequence each time.
    ///
    /// Use [`Adc::read`] instead if you only need a single one-shot transfer.
    ///
    /// # Parameters
    /// - `rx_dma`: The DMA channel to use for transfers.
    /// - `sequence`: Iterator of channels and sample times. Maximum 16 entries.
    /// - `buf`: Output buffer. Must have at least as many entries as `sequence`.
    ///
    /// # Returns
    /// A [`ConfiguredSequence`] whose [`read`](ConfiguredSequence::read) method triggers one
    /// DMA conversion of the pre-configured sequence per call.
    ///
    /// # Notes
    /// - The channel sequence is programmed into the ADC sequence registers once here and
    ///   remains fixed for the lifetime of the returned [`ConfiguredSequence`].
    pub fn configured_sequence<'adc, 'ch, D: RxDma<T>>(
        &'adc mut self,
        rx_dma: crate::Peri<'adc, D>,
        sequence: impl ExactSizeIterator<Item = (&'adc mut AnyAdcChannel<'ch, T>, <T::Regs as BasicAdcRegs>::SampleTime)>,
        buf: &'adc mut [u16],
    ) -> ConfiguredSequence<'adc, 'd, T, D>
    where
        'ch: 'adc,
    {
        assert!(sequence.len() != 0, "Sequence cannot be empty");
        assert!(sequence.len() <= 16, "Sequence cannot be more than 16 in length");
        assert!(
            buf.len() >= sequence.len(),
            "Buffer must have at least as many entries as the sequence"
        );

        // Ensure no conversions are ongoing
        T::regs().stop();
        T::regs().configure_sequence(
            sequence.map(|(channel, sample_time)| ((channel.channel, channel.is_differential), sample_time)),
        );

        T::regs().enable();

        // Configure DMA once, reused across all subsequent read() calls.
        T::regs().configure_dma(ConversionMode::ConfiguredSequence);

        ConfiguredSequence::new(self, rx_dma, buf)
    }

    #[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0, adc_wba, adc_c0))]
    /// Configures the ADC to use a DMA ring buffer for continuous data acquisition.
    ///
    /// Use the [`Self::read`] method to retrieve measurements from the DMA ring buffer. The read buffer
    /// should be exactly half the size of `dma_buf`. When using triggered mode, it is recommended
    /// to configure `dma_buf` as a double buffer so that one half can be read while the other half
    /// is being filled by the DMA, preventing data loss. The trigger period of the ADC effectively
    /// defines the period at which the buffer should be read.
    ///
    /// If continous conversion mode is selected, the provided `dma_buf` must be large enough to prevent
    /// DMA buffer overruns. Its length should be a multiple of the number of ADC channels being measured.
    /// For example, if 3 channels are measured and you want to store 40 samples per channel,
    /// the buffer length should be `3 * 40 = 120`.
    ///
    /// # Parameters
    /// - `dma`: The DMA peripheral used to transfer ADC data into the buffer.
    /// - `dma_buf`: The buffer where DMA stores ADC samples.
    /// - `regular_sequence`: Sequence of channels and sample times for regular ADC conversions.
    /// - `regular_conversion_mode`: Mode for regular conversions (continuous or triggered).
    ///
    /// # Returns
    /// A `RingBufferedAdc<'a, T>` instance configured for continuous DMA-based sampling.
    ///
    /// Note: Depending on hardware limitations, this method may require channels to be passed
    /// in order or require the sequence to have the same sample time for all channnels, depending
    /// on the number and properties of the channels in the sequence. This method will panic if
    /// the hardware cannot deliver the requested configuration.
    pub fn into_ring_buffered<'a, 'b, D: RxDma<T>>(
        self,
        dma: embassy_hal_internal::Peri<'a, D>,
        dma_buf: &'a mut [u16],
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
        sequence: impl ExactSizeIterator<Item = (AnyAdcChannel<'b, T>, <T::Regs as BasicAdcRegs>::SampleTime)>,
        trigger: Option<RegularAdcTrigger<T>>,
    ) -> RingBufferedAdc<'a, T> {
        let sequence_len = sequence.len();
        assert!(!dma_buf.is_empty() && dma_buf.len() <= 0xFFFF);
        assert!(sequence_len != 0, "Asynchronous read sequence cannot be empty");
        assert!(
            sequence_len <= 16,
            "Asynchronous read sequence cannot be more than 16 in length"
        );
        assert!(
            dma_buf.len() % sequence_len == 0,
            "DMA buffer length must be a multiple of the scan sequence length"
        );
        // Ensure no conversions are ongoing
        T::regs().stop();
        T::regs().configure_sequence(
            sequence.map(|(channel, sample_time)| ((channel.channel, channel.is_differential), sample_time)),
        );

        T::regs().enable();
        T::regs().configure_dma(ConversionMode::Repeated(trigger.map(|t| (t._trigger, t._edge))));

        core::mem::forget(self);

        RingBufferedAdc::new(dma, irq, dma_buf, sequence_len)
    }
}

#[cfg(any(adc_v2, adc_g4))]
impl<'d, T: Instance<Regs: InjectedAdcRegs>> Adc<'d, T> {
    #[cfg(any(adc_v2, adc_g4))]
    /// Configures the ADC for injected conversions.
    ///
    /// Injected conversions are separate from the regular conversion sequence and are typically
    /// triggered by software or an external event. This method sets up a fixed-length sequence of
    /// injected channels with specified sample times, the trigger source, and whether the end-of-sequence
    /// interrupt should be enabled.
    ///
    /// # Parameters
    /// - `sequence`: An array of tuples containing the ADC channels and their sample times. The length
    ///   `N` determines the number of injected ranks to configure (maximum 4 for STM32).
    /// - `trigger`: The trigger source that starts the injected conversion sequence.
    /// - `interrupt`: If `true`, enables the end-of-sequence (JEOS) interrupt for injected conversions.
    ///
    /// # Returns
    /// An `InjectedAdc<T, N>` instance that represents the configured injected sequence. The returned
    /// type encodes the sequence length `N` in its type, ensuring that reads return exactly `N` samples.
    ///
    /// # Panics
    /// This function will panic if:
    /// - `sequence` is empty.
    /// - `sequence` length exceeds the maximum number of injected ranks (`NR_INJECTED_RANKS`).
    ///
    /// # Notes
    /// - Injected conversions can run independently of regular ADC conversions.
    /// - The order of channels in `sequence` determines the rank order in the injected sequence.
    /// - Accessing samples beyond `N` will result in a panic; use the returned type
    ///   `InjectedAdc<T, N>` to enforce bounds at compile time.
    pub fn setup_injected_conversions<'a, const N: usize>(
        self,
        sequence: [(AnyAdcChannel<'a, T>, <T::Regs as BasicAdcRegs>::SampleTime); N],
        trigger: InjectedAdcTrigger<T>,
        interrupt: bool,
    ) -> InjectedAdc<'a, T, N> {
        assert!(N != 0, "Read sequence cannot be empty");
        assert!(
            N <= NR_INJECTED_RANKS,
            "Read sequence cannot be more than {} in length",
            NR_INJECTED_RANKS
        );

        // TODO: move enable after configure_sequence?
        T::regs().enable();
        T::regs().configure_injected_sequence(
            sequence
                .iter()
                .map(|(channel, sample_time)| ((channel.channel, channel.is_differential), *sample_time)),
        );

        T::regs().configure_injected_trigger((trigger._trigger, trigger._edge), interrupt);
        T::regs().start_injected();

        core::mem::forget(self);

        InjectedAdc::new(sequence) // InjectedAdc<'a, T, N> now borrows the channels
    }

    #[cfg(any(adc_v2, adc_g4))]
    /// Configures ADC for both regular conversions with a ring-buffered DMA and injected conversions.
    ///
    /// # Parameters
    /// - `dma`: The DMA peripheral to use for the ring-buffered ADC transfers.
    /// - `dma_buf`: The buffer to store DMA-transferred samples for regular conversions.
    /// - `regular_sequence`: The sequence of channels and their sample times for regular conversions.
    /// - `regular_conversion_mode`: The mode for regular conversions (e.g., continuous or triggered).
    /// - `injected_sequence`: An array of channels and sample times for injected conversions (length `N`).
    /// - `injected_trigger`: The trigger source for injected conversions.
    /// - `injected_interrupt`: Whether to enable the end-of-sequence interrupt for injected conversions.
    ///
    /// Injected conversions are typically used with interrupts. If ADC1 and ADC2 are used in dual mode,
    /// it is recommended to enable interrupts only for the ADC whose sequence takes the longest to complete.
    ///
    /// # Returns
    /// A tuple containing:
    /// 1. `RingBufferedAdc<'a, T>` — the configured ADC for regular conversions using DMA.
    /// 2. `InjectedAdc<T, N>` — the configured ADC for injected conversions.
    ///
    /// # Safety
    /// This function is `unsafe` because it clones the ADC peripheral handle unchecked. Both the
    /// `RingBufferedAdc` and `InjectedAdc` take ownership of the handle and drop it independently.
    /// Ensure no other code concurrently accesses the same ADC instance in a conflicting way.
    pub fn into_ring_buffered_and_injected<'a, 'b, const N: usize, D: RxDma<T>>(
        self,
        dma: embassy_hal_internal::Peri<'a, D>,
        dma_buf: &'a mut [u16],
        _irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
        regular_sequence: impl ExactSizeIterator<Item = (AnyAdcChannel<'b, T>, <T::Regs as BasicAdcRegs>::SampleTime)>,
        regular_trigger: Option<RegularAdcTrigger<T>>,
        injected_sequence: [(AnyAdcChannel<'b, T>, <T::Regs as BasicAdcRegs>::SampleTime); N],
        injected_trigger: InjectedAdcTrigger<T>,
        injected_interrupt: bool,
    ) -> (RingBufferedAdc<'a, T>, InjectedAdc<'b, T, N>) {
        unsafe {
            (
                Self {
                    adc: self.adc.clone_unchecked(),
                }
                .into_ring_buffered(dma, dma_buf, _irq, regular_sequence, regular_trigger),
                Self {
                    adc: self.adc.clone_unchecked(),
                }
                .setup_injected_conversions(injected_sequence, injected_trigger, injected_interrupt),
            )
        }
    }
}

pub(self) trait SpecialChannel {}

/// Implemented for ADCs that have a special channel
trait SealedSpecialConverter<T: SpecialChannel + Sized> {
    const CHANNEL: u8;
}

#[allow(private_bounds)]
pub trait SpecialConverter<T: SpecialChannel + Sized>: SealedSpecialConverter<T> {}

impl<C: SpecialChannel + Sized, T: SealedSpecialConverter<C>> SpecialConverter<C> for T {}

impl<C: SpecialChannel, T: Instance + SealedSpecialConverter<C>> AdcChannel<T> for C {}
impl<C: SpecialChannel, T: Instance + SealedSpecialConverter<C>> SealedAdcChannel<T> for C {
    fn channel(&self) -> u8 {
        T::CHANNEL
    }
}

pub struct VrefInt;
impl SpecialChannel for VrefInt {}

impl VrefInt {
    #[cfg(any(
        stm32f0,
        stm32f3,
        stm32f7,
        stm32g0,
        stm32g4,
        stm32l0,
        stm32l1,
        stm32l4,
        stm32l4_plus,
        stm32l5,
        stm32n6,
        stm32l5,
        stm32wb,
        stm32wl
    ))]
    /// The value that vref would be if vdda was at the factory calibration voltage `VREF_CALIB_MV`.
    pub fn calibrated_value(&self) -> u16 {
        crate::pac::VREFINTCAL.data().read()
    }
}

/// Internal temperature channel.
pub struct Temperature;
impl SpecialChannel for Temperature {}

/// Internal battery voltage channel.
pub struct Vbat;
impl SpecialChannel for Vbat {}

/// Vcore channel.
pub struct Vcore;
impl SpecialChannel for Vcore {}

/// Internal dac channel.
pub struct Dac;
impl SpecialChannel for Dac {}

/// ADC instance.
#[cfg(not(any(
    adc_f1, adc_v1, adc_l0, adc_v2, adc_v3, adc_v4, adc_g4, adc_f3v1, adc_f3v2, adc_g0, adc_u0, adc_h5, adc_h7rs,
    adc_u5, adc_u3, adc_c0, adc_wba,
)))]
#[allow(private_bounds)]
pub trait Instance: SealedInstance + crate::PeripheralType {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}
/// ADC instance.
#[cfg(any(
    adc_f1, adc_v1, adc_l0, adc_v2, adc_v3, adc_v4, adc_g4, adc_f3v1, adc_f3v2, adc_g0, adc_u0, adc_h5, adc_h7rs,
    adc_u5, adc_u3, adc_c0, adc_wba,
))]
#[allow(private_bounds)]
pub trait Instance: SealedInstance + crate::PeripheralType + crate::rcc::RccPeripheral {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// ADC channel.
#[allow(private_bounds)]
pub trait AdcChannel<T>: SealedAdcChannel<T> + Sized {
    #[allow(unused_mut)]
    fn degrade_adc<'a>(&'a mut self) -> AnyAdcChannel<'a, T> {
        #[cfg(any(adc_v1, adc_l0, adc_v2, adc_g4, adc_v3, adc_v4, adc_u3, adc_u5, adc_wba))]
        self.setup();

        AnyAdcChannel {
            channel: self.channel(),
            is_differential: self.is_differential(),
            _phantom: PhantomData,
        }
    }
}

/// A type-erased channel for a given ADC instance.
///
/// This is useful in scenarios where you need the ADC channels to have the same type, such as
/// storing them in an array.
pub struct AnyAdcChannel<'a, T> {
    channel: u8,
    is_differential: bool,
    _phantom: PhantomData<&'a mut T>,
}
impl<T: Instance> AdcChannel<T> for AnyAdcChannel<'_, T> {}
impl<T: Instance> SealedAdcChannel<T> for AnyAdcChannel<'_, T> {
    fn channel(&self) -> u8 {
        self.channel
    }

    fn is_differential(&self) -> bool {
        self.is_differential
    }
}

impl<T> AnyAdcChannel<'_, T> {
    #[allow(unused)]
    pub fn get_hw_channel(&self) -> u8 {
        self.channel
    }
}

#[cfg(not(adc_wba))]
impl BasicAdcRegs for crate::pac::adc::Adc {
    type SampleTime = SampleTime;
}

#[cfg(any(adc_wba, adc_u5))]
impl BasicAdcRegs for crate::pac::adc::Adc4 {
    type SampleTime = Adc4SampleTime;
}

trigger_trait!(RegularTrigger, Instance);
trigger_trait!(InjectedTrigger, Instance);

#[cfg(adc_wba)]
foreach_adc!(
    (ADC4, $common_inst:ident, $clock:ident) => {
        impl crate::adc::BasicInstance for peripherals::ADC4 {
            type Regs = crate::pac::adc::Adc4;
        }

        impl crate::adc::SealedInstance for peripherals::ADC4 {
            fn regs() -> Self::Regs {
                crate::pac::ADC4
            }

            fn common_regs() -> crate::pac::adccommon::AdcCommon {
                return crate::pac::$common_inst
            }
        }

        impl crate::adc::Instance for peripherals::ADC4 {
            type Interrupt = crate::_generated::peripheral_interrupts::ADC4::GLOBAL;
        }
    };

    ($inst:ident, $common_inst:ident, $clock:ident) => {
        impl crate::adc::SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::adc::Adc {
                crate::pac::$inst
            }

            fn common_regs() -> crate::pac::adccommon::AdcCommon {
                return crate::pac::$common_inst
            }
        }

        impl crate::adc::Instance for peripherals::$inst {
            type Interrupt = crate::_generated::peripheral_interrupts::$inst::GLOBAL;
        }
    };
);

#[cfg(adc_u5)]
foreach_adc!(
    (ADC4, $common_inst:ident, $clock:ident) => {
        impl crate::adc::BasicInstance for peripherals::ADC4 {
            type Regs = crate::pac::adc::Adc4;
        }

        impl crate::adc::SealedInstance for peripherals::ADC4 {
            fn regs() -> Self::Regs {
                crate::pac::ADC4
            }

            fn common_regs() -> crate::pac::adccommon::AdcCommon {
                return crate::pac::$common_inst
            }
        }

        impl crate::adc::Instance for peripherals::ADC4 {
            type Interrupt = crate::_generated::peripheral_interrupts::ADC4::GLOBAL;
        }
    };

    ($inst:ident, $common_inst:ident, $clock:ident) => {
        impl crate::adc::BasicInstance for peripherals::$inst {
            type Regs = crate::pac::adc::Adc;
        }

        impl crate::adc::SealedInstance for peripherals::$inst {
            fn regs() -> Self::Regs {
                crate::pac::$inst
            }

            fn common_regs() -> crate::pac::adccommon::AdcCommon {
                return crate::pac::$common_inst
            }
        }

        impl crate::adc::Instance for peripherals::$inst {
            type Interrupt = crate::_generated::peripheral_interrupts::$inst::GLOBAL;
        }
    };
);

#[cfg(not(any(adc_u5, adc_wba)))]
foreach_adc!(
    ($inst:ident, $common_inst:ident, $clock:ident) => {
        impl crate::adc::BasicInstance for peripherals::$inst {
            #[cfg(any(
                adc_v2, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u3, adc_u5, adc_wba, adc_g4, adc_c0
            ))]
            type Regs = crate::pac::adc::Adc;
        }

        impl crate::adc::SealedInstance for peripherals::$inst {
            #[cfg(any(
                adc_v2, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u3, adc_u5, adc_wba, adc_g4, adc_c0
            ))]
            fn regs() -> Self::Regs {
                crate::pac::$inst
            }

            #[cfg(any(adc_f1, adc_f3v1, adc_f3v2, adc_v1, adc_l0))]
            fn regs() -> crate::pac::adc::Adc {
                crate::pac::$inst
            }

            #[cfg(not(any(adc_f1, adc_v1, adc_l0, adc_f3v3, adc_f3v2, adc_g0, adc_u5, adc_wba)))]
            fn common_regs() -> crate::pac::adccommon::AdcCommon {
                return crate::pac::$common_inst
            }

            #[cfg(any(adc_f1, adc_f3v1, adc_v1, adc_l0, adc_f3v2))]
            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl crate::adc::Instance for peripherals::$inst {
            type Interrupt = crate::_generated::peripheral_interrupts::$inst::GLOBAL;
        }
    };
);

macro_rules! impl_adc_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::adc::AdcChannel<peripherals::$inst> for crate::Peri<'_, crate::peripherals::$pin> {}
        impl crate::adc::SealedAdcChannel<peripherals::$inst> for crate::Peri<'_, crate::peripherals::$pin> {
            #[cfg(any(
                adc_v1, adc_c0, adc_l0, adc_v2, adc_g4, adc_v3, adc_v4, adc_u3, adc_u5, adc_wba
            ))]
            fn setup(&mut self) {
                <crate::peripherals::$pin as crate::gpio::SealedPin>::set_as_analog(self);
            }

            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! impl_adc_pair {
    ($inst:ident, $pin:ident, $npin:ident, $ch:expr) => {
        impl crate::adc::AdcChannel<peripherals::$inst>
            for (
                crate::Peri<'_, crate::peripherals::$pin>,
                crate::Peri<'_, crate::peripherals::$npin>,
            )
        {
        }
        impl crate::adc::SealedAdcChannel<peripherals::$inst>
            for (
                crate::Peri<'_, crate::peripherals::$pin>,
                crate::Peri<'_, crate::peripherals::$npin>,
            )
        {
            #[cfg(any(
                adc_v1, adc_c0, adc_l0, adc_v2, adc_g4, adc_v3, adc_v4, adc_u3, adc_u5, adc_wba
            ))]
            fn setup(&mut self) {
                <crate::peripherals::$pin as crate::gpio::SealedPin>::set_as_analog(&mut self.0);
                <crate::peripherals::$npin as crate::gpio::SealedPin>::set_as_analog(&mut self.1);
            }

            fn channel(&self) -> u8 {
                $ch
            }

            fn is_differential(&self) -> bool {
                true
            }
        }
    };
}

/// Get the maximum reading value for this resolution.
///
/// This is `2**n - 1`.
#[cfg(not(any(adc_f1, adc_f3v3)))]
pub const fn resolution_to_max_count(res: Resolution) -> u32 {
    match res {
        #[cfg(adc_v4)]
        Resolution::BITS16 => (1 << 16) - 1,
        #[cfg(any(adc_v4, adc_u5))]
        Resolution::BITS14 => (1 << 14) - 1,
        #[cfg(adc_v4)]
        Resolution::BITS14V => (1 << 14) - 1,
        #[cfg(adc_v4)]
        Resolution::BITS12V => (1 << 12) - 1,
        Resolution::BITS12 => (1 << 12) - 1,
        Resolution::BITS10 => (1 << 10) - 1,
        Resolution::BITS8 => (1 << 8) - 1,
        #[cfg(any(adc_v1, adc_v2, adc_v3, adc_l0, adc_c0, adc_g0, adc_f3v1, adc_f3v2, adc_h5))]
        Resolution::BITS6 => (1 << 6) - 1,
        #[allow(unreachable_patterns)]
        _ => core::unreachable!(),
    }
}
