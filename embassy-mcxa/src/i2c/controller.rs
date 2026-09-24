//! # LPI2C Controller Driver
//!
//! This module provides a driver for the Low-Power Inter-Integrated
//! Circuit (LPI2C) controller, supporting blocking,
//! interrupt-only async, and DMA async modes of operation.
//!
//! The driver supports Standard, Fast, and Fast Plus modes.
//!
//! ## Features
//!
//! - **Blocking and Asynchronous Modes**: Supports both blocking and
//!   async APIs for flexibility in different runtime environments.
//! - **DMA Support**: Enables high-performance data transfers using
//!   DMA.
//! - **Configurable Bus Speeds**: Supports standard (100 kHz), fast
//!   (400 kHz), and fast-plus (1 MHz) modes. Ultra-fast (3.4 MHz) mode
//!   is not yet implemented.
//! - **Error Handling**: Comprehensive error reporting, including
//!   FIFO errors, arbitration loss, and address NACK conditions.
//! - **Embedded HAL Compatibility**: Implements traits from
//!   `embedded-hal` and `embedded-hal-async` for interoperability with
//!   other libraries.
//!
//! ### Error Types
//!
//! - `SetupError`: Errors related to hardware initialization, such as
//!   clock configuration issues.
//! - `IOError`: Errors during I2C operations, including FIFO errors,
//!   arbitration loss, and invalid buffer lengths.
//!
//! ## Example
//!
//! ```rust,no_run
//! #![no_std]
//! #![no_main]
//!
//! # extern crate panic_halt;
//! # extern crate embassy_mcxa;
//! # extern crate embassy_executor;
//! # use panic_halt as _;
//! use embassy_executor::Spawner;
//! use embassy_mcxa::clocks::config::Div8;
//! use embassy_mcxa::config::Config;
//! use embassy_mcxa::i2c::controller::{self, I2c, Speed};
//!
//! #[embassy_executor::main]
//! async fn main(_spawner: Spawner) {
//!     let mut config = Config::default();
//!     config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);
//!
//!     let p = embassy_mcxa::init(config);
//!
//!     let mut i2c = I2c::new_blocking(p.LPI2C2, p.P1_9, p.P1_8, Default::default()).unwrap();
//!
//!     // Write data
//!     i2c.blocking_write(0x50, &[0x01, 0x02, 0x03]).unwrap();
//!
//!     // Read data
//!     let mut buffer = [0u8; 3];
//!     i2c.blocking_read(0x50, &mut buffer).unwrap();
//! }
//! ```

use core::future::Future;
use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;

use super::{Async, AsyncMode, Blocking, Dma, Info, Instance, Mode, SclPin, SdaPin};
use crate::clocks::periph_helpers::{Div4, Lpi2cClockSel, Lpi2cConfig};
use crate::clocks::{ClockError, PoweredClock, WakeGuard, enable_and_reset};
use crate::dma::{Channel, DMA_MAX_TRANSFER_SIZE, DmaChannel, TransferOptions};
use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::lpi2c::{Alf, Cmd, Dmf, Dozen, Epf, McrRrf, McrRtf, Msr, MsrFef, MsrSdf, Ndf, Pltf, Prescale, Stf};

/// Errors exclusive to HW initialization
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Clock configuration error.
    ClockSetup(ClockError),
    /// The requested duty cycle does not meet the selected bus mode's minimum tHIGH.
    InvalidDutyCycle,
    /// The selected bus speed is not implemented by this driver.
    UnsupportedSpeed,
    /// The peripheral clock cannot produce the requested timing within the supported error bound.
    BaudrateNotAchievable,
    /// Other internal errors or unexpected state.
    Other,
}

/// I/O Errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum IOError {
    /// FIFO Error, the command in the FIFO queue expected the controller to be in a STARTed state, but it was not.
    ///
    /// Even though a START could have been issued earlier, the controller might now be in a different state.
    /// For example, a NAK condition was detected and the controller automatically issued a STOP.
    FifoError,
    /// Reading for I2C failed.
    ReadFail,
    /// Writing to I2C failed.
    WriteFail,
    /// I2C address NAK condition.
    AddressNack,
    /// Bus level arbitration loss.
    ArbitrationLoss,
    /// Address out of range.
    AddressOutOfRange(u8),
    /// Invalid write buffer length.
    InvalidWriteBufferLength,
    /// Invalid read buffer length.
    InvalidReadBufferLength,
    /// Other internal errors or unexpected state.
    Other,
}

impl From<crate::dma::InvalidParameters> for IOError {
    fn from(_value: crate::dma::InvalidParameters) -> Self {
        IOError::Other
    }
}

/// I2C interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        if T::info().regs().mier().read().0 != 0 {
            T::info().regs().mier().write(|w| {
                w.set_tdie(false);
                w.set_rdie(false);
                w.set_epie(false);
                w.set_sdie(false);
                w.set_ndie(false);
                w.set_alie(false);
                w.set_feie(false);
                w.set_pltie(false);
                w.set_dmie(false);
                w.set_stie(false);
            });

            T::PERF_INT_WAKE_INCR();
            T::info().wait_cell().wake();
        }
    }
}

/// Bus speed (nominal SCL, no clock stretching)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Speed {
    #[default]
    /// 100 kbit/sec
    Standard,
    /// 400 kbit/sec
    Fast,
    /// 1 Mbit/sec
    FastPlus,
    /// 3.4 Mbit/sec
    UltraFast,
}

impl From<Speed> for u32 {
    fn from(val: Speed) -> Self {
        match val {
            Speed::Standard => 100_000,
            Speed::Fast => 400_000,
            Speed::FastPlus => 1_000_000,
            Speed::UltraFast => 3_400_000,
        }
    }
}

impl Speed {
    const fn minimum_high_time_ns(self) -> Option<u32> {
        match self {
            Self::Standard => Some(4_000),
            Self::Fast => Some(600),
            Self::FastPlus => Some(260),
            Self::UltraFast => None,
        }
    }
}

/// SCL duty cycle: the share of the SCL period the clock is driven high.
///
/// CLKHI/CLKLO are 6-bit counters, and the high phase is additionally capped
/// by the tBUF clamp below, so the achieved ratio is best effort and lands at
/// ~48% for any request at or near [`MAX_PERCENT`](Self::MAX_PERCENT).
///
/// The minimum accepted value depends on [`Speed`]. The combination is
/// validated when the controller configuration is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DutyCycle {
    high_percent: u8,
}

impl DutyCycle {
    /// Minimum percentage accepted by [`new`](Self::new).
    pub const MIN_PERCENT: u8 = 24;
    /// Minimum percentage for Standard mode (4.0 us of a 10 us period).
    pub const STANDARD_MIN_PERCENT: u8 = 40;
    /// Minimum percentage for Fast mode (0.6 us of a 2.5 us period).
    pub const FAST_MIN_PERCENT: u8 = 24;
    /// Minimum percentage for Fast Plus mode (0.26 us of a 1 us period).
    pub const FAST_PLUS_MIN_PERCENT: u8 = 26;
    /// Largest accepted value: past this the tBUF clamp caps the high phase anyway.
    pub const MAX_PERCENT: u8 = 50;

    /// Creates a duty cycle from the percentage of the SCL period spent high.
    ///
    /// This validates the range representable by the driver. The configured
    /// [`Speed`] imposes a mode-specific minimum that is checked when the
    /// configuration is applied.
    ///
    /// Returns `None` if the value is outside
    /// [`MIN_PERCENT`](Self::MIN_PERCENT)..=[`MAX_PERCENT`](Self::MAX_PERCENT).
    pub const fn new(high_percent: u8) -> Option<Self> {
        if high_percent < Self::MIN_PERCENT || high_percent > Self::MAX_PERCENT {
            None
        } else {
            Some(Self { high_percent })
        }
    }

    /// Requested percentage of the SCL period spent high.
    pub const fn high_percent(&self) -> u8 {
        self.high_percent
    }

    /// Minimum accepted percentage for a supported bus speed.
    ///
    /// Returns `None` for [`Speed::UltraFast`], which is not implemented.
    pub const fn minimum_for_speed(speed: Speed) -> Option<u8> {
        match speed {
            Speed::Standard => Some(Self::STANDARD_MIN_PERCENT),
            Speed::Fast => Some(Self::FAST_MIN_PERCENT),
            Speed::FastPlus => Some(Self::FAST_PLUS_MIN_PERCENT),
            Speed::UltraFast => None,
        }
    }

    const fn is_valid_for(self, speed: Speed) -> bool {
        match Self::minimum_for_speed(speed) {
            Some(minimum) => self.high_percent >= minimum && self.high_percent <= Self::MAX_PERCENT,
            None => false,
        }
    }
}

impl Default for DutyCycle {
    /// Nominally even split; the tBUF clamp lands it at approximately 48%.
    fn default() -> Self {
        Self { high_percent: 50 }
    }
}

/// Nominal timing currently programmed into the controller.
///
/// Physical timing can differ due to SCL rise/fall time and clock stretching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConfiguredTiming {
    /// Nominal SCL frequency in hertz.
    pub frequency_hz: u32,
    /// Nominal share of the SCL period spent high.
    pub high_percent: u8,
}

/// Largest value the 6-bit MCCR0 CLKHI/CLKLO counters can hold.
const MAX_CLK_COUNT: u32 = 0x3F;

/// MCFGR2.FILTSCL value programmed by this driver.
const SCL_FILTER_CYCLES: u32 = 0;

/// Fixed internal cycles in addition to MCFGR2.FILTSCL.
const SCL_LATENCY_BASE_CYCLES: u32 = 2;

/// Maximum additional baud error accepted to make a duty cycle fit.
const MAX_DUTY_BAUD_DEGRADATION_PERCENT: u32 = 1;

#[derive(Clone, Copy)]
struct BaudCandidate {
    prescale: Prescale,
    clk_cycle: u32,
}

impl BaudCandidate {
    const fn divider(self) -> u32 {
        1u32 << (self.prescale as u8)
    }

    const fn scl_latency(self) -> u32 {
        (SCL_LATENCY_BASE_CYCLES + SCL_FILTER_CYCLES) / self.divider()
    }

    const fn period_cycles(self) -> u32 {
        self.clk_cycle + 2 + self.scl_latency()
    }

    /// Convert the requested physical high share to CLKHI. SCL latency is
    /// part of tHIGH on LPI2C, so it is subtracted from the register value.
    const fn requested_clk_high(self, duty_cycle: DutyCycle) -> u32 {
        let high_cycles = self.period_cycles() * duty_cycle.high_percent() as u32 / 100;
        high_cycles.saturating_sub(1 + self.scl_latency())
    }
}

#[derive(Clone, Copy)]
struct BaudParams {
    prescale: Prescale,
    clklo: u8,
    clkhi: u8,
    sethold: u8,
    datavd: u8,
}

impl BaudParams {
    const fn divider(self) -> u32 {
        1u32 << (self.prescale as u8)
    }

    const fn scl_latency(self) -> u32 {
        (SCL_LATENCY_BASE_CYCLES + SCL_FILTER_CYCLES) / self.divider()
    }

    const fn high_cycles(self) -> u32 {
        self.clkhi as u32 + 1 + self.scl_latency()
    }

    fn meets_minimum_high_time(self, src_hz: u32, speed: Speed) -> bool {
        let Some(minimum_ns) = speed.minimum_high_time_ns() else {
            return false;
        };

        u64::from(self.high_cycles()) * u64::from(self.divider()) * 1_000_000_000
            >= u64::from(minimum_ns) * u64::from(src_hz)
    }

    #[cfg(test)]
    fn configured_timing(self, src_hz: u32) -> ConfiguredTiming {
        configured_timing(src_hz, self.prescale, SCL_FILTER_CYCLES as u8, self.clklo, self.clkhi)
    }
}

fn configured_timing(src_hz: u32, prescale: Prescale, filter_cycles: u8, clklo: u8, clkhi: u8) -> ConfiguredTiming {
    let divider = 1u32 << (prescale as u8);
    let scl_latency = (SCL_LATENCY_BASE_CYCLES + u32::from(filter_cycles)) / divider;
    let period_cycles = u32::from(clklo) + u32::from(clkhi) + 2 + scl_latency;
    let high_cycles = u32::from(clkhi) + 1 + scl_latency;

    ConfiguredTiming {
        frequency_hz: (src_hz / divider) / period_cycles,
        high_percent: ((high_cycles * 100 + period_cycles / 2) / period_cycles) as u8,
    }
}

fn minimum_clk_high(src_hz: u32, speed: Speed, candidate: BaudCandidate) -> Option<u32> {
    let minimum_ns = u64::from(speed.minimum_high_time_ns()?);
    let denominator = 1_000_000_000u64 * u64::from(candidate.divider());
    let high_cycles = (minimum_ns * u64::from(src_hz)).div_ceil(denominator);

    Some((high_cycles as u32).saturating_sub(1 + candidate.scl_latency()))
}

fn params_for_candidate(
    src_hz: u32,
    speed: Speed,
    duty_cycle: DutyCycle,
    candidate: BaudCandidate,
) -> Option<BaudParams> {
    let baud_hz: u32 = speed.into();
    let divider = candidate.divider();
    let requested_high = candidate.requested_clk_high(duty_cycle);
    let minimum_high = minimum_clk_high(src_hz, speed, candidate)?;

    // Preserve tBUF >= 0.52 * SCL period, matching the NXP SDK bound.
    let tbuf_cycles = (13u64 * u64::from(src_hz) / u64::from(baud_hz) / u64::from(divider) / 25) as u32;
    let maximum_high = candidate.clk_cycle.saturating_sub(tbuf_cycles).saturating_add(1);
    let clk_high = requested_high.max(minimum_high).min(maximum_high);
    let clk_low = candidate.clk_cycle.checked_sub(clk_high)?;

    if clk_high > MAX_CLK_COUNT || clk_low > MAX_CLK_COUNT {
        return None;
    }

    let clk_bdr = src_hz / baud_hz;
    let tmp_hold = (clk_bdr / divider / 2).saturating_sub(1);
    let tmp_datavd = (clk_bdr / divider / 4).saturating_sub(1);
    let params = BaudParams {
        prescale: candidate.prescale,
        clklo: clk_low as u8,
        clkhi: clk_high as u8,
        sethold: tmp_hold.min(MAX_CLK_COUNT) as u8,
        datavd: tmp_datavd.min(MAX_CLK_COUNT) as u8,
    };

    params.meets_minimum_high_time(src_hz, speed).then_some(params)
}

/// Compute LPI2C controller MCFGR1.PRESCALE + MCCR0 fields from peripheral
/// input frequency, target SCL frequency and target duty cycle.
///
/// Based on the NXP SDK `LPI2C_MasterSetBaudRate` algorithm
/// (see `fsl_lpi2c.c`). For each prescaler 0..=7, computes the period
/// in periph cycles using round-to-nearest division and keeps the smallest
/// absolute error to the target, preferring prescalers whose cycle budget can
/// express the requested duty cycle within the 6-bit counters. A candidate is
/// rejected if accommodating the duty cycle adds more than 1% baud error.
/// Then derives:
///   - CLKHI from the complete SCL period, including SCL latency, capped so
///     tBUF >= 0.52/baud and raised as needed to meet the mode's minimum tHIGH.
///   - CLKLO = clkCycle - CLKHI.
///   - SETHOLD = clk_bdr/divider/2 - 1   (~half SCL period).
///   - DATAVD  = clk_bdr/divider/4 - 1   (~quarter SCL period).
fn compute_baud_params(src_hz: u32, speed: Speed, duty_cycle: DutyCycle) -> Result<BaudParams, SetupError> {
    if speed == Speed::UltraFast {
        return Err(SetupError::UnsupportedSpeed);
    }
    if !duty_cycle.is_valid_for(speed) {
        return Err(SetupError::InvalidDutyCycle);
    }
    if src_hz == 0 {
        return Err(SetupError::BaudrateNotAchievable);
    }

    let baud_hz: u32 = speed.into();
    let prescalers = [
        Prescale::DivideBy1,
        Prescale::DivideBy2,
        Prescale::DivideBy4,
        Prescale::DivideBy8,
        Prescale::DivideBy16,
        Prescale::DivideBy32,
        Prescale::DivideBy64,
        Prescale::DivideBy128,
    ];

    let mut best_err = u32::MAX;
    let mut found_candidate = false;
    let mut best_fit = None;
    let mut best_fit_err = u32::MAX;

    for &prescale in &prescalers {
        let divider: u32 = 1u32 << (prescale as u8);
        let scl_lat = (SCL_LATENCY_BASE_CYCLES + SCL_FILTER_CYCLES) / divider;

        // a = round(src / divider / baud)
        let a = ((10u64 * u64::from(src_hz) / u64::from(divider) / u64::from(baud_hz) + 5) / 10) as u32;
        let b = scl_lat + 2;
        if a <= b {
            continue;
        }
        let clk_cycle = a - b;
        if clk_cycle > 120u32.saturating_sub(scl_lat) {
            continue;
        }

        let computed = (src_hz / divider) / (clk_cycle + 2 + scl_lat);
        let abs_err = computed.abs_diff(baud_hz);
        found_candidate = true;

        if abs_err < best_err {
            best_err = abs_err;
        }

        let candidate = BaudCandidate { prescale, clk_cycle };
        if let Some(params) = params_for_candidate(src_hz, speed, duty_cycle, candidate)
            && abs_err < best_fit_err
        {
            best_fit_err = abs_err;
            best_fit = Some(params);
        }
    }

    if !found_candidate {
        return Err(SetupError::BaudrateNotAchievable);
    }

    let params = best_fit.ok_or(SetupError::BaudrateNotAchievable)?;
    let added_error = best_fit_err.saturating_sub(best_err);
    if u64::from(added_error) * 100 > u64::from(baud_hz) * u64::from(MAX_DUTY_BAUD_DEGRADATION_PERCENT) {
        return Err(SetupError::BaudrateNotAchievable);
    }

    Ok(params)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum SendStop {
    No,
    Yes,
}

/// I2C controller configuration
#[derive(Clone, Copy, Default)]
#[non_exhaustive]
pub struct Config {
    /// Bus speed
    pub speed: Speed,

    /// Share of the SCL period the clock is driven high (best effort).
    ///
    /// Must be at least [`DutyCycle::minimum_for_speed`] for [`Self::speed`].
    pub duty_cycle: DutyCycle,

    /// Clock configuration
    pub clock_config: ClockConfig,
}

/// I2C controller clock configuration
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct ClockConfig {
    /// Powered clock configuration
    pub power: PoweredClock,
    /// LPI2C clock source
    pub source: Lpi2cClockSel,
    /// LPI2C pre-divider
    pub div: Div4,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: Lpi2cClockSel::FroLfDiv,
            div: const { Div4::no_div() },
        }
    }
}

/// I2C Controller Driver.
pub struct I2c<'d, M: Mode> {
    info: &'static Info,
    _scl: Peri<'d, AnyPin>,
    _sda: Peri<'d, AnyPin>,
    mode: M,
    is_hs: bool,
    /// Peripheral input clock frequency in Hz, captured at construction.
    /// Used to compute MCCR0 timing parameters (e.g. when [`set_config`]
    /// changes the bus speed).
    freq: u32,
    _wg: Option<WakeGuard>,
}

impl<'d> I2c<'d, Blocking> {
    /// Creates a new blocking instance of the I2C Controller bus driver.
    ///
    /// This method initializes the I2C controller in blocking mode, allowing
    /// synchronous read and write operations.  The I2C bus is configured based
    /// on the provided `Config` structure, which specifies parameters such as
    /// bus speed and clock settings.
    ///
    /// # Arguments
    ///
    /// - `peri`: The peripheral instance representing the I2C controller hardware.
    /// - `scl`: The pin to be used for the I2C clock line (SCL).
    /// - `sda`: The pin to be used for the I2C data line (SDA).
    /// - `config`: A `Config` structure specifying the desired I2C configuration, including bus speed and clock settings.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: A new instance of the I2C driver in blocking mode if initialization is successful.
    /// - `Err(SetupError)`: An error if the initialization fails, such as due to invalid clock configuration.
    ///
    /// # Behavior
    ///
    /// - The I2C controller is configured and enabled based on the provided `Config`.
    /// - Any external pins used for SCL and SDA will be placed into a disabled state when the driver instance is dropped.
    ///
    /// # Errors
    ///
    /// - `SetupError::ClockSetup`: If there is an issue with the clock configuration.
    /// - `SetupError::InvalidDutyCycle`: If the duty cycle violates the selected mode's minimum tHIGH.
    /// - `SetupError::UnsupportedSpeed`: If UltraFast mode is selected.
    /// - `SetupError::BaudrateNotAchievable`: If the requested timing cannot be represented.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
    ) -> Result<Self, SetupError> {
        Self::new_inner(peri, scl, sda, config, Blocking)
    }
}

impl<'d, M: Mode> I2c<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
        mode: M,
    ) -> Result<Self, SetupError> {
        let ClockConfig { power, source, div } = config.clock_config;

        // Enable clocks
        let conf = Lpi2cConfig {
            power,
            source,
            div,
            instance: T::CLOCK_INSTANCE,
        };

        let parts = unsafe { enable_and_reset::<T>(&conf).map_err(SetupError::ClockSetup)? };

        scl.mux();
        sda.mux();

        let _scl = scl.into();
        let _sda = sda.into();

        let mut inst = Self {
            info: T::info(),
            _scl,
            _sda,
            mode,
            is_hs: config.speed == Speed::UltraFast,
            freq: parts.freq,
            _wg: parts.wake_guard,
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    /// Returns the nominal frequency and high-phase share currently programmed.
    ///
    /// This reads the hardware registers, so it also reflects configurations
    /// applied through [`embassy_embedded_hal::SetConfig`].
    pub fn configured_timing(&self) -> ConfiguredTiming {
        let mcfgr1 = self.info.regs().mcfgr1().read();
        let mcfgr2 = self.info.regs().mcfgr2().read();
        let mccr0 = self.info.regs().mccr0().read();

        configured_timing(
            self.freq,
            mcfgr1.prescale(),
            mcfgr2.filtscl(),
            mccr0.clklo(),
            mccr0.clkhi(),
        )
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), SetupError> {
        let params = compute_baud_params(self.freq, config.speed, config.duty_cycle)?;

        // Disable the controller.
        critical_section::with(|_| self.info.regs().mcr().modify(|w| w.set_men(false)));

        // Soft-reset the controller, read and write FIFOs.
        self.reset_fifos();
        critical_section::with(|_| {
            self.info.regs().mcr().modify(|w| w.set_rst(true));
            // According to Reference Manual section 40.7.1.4, "There
            // is no minimum delay required before clearing the
            // software reset", therefore we clear it immediately.
            self.info.regs().mcr().modify(|w| w.set_rst(false));

            self.info.regs().mcr().modify(|w| {
                w.set_dozen(Dozen::Enabled);
                w.set_dbgen(false);
            });
        });

        critical_section::with(|_| {
            // The timing calculation assumes FILTSCL=0, so program it
            // explicitly rather than relying on the reset value.
            self.info
                .regs()
                .mcfgr2()
                .modify(|w| w.set_filtscl(SCL_FILTER_CYCLES as u8));
            self.info.regs().mcfgr1().modify(|w| w.set_prescale(params.prescale));
            self.info.regs().mccr0().modify(|w| {
                w.set_clklo(params.clklo);
                w.set_clkhi(params.clkhi);
                w.set_sethold(params.sethold);
                w.set_datavd(params.datavd);
            });

            // Enable the controller.
            self.info.regs().mcr().modify(|w| w.set_men(true));
        });

        // Clear all flags
        self.info.regs().msr().write(|w| {
            w.set_epf(Epf::IntYes);
            w.set_sdf(MsrSdf::IntYes);
            w.set_ndf(Ndf::IntYes);
            w.set_alf(Alf::IntYes);
            w.set_fef(MsrFef::IntYes);
            w.set_pltf(Pltf::IntYes);
            w.set_dmf(Dmf::IntYes);
            w.set_stf(Stf::IntYes);
        });

        Ok(())
    }

    fn remediation(&self) {
        #[cfg(feature = "defmt")]
        defmt::trace!("Future dropped, recovering controller",);

        // Send a STOP. After an address NACK with empty TX FIFO and
        // autostop disabled, this releases the bus.
        //
        // Important: `stop()` busy-waits on `is_tx_fifo_empty_or_error`,
        // which returns true on *any* error (e.g., FEF raised because
        // the master was already in idle when STOP was queued). In
        // that case the STOP command may still be sitting in the TX
        // FIFO, ready to confuse the next transaction. Always reset
        // the FIFOs after the STOP attempt to guarantee a clean slate.
        let _ = self.stop();
        self.reset_fifos();

        // Clear any residual MSR flags raised by the recovery STOP
        // (FEF in particular) so the next transaction starts clean.
        let msr = self.info.regs().msr().read();
        self.info.regs().msr().write(|w| *w = msr);
    }

    /// Resets both TX and RX FIFOs dropping their contents.
    fn reset_fifos(&self) {
        critical_section::with(|_| {
            self.info.regs().mcr().modify(|w| {
                w.set_rtf(McrRtf::Reset);
                w.set_rrf(McrRrf::Reset);
            });
        });
    }

    /// Recover from an I2C error by resetting FIFOs and clearing all
    /// status flags.  Without this, a NACK or FIFO error leaves the
    /// LPI2C controller in a state where every subsequent transaction
    /// fails with FifoError.
    fn recover_from_error(&self) {
        self.reset_fifos();
        self.info.regs().msr().write(|w| {
            w.set_epf(Epf::IntYes);
            w.set_sdf(MsrSdf::IntYes);
            w.set_ndf(Ndf::IntYes);
            w.set_alf(Alf::IntYes);
            w.set_fef(MsrFef::IntYes);
            w.set_pltf(Pltf::IntYes);
            w.set_dmf(Dmf::IntYes);
            w.set_stf(Stf::IntYes);
        });
    }

    /// Checks whether the TX FIFO is full
    fn is_tx_fifo_full(&self) -> bool {
        let txfifo_size = 1 << self.info.regs().param().read().mtxfifo();
        self.info.regs().mfsr().read().txcount() == txfifo_size
    }

    /// Checks whether the TX FIFO is empty
    fn is_tx_fifo_empty(&self) -> bool {
        self.info.regs().mfsr().read().txcount() == 0
    }

    /// Checks whether the TX FIFO or if there is an error condition active.
    fn is_tx_fifo_empty_or_error(&self) -> bool {
        self.is_tx_fifo_empty() || self.status().is_err()
    }

    /// Checks whether the RX FIFO is empty.
    fn is_rx_fifo_empty(&self) -> bool {
        self.info.regs().mfsr().read().rxcount() == 0
    }

    /// Parses the controller status producing an
    /// appropriate `Result<(), Error>` variant.
    fn parse_status(&self, msr: &Msr) -> Result<(), IOError> {
        if msr.ndf() == Ndf::IntYes {
            Err(IOError::AddressNack)
        } else if msr.alf() == Alf::IntYes {
            Err(IOError::ArbitrationLoss)
        } else if msr.fef() == MsrFef::IntYes {
            Err(IOError::FifoError)
        } else {
            Ok(())
        }
    }

    /// Reads, parses and clears the controller status producing an
    /// appropriate `Result<(), Error>` variant.
    ///
    /// Will also send a STOP command if the tx_fifo is empty.
    fn status_and_act(&self) -> Result<(), IOError> {
        let msr = self.info.regs().msr().read();
        self.info.regs().msr().write(|w| *w = msr);

        let status = self.parse_status(&msr);

        if let Err(IOError::AddressNack) = status {
            // According to the Reference Manual, section 40.7.1.5
            // Controller Status (MSR), the controller will
            // automatically send a STOP condition if
            // `MCFGR1[AUTOSTOP]` is enabled or if the transmit FIFO
            // is *not* empty.
            //
            // If neither of those conditions is true, we will send a
            // STOP ourselves.
            if !self.info.regs().mcfgr1().read().autostop() && self.is_tx_fifo_empty() {
                self.remediation();
            }
        }

        status
    }

    /// Reads and parses the controller status producing an
    /// appropriate `Result<(), Error>` variant.
    fn status(&self) -> Result<(), IOError> {
        self.parse_status(&self.info.regs().msr().read())
    }

    /// Inserts the given command into the outgoing FIFO.
    ///
    /// Caller must ensure there is space in the FIFO for the new
    /// command.
    fn send_cmd(&self, cmd: Cmd, data: u8) {
        #[cfg(feature = "defmt")]
        defmt::trace!(
            "Sending cmd '{}' ({}) with data '{:08x}' MSR: {:08x}",
            cmd,
            cmd as u8,
            data,
            self.info.regs().msr().read()
        );

        self.info.regs().mtdr().write(|w| {
            w.set_data(data);
            w.set_cmd(cmd);
        });
    }

    /// Prepares an appropriate Start condition on bus by issuing a
    /// `Start` command together with the device address and R/w bit.
    ///
    /// Blocks waiting for space in the FIFO to become available, then
    /// sends the command and blocks waiting for the FIFO to become
    /// empty ensuring the command was sent.
    fn start(&self, address: u8, read: bool) -> Result<(), IOError> {
        if address >= 0x80 {
            return Err(IOError::AddressOutOfRange(address));
        }

        // Wait until we have space in the TxFIFO
        while self.is_tx_fifo_full() {}

        let addr_rw = address << 1 | if read { 1 } else { 0 };
        self.send_cmd(if self.is_hs { Cmd::START_HS } else { Cmd::START }, addr_rw);

        // Wait for TxFIFO to be drained
        while !self.is_tx_fifo_empty_or_error() {}

        // Check controller status
        self.status_and_act()
    }

    /// Prepares a Stop condition on the bus.
    ///
    /// Analogous to `start`, this blocks waiting for space in the
    /// FIFO to become available, then sends the command and blocks
    /// waiting for the FIFO to become empty ensuring the command was
    /// sent.
    fn stop(&self) -> Result<(), IOError> {
        // Wait until we have space in the TxFIFO
        while self.is_tx_fifo_full() {}

        self.send_cmd(Cmd::STOP, 0);

        // Wait for TxFIFO to be drained
        while !self.is_tx_fifo_empty_or_error() {}

        self.status_and_act()
    }

    fn blocking_read_internal(&self, address: u8, read: &mut [u8], send_stop: SendStop) -> Result<(), IOError> {
        if read.is_empty() {
            return Err(IOError::InvalidReadBufferLength);
        }

        for chunk in read.chunks_mut(256) {
            self.start(address, true)?;

            // Wait until we have space in the TxFIFO
            while self.is_tx_fifo_full() {}

            self.send_cmd(Cmd::RECEIVE, (chunk.len() - 1) as u8);

            for byte in chunk.iter_mut() {
                // Wait until there's data in the RxFIFO
                while self.is_rx_fifo_empty() {}

                *byte = self.info.regs().mrdr().read().data();
            }
        }

        if send_stop == SendStop::Yes {
            self.stop()?;
        }

        Ok(())
    }

    fn blocking_write_internal(&self, address: u8, write: &[u8], send_stop: SendStop) -> Result<(), IOError> {
        self.start(address, false)?;

        // Usually, embassy HALs error out with an empty write,
        // however empty writes are useful for writing I2C scanning
        // logic through write probing. That is, we send a start with
        // R/w bit cleared, but instead of writing any data, just send
        // the stop onto the bus. This has the effect of checking if
        // the resulting address got an ACK but causing no
        // side-effects to the device on the other end.
        //
        // Because of this, we are not going to error out in case of
        // empty writes.
        if write.is_empty() {
            #[cfg(feature = "defmt")]
            defmt::trace!("Empty write, write probing?");
            if send_stop == SendStop::Yes {
                self.stop()?;
            }
            return Ok(());
        }

        for byte in write {
            // Wait until we have space in the TxFIFO
            while self.is_tx_fifo_full() {}

            self.send_cmd(Cmd::TRANSMIT, *byte);
        }

        if send_stop == SendStop::Yes {
            self.stop()?;
        }

        Ok(())
    }

    // Public API: Blocking

    /// Reads data from the specified I2C address into the provided buffer.
    ///
    /// This method blocks the caller until the operation is complete.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `read`: A mutable buffer to store the data read from the device.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the read operation is successful.
    /// - `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    ///
    /// # Notes
    ///
    /// The driver will attempt to fill the buffer with data. If the
    /// buffer length exceeds the maximum transfer size of the
    /// controller, the read operation will be performed in multiple
    /// chunks. This will be transparent to the caller.
    pub fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), IOError> {
        self.blocking_read_internal(address, read, SendStop::Yes)
    }

    /// Writes data to the specified I2C address from the provided buffer.
    ///
    /// This method blocks the caller until the operation is complete.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `write`: A buffer containing the data to be written to the device.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the write operation is successful.
    /// - `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    pub fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<(), IOError> {
        self.blocking_write_internal(address, write, SendStop::Yes)
    }

    /// Performs a combined write and read operation on the specified I2C
    /// address.
    ///
    /// This method first writes data to the device, then reads data from the
    /// device into the provided buffer.  The caller is blocked until the
    /// operation is complete.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `write`: A buffer containing the data to be written to the device.
    /// - `read`: A mutable buffer to store the data read from the device.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the write-read operation is successful.
    /// - `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), IOError> {
        self.blocking_write_internal(address, write, SendStop::No)?;
        self.blocking_read_internal(address, read, SendStop::Yes)
    }
}

#[allow(private_bounds)]
impl<'d, M: AsyncMode> I2c<'d, M>
where
    Self: AsyncEngine,
{
    fn enable_rx_ints(&self) {
        self.info.regs().mier().write(|w| {
            w.set_rdie(true);
            w.set_ndie(true);
            w.set_alie(true);
            w.set_feie(true);
            w.set_pltie(true);
        });
    }

    fn enable_tx_ints(&self) {
        self.info.regs().mier().write(|w| {
            w.set_tdie(true);
            w.set_ndie(true);
            w.set_alie(true);
            w.set_feie(true);
            w.set_pltie(true);
        });
    }

    /// Schedule sending a START command and await it being pulled from the FIFO.
    ///
    /// Does not indicate that the command was responded to.
    async fn async_start(&self, address: u8, read: bool) -> Result<(), IOError> {
        if address >= 0x80 {
            return Err(IOError::AddressOutOfRange(address));
        }

        // send the start command
        let addr_rw = address << 1 | if read { 1 } else { 0 };
        self.send_cmd(if self.is_hs { Cmd::START_HS } else { Cmd::START }, addr_rw);

        self.info
            .wait_cell()
            .wait_for(|| {
                // enable interrupts
                self.enable_tx_ints();
                // if the command FIFO is empty, we're done sending start
                self.is_tx_fifo_empty_or_error()
            })
            .await
            .map_err(|_| IOError::Other)?;

        // Note: the START + ACK/NACK have not necessarily been finished here.
        // thus this might return Ok(()), but might at a later state result in NAK or FifoError.
        self.status_and_act()
    }

    async fn async_stop(&self) -> Result<(), IOError> {
        // send the stop command
        self.send_cmd(Cmd::STOP, 0);

        self.info
            .wait_cell()
            .wait_for(|| {
                // enable interrupts
                self.enable_tx_ints();
                // if the command FIFO is empty, we're done sending stop
                self.is_tx_fifo_empty_or_error()
            })
            .await
            .map_err(|_| IOError::Other)?;

        self.status_and_act()
    }

    // Public API: Async

    /// Reads data from the specified I2C address into the provided buffer asynchronously.
    ///
    /// This method performs the read operation without blocking the caller,
    /// returning a `Future` that resolves when the operation is complete.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `read`: A mutable buffer to store the data read from the device.
    ///
    /// # Returns
    ///
    /// - A `Future` that resolves to `Ok(())` if the read operation is successful.
    /// - Resolves to `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    pub fn async_read<'a>(
        &'a mut self,
        address: u8,
        read: &'a mut [u8],
    ) -> impl Future<Output = Result<(), IOError>> + 'a {
        <Self as AsyncEngine>::async_read_internal(self, address, read, SendStop::Yes)
    }

    /// Writes data to the specified I2C address from the provided buffer asynchronously.
    ///
    /// This method performs the write operation without blocking the caller, returning a `Future` that resolves when the operation is complete.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `write`: A buffer containing the data to be written to the device.
    ///
    /// # Returns
    ///
    /// - A `Future` that resolves to `Ok(())` if the write operation is successful.
    /// - Resolves to `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    pub fn async_write<'a>(
        &'a mut self,
        address: u8,
        write: &'a [u8],
    ) -> impl Future<Output = Result<(), IOError>> + 'a {
        <Self as AsyncEngine>::async_write_internal(self, address, write, SendStop::Yes)
    }

    /// Performs a combined write and read operation on the specified I2C
    /// address asynchronously.
    ///
    /// This method first writes data to the device, then reads data from the
    /// device into the provided buffer. The operation is performed without
    /// blocking the caller.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `write`: A buffer containing the data to be written to the device.
    /// - `read`: A mutable buffer to store the data read from the device.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the write-read operation is successful.
    /// - `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    pub async fn async_write_read<'a>(
        &'a mut self,
        address: u8,
        write: &'a [u8],
        read: &'a mut [u8],
    ) -> Result<(), IOError> {
        <Self as AsyncEngine>::async_write_internal(self, address, write, SendStop::No).await?;
        <Self as AsyncEngine>::async_read_internal(self, address, read, SendStop::Yes).await
    }
}

trait AsyncEngine {
    fn async_read_internal<'a>(
        &'a mut self,
        address: u8,
        read: &'a mut [u8],
        send_stop: SendStop,
    ) -> impl Future<Output = Result<(), IOError>> + 'a;

    fn async_write_internal<'a>(
        &'a mut self,
        address: u8,
        write: &'a [u8],
        send_stop: SendStop,
    ) -> impl Future<Output = Result<(), IOError>> + 'a;
}

impl<'d> I2c<'d, Async> {
    /// Creates a new interrupt-only asynchronous instance of the I2C Controller
    /// bus driver.
    ///
    /// This method initializes the I2C controller in asynchronous mode,
    /// enabling non-blocking operations using futures.  The I2C bus is
    /// configured based on the provided `Config` structure, which specifies
    /// parameters such as bus speed and clock settings.
    ///
    /// # Arguments
    ///
    /// - `peri`: The peripheral instance representing the I2C controller hardware.
    /// - `scl`: The pin to be used for the I2C clock line (SCL).
    /// - `sda`: The pin to be used for the I2C data line (SDA).
    /// - `_irq`: The interrupt binding for the I2C controller, ensuring that an interrupt handler is registered.
    /// - `config`: A `Config` structure specifying the desired I2C configuration, including bus speed and clock settings.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: A new instance of the I2C driver in asynchronous mode if initialization is successful.
    /// - `Err(SetupError)`: An error if the initialization fails, such as due to invalid clock configuration.
    ///
    /// # Behavior
    ///
    /// - The I2C controller is configured and enabled based on the provided `Config`.
    /// - The interrupt for the I2C controller is enabled to support asynchronous operations.
    /// - Any external pins used for SCL and SDA will be placed into a disabled state when the driver instance is dropped.
    ///
    /// # Errors
    ///
    /// - `SetupError::ClockSetup`: If there is an issue with the clock configuration.
    /// - `SetupError::InvalidDutyCycle`: If the duty cycle violates the selected mode's minimum tHIGH.
    /// - `SetupError::UnsupportedSpeed`: If UltraFast mode is selected.
    /// - `SetupError::BaudrateNotAchievable`: If the requested timing cannot be represented.
    pub fn new_async<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, SetupError> {
        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        Self::new_inner(peri, scl, sda, config, Async)
    }
}

impl<'d> AsyncEngine for I2c<'d, Async> {
    async fn async_read_internal(&mut self, address: u8, read: &mut [u8], send_stop: SendStop) -> Result<(), IOError> {
        if read.is_empty() {
            return Err(IOError::InvalidReadBufferLength);
        }

        for chunk in read.chunks_mut(256) {
            self.async_start(address, true).await?;

            // perform corrective action if the future is dropped or an
            // error happens between here and the end of the read.
            //
            // NOTE: this *must* be set up *after* async_start. async_start
            // already runs `status_and_act`, which on NACK performs its
            // own remediation; if we set OnDrop earlier, the early `?`
            // return would invoke remediation a second time and corrupt
            // the controller state for the next transaction.
            let on_drop = OnDrop::new(|| self.remediation());

            // send receive command
            self.send_cmd(Cmd::RECEIVE, (chunk.len() - 1) as u8);

            self.info
                .wait_cell()
                .wait_for(|| {
                    // enable interrupts
                    self.enable_tx_ints();
                    // if the command FIFO is empty, we're done sending start
                    self.is_tx_fifo_empty_or_error()
                })
                .await
                .map_err(|_| IOError::Other)?;

            for byte in chunk.iter_mut() {
                self.info
                    .wait_cell()
                    .wait_for(|| {
                        // enable interrupts
                        self.enable_rx_ints();
                        // if the rx FIFO is not empty, we need to read a byte
                        !self.is_rx_fifo_empty()
                    })
                    .await
                    .map_err(|_| IOError::ReadFail)?;

                *byte = self.info.regs().mrdr().read().data();
            }

            // defuse it; we'll re-arm on the next chunk if any.
            on_drop.defuse();
        }

        if send_stop == SendStop::Yes {
            self.async_stop().await?;
        }

        Ok(())
    }

    async fn async_write_internal(&mut self, address: u8, write: &[u8], send_stop: SendStop) -> Result<(), IOError> {
        self.async_start(address, false).await?;

        // perform corrective action if the future is dropped
        let on_drop = OnDrop::new(|| self.remediation());

        // Usually, embassy HALs error out with an empty write,
        // however empty writes are useful for writing I2C scanning
        // logic through write probing. That is, we send a start with
        // R/w bit cleared, but instead of writing any data, just send
        // the stop onto the bus. This has the effect of checking if
        // the resulting address got an ACK but causing no
        // side-effects to the device on the other end.
        //
        // Because of this, we are not going to error out in case of
        // empty writes.
        if write.is_empty() {
            #[cfg(feature = "defmt")]
            defmt::trace!("Empty write, write probing?");
            if send_stop == SendStop::Yes {
                self.async_stop().await?;
            }
            return Ok(());
        }

        for byte in write {
            // initiate transmit
            self.send_cmd(Cmd::TRANSMIT, *byte);

            self.info
                .wait_cell()
                .wait_for(|| {
                    // enable interrupts
                    self.enable_tx_ints();
                    // if the tx FIFO is empty, we're done transmiting
                    self.is_tx_fifo_empty_or_error()
                })
                .await
                .map_err(|_| IOError::WriteFail)?;

            self.status_and_act()?;
        }

        if send_stop == SendStop::Yes {
            self.async_stop().await?;
        }

        // defuse it if the future is not dropped
        on_drop.defuse();

        Ok(())
    }
}

impl<'d> I2c<'d, Dma<'d>> {
    /// Creates a new asynchronous instance of the I2C Controller bus driver with DMA support.
    ///
    /// This method initializes the I2C controller in asynchronous mode with
    /// Direct Memory Access (DMA) support, enabling efficient non-blocking
    /// operations for large data transfers.  The I2C bus is configured based on
    /// the provided `Config` structure, which specifies parameters such as bus
    /// speed and clock settings.
    ///
    /// # Arguments
    ///
    /// - `peri`: The peripheral instance representing the I2C controller hardware.
    /// - `scl`: The pin to be used for the I2C clock line (SCL).
    /// - `sda`: The pin to be used for the I2C data line (SDA).
    /// - `tx_dma`: The DMA channel to be used for transmitting data.
    /// - `rx_dma`: The DMA channel to be used for receiving data.
    /// - `_irq`: The interrupt binding for the I2C controller, ensuring that an interrupt handler is registered.
    /// - `config`: A `Config` structure specifying the desired I2C configuration, including bus speed and clock settings.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: A new instance of the I2C driver in asynchronous mode with DMA support if initialization is successful.
    /// - `Err(SetupError)`: An error if the initialization fails, such as due to invalid clock configuration.
    ///
    /// # Behavior
    ///
    /// - The I2C controller is configured and enabled based on the provided `Config`.
    /// - The interrupt for the I2C controller is enabled to support asynchronous operations.
    /// - The specified DMA channels are initialized and their interrupts are enabled.
    /// - Any external pins used for SCL and SDA will be placed into a disabled state when the driver instance is dropped.
    ///
    /// # Errors
    ///
    /// - `SetupError::ClockSetup`: If there is an issue with the clock configuration.
    /// - `SetupError::InvalidDutyCycle`: If the duty cycle violates the selected mode's minimum tHIGH.
    /// - `SetupError::UnsupportedSpeed`: If UltraFast mode is selected.
    /// - `SetupError::BaudrateNotAchievable`: If the requested timing cannot be represented.
    pub fn new_async_with_dma<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, SetupError> {
        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        // enable this channel's interrupt
        let tx_dma = DmaChannel::new(tx_dma);
        let rx_dma = DmaChannel::new(rx_dma);

        tx_dma.enable_interrupt();
        rx_dma.enable_interrupt();

        Self::new_inner(
            peri,
            scl,
            sda,
            config,
            Dma {
                tx_dma,
                rx_dma,
                tx_request: T::TX_DMA_REQUEST,
                rx_request: T::RX_DMA_REQUEST,
            },
        )
    }
}

impl<'d> AsyncEngine for I2c<'d, Dma<'d>> {
    async fn async_read_internal(&mut self, address: u8, read: &mut [u8], send_stop: SendStop) -> Result<(), IOError> {
        if read.is_empty() {
            return Err(IOError::InvalidReadBufferLength);
        }

        // Issue a single START for the whole read
        self.async_start(address, true).await?;

        // perform corrective action if the future is dropped or an
        // error happens between here and the end of the read.
        //
        // NOTE: this *must* be set up *after* async_start. async_start
        // already runs `status_and_act`, which on NACK performs its
        // own remediation; if we set OnDrop earlier, the early `?`
        // return would invoke remediation a second time and corrupt
        // the controller state for the next transaction.
        let on_drop = OnDrop::new(|| {
            self.remediation();
            self.info.regs().mder().modify(|w| w.set_rdde(false));
        });

        // Drain the *entire* read with a single continuous DMA transfer
        let peri_addr = self.info.regs().mrdr().as_ptr() as *const u8;
        unsafe {
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();
            self.mode.rx_dma.clear_interrupt();
            self.mode.rx_dma.set_request_source(self.mode.rx_request);
            self.mode
                .rx_dma
                .setup_read_from_peripheral(peri_addr, read, false, TransferOptions::COMPLETE_INTERRUPT)?;
            self.info.regs().mder().modify(|w| w.set_rdde(true));
            self.mode.rx_dma.enable_request();
        }

        // A single RECEIVE command can request at most 256 bytes (its count
        // field is 8-bit), and the command FIFO is only a few entries deep, so
        // a large read needs many RECEIVE commands issued over the life of the
        // transfer. Refills are driven by the LPI2C transmit-data flag (TDF) —
        // i.e. by *command-FIFO space*

        let mut to_request = read.len();
        let result = core::future::poll_fn(|cx| {
            // Register wakers for both completion sources before touching
            // hardware state: DMA-complete (whole buffer received) and the
            // shared I2C interrupt (TDF command-FIFO space, plus bus errors).
            let _ = self.mode.rx_dma.wait_cell().poll_wait(cx);
            let _ = self.info.wait_cell().poll_wait(cx);

            // Surface a bus error (NACK, arbitration loss, FIFO error) rather
            // than waiting forever for data that will never arrive.
            if let Err(e) = self.status() {
                return core::task::Poll::Ready(Err(e));
            }

            // Refill the command FIFO while it has space and commands remain.
            while to_request > 0 && !self.is_tx_fifo_full() {
                let n = to_request.min(256);
                self.send_cmd(Cmd::RECEIVE, (n - 1) as u8);
                to_request -= n;
            }

            // Re-arm interrupts every poll: the shared I2C ISR disables MIER on
            // each fire. Always keep the error interrupts armed so a NACK wakes
            // us
            self.info.regs().mier().write(|w| {
                w.set_ndie(true);
                w.set_alie(true);
                w.set_feie(true);
                w.set_pltie(true);
                w.set_tdie(to_request > 0);
            });

            if self.mode.rx_dma.is_done() {
                core::task::Poll::Ready(Ok(()))
            } else {
                core::task::Poll::Pending
            }
        })
        .await;

        cortex_m::asm::dsb();

        self.info.regs().mder().modify(|w| w.set_rdde(false));
        unsafe {
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();
        }

        result?;

        if send_stop == SendStop::Yes {
            self.async_stop().await?;
        }

        // defuse it if the future is not dropped
        on_drop.defuse();

        Ok(())
    }

    async fn async_write_internal(&mut self, address: u8, write: &[u8], send_stop: SendStop) -> Result<(), IOError> {
        self.async_start(address, false).await?;

        // Usually, embassy HALs error out with an empty write,
        // however empty writes are useful for writing I2C scanning
        // logic through write probing. That is, we send a start with
        // R/w bit cleared, but instead of writing any data, just send
        // the stop onto the bus. This has the effect of checking if
        // the resulting address got an ACK but causing no
        // side-effects to the device on the other end.
        //
        // Because of this, we are not going to error out in case of
        // empty writes.
        if write.is_empty() {
            #[cfg(feature = "defmt")]
            defmt::trace!("Empty write, write probing?");
            if send_stop == SendStop::Yes {
                self.async_stop().await?;
            }
            return Ok(());
        }

        // perform corrective action if the future is dropped
        let on_drop = OnDrop::new(|| {
            self.remediation();
            self.info.regs().mder().modify(|w| w.set_tdde(false));
        });

        for chunk in write.chunks(DMA_MAX_TRANSFER_SIZE) {
            let peri_addr = self.info.regs().mtdr().as_ptr() as *mut u8;

            unsafe {
                // Clean up channel state
                self.mode.tx_dma.disable_request();
                self.mode.tx_dma.clear_done();
                self.mode.tx_dma.clear_interrupt();

                // Set DMA request source from instance type (type-safe)
                self.mode.tx_dma.set_request_source(self.mode.tx_request);

                // Configure TCD for memory-to-peripheral transfer
                self.mode.tx_dma.setup_write_to_peripheral(
                    chunk,
                    peri_addr,
                    false,
                    TransferOptions::COMPLETE_INTERRUPT,
                )?;

                // Enable I2C TX DMA request
                self.info.regs().mder().modify(|w| w.set_tdde(true));

                // Enable DMA channel request
                self.mode.tx_dma.enable_request();
            }

            // Wait for completion asynchronously
            core::future::poll_fn(|cx| {
                let _ = self.mode.tx_dma.wait_cell().poll_wait(cx);
                if self.mode.tx_dma.is_done() {
                    core::task::Poll::Ready(())
                } else {
                    core::task::Poll::Pending
                }
            })
            .await;

            // Ensure DMA writes are visible to CPU
            cortex_m::asm::dsb();
            // Cleanup
            self.info.regs().mder().modify(|w| w.set_tdde(false));
            unsafe {
                self.mode.tx_dma.disable_request();
                self.mode.tx_dma.clear_done();
            }
        }

        if send_stop == SendStop::Yes {
            self.async_stop().await?;
        }

        // defuse it if the future is not dropped
        on_drop.defuse();

        Ok(())
    }
}

impl<'d, M: Mode> Drop for I2c<'d, M> {
    fn drop(&mut self) {
        self._scl.set_as_disabled();
        self._sda.set_as_disabled();
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Read for I2c<'d, M> {
    type Error = IOError;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, buffer)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Write for I2c<'d, M> {
    type Error = IOError;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, bytes)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::WriteRead for I2c<'d, M> {
    type Error = IOError;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(address, bytes, buffer)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Transactional for I2c<'d, M> {
    type Error = IOError;

    fn exec(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_02::blocking::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        if let Some((last, rest)) = operations.split_last_mut() {
            for op in rest {
                match op {
                    embedded_hal_02::blocking::i2c::Operation::Read(buf) => {
                        self.blocking_read_internal(address, buf, SendStop::No)?
                    }
                    embedded_hal_02::blocking::i2c::Operation::Write(buf) => {
                        self.blocking_write_internal(address, buf, SendStop::No)?
                    }
                }
            }

            match last {
                embedded_hal_02::blocking::i2c::Operation::Read(buf) => {
                    self.blocking_read_internal(address, buf, SendStop::Yes)
                }
                embedded_hal_02::blocking::i2c::Operation::Write(buf) => {
                    self.blocking_write_internal(address, buf, SendStop::Yes)
                }
            }
        } else {
            Ok(())
        }
    }
}

impl embedded_hal_1::i2c::Error for IOError {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match *self {
            Self::ArbitrationLoss => embedded_hal_1::i2c::ErrorKind::ArbitrationLoss,
            Self::AddressNack => {
                embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Address)
            }
            _ => embedded_hal_1::i2c::ErrorKind::Other,
        }
    }
}

impl<'d, M: Mode> embedded_hal_1::i2c::ErrorType for I2c<'d, M> {
    type Error = IOError;
}

impl<'d, M: Mode> embedded_hal_1::i2c::I2c for I2c<'d, M> {
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let result = (|| {
            if let Some((last, rest)) = operations.split_last_mut() {
                for op in rest {
                    match op {
                        embedded_hal_1::i2c::Operation::Read(buf) => {
                            self.blocking_read_internal(address, buf, SendStop::No)?
                        }
                        embedded_hal_1::i2c::Operation::Write(buf) => {
                            self.blocking_write_internal(address, buf, SendStop::No)?
                        }
                    }
                }

                match last {
                    embedded_hal_1::i2c::Operation::Read(buf) => {
                        self.blocking_read_internal(address, buf, SendStop::Yes)
                    }
                    embedded_hal_1::i2c::Operation::Write(buf) => {
                        self.blocking_write_internal(address, buf, SendStop::Yes)
                    }
                }
            } else {
                Ok(())
            }
        })();

        if result.is_err() {
            self.recover_from_error();
        }
        result
    }
}

impl<'d, M: AsyncMode> embedded_hal_async::i2c::I2c for I2c<'d, M>
where
    I2c<'d, M>: AsyncEngine,
{
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let result = async {
            if let Some((last, rest)) = operations.split_last_mut() {
                for op in rest {
                    match op {
                        embedded_hal_async::i2c::Operation::Read(buf) => {
                            <Self as AsyncEngine>::async_read_internal(self, address, buf, SendStop::No).await?
                        }
                        embedded_hal_async::i2c::Operation::Write(buf) => {
                            <Self as AsyncEngine>::async_write_internal(self, address, buf, SendStop::No).await?
                        }
                    }
                }

                match last {
                    embedded_hal_async::i2c::Operation::Read(buf) => {
                        <Self as AsyncEngine>::async_read_internal(self, address, buf, SendStop::Yes).await
                    }
                    embedded_hal_async::i2c::Operation::Write(buf) => {
                        <Self as AsyncEngine>::async_write_internal(self, address, buf, SendStop::Yes).await
                    }
                }
            } else {
                Ok(())
            }
        }
        .await;

        if result.is_err() {
            self.recover_from_error();
        }
        result
    }
}

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for I2c<'d, M> {
    type Config = Config;
    type ConfigError = SetupError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), SetupError> {
        self.set_configuration(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duty_cycle_validation_depends_on_speed() {
        assert!(DutyCycle::new(DutyCycle::STANDARD_MIN_PERCENT - 1).is_some());

        let below_standard = DutyCycle::new(DutyCycle::STANDARD_MIN_PERCENT - 1).unwrap();
        assert!(!below_standard.is_valid_for(Speed::Standard));
        assert!(
            DutyCycle::new(DutyCycle::STANDARD_MIN_PERCENT)
                .unwrap()
                .is_valid_for(Speed::Standard)
        );

        assert!(DutyCycle::new(DutyCycle::FAST_MIN_PERCENT - 1).is_none());
        assert!(
            DutyCycle::new(DutyCycle::FAST_MIN_PERCENT)
                .unwrap()
                .is_valid_for(Speed::Fast)
        );

        assert!(
            !DutyCycle::new(DutyCycle::FAST_PLUS_MIN_PERCENT - 1)
                .unwrap()
                .is_valid_for(Speed::FastPlus)
        );
        assert!(
            DutyCycle::new(DutyCycle::FAST_PLUS_MIN_PERCENT)
                .unwrap()
                .is_valid_for(Speed::FastPlus)
        );
    }

    #[test]
    fn minimum_duty_meets_each_modes_minimum_high_time() {
        let sources = [6_000_000, 12_000_000, 24_000_000, 48_000_000, 96_000_000, 150_000_000];
        let speeds = [Speed::Standard, Speed::Fast, Speed::FastPlus];

        for source in sources {
            for speed in speeds {
                let minimum = DutyCycle::minimum_for_speed(speed).unwrap();
                let duty_cycle = DutyCycle::new(minimum).unwrap();
                let params = compute_baud_params(source, speed, duty_cycle).unwrap();

                assert!(
                    params.meets_minimum_high_time(source, speed),
                    "{source} Hz source did not meet {speed:?} tHIGH"
                );
            }
        }
    }

    #[test]
    fn fast_mode_review_example_meets_thigh() {
        let params = compute_baud_params(48_000_000, Speed::Fast, DutyCycle::new(24).unwrap()).unwrap();

        assert_eq!((params.prescale as u8, params.clklo, params.clkhi), (1, 44, 13));
        assert!(params.meets_minimum_high_time(48_000_000, Speed::Fast));
        assert_eq!(
            params.configured_timing(48_000_000),
            ConfiguredTiming {
                frequency_hz: 400_000,
                high_percent: 25,
            }
        );
    }

    #[test]
    fn default_duty_preserves_legacy_register_values() {
        let cases = [
            (6_000_000, Speed::Standard, (0, 30, 26, 29, 14)),
            (6_000_000, Speed::Fast, (0, 7, 4, 6, 2)),
            (6_000_000, Speed::FastPlus, (0, 2, 0, 2, 0)),
            (12_000_000, Speed::Standard, (0, 61, 55, 59, 29)),
            (12_000_000, Speed::Fast, (0, 14, 12, 14, 6)),
            (12_000_000, Speed::FastPlus, (0, 5, 3, 5, 2)),
            (24_000_000, Speed::Standard, (1, 61, 56, 59, 29)),
            (24_000_000, Speed::Fast, (0, 30, 26, 29, 14)),
            (24_000_000, Speed::FastPlus, (0, 11, 9, 11, 5)),
            (48_000_000, Speed::Standard, (2, 61, 57, 59, 29)),
            (48_000_000, Speed::Fast, (0, 61, 55, 59, 29)),
            (48_000_000, Speed::FastPlus, (0, 23, 21, 23, 11)),
            (96_000_000, Speed::Standard, (3, 61, 57, 59, 29)),
            (96_000_000, Speed::Fast, (1, 61, 56, 59, 29)),
            (96_000_000, Speed::FastPlus, (0, 48, 44, 47, 23)),
            (150_000_000, Speed::Standard, (4, 47, 45, 45, 22)),
            (150_000_000, Speed::Fast, (2, 47, 45, 45, 22)),
            (150_000_000, Speed::FastPlus, (1, 38, 34, 36, 17)),
        ];

        for (source, speed, expected) in cases {
            let params = compute_baud_params(source, speed, DutyCycle::default()).unwrap();
            let actual = (
                params.prescale as u8,
                params.clklo,
                params.clkhi,
                params.sethold,
                params.datavd,
            );

            assert_eq!(actual, expected, "source={source}, speed={speed:?}");
        }
    }

    #[test]
    fn duty_cycle_frequency_degradation_is_bounded() {
        let duty_cycle = DutyCycle::new(DutyCycle::FAST_MIN_PERCENT).unwrap();

        assert!(compute_baud_params(42_000_000, Speed::Fast, duty_cycle).is_ok());
        assert!(matches!(
            compute_baud_params(38_000_000, Speed::Fast, duty_cycle),
            Err(SetupError::BaudrateNotAchievable)
        ));
    }

    #[test]
    fn invalid_or_unachievable_timing_returns_an_error() {
        let fast_only_duty = DutyCycle::new(DutyCycle::FAST_MIN_PERCENT).unwrap();
        assert!(matches!(
            compute_baud_params(48_000_000, Speed::Standard, fast_only_duty),
            Err(SetupError::InvalidDutyCycle)
        ));
        assert!(matches!(
            compute_baud_params(1_000_000, Speed::FastPlus, DutyCycle::default()),
            Err(SetupError::BaudrateNotAchievable)
        ));
        assert!(matches!(
            compute_baud_params(48_000_000, Speed::UltraFast, DutyCycle::default()),
            Err(SetupError::UnsupportedSpeed)
        ));
    }
}
