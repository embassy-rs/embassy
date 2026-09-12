//! Analog to Digital Converter (ADC)

// One driver for every STM32 ADC. The register-level differences between the three ADC
// generations are handled in [`v1`], [`v2`] and [`v3`]; everything above that is shared, so the
// API is the same on every chip and only differs where the hardware lacks a feature (in which
// case the method or configuration option is absent for that chip).
//
// | module | register layout                   | families |
// |--------|-----------------------------------|----------|
// | `v1`   | `SR`/`CR1`/`CR2`, `SQR1..3`       | F1, F2, F4, F7, L1 |
// | `v2`   | `CFGR1`, `CHSELR` bitmask         | F0, L0, G0, C0, U0, WL, WB1x, WBA, U5 (`ADC4`) |
// | `v3`   | `ISR`/`CFGR`, `SQR1..4`, `DIFSEL` | F3, L4, L5, WB, G4, H5, H7, H7RS, U5, U3, N6, C5 |
//
// The register versions in stm32-data follow the same scheme (`adc_v1_f4`, `adc_v2_g0`,
// `adc_v3_h7`, ...): the first component is the generation, the second the first chip family
// with that exact register layout.
//
// # Common registers
//
// Several ADC instances can share an `ADCx_COMMON` register block (clock prescaler, internal
// channel enables, dual mode). The driver only ever touches those registers with
// read-modify-write sequences inside a critical section, so several `Adc` drivers on the same
// common block can be created and used from different tasks. Settings that apply to the whole
// block (the clock configuration, dual mode) are written by whichever instance is constructed
// last; the internal channel switches are only ever enabled, never disabled, by the `enable_*`
// calls (`disable_vbat` is the exception, since the VBAT divider draws battery current).

#![macro_use]

use core::marker::PhantomData;
use core::sync::atomic::AtomicBool;

use embassy_hal_internal::drop::OnDrop;
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::time::Hertz;
use crate::{Peri, interrupt, peripherals, rcc};

mod configured_sequence;
#[cfg(any(adc_v1, adc_v3))]
mod injected;
mod internal;
mod ringbuffered;
#[cfg(adc_v1)]
mod v1;
#[cfg(adc_v2)]
mod v2;
#[cfg(adc_v3)]
mod v3;
mod watchdog;

pub use configured_sequence::ConfiguredSequence;
#[cfg(any(adc_v1, adc_v3))]
pub use injected::{InjectedAdc, InjectedMode};
pub use internal::*;
pub use ringbuffered::{OverrunError, RingBufferedAdc};
pub use watchdog::{AnalogWatchdog, WatchdogChannels, WatchdogIndex};

/// Register-level enums of the ADC, re-exported from the PAC.
pub use crate::pac::adc::vals;
/// Sample time of the STM32U5 `ADC4`, in ADC clock cycles.
#[cfg(adc_v2_u5)]
pub use crate::pac::adc::vals::Adc4SampleTime;
/// External trigger edge selection.
#[cfg(not(adc_v1_f1))]
pub use crate::pac::adc::vals::Exten;
/// Sample time, in ADC clock cycles.
///
/// The variants are the cycle counts of this chip's ADC. On the STM32U5 the two ADC generations
/// coexist: `ADC1`/`ADC2` use `SampleTime`, `ADC4` uses [`Adc4SampleTime`].
pub use crate::pac::adc::vals::SampleTime;
/// External trigger edge selection.
///
/// The STM32F1 ADC only triggers on rising edges.
#[cfg(adc_v1_f1)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Exten {
    /// Hardware trigger detection disabled.
    Disabled,
    /// Hardware trigger detection on the rising edge.
    RisingEdge,
}

#[cfg(adccommon_v4)]
pub use crate::pac::adccommon::vals::Damdf;
#[cfg(any(adccommon_v3, adccommon_v4))]
pub use crate::pac::adccommon::vals::Dual;

dma_trait!(RxDma, Instance);
trigger_trait!(RegularTrigger, Instance);
#[cfg(any(adc_v1, adc_v3))]
trigger_trait!(InjectedTrigger, Instance);

// ----------------------------------------------------------------------------------------------
// Configuration

/// Conversion resolution.
///
/// Only the resolutions of this chip's ADCs exist.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Resolution {
    /// 6-bit.
    #[cfg(not(any(adc_v1_f1, adc_v3_h7)))]
    Bits6,
    /// 8-bit.
    #[cfg(not(adc_v1_f1))]
    Bits8,
    /// 10-bit.
    #[cfg(not(adc_v1_f1))]
    Bits10,
    /// 12-bit.
    Bits12,
    /// 14-bit.
    #[cfg(adc_res14)]
    Bits14,
    /// 16-bit.
    #[cfg(adc_res16)]
    Bits16,
}

impl Resolution {
    /// Number of bits.
    pub const fn bits(self) -> u8 {
        match self {
            #[cfg(not(any(adc_v1_f1, adc_v3_h7)))]
            Self::Bits6 => 6,
            #[cfg(not(adc_v1_f1))]
            Self::Bits8 => 8,
            #[cfg(not(adc_v1_f1))]
            Self::Bits10 => 10,
            Self::Bits12 => 12,
            #[cfg(adc_res14)]
            Self::Bits14 => 14,
            #[cfg(adc_res16)]
            Self::Bits16 => 16,
        }
    }

    /// The maximum reading at this resolution, `2**bits - 1`.
    pub const fn max_count(self) -> u32 {
        (1 << self.bits()) - 1
    }
}

/// Get the maximum reading value for this resolution.
///
/// This is `2**n - 1`.
pub const fn resolution_to_max_count(res: Resolution) -> u32 {
    res.max_count()
}

/// Number of samples the oversampler accumulates.
///
/// Only the ratios of this chip's ADCs exist.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(adc_oversampler)]
#[allow(missing_docs)]
pub enum OversamplingRatio {
    X2,
    X4,
    X8,
    X16,
    X32,
    X64,
    X128,
    X256,
    #[cfg(adc_oversampler_1024)]
    X512,
    #[cfg(adc_oversampler_1024)]
    X1024,
}

#[cfg(adc_oversampler)]
impl OversamplingRatio {
    /// log2 of the number of samples.
    pub const fn log2(self) -> u8 {
        match self {
            Self::X2 => 1,
            Self::X4 => 2,
            Self::X8 => 3,
            Self::X16 => 4,
            Self::X32 => 5,
            Self::X64 => 6,
            Self::X128 => 7,
            Self::X256 => 8,
            #[cfg(adc_oversampler_1024)]
            Self::X512 => 9,
            #[cfg(adc_oversampler_1024)]
            Self::X1024 => 10,
        }
    }
}

/// Raw control of the hardware oversampler.
///
/// The oversampler accumulates `ratio` samples and right-shifts the sum by `shift` bits.
/// Averaging (`Config::averaging`) is the common special case where the shift is `ratio.log2()`;
/// use this to get extra resolution instead (for example 16 samples and a shift of 2 turns a
/// 12-bit ADC into a 14-bit one).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(adc_oversampler)]
pub struct Oversampling {
    /// Number of samples to accumulate.
    pub ratio: OversamplingRatio,
    /// Number of bits to right-shift the accumulated result: 0 to 8, or 0 to 11 on ADCs with the
    /// `X1024` ratio. Panics at construction when out of range.
    pub shift: u8,
    /// Triggered mode: each oversampled conversion needs a trigger instead of all of them being
    /// run back to back after a single trigger.
    pub triggered: bool,
    /// Resume (instead of continue) a regular oversampling sequence interrupted by an injected
    /// conversion. Only on ADCs with injected conversions.
    pub resumed: bool,
}

#[cfg(adc_oversampler)]
impl Oversampling {
    /// Oversample by `ratio` and right-shift the sum by `shift` bits, in continuous mode.
    pub const fn new(ratio: OversamplingRatio, shift: u8) -> Self {
        Self {
            ratio,
            shift,
            triggered: false,
            resumed: false,
        }
    }

    /// Oversampling settings that average `ratio` samples, keeping the resolution.
    pub const fn averaging(ratio: OversamplingRatio) -> Self {
        Self::new(ratio, ratio.log2())
    }
}

/// ADC clock prescaler, applied to the kernel clock in asynchronous clock mode.
///
/// Only the ratios of this chip's ADCs exist.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(any(adc_presc_f4, adc_presc_l1, adc_presc_full))]
#[allow(missing_docs)]
pub enum Prescaler {
    #[cfg(any(adc_presc_l1, adc_presc_full))]
    Div1,
    Div2,
    Div4,
    #[cfg(any(adc_presc_f4, adc_presc_full))]
    Div6,
    #[cfg(any(adc_presc_f4, adc_presc_full))]
    Div8,
    #[cfg(adc_presc_full)]
    Div10,
    #[cfg(adc_presc_full)]
    Div12,
    #[cfg(adc_presc_full)]
    Div16,
    #[cfg(adc_presc_full)]
    Div32,
    #[cfg(adc_presc_full)]
    Div64,
    #[cfg(adc_presc_full)]
    Div128,
    #[cfg(adc_presc_full)]
    Div256,
}

#[cfg(any(adc_presc_f4, adc_presc_l1, adc_presc_full))]
impl Prescaler {
    /// The division factor.
    pub const fn divisor(self) -> u32 {
        match self {
            #[cfg(any(adc_presc_l1, adc_presc_full))]
            Self::Div1 => 1,
            Self::Div2 => 2,
            Self::Div4 => 4,
            #[cfg(any(adc_presc_f4, adc_presc_full))]
            Self::Div6 => 6,
            #[cfg(any(adc_presc_f4, adc_presc_full))]
            Self::Div8 => 8,
            #[cfg(adc_presc_full)]
            Self::Div10 => 10,
            #[cfg(adc_presc_full)]
            Self::Div12 => 12,
            #[cfg(adc_presc_full)]
            Self::Div16 => 16,
            #[cfg(adc_presc_full)]
            Self::Div32 => 32,
            #[cfg(adc_presc_full)]
            Self::Div64 => 64,
            #[cfg(adc_presc_full)]
            Self::Div128 => 128,
            #[cfg(adc_presc_full)]
            Self::Div256 => 256,
        }
    }
}

/// ADC clock divider, applied to the bus clock in synchronous clock mode.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(adc_sync_clock)]
#[allow(missing_docs)]
pub enum SyncDiv {
    #[cfg(adc_sync_div1)]
    Div1,
    Div2,
    Div4,
}

#[cfg(adc_sync_clock)]
impl SyncDiv {
    /// The division factor.
    pub const fn divisor(self) -> u32 {
        match self {
            #[cfg(adc_sync_div1)]
            Self::Div1 => 1,
            Self::Div2 => 2,
            Self::Div4 => 4,
        }
    }
}

/// ADC clock selection.
///
/// Only the modes of this chip's ADCs exist; ADCs without a prescaler take their clock from the
/// RCC as is.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Clock {
    /// Use the ADC kernel clock selected in the RCC, with the smallest prescaler that keeps the
    /// ADC clock within the limit of the chip.
    #[default]
    Auto,
    /// Use the ADC kernel clock selected in the RCC, divided by the given prescaler.
    #[cfg(any(adc_presc_f4, adc_presc_l1, adc_presc_full))]
    Async(Prescaler),
    /// Use the ADC bus clock (synchronous mode), divided by the given divider.
    #[cfg(adc_sync_clock)]
    Sync(SyncDiv),
}

/// ADC configuration.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// Conversion resolution. `None` keeps the hardware default, which is the highest resolution
    /// of the ADC.
    #[cfg(not(adc_v1_f1))]
    pub resolution: Option<Resolution>,
    /// Hardware averaging: accumulate this many samples per conversion and scale the sum back
    /// to the configured resolution.
    #[cfg(adc_oversampler)]
    pub averaging: Option<OversamplingRatio>,
    /// Raw oversampler control; takes precedence over `averaging`.
    #[cfg(adc_oversampler)]
    pub oversampling: Option<Oversampling>,
    /// ADC clock selection.
    pub clock: Clock,
    /// Dual ADC mode, in the common registers shared with the other ADC of the pair.
    #[cfg(any(adccommon_v3, adccommon_v4))]
    pub dual_mode: Option<Dual>,
    /// Dual ADC mode data format, in the common registers shared with the other ADC of the pair.
    #[cfg(adccommon_v4)]
    pub dual_data_format: Option<Damdf>,
    /// Delay between the two sampling phases in dual ADC mode, in the common registers shared
    /// with the other ADC of the pair.
    #[cfg(any(adccommon_v3, adccommon_v4))]
    pub dual_delay: Option<u8>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(not(adc_v1_f1))]
            resolution: None,
            #[cfg(adc_oversampler)]
            averaging: None,
            #[cfg(adc_oversampler)]
            oversampling: None,
            clock: Clock::Auto,
            #[cfg(any(adccommon_v3, adccommon_v4))]
            dual_mode: None,
            #[cfg(adccommon_v4)]
            dual_data_format: None,
            #[cfg(any(adccommon_v3, adccommon_v4))]
            dual_delay: None,
        }
    }
}

#[cfg(adc_oversampler)]
impl Config {
    /// The oversampler settings to apply, from `oversampling` or `averaging`.
    pub(crate) fn oversampler(&self) -> Option<Oversampling> {
        self.oversampling.or(self.averaging.map(Oversampling::averaging))
    }
}

// ----------------------------------------------------------------------------------------------
// Triggers

/// External trigger for regular conversions.
pub struct RegularAdcTrigger<T: Instance> {
    trigger: u8,
    edge: Exten,
    _marker: PhantomData<T>,
}

impl<T: Instance> RegularAdcTrigger<T> {
    /// Build a trigger from a trigger source and an edge.
    pub fn from(trigger: impl RegularTrigger<T>, edge: Exten) -> Self {
        Self {
            trigger: trigger.signal(),
            edge,
            _marker: PhantomData,
        }
    }
}

/// External trigger for injected conversions.
#[cfg(any(adc_v1, adc_v3))]
pub struct InjectedAdcTrigger<T: Instance> {
    trigger: u8,
    edge: Exten,
    _marker: PhantomData<T>,
}

#[cfg(any(adc_v1, adc_v3))]
impl<T: Instance> InjectedAdcTrigger<T> {
    /// Build a trigger from a trigger source and an edge.
    pub fn from(trigger: impl InjectedTrigger<T>, edge: Exten) -> Self {
        Self {
            trigger: trigger.signal(),
            edge,
            _marker: PhantomData,
        }
    }
}

/// How the ADC and the DMA are configured for a set of conversions.
#[derive(Copy, Clone)]
pub(crate) enum ConversionMode {
    /// Software-started, no DMA, one sequence per start.
    NoDma,
    /// Software-started, DMA requests, one sequence per start.
    Singular,
    /// DMA requests, sequences repeat: continuously when there is no trigger, or once per trigger
    /// event.
    Repeated(Option<(u8, Exten)>),
}

// ----------------------------------------------------------------------------------------------
// Register abstraction implemented by each ADC generation

/// Interrupt state shared between the driver and the interrupt handler.
pub struct State {
    pub(crate) waker: AtomicWaker,
    #[cfg(any(adc_v1, adc_v3))]
    pub(crate) injected_done: AtomicBool,
    pub(crate) awd_triggered: [AtomicBool; 3],
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    /// Create a new state.
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            #[cfg(any(adc_v1, adc_v3))]
            injected_done: AtomicBool::new(false),
            awd_triggered: [AtomicBool::new(false), AtomicBool::new(false), AtomicBool::new(false)],
        }
    }
}

/// Which internal channel to switch on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum InternalChannel {
    VrefInt,
    Temperature,
    Vbat,
    VddCore,
    /// DAC output loopback, with the selected DAC channel where several are muxed onto one ADC
    /// channel.
    Dac(u8),
}

/// Register block of an ADC instance: the sample time type of the ADC.
pub trait BasicAdcRegs: Copy + 'static {
    /// The sample time selection of this ADC.
    type SampleTime: Copy;
}

/// The register-level operations every ADC generation implements.
///
/// `Self` is the PAC register block; `Common` the block holding the shared registers (`()` when
/// they live in the ADC block itself).
pub(crate) trait AdcRegs: BasicAdcRegs {
    type Common: Copy;

    /// Number of analog watchdogs.
    const AWD_COUNT: usize;
    /// Maximum number of conversions in a regular sequence.
    const MAX_SEQUENCE_LEN: usize;
    /// Number of injected ranks.
    #[cfg(any(adc_v1, adc_v3))]
    const INJECTED_RANKS: usize;

    /// Power up, calibrate and enable the ADC, and apply the configuration.
    ///
    /// `kernel_clock` is the RCC clock feeding the ADC.
    fn init(self, common: Self::Common, kernel_clock: Hertz, config: &Config);
    /// The ADC clock after `init`.
    fn clock(self, common: Self::Common, kernel_clock: Hertz) -> Hertz;
    /// Stop conversions and fully power down the ADC.
    fn power_down(self);
    /// Enable the ADC if it is not already enabled.
    fn enable(self);

    #[cfg(not(adc_v1_f1))]
    fn set_resolution(self, res: Resolution);
    fn resolution(self) -> Resolution;

    /// Program a regular or injected sequence. Must be called with conversions stopped. May
    /// disable the ADC if the hardware requires it; callers re-enable it with `enable`.
    fn configure_sequence(
        self,
        sequence: impl ExactSizeIterator<Item = ((u8, bool), Self::SampleTime)>,
        injected: bool,
    );
    /// Configure DMA, continuous mode and the regular trigger.
    fn configure_dma(self, mode: ConversionMode);
    /// Start regular conversions (software trigger, or arm the hardware trigger).
    fn start(self);
    /// Stop regular conversions, leaving the ADC enabled and configured.
    fn stop(self);
    /// Whether the conversion started with `start` has completed.
    fn done(self) -> bool;
    /// The data register, read as `u16`.
    fn data(self) -> *mut u16;
    /// Enable or disable the end-of-conversion interrupt used by the async single read.
    fn set_eoc_interrupt(self, enable: bool);
    /// Interrupt handler body.
    fn on_interrupt(self, state: &State);

    /// Switch an internal channel (the `enable` bit in the common registers) on or off.
    fn enable_internal(self, common: Self::Common, channel: InternalChannel, enable: bool);

    /// Configure and enable analog watchdog `index` (0-based). The thresholds are in the
    /// same units as data register readings; the implementation scales them for the
    /// hardware comparison.
    fn configure_awd(self, index: usize, channels: WatchdogChannels, low: u32, high: u32);
    fn disable_awd(self, index: usize);
    fn set_awd_interrupt(self, index: usize, enable: bool);
    /// Read and clear the watchdog `index` flag.
    fn clear_awd_flag(self, index: usize) -> bool;
    /// Set continuous conversion mode (used by the watchdog monitor).
    fn set_continuous(self, enable: bool);
    /// Set low-frequency trigger mode.
    #[cfg(any(
        adc_v2_wb1, adc_v2_g0, adc_v2_u5, adc_v2_wba, adc_v3_u5, adc_v3_u3, stm32h5, stm32h7rs
    ))]
    fn set_low_frequency_trigger(self, enable: bool);
}

/// The register block type of an ADC instance.
#[allow(private_bounds)]
pub trait BasicInstance {
    /// Register block.
    type Regs: AdcRegs;
}

trait SealedInstance: BasicInstance {
    fn regs() -> Self::Regs;
    fn common() -> <Self::Regs as AdcRegs>::Common;
    fn state() -> &'static State;
}

/// ADC instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + BasicInstance + crate::PeripheralType + rcc::RccPeripheral {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::regs().on_interrupt(T::state());
    }
}

// ----------------------------------------------------------------------------------------------
// Channels

pub(crate) trait SealedAdcChannel<T> {
    fn setup(&mut self) {}

    fn channel(&self) -> u8;

    fn is_differential(&self) -> bool {
        false
    }
}

/// ADC channel.
#[allow(private_bounds)]
pub trait AdcChannel<'d, T>: SealedAdcChannel<T> + Sized {
    /// Type-erase the channel. The pin (if any) is configured for the ADC once, here.
    fn degrade_adc(mut self) -> BorrowedAdcChannel<'d, T> {
        self.setup();

        BorrowedAdcChannel {
            channel: self.channel(),
            is_differential: self.is_differential(),
            _marker: PhantomData,
        }
    }

    /// Borrow the channel, type-erased. The pin (if any) is configured for the ADC.
    #[allow(unused_mut)]
    fn reborrow_adc<'a>(&'a mut self) -> BorrowedAdcChannel<'a, T> {
        self.setup();

        BorrowedAdcChannel {
            channel: self.channel(),
            is_differential: self.is_differential(),
            _marker: PhantomData,
        }
    }
}

/// A type-erased borrowed channel for a given ADC instance.
pub struct BorrowedAdcChannel<'a, T> {
    channel: u8,
    is_differential: bool,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T: Instance> BorrowedAdcChannel<'a, T> {
    /// The hardware channel number.
    #[inline]
    pub fn get_hw_channel(&self) -> u8 {
        self.channel
    }
}

impl<'a, T: Instance> SealedAdcChannel<T> for BorrowedAdcChannel<'a, T> {
    fn channel(&self) -> u8 {
        self.channel
    }

    fn is_differential(&self) -> bool {
        self.is_differential
    }
}

impl<'a, T: Instance> AdcChannel<'a, T> for BorrowedAdcChannel<'a, T> {
    fn degrade_adc(self) -> BorrowedAdcChannel<'a, T> {
        self
    }

    #[inline]
    fn reborrow_adc<'b>(&'b mut self) -> BorrowedAdcChannel<'b, T> {
        Self { ..*self }
    }
}

trait SealedBorrowedChannel<'a, T> {
    fn reborrow_adc(self) -> BorrowedAdcChannel<'a, T>;
}

/// Anything that can be turned into a [`BorrowedAdcChannel`]: a `&mut` to a channel, or a
/// borrowed channel itself.
#[allow(private_bounds)]
pub trait BorrowedChannel<'a, T>: SealedBorrowedChannel<'a, T> {}
impl<'a, T, C: SealedBorrowedChannel<'a, T>> BorrowedChannel<'a, T> for C {}

impl<'a, 'd, T, C: AdcChannel<'d, T>> SealedBorrowedChannel<'a, T> for &'a mut C {
    #[inline]
    fn reborrow_adc(self) -> BorrowedAdcChannel<'a, T> {
        AdcChannel::reborrow_adc(self)
    }
}

impl<'a, T> SealedBorrowedChannel<'a, T> for BorrowedAdcChannel<'a, T> {
    fn reborrow_adc(self) -> BorrowedAdcChannel<'a, T> {
        self
    }
}

pub(crate) trait AnalogPin {
    fn set_as_analog(&self) {}
}

impl<T: crate::gpio::SealedPin> AnalogPin for T {
    fn set_as_analog(&self) {
        T::set_as_analog(self);
    }
}

#[allow(unused_macros)]
macro_rules! impl_analog_pin {
    ($pin:ident) => {
        impl crate::adc::AnalogPin for crate::peripherals::$pin {}
    };
}

macro_rules! impl_adc_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl<'d> crate::adc::AdcChannel<'d, peripherals::$inst> for crate::Peri<'d, crate::peripherals::$pin> {}
        impl crate::adc::SealedAdcChannel<peripherals::$inst> for crate::Peri<'_, crate::peripherals::$pin> {
            fn setup(&mut self) {
                <crate::peripherals::$pin as crate::adc::AnalogPin>::set_as_analog(self);
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
        impl<'d> crate::adc::AdcChannel<'d, peripherals::$inst>
            for (
                crate::Peri<'d, crate::peripherals::$pin>,
                crate::Peri<'d, crate::peripherals::$npin>,
            )
        {
        }
        impl crate::adc::SealedAdcChannel<peripherals::$inst>
            for (
                crate::Peri<'_, crate::peripherals::$pin>,
                crate::Peri<'_, crate::peripherals::$npin>,
            )
        {
            fn setup(&mut self) {
                <crate::peripherals::$pin as crate::adc::AnalogPin>::set_as_analog(&mut self.0);
                <crate::peripherals::$npin as crate::adc::AnalogPin>::set_as_analog(&mut self.1);
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

// ----------------------------------------------------------------------------------------------
// Driver

/// Analog to Digital driver.
pub struct Adc<'d, T: Instance, M: Mode> {
    _adc: Peri<'d, T>,
    _mode: PhantomData<M>,
}

/// Sample time type of an ADC instance.
pub type SampleTimeOf<T> = <<T as BasicInstance>::Regs as BasicAdcRegs>::SampleTime;

impl<'d, T: Instance> Adc<'d, T, Blocking> {
    /// Create a new ADC driver without an interrupt binding.
    ///
    /// DMA reads still work; the interrupt-driven single [`read`](Adc::read), injected
    /// conversions with interrupts and the asynchronous watchdog need [`Adc::new`].
    pub fn new_blocking(adc: Peri<'d, T>, config: Config) -> Self {
        Self::new_inner(adc, config)
    }
}

impl<'d, T: Instance> Adc<'d, T, Async> {
    /// Create a new ADC driver with its interrupt bound.
    pub fn new(
        adc: Peri<'d, T>,
        _irqs: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        let s = Self::new_inner(adc, config);
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        s
    }

    /// Read a channel once, waiting for the end-of-conversion interrupt.
    pub async fn read<'a>(&mut self, channel: impl BorrowedChannel<'a, T>, sample_time: SampleTimeOf<T>) -> u16 {
        use core::future::poll_fn;
        use core::sync::atomic::{Ordering, compiler_fence};
        use core::task::Poll;

        let _scoped_wake_guard = <T as rcc::SealedRccPeripheral>::RCC_INFO.wake_guard();
        let channel = channel.reborrow_adc();

        let r = T::regs();
        r.stop();
        r.configure_sequence(
            [((channel.channel(), channel.is_differential()), sample_time)].into_iter(),
            false,
        );
        r.enable();
        r.configure_dma(ConversionMode::NoDma);
        r.set_eoc_interrupt(true);
        r.start();

        let _stop = OnDrop::new(|| {
            r.set_eoc_interrupt(false);
            r.stop();
        });

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());
            compiler_fence(Ordering::SeqCst);
            if r.done() { Poll::Ready(()) } else { Poll::Pending }
        })
        .await;

        unsafe { core::ptr::read_volatile(r.data()) }
    }
}

impl<'d, T: Instance, M: Mode> Adc<'d, T, M> {
    fn new_inner(adc: Peri<'d, T>, config: Config) -> Self {
        rcc::enable_and_reset::<T>();
        let kernel_clock = T::frequency();
        T::regs().init(T::common(), kernel_clock, &config);
        trace!("adc: clock {}", T::regs().clock(T::common(), kernel_clock));
        Self {
            _adc: adc,
            _mode: PhantomData,
        }
    }

    /// The ADC clock frequency.
    pub fn clock(&self) -> Hertz {
        T::regs().clock(T::common(), T::frequency())
    }

    /// Enable or disable low-frequency trigger mode, required when hardware triggers arrive at
    /// less than about 1 kHz.
    #[cfg(any(
        adc_v2_wb1, adc_v2_g0, adc_v2_u5, adc_v2_wba, adc_v3_u5, adc_v3_u3, stm32h5, stm32h7rs
    ))]
    pub fn set_low_frequency_trigger(&mut self, enable: bool) {
        T::regs().set_low_frequency_trigger(enable);
    }

    /// Set the conversion resolution.
    #[cfg(not(adc_v1_f1))]
    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().stop();
        T::regs().set_resolution(resolution);
    }

    /// The currently configured resolution.
    pub fn resolution(&self) -> Resolution {
        T::regs().resolution()
    }

    /// Pick the shortest sample time that samples for at least `us` microseconds.
    pub fn sample_time_for_us(&self, us: u32) -> SampleTimeOf<T>
    where
        T::Regs: SampleTimes,
    {
        let clock = self.clock();
        let wanted_half_cycles = (us as u64 * clock.0 as u64 * 2).div_ceil(1_000_000) as u32;
        T::Regs::sample_time_for_half_cycles(wanted_half_cycles)
    }

    /// Read a channel once, busy-waiting for the conversion.
    pub fn blocking_read<'a>(&mut self, channel: impl BorrowedChannel<'a, T>, sample_time: SampleTimeOf<T>) -> u16 {
        let channel = channel.reborrow_adc();

        let r = T::regs();
        r.stop();
        r.configure_sequence(
            [((channel.channel(), channel.is_differential()), sample_time)].into_iter(),
            false,
        );
        r.enable();
        r.configure_dma(ConversionMode::NoDma);
        r.start();
        while !r.done() {}

        unsafe { core::ptr::read_volatile(r.data()) }
    }

    /// Read one or multiple regular channels using DMA.
    ///
    /// `readings` must have a length that is a multiple of the length of the `sequence` iterator;
    /// the sequence is repeated until it is full, continuously or once per `trigger` event.
    ///
    /// Example
    /// ```rust,ignore
    /// use embassy_stm32::adc::{Adc, AdcChannel}
    ///
    /// let mut adc = Adc::new_blocking(p.ADC1, Default::default());
    /// let mut adc_pin0 = p.PA0.into();
    /// let mut adc_pin1 = p.PA1.into();
    /// let mut measurements = [0u16; 2];
    ///
    /// adc.read_sequence(
    ///     p.DMA1_CH2.reborrow(),
    ///     Irqs,
    ///     [
    ///         (&mut *adc_pin0, SampleTime::CYCLES160_5),
    ///         (&mut *adc_pin1, SampleTime::CYCLES160_5),
    ///     ]
    ///     .into_iter(),
    ///     None,
    ///     &mut measurements,
    /// )
    /// .await;
    /// defmt::info!("measurements: {}", measurements);
    /// ```
    ///
    /// Note: the ADC is reconfigured on each call. Use [`configure_sequence`](Self::configure_sequence)
    /// or [`into_ring_buffered`](Self::into_ring_buffered) to convert the same sequence repeatedly.
    ///
    /// Note: ADCs without a fully configurable sequencer (F0, L0, and G0-class ADCs with more than
    /// 8 channels or channels above 14) can only scan channels in ascending or descending order,
    /// with a single sample time. This method panics if the hardware cannot deliver the requested
    /// sequence.
    #[inline]
    pub async fn read_sequence<'a, 'ch: 'a, D: RxDma<T>>(
        &mut self,
        rx_dma: Peri<'a, D>,
        irq: impl interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
        sequence: impl ExactSizeIterator<Item = (BorrowedAdcChannel<'ch, T>, SampleTimeOf<T>)>,
        trigger: Option<RegularAdcTrigger<T>>,
        readings: &mut [u16],
    ) {
        let _scoped_wake_guard = <T as rcc::SealedRccPeripheral>::RCC_INFO.wake_guard();

        check_dma_len::<T>(sequence.len(), Some(readings.len()));

        let r = T::regs();
        r.stop();
        r.configure_sequence(
            sequence.map(|(channel, sample_time)| ((channel.channel, channel.is_differential), sample_time)),
            false,
        );
        r.enable();
        r.configure_dma(ConversionMode::Repeated(trigger.map(|t| (t.trigger, t.edge))));

        let mut dma_channel = new_dma_nonopt!(rx_dma, irq);

        let transfer = unsafe {
            dma_channel.read_raw(
                r.data() as *mut Word,
                readings,
                crate::dma::TransferOptions {
                    #[cfg(stm32n6)]
                    // DMA will read 0 unless it is marked as secure along with RISUP 64 (ADC12)
                    secure: true,
                    #[cfg(stm32n6)]
                    packing: crate::dma::Packing::ZeroExtendOrLeftTruncate,
                    ..Default::default()
                },
            )
        };

        // Stop conversions even if the future is dropped.
        let _stop_adc = OnDrop::new(|| r.stop());
        r.start();

        transfer.await;
    }

    /// Configure an ADC channel sequence once and return a [`ConfiguredTransfer`] for repeated
    /// DMA reads to peripherals such as FMAC or CORDIC.
    ///
    /// Use [`Adc::configure_sequence`] instead if you don't want to pipe the results directly
    /// to a peripheral.
    ///
    /// # Safety
    ///
    /// `dst` must be a valid peripheral data register for the lifetime of the returned value.
    ///
    /// # Notes
    /// - The channel sequence is programmed into the ADC sequence registers once here and
    ///   remains fixed for the lifetime of the returned [`ConfiguredTransfer`].
    /// - Call this method AFTER the targeted peripheral is ready to begin receiving the transfer.
    pub unsafe fn configure_transfer<'adc, 'ch, D: RxDma<T>, W: crate::dma::word::Word>(
        &'adc mut self,
        rx_dma: Peri<'adc, D>,
        irq: impl interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'd,
        sequence: impl ExactSizeIterator<Item = (BorrowedAdcChannel<'ch, T>, SampleTimeOf<T>)>,
        trigger: RegularAdcTrigger<T>,
        dst: *mut W,
    ) -> ConfiguredTransfer<'adc, T::Regs>
    where
        'ch: 'adc,
    {
        check_dma_len::<T>(sequence.len(), None);

        let r = T::regs();
        r.stop();
        r.configure_sequence(
            sequence.map(|(channel, sample_time)| ((channel.channel, channel.is_differential), sample_time)),
            false,
        );
        r.enable();
        r.configure_dma(ConversionMode::Repeated(Some((trigger.trigger, trigger.edge))));

        let mut dma_channel = new_dma_nonopt!(rx_dma, irq);

        let transfer = unsafe {
            dma_channel
                .read_raw_repeated(
                    dst,
                    1,
                    r.data(),
                    crate::dma::TransferOptions {
                        #[cfg(any(bdma, dma, mdma))]
                        circular: true,
                        ..Default::default()
                    },
                )
                .unchecked_extend_lifetime()
        };

        r.start();

        ConfiguredTransfer {
            _transfer: transfer,
            _marker: PhantomData,
        }
    }

    /// Configure an ADC channel sequence once and return a [`ConfiguredSequence`] for repeated
    /// DMA reads without reprogramming the sequence each time.
    ///
    /// Use [`Adc::read_sequence`] instead if you only need a single one-shot transfer.
    ///
    /// # Parameters
    /// - `rx_dma`: The DMA channel to use for transfers.
    /// - `sequence`: Iterator of channels and sample times.
    ///
    /// # Returns
    /// A [`ConfiguredSequence`] whose [`read`](ConfiguredSequence::read) method triggers one
    /// DMA conversion of the pre-configured sequence per call.
    pub fn configure_sequence<'adc, 'ch, D: RxDma<T>>(
        &'adc mut self,
        rx_dma: Peri<'adc, D>,
        sequence: impl ExactSizeIterator<Item = (BorrowedAdcChannel<'ch, T>, SampleTimeOf<T>)>,
        irq: impl interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'd,
    ) -> ConfiguredSequence<'adc, T::Regs>
    where
        'ch: 'adc,
    {
        check_dma_len::<T>(sequence.len(), None);

        let len = sequence.len();

        let r = T::regs();
        r.stop();
        r.configure_sequence(
            sequence.map(|(channel, sample_time)| ((channel.channel, channel.is_differential), sample_time)),
            false,
        );
        r.enable();
        r.configure_dma(ConversionMode::Singular);

        ConfiguredSequence::new(self, rx_dma, len, irq)
    }

    /// Configures the ADC to use a DMA ring buffer for continuous data acquisition.
    ///
    /// Use the [`RingBufferedAdc::read`] method to retrieve measurements from the DMA ring buffer.
    /// The read buffer should be exactly half the size of `dma_buf`. When using triggered mode, it
    /// is recommended to configure `dma_buf` as a double buffer so that one half can be read while
    /// the other half is being filled by the DMA, preventing data loss. The trigger period of the
    /// ADC effectively defines the period at which the buffer should be read.
    ///
    /// If continuous conversion mode is selected (no trigger), the provided `dma_buf` must be
    /// large enough to prevent DMA buffer overruns. Its length should be a multiple of the number
    /// of ADC channels being measured. For example, if 3 channels are measured and you want to
    /// store 40 samples per channel, the buffer length should be `3 * 40 = 120`.
    ///
    /// Note: ADCs without a fully configurable sequencer (see [`read_sequence`](Self::read_sequence))
    /// restrict the channel order and sample times; this method panics if the hardware cannot
    /// deliver the requested sequence.
    pub fn into_ring_buffered<'a, 'ch, D: RxDma<T>>(
        self,
        dma: Peri<'a, D>,
        dma_buf: &'a mut [u16],
        irq: impl interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
        sequence: impl ExactSizeIterator<Item = (BorrowedAdcChannel<'ch, T>, SampleTimeOf<T>)>,
        trigger: Option<RegularAdcTrigger<T>>,
    ) -> RingBufferedAdc<'a, T::Regs> {
        let sequence_len = sequence.len();

        check_dma_len::<T>(sequence_len, Some(dma_buf.len()));

        let r = T::regs();
        r.stop();
        r.configure_sequence(
            sequence.map(|(channel, sample_time)| ((channel.channel, channel.is_differential), sample_time)),
            false,
        );
        r.enable();
        r.configure_dma(ConversionMode::Repeated(trigger.map(|t| (t.trigger, t.edge))));

        core::mem::forget(self);

        RingBufferedAdc::new(dma, irq, dma_buf, sequence_len)
    }

    fn enable_internal(&mut self, channel: InternalChannel) {
        T::regs().enable_internal(T::common(), channel, true);
    }
}

impl<'d, T: Instance, M: Mode> Drop for Adc<'d, T, M> {
    fn drop(&mut self) {
        T::regs().stop();
        T::regs().power_down();
        rcc::disable::<T>();
    }
}

/// Sample-time lookup for [`Adc::sample_time_for_us`].
pub trait SampleTimes: BasicAdcRegs {
    #[doc(hidden)]
    fn sample_time_for_half_cycles(half_cycles: u32) -> Self::SampleTime;
}

#[cfg(stm32n6)]
type Word = u32;
#[cfg(not(stm32n6))]
type Word = u16;

/// An ADC with a pre-configured channel sequence for repeated DMA to peripheral reads.
///
/// Just like [`Adc::configure_sequence`], this type programs the ADC channel sequence
/// registers. However, while `ConfiguredSequence` is targeted at ADC to mem transfers,
/// `ConfiguredTransfer` is designed for ADC to peripheral transfers such as to FMAC or CORDIC
///
/// Obtain via [`Adc::configure_transfer`].
#[allow(private_bounds)]
pub struct ConfiguredTransfer<'adc, R: AdcRegs> {
    _transfer: crate::dma::Transfer<'adc>,
    _marker: PhantomData<R>,
}

fn check_dma_len<T: Instance>(sequence_len: usize, dma_len: Option<usize>) {
    assert!(sequence_len != 0, "Asynchronous read sequence cannot be empty");
    assert!(
        sequence_len <= T::Regs::MAX_SEQUENCE_LEN,
        "Asynchronous read sequence cannot be more than {} in length",
        T::Regs::MAX_SEQUENCE_LEN
    );
    if let Some(dma_len) = dma_len {
        assert!(dma_len != 0 && dma_len <= 0xFFFF);
        assert!(
            dma_len % sequence_len == 0,
            "Readings length must be a multiple of sequence length"
        );
    }
}

// ----------------------------------------------------------------------------------------------
// Instances

macro_rules! impl_adc_instance {
    ($inst:ident, $regs:ty, $common:expr) => {
        impl crate::adc::BasicInstance for peripherals::$inst {
            type Regs = $regs;
        }

        impl crate::adc::SealedInstance for peripherals::$inst {
            fn regs() -> Self::Regs {
                crate::pac::$inst
            }

            fn common() -> <Self::Regs as crate::adc::AdcRegs>::Common {
                $common
            }

            fn state() -> &'static crate::adc::State {
                static STATE: crate::adc::State = crate::adc::State::new();
                &STATE
            }
        }

        impl crate::adc::Instance for peripherals::$inst {
            type Interrupt = crate::_generated::peripheral_interrupts::$inst::GLOBAL;
        }
    };
}

// Rows: (instance, common instance or `none`, register block type name, family)
foreach_adc!(
    ($inst:ident, $common:ident, $block:ident, v2) => {
        impl_adc_instance!($inst, crate::pac::adc::$block, ());
    };
    ($inst:ident, none, $block:ident, v1) => {
        impl_adc_instance!($inst, crate::pac::adc::$block, ());
    };
    ($inst:ident, $common:ident, $block:ident, v1) => {
        impl_adc_instance!($inst, crate::pac::adc::$block, crate::pac::$common);
    };
    ($inst:ident, none, $block:ident, v3) => {
        compile_error!("advanced ADC without a common register block");
    };
    ($inst:ident, $common:ident, $block:ident, v3) => {
        impl_adc_instance!($inst, crate::pac::adc::$block, crate::pac::$common);
    };
);
