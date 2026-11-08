//! One-shot and continuous frequency measurements of internal clock signals.
//!
//! Configure both input clocks through [`crate::init`] before constructing the
//! driver. This driver only routes already-enabled clocks; it does not change
//! the clock tree. Pulse-width measurement and external pins are not
//! supported. The configured power requirement controls the clock wake guard.
//!
//! Blocking measurements require running input clocks and have no timeout.
//! Async measurements may be wrapped in [`embassy_time::with_timeout`]; dropping
//! the future stops the measurement and masks/clears its interrupt source.

use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};

use embassy_hal_internal::drop::OnDrop;
use maitake_sync::WaitCell;

pub use crate::clocks::periph_helpers::FreqmeClockSel as ClockSource;
use crate::clocks::periph_helpers::{FreqmeConfig, FreqmeTargetClockSel, SPConfHelper};
use crate::clocks::{ClockError, Gate, PoweredClock, WakeGuard, enable_and_reset, with_clocks};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::freqme::CtrlWMeasureInProgress;
use crate::{Peri, PeripheralType, interrupt, pac, peripherals};

// Software completion tag; hardware RESULT occupies only the lower 31 bits.
const COMPLETED: u32 = 1 << 31;

/// Measurement window of 2^exponent reference-clock cycles.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReferenceScale(u8);

impl ReferenceScale {
    /// Select an exponent in 0..=31, or return `None` if it is out of range.
    pub const fn new(exponent: u8) -> Option<Self> {
        if exponent <= 31 { Some(Self(exponent)) } else { None }
    }

    /// Reference-clock cycles in this measurement window.
    pub const fn cycles(self) -> u32 {
        1u32 << self.0
    }

    const fn exponent(self) -> u8 {
        self.0
    }
}

// Reserve two counts for the SDK's correction. Use cross multiplication so
// fractional target cycles are not rounded down at the capacity boundary.
pub(crate) fn window_fits(reference_hz: u32, target_hz: u32, scale: ReferenceScale) -> bool {
    u64::from(target_hz) * u64::from(scale.cycles()) <= u64::from(0x7fff_fffdu32) * u64::from(reference_hz)
}

/// A raw measurement cannot be represented as a frequency in Hz.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConversionError {
    /// Raw count is below two; the SDK's subtraction would underflow.
    ResultTooSmall,
    /// Calculated frequency exceeds `u32::MAX` Hz.
    Overflow,
}

/// A completed frequency measurement, with its reference frequency and window.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Measurement {
    raw_count: u32,
    reference_hz: u32,
    scale: ReferenceScale,
}

impl Measurement {
    const fn new(raw_count: u32, reference_hz: u32, scale: ReferenceScale) -> Self {
        Self {
            raw_count,
            reference_hz,
            scale,
        }
    }

    /// Unmodified 31-bit hardware RESULT, before the SDK count correction.
    pub const fn raw_count(self) -> u32 {
        self.raw_count
    }

    /// Target frequency in Hz, rounded down using the SDK equation.
    pub fn frequency_hz(self) -> Result<u32, ConversionError> {
        let count = self.raw_count.checked_sub(2).ok_or(ConversionError::ResultTooSmall)?;
        let hz = u64::from(count) * u64::from(self.reference_hz) / u64::from(self.scale.cycles());
        u32::try_from(hz).map_err(|_| ConversionError::Overflow)
    }
}

/// One-shot and continuous frequency measurement configuration.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct Config {
    /// Required input-clock availability. Active-only operation retains a
    /// wake guard; `AlwaysEnabled` requires both inputs to run in deep sleep.
    pub power: PoweredClock,
    /// Reference clock; its configured frequency is used in Hz conversion.
    pub reference: ClockSource,
    /// Target signal to measure, independent of the reference-clock selection.
    pub target: FreqmeTargetClockSel,
    /// Reference window. Longer windows reduce count quantization error, but
    /// must not overflow the 31-bit target counter. Clock setup checks the
    /// nominal clock ratio with two counts of correction headroom. Leave
    /// additional margin for oscillator tolerance; this is not an overflow
    /// detector for the actual hardware counter.
    pub reference_scale: ReferenceScale,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            reference: ClockSource::Fro12m,
            target: FreqmeTargetClockSel::Fro12m,
            reference_scale: ReferenceScale::default(),
        }
    }
}

/// Driver setup or measurement error.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Clock setup failed or a selected input is not enabled.
    ClockSetup(ClockError),
    /// No measurement has completed since the last start, restart or stop.
    NotReady,
    /// The raw measurement is below the hardware MIN threshold.
    BelowMinimum,
    /// The raw measurement is above the hardware MAX threshold.
    AboveMaximum,
    /// The per-instance wait cell was closed.
    WaitClosed,
}

mod sealed {
    pub trait SealedMode {}
}

/// Sealed driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::SealedMode {}

/// Blocking measurement mode.
pub struct Blocking;
impl sealed::SealedMode for Blocking {}
impl Mode for Blocking {}

/// Interrupt-driven measurement mode.
pub struct Async;
impl sealed::SealedMode for Async {}
impl Mode for Async {}

/// Owned frequency measurement driver, independent of the peripheral instance.
pub struct Freqme<'d, M: Mode> {
    info: &'static Info,
    reference_hz: u32,
    scale: ReferenceScale,
    _wake_guard: Option<WakeGuard>,
    _phantom: PhantomData<&'d mut M>,
}

impl<'d, M: Mode> Freqme<'d, M> {
    fn new_inner<T: Instance>(_peri: Peri<'d, T>, config: Config) -> Result<Self, Error> {
        let clock_config = FreqmeConfig {
            power: config.power,
            reference: config.reference,
            target: config.target,
            reference_scale: config.reference_scale,
        };
        let info = T::info();
        let parts = critical_section::with(|_| {
            T::Interrupt::disable();
            // The clock helper validates and routes both inputs and manages
            // the source-power vote. Serialize shared MRCC gate updates.
            // SAFETY: the peripheral token proves exclusive ownership.
            let parts = unsafe { enable_and_reset::<T>(&clock_config) }.map_err(Error::ClockSetup)?;
            info.stop();
            Ok(parts)
        })?;
        Ok(Self {
            info,
            reference_hz: parts.freq,
            scale: config.reference_scale,
            _wake_guard: parts.wake_guard,
            _phantom: PhantomData,
        })
    }

    /// Start repeated measurements, replacing any previous measurement.
    ///
    /// Each result still covers the configured reference window. Poll
    /// [`Self::result`] for the most recently completed measurement; result
    /// reads do not stop counting. Peripheral interrupts remain disabled,
    /// regardless of the driver's blocking or async mode.
    pub fn continuous(&mut self) {
        self.info
            .start(self.scale, InterruptEnable::Disabled, MeasurementMode::Continuous);
    }

    /// Start one measurement without waiting, replacing any previous one.
    ///
    /// Poll [`Self::result`] for completion. For a blocking wait or an
    /// interrupt-driven wait, use [`Freqme::blocking_measure`] or
    /// [`Freqme::measure`] respectively. This polling operation leaves the
    /// peripheral interrupt disabled in either driver mode.
    pub fn oneshot(&mut self) {
        self.info
            .start(self.scale, InterruptEnable::Disabled, MeasurementMode::OneShot);
    }

    /// Read the latest completed measurement without consuming it.
    ///
    /// Returns [`Error::NotReady`] before the first completion after starting
    /// or restarting, and after [`Self::stop`]. The latest continuous result
    /// remains readable while the next measurement is in progress;
    /// successive reads need not return a new sample.
    ///
    /// Threshold errors take precedence over readiness and remain latched
    /// until stop/restart. Hardware suspends continuous operation on either
    /// threshold violation. If both flags are set, `BelowMinimum` is reported.
    pub fn result(&self) -> Result<Measurement, Error> {
        if !self.info.read_status()?.result_ready_stat() {
            return Err(Error::NotReady);
        }
        let raw_count = self.info.regs.ctrl_r().read().result();
        // A continuous window can complete between the status and result reads.
        // Recheck latched errors before accepting the newly captured result.
        self.info.read_status()?;
        Ok(Measurement::new(raw_count, self.reference_hz, self.scale))
    }

    /// Stop counting, mask peripheral interrupts and clear result readiness.
    pub fn stop(&mut self) {
        self.info.stop();
    }
}

impl<'d> Freqme<'d, Blocking> {
    /// Create a blocking driver; no interrupt binding is needed.
    pub fn new_blocking<T: Instance>(peri: Peri<'d, T>, config: Config) -> Result<Self, Error> {
        Self::new_inner(peri, config)
    }

    /// Start a fresh measurement and busy-wait for its result.
    ///
    /// Both input clocks must remain running. This method has no timeout.
    /// Any continuous measurement is stopped before the new one-shot starts.
    pub fn blocking_measure(&mut self) -> Result<Measurement, Error> {
        self.oneshot();
        loop {
            match self.result() {
                Err(Error::NotReady) => cortex_m::asm::nop(),
                result => {
                    self.stop();
                    return result;
                }
            }
        }
    }
}

impl<'d> Freqme<'d, Async> {
    /// Create an interrupt-driven driver with its FREQME interrupt binding.
    pub fn new_async<T: Instance>(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        let driver = Self::new_inner(peri, config)?;
        T::Interrupt::unpend();
        // SAFETY: the binding ensures this instance's handler is installed.
        unsafe { T::Interrupt::enable() };
        Ok(driver)
    }

    /// Start a fresh measurement and await its completion.
    ///
    /// Cancellation stops counting, masks interrupts and clears status. The
    /// retained hardware RESULT is never returned as a fresh result on reuse.
    /// Wrap this in a timeout if the input clocks might stop.
    /// Any continuous measurement is stopped before the new one-shot starts.
    pub async fn measure(&mut self) -> Result<Measurement, Error> {
        let _stop = OnDrop::new(|| self.info.stop());
        self.info
            .start(self.scale, InterruptEnable::Enabled, MeasurementMode::OneShot);
        self.info
            .wait_cell
            .wait_for(|| self.info.completed_result.load(Ordering::Acquire) & COMPLETED != 0)
            .await
            .map_err(|_| Error::WaitClosed)?;
        self.info.read_status()?;
        let raw_count = self.info.completed_result.load(Ordering::Acquire) & !COMPLETED;
        Ok(Measurement::new(raw_count, self.reference_hz, self.scale))
    }
}

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for Freqme<'d, M> {
    type Config = Config;
    type ConfigError = Error;

    /// Stop acquisition and apply new clock inputs, power policy and window.
    ///
    /// Acquisition remains stopped on success or failure. On error the last
    /// accepted routing, cached frequency/window and WakeGuard are retained.
    /// Start again with `oneshot`, `continuous` or a mode-specific wait method.
    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.stop();
        let clock_config = FreqmeConfig {
            power: config.power,
            reference: config.reference,
            target: config.target,
            reference_scale: config.reference_scale,
        };
        let parts = with_clocks(|clocks| clock_config.pre_enable_config(clocks))
            .ok_or(Error::ClockSetup(ClockError::NeverInitialized))?
            .map_err(Error::ClockSetup)?;
        self.reference_hz = parts.freq;
        self.scale = config.reference_scale;
        self._wake_guard = parts.wake_guard;
        Ok(())
    }
}

impl<M: Mode> Drop for Freqme<'_, M> {
    fn drop(&mut self) {
        // Quiesce before the clock helper's WakeGuard is released. Like the
        // other instance-erased drivers, leave gate policy to the clock layer.
        self.info.stop();
    }
}

trait SealedInstance: Gate<MrccPeriphConfig = FreqmeConfig> {
    fn info() -> &'static Info;
    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
}

/// FREQME peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Associated interrupt.
    type Interrupt: interrupt::typelevel::Interrupt;
}

enum InterruptEnable {
    Disabled,
    Enabled,
}

impl From<InterruptEnable> for bool {
    fn from(interrupt: InterruptEnable) -> Self {
        matches!(interrupt, InterruptEnable::Enabled)
    }
}

enum MeasurementMode {
    OneShot,
    Continuous,
}

impl From<MeasurementMode> for bool {
    fn from(mode: MeasurementMode) -> Self {
        matches!(mode, MeasurementMode::Continuous)
    }
}

struct Info {
    regs: pac::freqme::Freqme,
    wait_cell: WaitCell,
    completed_result: AtomicU32,
}

// SAFETY: register mutation is serialized by exclusive driver ownership and
// critical sections shared with the interrupt handler; WaitCell is Sync.
unsafe impl Sync for Info {}

impl Info {
    fn read_status(&self) -> Result<pac::freqme::Ctrlstat, Error> {
        let status = self.regs.ctrlstat().read();
        if status.lt_min_stat() {
            return Err(Error::BelowMinimum);
        }
        if status.gt_max_stat() {
            return Err(Error::AboveMaximum);
        }
        Ok(status)
    }

    fn stop(&self) {
        critical_section::with(|_| {
            // Do not assume this clears RESULT: hardware retains it on abort.
            self.regs.ctrl_w().write(|_| {});
            self.regs.ctrlstat().write(|w| {
                w.set_lt_min_stat(true);
                w.set_gt_max_stat(true);
                w.set_result_ready_stat(true);
            });
            cortex_m::asm::dsb();
            self.completed_result.store(0, Ordering::Release);
        });
    }

    fn start(&self, scale: ReferenceScale, interrupt: InterruptEnable, mode: MeasurementMode) {
        critical_section::with(|_| {
            self.stop();
            self.regs.ctrl_w().write(|w| {
                w.set_ref_scale(scale.exponent());
                w.set_result_ready_int_en(interrupt.into());
                w.set_continuous_mode_en(mode.into());
                w.set_measure_in_progress(CtrlWMeasureInProgress::from_bits(1));
            });
        });
    }
}

/// FREQME interrupt handler. Bind it to the instance's interrupt.
pub struct InterruptHandler<T: Instance>(PhantomData<T>);

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        critical_section::with(|_| {
            let info = T::info();
            // An active lower-priority ISR may resume after a higher-priority
            // task drops the driver. NVIC masking alone cannot stop it. Check
            // the gate inside the same CS as teardown before accessing MMIO.
            if !T::is_clock_enabled() {
                T::Interrupt::disable();
                return;
            }
            let status = info.regs.ctrlstat().read();
            if status.result_ready_stat() && status.result_ready_int_en() {
                // Capture RESULT before masking via CTRL_W: a zero write is
                // documented to reset RESULT, so never read it after masking.
                // CTRLSTAT's configuration fields are only readback mirrors.
                let result = info.regs.ctrl_r().read().result();
                info.regs.ctrl_w().write(|_| {});
                info.completed_result.store(COMPLETED | result, Ordering::Release);
                T::PERF_INT_WAKE_INCR();
                info.wait_cell.wake();
            }
        });
    }
}

impl SealedInstance for peripherals::FREQME0 {
    fn info() -> &'static Info {
        static INFO: Info = Info {
            regs: pac::FREQME0,
            wait_cell: WaitCell::new(),
            completed_result: AtomicU32::new(0),
        };
        &INFO
    }

    const PERF_INT_INCR: fn() = crate::perf_counters::incr_interrupt_freqme0;
    const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::incr_interrupt_freqme0_wake;
}

impl Instance for peripherals::FREQME0 {
    type Interrupt = interrupt::typelevel::FREQME0;
}
