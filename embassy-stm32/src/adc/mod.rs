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

#[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0, adc_wba))]
mod ringbuffered;

use core::marker::PhantomData;

#[allow(unused)]
#[cfg(not(any(adc_f3v3, adc_wba)))]
pub use _version::*;
#[allow(unused)]
use embassy_hal_internal::PeripheralType;
#[cfg(any(adc_f1, adc_f3v1, adc_v1, adc_l0, adc_f3v2))]
use embassy_sync::waitqueue::AtomicWaker;
#[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0, adc_wba))]
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
#[cfg(any(adc_v2, adc_g4))]
pub use crate::pac::adc::vals::Exten;
#[cfg(not(any(adc_f1, adc_f3v3)))]
pub use crate::pac::adc::vals::Res as Resolution;
pub use crate::pac::adc::vals::SampleTime;
use crate::peripherals;

dma_trait!(RxDma, Instance);

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
    type SampleTime;
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
        adc_g4, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u5, adc_u3, adc_wba, adc_c0
    ))]
    Singular,
    // Should match the cfg on "into_ring_buffered" below
    #[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0, adc_wba))]
    Repeated(RegularConversionMode),
}

// Trigger source for ADC conversions¨
#[cfg(any(adc_v2, adc_g4))]
#[derive(Copy, Clone)]
pub struct ConversionTrigger {
    // Note that Injected and Regular channels uses different mappings
    pub channel: u8,
    pub edge: Exten,
}

// Should match the cfg on "into_ring_buffered" below
#[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0, adc_wba))]
// Conversion mode for regular ADC channels
#[derive(Copy, Clone)]
pub enum RegularConversionMode {
    // Samples as fast as possible
    Continuous,
    #[cfg(any(adc_g4, adc_v2))]
    // Sample at rate determined by external trigger
    Triggered(ConversionTrigger),
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

        // Ensure no conversions are ongoing
        T::regs().stop();
        #[cfg(any(adc_v2, adc_v3, adc_g0, adc_h7rs, adc_u0, adc_u3, adc_u5, adc_wba))]
        T::regs().enable();
        T::regs().configure_sequence([((channel.channel(), channel.is_differential()), sample_time)].into_iter());

        // On chips with differential channels, enable after configure_sequence to allow setting differential channels
        //
        // TODO: If hardware allows, enable after configure_sequence on all chips
        #[cfg(any(adc_g4, adc_h5, adc_c0))]
        T::regs().enable();
        T::regs().convert();

        unsafe { core::ptr::read_volatile(T::regs().data()) }
    }

    #[cfg(any(
        adc_g4, adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0, adc_v4, adc_u5, adc_u3, adc_wba, adc_c0
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
    pub async fn read<'a, 'b: 'a>(
        &mut self,
        rx_dma: embassy_hal_internal::Peri<'_, impl RxDma<T>>,
        sequence: impl ExactSizeIterator<Item = (&'a mut AnyAdcChannel<'b, T>, <T::Regs as BasicAdcRegs>::SampleTime)>,
        readings: &mut [u16],
    ) {
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
        #[cfg(any(adc_g0, adc_v3, adc_h7rs, adc_u0, adc_v4, adc_u3, adc_u5, adc_wba))]
        T::regs().enable();

        T::regs().configure_sequence(
            sequence.map(|(channel, sample_time)| ((channel.channel, channel.is_differential), sample_time)),
        );

        // On chips with differential channels, enable after configure_sequence to allow setting differential channels
        //
        // TODO: If hardware allows, enable after configure_sequence on all chips
        #[cfg(any(adc_g4, adc_h5, adc_c0))]
        T::regs().enable();
        T::regs().configure_dma(ConversionMode::Singular);

        let request = rx_dma.request();
        let transfer =
            unsafe { crate::dma::Transfer::new_read(rx_dma, request, T::regs().data(), readings, Default::default()) };

        T::regs().start();

        // Wait for conversion sequence to finish.
        transfer.await;

        // Ensure conversions are finished.
        T::regs().stop();
    }

    #[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0, adc_wba))]
    /// Configures the ADC to use a DMA ring buffer for continuous data acquisition.
    ///
    /// Use the [`read`] method to retrieve measurements from the DMA ring buffer. The read buffer
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
    pub fn into_ring_buffered<'a, 'b>(
        self,
        dma: embassy_hal_internal::Peri<'a, impl RxDma<T>>,
        dma_buf: &'a mut [u16],
        sequence: impl ExactSizeIterator<Item = (AnyAdcChannel<'b, T>, <T::Regs as BasicAdcRegs>::SampleTime)>,
        mode: RegularConversionMode,
    ) -> RingBufferedAdc<'a, T> {
        assert!(!dma_buf.is_empty() && dma_buf.len() <= 0xFFFF);
        assert!(sequence.len() != 0, "Asynchronous read sequence cannot be empty");
        assert!(
            sequence.len() <= 16,
            "Asynchronous read sequence cannot be more than 16 in length"
        );
        // Ensure no conversions are ongoing
        T::regs().stop();
        #[cfg(any(adc_g0, adc_v3, adc_h7rs, adc_u0, adc_v4, adc_u5, adc_wba, adc_c0))]
        T::regs().enable();

        T::regs().configure_sequence(
            sequence.map(|(channel, sample_time)| ((channel.channel, channel.is_differential), sample_time)),
        );

        // On chips with differential channels, enable after configure_sequence to allow setting differential channels
        //
        // TODO: If hardware allows, enable after configure_sequence on all chips
        #[cfg(any(adc_g4, adc_h5))]
        T::regs().enable();
        T::regs().configure_dma(ConversionMode::Repeated(mode));

        core::mem::forget(self);

        RingBufferedAdc::new(dma, dma_buf)
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
    #[cfg(any(adc_f3v1, adc_f3v2))]
    /// The value that vref would be if vdda was at 3300mv
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
    fn degrade_adc<'a>(mut self) -> AnyAdcChannel<'a, T>
    where
        Self: 'a,
    {
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
