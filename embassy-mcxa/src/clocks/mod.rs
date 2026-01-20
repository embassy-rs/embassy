//! # Clock Module
//!
//! For the MCX-A, we separate clock and peripheral control into two main stages:
//!
//! 1. At startup, e.g. when `embassy_mcxa::init()` is called, we configure the
//!    core system clocks, including external and internal oscillators. This
//!    configuration is then largely static for the duration of the program.
//! 2. When HAL drivers are created, e.g. `Lpuart::new()` is called, the driver
//!    is responsible for two main things:
//!     * Ensuring that any required "upstream" core system clocks necessary for
//!       clocking the peripheral is active and configured to a reasonable value
//!     * Enabling the clock gates for that peripheral, and resetting the peripheral
//!
//! From a user perspective, only step 1 is visible. Step 2 is automatically handled
//! by HAL drivers, using interfaces defined in this module.
//!
//! It is also possible to *view* the state of the clock configuration after [`init()`]
//! has been called, using the [`with_clocks()`] function, which provides a view of the
//! [`Clocks`] structure.
//!
//! ## For HAL driver implementors
//!
//! The majority of peripherals in the MCXA chip are fed from either a "hard-coded" or
//! configurable clock source, e.g. selecting the FROM12M or `clk_1m` as a source. This
//! selection, as well as often any pre-scaler division from that source clock, is made
//! through MRCC registers.
//!
//! Any peripheral that is controlled through the MRCC register can automatically implement
//! the necessary APIs using the `impl_cc_gate!` macro in this module. You will also need
//! to define the configuration surface and steps necessary to fully configure that peripheral
//! from a clocks perspective by:
//!
//! 1. Defining a configuration type in the [`periph_helpers`] module that contains any selects
//!    or divisions available to the HAL driver
//! 2. Implementing the [`periph_helpers::SPConfHelper`] trait, which should check that the
//!    necessary input clocks are reasonable

use core::cell::RefCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use config::{
    ClocksConfig, CoreSleep, FircConfig, FircFreqSel, Fro16KConfig, MainClockSource, SircConfig, VddDriveStrength,
    VddLevel,
};
use paste::paste;
use periph_helpers::{PreEnableParts, SPConfHelper};

use crate::pac;
use crate::pac::cmc::vals::CkctrlCkmode;
use crate::pac::scg::vals::{
    Erefs, Fircacc, FircaccIe, FirccsrLk, Fircerr, FircerrIe, Fircsten, FreqSel, Range, Scs, SirccsrLk, Sircerr,
    Sircvld, SosccsrLk, Soscerr, Source, SpllLock, SpllcsrLk, Spllerr, Spllsten, TrimUnlock,
};
use crate::pac::spc::vals::{
    ActiveCfgBgmode, ActiveCfgCoreldoVddDs, ActiveCfgCoreldoVddLvl, LpCfgBgmode, LpCfgCoreldoVddLvl, Vsm,
};
use crate::pac::syscon::vals::{
    AhbclkdivUnstab, FrohfdivHalt, FrohfdivReset, FrohfdivUnstab, FrolfdivHalt, FrolfdivReset, FrolfdivUnstab,
    Pll1clkdivHalt, Pll1clkdivReset, Pll1clkdivUnstab,
};
pub mod config;
pub mod periph_helpers;

//
// Statics/Consts
//

// TODO: Different for different CPUs?
const VDD_CORE_MID_DRIVE_WAIT_STATE_LIMITS: &[(u32, u8)] = &[(22_500_000, 0b0000)];
const VDD_CORE_MID_DRIVE_MAX_WAIT_STATES: u8 = 0b0001;

const VDD_CORE_OVER_DRIVE_WAIT_STATE_LIMITS: &[(u32, u8)] = &[
    (40_000_000, 0b0000),
    (80_000_000, 0b0001),
    (120_000_000, 0b0010),
    (160_000_000, 0b0011),
];
const VDD_CORE_OVER_DRIVE_MAX_WAIT_STATES: u8 = 0b0100;

/// The state of system core clocks.
///
/// Initialized by [`init()`], and then unchanged for the remainder of the program.
static CLOCKS: critical_section::Mutex<RefCell<Option<Clocks>>> = critical_section::Mutex::new(RefCell::new(None));
static LIVE_HP_TOKENS: AtomicUsize = AtomicUsize::new(0);

//
// Free functions
//

/// Initialize the core system clocks with the given [`ClocksConfig`].
///
/// This function should be called EXACTLY once at start-up, usually via a
/// call to [`embassy_mcxa::init()`](crate::init()). Subsequent calls will
/// return an error.
pub fn init(settings: ClocksConfig) -> Result<(), ClockError> {
    critical_section::with(|cs| {
        if CLOCKS.borrow_ref(cs).is_some() {
            Err(ClockError::AlreadyInitialized)
        } else {
            Ok(())
        }
    })?;

    let mut clocks = Clocks::default();
    let mut operator = ClockOperator {
        clocks: &mut clocks,
        config: &settings,
        sirc_forced: false,

        _mrcc0: pac::MRCC0,
        scg0: pac::SCG0,
        syscon: pac::SYSCON,
        vbat0: pac::VBAT0,
        spc0: pac::SPC0,
        fmu0: pac::FMU0,
        cmc: pac::CMC,
    };

    // Before applying any requested clocks, apply the requested VDD_CORE
    // voltage level
    operator.configure_voltages()?;

    // Enable SIRC clocks FIRST, in case we need to use SIRC as main_clk for
    // a short while.
    operator.configure_sirc_clocks_early()?;
    operator.configure_firc_clocks()?;
    operator.configure_fro16k_clocks()?;
    #[cfg(not(feature = "sosc-as-gpio"))]
    operator.configure_sosc()?;
    operator.configure_spll()?;

    // Finally, setup main clock
    operator.configure_main_clk()?;

    // If we were keeping SIRC enabled, now we can release it.
    operator.configure_sirc_clocks_late();

    critical_section::with(|cs| {
        let mut clks = CLOCKS.borrow_ref_mut(cs);
        assert!(clks.is_none(), "Clock setup race!");
        *clks = Some(clocks);
    });

    Ok(())
}

/// Obtain the full clocks structure, calling the given closure in a critical section.
///
/// The given closure will be called with read-only access to the state of the system
/// clocks. This can be used to query and return the state of a given clock.
///
/// As the caller's closure will be called in a critical section, care must be taken
/// not to block or cause any other undue delays while accessing.
///
/// Calls to this function will not succeed until after a successful call to `init()`,
/// and will always return None.
pub fn with_clocks<R: 'static, F: FnOnce(&Clocks) -> R>(f: F) -> Option<R> {
    critical_section::with(|cs| {
        let c = CLOCKS.borrow_ref(cs);
        let c = c.as_ref()?;
        Some(f(c))
    })
}

//
// Structs/Enums
//

/// A guard that will inhibit the device from entering deep sleep while
/// it exists.
#[must_use = "Wake Guard must be kept in order to prevent deep sleep"]
pub struct WakeGuard {
    _x: (),
}

impl WakeGuard {
    /// Create a new wake guard, that increments the "live high power token" counts.
    ///
    /// This is typically used by HAL drivers (when a peripheral is clocked from an
    /// active-mode-only source) to inhibit sleep, OR by application code to prevent
    /// deep sleep as well.
    pub fn new() -> Self {
        _ = LIVE_HP_TOKENS.fetch_add(1, Ordering::AcqRel);
        Self { _x: () }
    }

    /// Helper method to potentially create a guard if necessary for a clock.
    pub fn for_power(level: &PoweredClock) -> Option<Self> {
        match level {
            PoweredClock::NormalEnabledDeepSleepDisabled => Some(Self::new()),
            PoweredClock::AlwaysEnabled => None,
        }
    }
}

impl Clone for WakeGuard {
    fn clone(&self) -> Self {
        // NOTE: Call load-bearing-new to clone, DO NOT just use the derive to
        // copy the ZST!
        Self::new()
    }
}

impl Default for WakeGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WakeGuard {
    fn drop(&mut self) {
        let old = LIVE_HP_TOKENS.fetch_sub(1, Ordering::AcqRel);
        // Ensure we didn't just underflow.
        assert!(old != 0);
    }
}

/// The `Clocks` structure contains the initialized state of the core system clocks
///
/// These values are configured by providing [`config::ClocksConfig`] to the [`init()`] function
/// at boot time.
#[derive(Default, Debug, Clone)]
#[non_exhaustive]
pub struct Clocks {
    /// Active power config
    pub active_power: VddLevel,

    /// Low-power power config
    pub lp_power: VddLevel,

    /// The `clk_in` is a clock provided by an external oscillator
    /// AKA SOSC
    #[cfg(not(feature = "sosc-as-gpio"))]
    pub clk_in: Option<Clock>,

    // FRO180M stuff
    //
    /// `fro_hf_root` is the direct output of the `FRO180M` internal oscillator
    ///
    /// It is used to feed downstream clocks, such as `fro_hf`, `clk_45m`,
    /// and `fro_hf_div`.
    pub fro_hf_root: Option<Clock>,

    /// `fro_hf` is the same frequency as `fro_hf_root`, but behind a gate.
    pub fro_hf: Option<Clock>,

    /// `clk_45` is a 45MHz clock, sourced from `fro_hf`.
    pub clk_45m: Option<Clock>,

    /// `fro_hf_div` is a configurable frequency clock, sourced from `fro_hf`.
    pub fro_hf_div: Option<Clock>,

    //
    // End FRO180M

    // FRO12M stuff
    //
    /// `fro_12m_root` is the direct output of the `FRO12M` internal oscillator
    ///
    /// It is used to feed downstream clocks, such as `fro_12m`, `clk_1m`,
    /// `and `fro_lf_div`.
    pub fro_12m_root: Option<Clock>,

    /// `fro_12m` is the same frequency as `fro_12m_root`, but behind a gate.
    pub fro_12m: Option<Clock>,

    /// `clk_1m` is a 1MHz clock, sourced from `fro_12m`
    pub clk_1m: Option<Clock>,

    /// `fro_lf_div` is a configurable frequency clock, sourced from `fro_12m`
    pub fro_lf_div: Option<Clock>,
    //
    // End FRO12M stuff
    /// `clk_16k_vsys` is one of two outputs of the `FRO16K` internal oscillator.
    ///
    /// Also referred to as `clk_16k[0]` in the datasheet, it feeds peripherals in
    /// the system domain, such as the CMP and RTC.
    pub clk_16k_vsys: Option<Clock>,

    /// `clk_16k_vdd_core` is one of two outputs of the `FRO16K` internal oscillator.
    ///
    /// Also referred to as `clk_16k[1]` in the datasheet, it feeds peripherals in
    /// the VDD Core domain, such as the OSTimer or LPUarts.
    pub clk_16k_vdd_core: Option<Clock>,

    /// `main_clk` is the main clock, upstream of the cpu/system clock.
    pub main_clk: Option<Clock>,

    /// `CPU_CLK` or `SYSTEM_CLK` is the output of `main_clk`, run through the `AHBCLKDIV`,
    /// used for the CPU, AHB, APB, IPS bus, and some high speed peripherals.
    pub cpu_system_clk: Option<Clock>,

    /// `pll1_clk` is the output of the main system PLL, `pll1`.
    pub pll1_clk: Option<Clock>,

    /// `pll1_clk_div` is a configurable frequency clock, sourced from `pll1_clk`
    pub pll1_clk_div: Option<Clock>,
}

/// `ClockError` is the main error returned when configuring or checking clock state
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ClockError {
    /// The system clocks were never initialized by calling [`init()`]
    NeverInitialized,
    /// The [`init()`] function was called more than once
    AlreadyInitialized,
    /// The requested configuration was not possible to fulfill, as the system clocks
    /// were not configured in a compatible way
    BadConfig { clock: &'static str, reason: &'static str },
    /// The requested configuration was not possible to fulfill, as the required system
    /// clocks have not yet been implemented.
    NotImplemented { clock: &'static str },
    /// The requested peripheral could not be configured, as the steps necessary to
    /// enable it have not yet been implemented.
    UnimplementedConfig,
}

/// Information regarding a system clock
#[derive(Debug, Clone)]
pub struct Clock {
    /// The frequency, in Hz, of the given clock
    pub frequency: u32,
    /// The power state of the clock, e.g. whether it is active in deep sleep mode
    /// or not.
    pub power: PoweredClock,
}

/// The power state of a given clock.
///
/// On the MCX-A, when Deep-Sleep is entered, any clock not configured for Deep Sleep
/// mode will be stopped. This means that any downstream usage, e.g. by peripherals,
/// will also stop.
///
/// In the future, we will provide an API for entering Deep Sleep, and if there are
/// any peripherals that are NOT using an `AlwaysEnabled` clock active, entry into
/// Deep Sleep will be prevented, in order to avoid misbehaving peripherals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoweredClock {
    /// The given clock will NOT continue running in Deep Sleep mode
    NormalEnabledDeepSleepDisabled,
    /// The given clock WILL continue running in Deep Sleep mode
    AlwaysEnabled,
}

/// The ClockOperator is a private helper type that contains the methods used
/// during system clock initialization.
///
/// # SAFETY
///
/// Concurrent access to clock-relevant peripheral registers, such as `MRCC`, `SCG`,
/// `SYSCON`, and `VBAT` should not be allowed for the duration of the [`init()`] function.
struct ClockOperator<'a> {
    /// A mutable reference to the current state of system clocks
    clocks: &'a mut Clocks,
    /// A reference to the requested configuration provided by the caller of [`init()`]
    config: &'a ClocksConfig,

    /// SIRC is forced-on until we set `main_clk`
    sirc_forced: bool,

    // We hold on to stolen peripherals
    _mrcc0: pac::mrcc::Mrcc,
    scg0: pac::scg::Scg,
    syscon: pac::syscon::Syscon,
    vbat0: pac::vbat::Vbat,
    spc0: pac::spc::Spc,
    fmu0: pac::fmu::Fmu,
    cmc: pac::cmc::Cmc,
}

// From Table 165 - Max Clock Frequencies
struct ClockLimits {
    fro_hf: u32,
    fro_hf_div: u32,
    pll1_clk: u32,
    main_clk: u32,
    cpu_clk: u32,
    // The following items are LISTED in Table 165, but are not necessary
    // to check at runtime either because they are physically fixed, the
    // HAL exposes no way for them to exceed their limits, or they cannot
    // exceed their limits due to some upstream clock enforcement. They
    // are included here as documentation.
    //
    // clk_16k: u32,        // fixed (16.384kHz), no need to check
    // clk_in: u32,         // Checked already in configure_sosc method, 50MHz in all modes
    // clk_48m: u32,        // clk_48m is fixed (to 45mhz actually)
    // fro_12m: u32,        // We don't allow modifying from 12mhz
    // fro_12m_div: u32,    // div can never exceed 12mhz
    // pll1_clk_div: u32,   // if pll1_clk is in range, so is pll1_clk_div
    // clk_1m: u32,         // fro_12m / 12 can never exceed 12mhz
    // system_clk: u32,     // cpu_clk == system_clk
    // bus_clk: u32,        // bus_clk == (cpu_clk / 2), if cpu_clk is good so is bus_clk
    // slow_clk: u32,       // slow_clk == (cpu_clk / 6), if cpu_clk is good so is slow_clock
}

impl ClockLimits {
    const MID_DRIVE: Self = Self {
        fro_hf: 90_000_000,
        fro_hf_div: 45_000_000,
        pll1_clk: 48_000_000,
        main_clk: 90_000_000,
        cpu_clk: 45_000_000,
        // clk_16k: 16_384,
        // clk_in: 50_000_000,
        // clk_48m: 48_000_000,
        // fro_12m: 24_000_000, // what?
        // fro_12m_div: 24_000_000, // what?
        // pll1_clk_div: 48_000_000,
        // clk_1m: 1_000_000,
        // system_clk: 45_000_000,
        // bus_clk: 22_500_000,
        // slow_clk: 7_500_000,
    };

    const OVER_DRIVE: Self = Self {
        fro_hf: 180_000_000,
        fro_hf_div: 180_000_000,
        pll1_clk: 240_000_000,
        main_clk: 180_000_000,
        cpu_clk: 180_000_000,
        // clk_16k: 16_384,
        // clk_in: 50_000_000,
        // clk_48m: 48_000_000,
        // fro_12m: 24_000_000, // what?
        // fro_12m_div: 24_000_000, // what?
        // pll1_clk_div: 240_000_000,
        // clk_1m: 1_000_000,
        // system_clk: 180_000_000,
        // bus_clk: 90_000_000,
        // slow_clk: 36_000_000,
    };
}

/// Trait describing an AHB clock gate that can be toggled through MRCC.
pub trait Gate {
    type MrccPeriphConfig: SPConfHelper;

    /// Enable the clock gate.
    ///
    /// # SAFETY
    ///
    /// The current peripheral must be disabled prior to calling this method
    unsafe fn enable_clock();

    /// Disable the clock gate.
    ///
    /// # SAFETY
    ///
    /// There must be no active user of this peripheral when calling this method
    unsafe fn disable_clock();

    /// Drive the peripheral into reset.
    ///
    /// # SAFETY
    ///
    /// There must be no active user of this peripheral when calling this method
    unsafe fn assert_reset();

    /// Drive the peripheral out of reset.
    ///
    /// # SAFETY
    ///
    /// There must be no active user of this peripheral when calling this method
    unsafe fn release_reset();

    /// Return whether the clock gate for this peripheral is currently enabled.
    fn is_clock_enabled() -> bool;

    /// Return whether the peripheral is currently held in reset.
    fn is_reset_released() -> bool;
}

/// This is the primary helper method HAL drivers are expected to call when creating
/// an instance of the peripheral.
///
/// This method:
///
/// 1. Enables the MRCC clock gate for this peripheral
/// 2. Calls the `G::MrccPeriphConfig::post_enable_config()` method, returning an error
///    and re-disabling the peripheral if this fails.
/// 3. Pulses the MRCC reset line, to reset the peripheral to the default state
/// 4. Returns the frequency, in Hz that is fed into the peripheral, taking into account
///    the selected upstream clock, as well as any division specified by `cfg`.
///
/// NOTE: if a clock is disabled, sourced from an "ambient" clock source, this method
/// may return `Ok(0)`. In the future, this might be updated to return the correct
/// "ambient" clock, e.g. the AHB/APB frequency.
///
/// # SAFETY
///
/// This peripheral must not yet be in use prior to calling `enable_and_reset`.
#[inline]
pub unsafe fn enable_and_reset<G: Gate>(cfg: &G::MrccPeriphConfig) -> Result<PreEnableParts, ClockError> {
    unsafe {
        let freq = enable::<G>(cfg)?;
        pulse_reset::<G>();
        Ok(freq)
    }
}

/// Enable the clock gate for the given peripheral.
///
/// Prefer [`enable_and_reset`] unless you are specifically avoiding a pulse of the reset, or need
/// to control the duration of the pulse more directly.
///
/// If an `Err` is returned, the given clock is guaranteed to be disabled.
///
/// # SAFETY
///
/// This peripheral must not yet be in use prior to calling `enable`.
#[inline]
pub unsafe fn enable<G: Gate>(cfg: &G::MrccPeriphConfig) -> Result<PreEnableParts, ClockError> {
    unsafe {
        // Instead of checking, just disable the clock if it is currently enabled.
        G::disable_clock();

        let freq = critical_section::with(|cs| {
            let clocks = CLOCKS.borrow_ref(cs);
            let clocks = clocks.as_ref().ok_or(ClockError::NeverInitialized)?;
            cfg.pre_enable_config(clocks)
        })?;

        G::enable_clock();
        while !G::is_clock_enabled() {}
        core::arch::asm!("dsb sy; isb sy", options(nomem, nostack, preserves_flags));

        Ok(freq)
    }
}

/// Disable the clock gate for the given peripheral.
///
/// # SAFETY
///
/// This peripheral must no longer be in use prior to calling `enable`.
#[allow(dead_code)]
#[inline]
pub unsafe fn disable<G: Gate>() {
    unsafe {
        G::disable_clock();
    }
}

/// Check whether a gate is currently enabled.
#[allow(dead_code)]
#[inline]
pub fn is_clock_enabled<G: Gate>() -> bool {
    G::is_clock_enabled()
}

/// Release a reset line for the given peripheral set.
///
/// Prefer [`enable_and_reset`].
///
/// # SAFETY
///
/// This peripheral must not yet be in use prior to calling `release_reset`.
#[inline]
pub unsafe fn release_reset<G: Gate>() {
    unsafe {
        G::release_reset();
    }
}

/// Assert a reset line for the given peripheral set.
///
/// Prefer [`enable_and_reset`].
///
/// # SAFETY
///
/// This peripheral must not yet be in use prior to calling `assert_reset`.
#[inline]
pub unsafe fn assert_reset<G: Gate>() {
    unsafe {
        G::assert_reset();
    }
}

/// Check whether the peripheral is held in reset.
///
/// # Safety
///
/// Must be called with a valid peripheral gate type.
#[inline]
pub unsafe fn is_reset_released<G: Gate>() -> bool {
    G::is_reset_released()
}

/// Pulse a reset line (assert then release) with a short delay.
///
/// Prefer [`enable_and_reset`].
///
/// # SAFETY
///
/// This peripheral must not yet be in use prior to calling `release_reset`.
#[inline]
pub unsafe fn pulse_reset<G: Gate>() {
    unsafe {
        G::assert_reset();
        cortex_m::asm::nop();
        cortex_m::asm::nop();
        G::release_reset();
    }
}

//
// `impl`s for structs/enums
//

/// The [`Clocks`] type's methods generally take the form of "ensure X clock is active".
///
/// These methods are intended to be used by HAL peripheral implementors to ensure that their
/// selected clocks are active at a suitable level at time of construction. These methods
/// return the frequency of the requested clock, in Hertz, or a [`ClockError`].
impl Clocks {
    fn ensure_clock_active(
        &self,
        clock: &Option<Clock>,
        name: &'static str,
        at_level: &PoweredClock,
    ) -> Result<u32, ClockError> {
        let Some(clk) = clock.as_ref() else {
            return Err(ClockError::BadConfig {
                clock: name,
                reason: "required but not active",
            });
        };
        if !clk.power.meets_requirement_of(at_level) {
            return Err(ClockError::BadConfig {
                clock: name,
                reason: "not low power active",
            });
        }
        Ok(clk.frequency)
    }

    /// Ensure the `fro_lf_div` clock is active and valid at the given power state.
    #[inline]
    pub fn ensure_fro_lf_div_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        self.ensure_clock_active(&self.fro_lf_div, "fro_lf_div", at_level)
    }

    /// Ensure the `fro_hf` clock is active and valid at the given power state.
    #[inline]
    pub fn ensure_fro_hf_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        self.ensure_clock_active(&self.fro_hf, "fro_hf", at_level)
    }

    /// Ensure the `fro_hf_div` clock is active and valid at the given power state.
    #[inline]
    pub fn ensure_fro_hf_div_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        self.ensure_clock_active(&self.fro_hf_div, "fro_hf_div", at_level)
    }

    /// Ensure the `clk_in` clock is active and valid at the given power state.
    #[cfg(not(feature = "sosc-as-gpio"))]
    #[inline]
    pub fn ensure_clk_in_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        self.ensure_clock_active(&self.clk_in, "clk_in", at_level)
    }

    /// Ensure the `clk_16k_vsys` clock is active and valid at the given power state.
    pub fn ensure_clk_16k_vsys_active(&self, _at_level: &PoweredClock) -> Result<u32, ClockError> {
        // NOTE: clk_16k is always active in low power mode
        Ok(self
            .clk_16k_vsys
            .as_ref()
            .ok_or(ClockError::BadConfig {
                clock: "clk_16k_vsys",
                reason: "required but not active",
            })?
            .frequency)
    }

    /// Ensure the `clk_16k_vdd_core` clock is active and valid at the given power state.
    pub fn ensure_clk_16k_vdd_core_active(&self, _at_level: &PoweredClock) -> Result<u32, ClockError> {
        // NOTE: clk_16k is always active in low power mode
        Ok(self
            .clk_16k_vdd_core
            .as_ref()
            .ok_or(ClockError::BadConfig {
                clock: "clk_16k_vdd_core",
                reason: "required but not active",
            })?
            .frequency)
    }

    /// Ensure the `clk_1m` clock is active and valid at the given power state.
    #[inline]
    pub fn ensure_clk_1m_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        self.ensure_clock_active(&self.clk_1m, "clk_1m", at_level)
    }

    /// Ensure the `pll1_clk` clock is active and valid at the given power state.
    #[inline]
    pub fn ensure_pll1_clk_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        self.ensure_clock_active(&self.pll1_clk, "pll1_clk", at_level)
    }

    /// Ensure the `pll1_clk_div` clock is active and valid at the given power state.
    #[inline]
    pub fn ensure_pll1_clk_div_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        self.ensure_clock_active(&self.pll1_clk_div, "pll1_clk_div", at_level)
    }

    /// Ensure the `CPU_CLK` or `SYSTEM_CLK` is active
    pub fn ensure_cpu_system_clk_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        let Some(clk) = self.cpu_system_clk.as_ref() else {
            return Err(ClockError::BadConfig {
                clock: "cpu_system_clk",
                reason: "required but not active",
            });
        };
        // Can the main_clk ever be active in deep sleep? I think it is gated?
        match at_level {
            PoweredClock::NormalEnabledDeepSleepDisabled => {}
            PoweredClock::AlwaysEnabled => {
                return Err(ClockError::BadConfig {
                    clock: "main_clk",
                    reason: "not low power active",
                });
            }
        }

        Ok(clk.frequency)
    }

    pub fn ensure_slow_clk_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        let freq = self.ensure_cpu_system_clk_active(at_level)?;

        Ok(freq / 6)
    }
}

impl PoweredClock {
    /// Does THIS clock meet the power requirements of the OTHER clock?
    pub fn meets_requirement_of(&self, other: &Self) -> bool {
        match (self, other) {
            (PoweredClock::NormalEnabledDeepSleepDisabled, PoweredClock::AlwaysEnabled) => false,
            (PoweredClock::NormalEnabledDeepSleepDisabled, PoweredClock::NormalEnabledDeepSleepDisabled) => true,
            (PoweredClock::AlwaysEnabled, PoweredClock::NormalEnabledDeepSleepDisabled) => true,
            (PoweredClock::AlwaysEnabled, PoweredClock::AlwaysEnabled) => true,
        }
    }
}

impl ClockOperator<'_> {
    fn active_limits(&self) -> &ClockLimits {
        match self.config.vdd_power.active_mode.level {
            VddLevel::MidDriveMode => &ClockLimits::MID_DRIVE,
            VddLevel::OverDriveMode => &ClockLimits::OVER_DRIVE,
        }
    }

    /// Configure the FIRC/FRO180M clock family
    ///
    /// NOTE: Currently we require this to be a fairly hardcoded value, as this clock is used
    /// as the main clock used for the CPU, AHB, APB, etc.
    fn configure_firc_clocks(&mut self) -> Result<(), ClockError> {
        // Three options here:
        //
        // * Firc is disabled -> Switch main clock to SIRC and return
        // * Firc is enabled and !default ->
        //   * Switch main clock to SIRC
        //   * Make FIRC changes
        //   * Switch main clock back to FIRC
        // * Firc is enabled and default -> nop
        let is_default = self
            .config
            .firc
            .as_ref()
            .is_some_and(|c| matches!(c.frequency, FircFreqSel::Mhz45));

        // If we are not default, then we need to switch to SIRC
        if !is_default {
            // Set SIRC (fro_12m) as the source
            self.scg0.rccr().modify(|w| w.set_scs(Scs::SIRC));

            // Wait for the change to complete
            while self.scg0.csr().read().scs() == Scs::SIRC {}
        }

        // Enable CSR writes
        self.scg0.firccsr().modify(|w| w.set_lk(FirccsrLk::WRITE_ENABLED));

        // Did the user give us a FIRC config?
        let Some(firc) = self.config.firc.as_ref() else {
            // Nope, and we've already switched to fro_12m. Disable FIRC.
            self.scg0.firccsr().modify(|w| {
                w.set_fircsten(Fircsten::DISABLED_IN_STOP_MODES);
                w.set_fircerr_ie(FircerrIe::ERROR_NOT_DETECTED);
                w.set_fircen(false);
            });

            self.scg0.firccsr().modify(|w| w.set_lk(FirccsrLk::WRITE_DISABLED));
            return Ok(());
        };

        // If we are here, we WANT FIRC. If we are !default, let's disable FIRC before
        // we mess with it. If we are !default, we have already switched to SIRC instead!
        if !is_default {
            // Unlock
            self.scg0.firccsr().modify(|w| w.set_lk(FirccsrLk::WRITE_ENABLED));

            // Disable FIRC
            self.scg0.firccsr().modify(|w| {
                w.set_fircen(false);
                w.set_fircsten(Fircsten::DISABLED_IN_STOP_MODES);
                w.set_fircerr_ie(FircerrIe::ERROR_NOT_DETECTED);
                w.set_fircacc_ie(FircaccIe::FIRCACCNOT);
                w.set_firc_sclk_periph_en(false);
                w.set_firc_fclk_periph_en(false);
            });
        }

        // Set frequency (if not the default 45MHz!), re-enable FIRC, and return the base frequency
        //
        // NOTE: the SVD currently has the wrong(?) values for these:
        // 45 -> 48
        // 60 -> 64
        // 90 -> 96
        // 180 -> 192
        //
        // Probably correct-ish, but for a different trim value?
        let base_freq = match firc.frequency {
            FircFreqSel::Mhz45 => {
                // We are default, there's nothing to do here.
                45_000_000
            }
            FircFreqSel::Mhz60 => {
                self.scg0.firccfg().modify(|w| w.set_freq_sel(FreqSel::FIRC_64MHZ));
                self.scg0.firccsr().modify(|w| w.set_fircen(true));
                60_000_000
            }
            FircFreqSel::Mhz90 => {
                self.scg0.firccfg().modify(|w| w.set_freq_sel(FreqSel::FIRC_96MHZ));
                self.scg0.firccsr().modify(|w| w.set_fircen(true));
                90_000_000
            }
            FircFreqSel::Mhz180 => {
                self.scg0.firccfg().modify(|w| w.set_freq_sel(FreqSel::FIRC_192MHZ));
                self.scg0.firccsr().modify(|w| w.set_fircen(true));
                180_000_000
            }
        };

        // Wait for FIRC to be enabled, error-free, and accurate
        let mut firc_ok = false;
        while !firc_ok {
            let csr = self.scg0.firccsr().read();

            firc_ok = csr.fircen()
                && csr.fircacc() == Fircacc::ENABLED_AND_VALID
                && csr.fircerr() == Fircerr::ERROR_NOT_DETECTED;
        }

        // Note that the fro_hf_root is active
        self.clocks.fro_hf_root = Some(Clock {
            frequency: base_freq,
            power: firc.power,
        });

        // Okay! Now we're past that, let's enable all the downstream clocks.
        let FircConfig {
            frequency: _,
            power,
            fro_hf_enabled,
            clk_45m_enabled,
            fro_hf_div,
        } = firc;

        // When is the FRO enabled?
        let pow_set = match power {
            PoweredClock::NormalEnabledDeepSleepDisabled => Fircsten::DISABLED_IN_STOP_MODES,
            PoweredClock::AlwaysEnabled => Fircsten::ENABLED_IN_STOP_MODES,
        };

        // Do we enable the `fro_hf` output?
        let fro_hf_set = if *fro_hf_enabled {
            if base_freq > self.active_limits().fro_hf {
                return Err(ClockError::BadConfig {
                    clock: "fro_hf",
                    reason: "exceeds max",
                });
            }

            self.clocks.fro_hf = Some(Clock {
                frequency: base_freq,
                power: *power,
            });
            true
        } else {
            false
        };

        // Do we enable the `clk_45m` output?
        let clk_45m_set = if *clk_45m_enabled {
            self.clocks.clk_45m = Some(Clock {
                frequency: 45_000_000,
                power: *power,
            });
            true
        } else {
            false
        };

        self.scg0.firccsr().modify(|w| {
            w.set_fircsten(pow_set);
            w.set_firc_fclk_periph_en(fro_hf_set);
            w.set_firc_sclk_periph_en(clk_45m_set);
        });

        // Last write to CSR, re-lock
        self.scg0.firccsr().modify(|w| w.set_lk(FirccsrLk::WRITE_DISABLED));

        // Do we enable the `fro_hf_div` output?
        if let Some(d) = fro_hf_div.as_ref() {
            // We need `fro_hf` to be enabled
            if !*fro_hf_enabled {
                return Err(ClockError::BadConfig {
                    clock: "fro_hf_div",
                    reason: "fro_hf not enabled",
                });
            }

            let div_freq = base_freq / d.into_divisor();
            if div_freq > self.active_limits().fro_hf_div {
                return Err(ClockError::BadConfig {
                    clock: "fro_hf_root",
                    reason: "exceeds max frequency",
                });
            }

            // Halt and reset the div; then set our desired div.
            self.syscon.frohfdiv().write(|w| {
                w.set_halt(FrohfdivHalt::HALT);
                w.set_reset(FrohfdivReset::ASSERTED);
                w.set_div(d.into_bits());
            });
            // Then unhalt it, and reset it
            self.syscon.frohfdiv().write(|w| {
                w.set_halt(FrohfdivHalt::RUN);
                w.set_reset(FrohfdivReset::RELEASED);
            });

            // Wait for clock to stabilize
            while self.syscon.frohfdiv().read().unstab() == FrohfdivUnstab::ONGOING {}

            // Store off the clock info
            self.clocks.fro_hf_div = Some(Clock {
                frequency: div_freq,
                power: *power,
            });
        }

        Ok(())
    }

    /// Configure the SIRC/FRO12M clock family
    fn configure_sirc_clocks_early(&mut self) -> Result<(), ClockError> {
        let SircConfig {
            power,
            fro_12m_enabled,
            fro_lf_div,
        } = &self.config.sirc;
        let base_freq = 12_000_000;

        // Allow writes
        self.scg0.sirccsr().modify(|w| w.set_lk(SirccsrLk::WRITE_ENABLED));
        self.clocks.fro_12m_root = Some(Clock {
            frequency: base_freq,
            power: *power,
        });

        let deep = match power {
            PoweredClock::NormalEnabledDeepSleepDisabled => false,
            PoweredClock::AlwaysEnabled => true,
        };

        // clk_1m is *before* the fro_12m clock gate
        self.clocks.clk_1m = Some(Clock {
            frequency: base_freq / 12,
            power: *power,
        });

        // If the user wants fro_12m to be disabled, FOR now, we ignore their
        // wish to ensure fro_12m is selectable as a main_clk source at least until
        // we select the CPU clock. We still mark it as not enabled though, to prevent
        // other peripherals using it, as we will gate if off at `configure_sirc_clocks_late`.
        if *fro_12m_enabled {
            self.clocks.fro_12m = Some(Clock {
                frequency: base_freq,
                power: *power,
            });
        } else {
            self.sirc_forced = true;
        };

        // Set sleep/peripheral usage
        self.scg0.sirccsr().modify(|w| {
            w.set_sircsten(deep);
            // Always on, for now at least! Will be resolved in `configure_sirc_clocks_late`
            w.set_sirc_clk_periph_en(true);
        });

        while self.scg0.sirccsr().read().sircvld() == Sircvld::DISABLED_OR_NOT_VALID {}
        if self.scg0.sirccsr().read().sircerr() == Sircerr::ERROR_DETECTED {
            return Err(ClockError::BadConfig {
                clock: "sirc",
                reason: "error set",
            });
        }

        // reset lock
        self.scg0.sirccsr().modify(|w| w.set_lk(SirccsrLk::WRITE_DISABLED));

        // Do we enable the `fro_lf_div` output?
        if let Some(d) = fro_lf_div.as_ref() {
            // We need `fro_lf` to be enabled
            if !*fro_12m_enabled {
                return Err(ClockError::BadConfig {
                    clock: "fro_lf_div",
                    reason: "fro_12m not enabled",
                });
            }

            // Halt and reset the div; then set our desired div.
            self.syscon.frolfdiv().write(|w| {
                w.set_halt(FrolfdivHalt::HALT);
                w.set_reset(FrolfdivReset::ASSERTED);
                w.set_div(d.into_bits());
            });
            // Then unhalt it, and reset it
            self.syscon.frolfdiv().modify(|w| {
                w.set_halt(FrolfdivHalt::RUN);
                w.set_reset(FrolfdivReset::RELEASED);
            });

            // Wait for clock to stabilize
            while self.syscon.frolfdiv().read().unstab() == FrolfdivUnstab::ONGOING {}

            // Store off the clock info
            self.clocks.fro_lf_div = Some(Clock {
                frequency: base_freq / d.into_divisor(),
                power: *power,
            });
        }

        Ok(())
    }

    fn configure_sirc_clocks_late(&mut self) {
        // If we forced SIRC's fro_12m to be enabled, disable it now.
        if self.sirc_forced {
            // Allow writes
            self.scg0.sirccsr().modify(|w| w.set_lk(SirccsrLk::WRITE_ENABLED));

            // Disable clk_12m
            self.scg0.sirccsr().modify(|w| w.set_sirc_clk_periph_en(false));

            // reset lock
            self.scg0.sirccsr().modify(|w| w.set_lk(SirccsrLk::WRITE_DISABLED));
        }
    }

    /// Configure the ROSC/FRO16K/clk_16k clock family
    fn configure_fro16k_clocks(&mut self) -> Result<(), ClockError> {
        // If we have a config: ensure fro16k is enabled. If not: ensure it is disabled.
        let enable = self.config.fro16k.is_some();
        self.vbat0.froctla().modify(|w| w.set_fro_en(enable));

        // Lock the control register
        self.vbat0.frolcka().modify(|w| w.set_lock(true));

        // If we're disabled, we're done!
        let Some(fro16k) = self.config.fro16k.as_ref() else {
            return Ok(());
        };

        // Enabled, now set up.
        let Fro16KConfig {
            vsys_domain_active,
            vdd_core_domain_active,
        } = fro16k;

        // Enable clock outputs to both VSYS and VDD_CORE domains
        // Bit 0: clk_16k0 to VSYS domain
        // Bit 1: clk_16k1 to VDD_CORE domain
        //
        // TODO: Define sub-fields for this register with a PAC patch?
        let mut bits = 0;
        if *vsys_domain_active {
            bits |= 0b01;
            self.clocks.clk_16k_vsys = Some(Clock {
                frequency: 16_384,
                power: PoweredClock::AlwaysEnabled,
            });
        }
        if *vdd_core_domain_active {
            bits |= 0b10;
            self.clocks.clk_16k_vdd_core = Some(Clock {
                frequency: 16_384,
                power: PoweredClock::AlwaysEnabled,
            });
        }
        self.vbat0.froclke().modify(|w| w.set_clke(bits));

        Ok(())
    }

    fn ensure_ldo_active(&mut self) {
        // TODO: Config for the LDO? For now, just enable
        // using the default settings:
        // LDOBYPASS: 0/not bypassed
        // VOUT_SEL: 0b100: 1.1v
        // LDOEN: 0/Disabled
        let already_enabled = {
            let ldocsr = self.scg0.ldocsr().read();
            ldocsr.ldoen() && ldocsr.vout_ok()
        };
        if !already_enabled {
            self.scg0.ldocsr().modify(|w| w.set_ldoen(true));
            while !self.scg0.ldocsr().read().vout_ok() {}
        }
    }

    /// Configure the SOSC/clk_in oscillator
    #[cfg(not(feature = "sosc-as-gpio"))]
    fn configure_sosc(&mut self) -> Result<(), ClockError> {
        let Some(parts) = self.config.sosc.as_ref() else {
            return Ok(());
        };

        // Enable (and wait for) LDO to be active
        self.ensure_ldo_active();

        let eref = match parts.mode {
            config::SoscMode::CrystalOscillator => Erefs::INTERNAL,
            config::SoscMode::ActiveClock => Erefs::EXTERNAL,
        };
        let freq = parts.frequency;

        // TODO: Fix PAC names here
        //
        // #[doc = "0: Frequency range select of 8-16 MHz."]
        // Freq16to20mhz = 0,
        // #[doc = "1: Frequency range select of 16-25 MHz."]
        // LowFreq = 1,
        // #[doc = "2: Frequency range select of 25-40 MHz."]
        // MediumFreq = 2,
        // #[doc = "3: Frequency range select of 40-50 MHz."]
        // HighFreq = 3,
        let range = match freq {
            0..8_000_000 => {
                return Err(ClockError::BadConfig {
                    clock: "clk_in",
                    reason: "freq too low",
                });
            }
            8_000_000..16_000_000 => Range::FREQ_16TO20MHZ,
            16_000_000..25_000_000 => Range::LOW_FREQ,
            25_000_000..40_000_000 => Range::MEDIUM_FREQ,
            40_000_000..50_000_001 => Range::HIGH_FREQ,
            50_000_001.. => {
                return Err(ClockError::BadConfig {
                    clock: "clk_in",
                    reason: "freq too high",
                });
            }
        };

        // Set source/erefs and range
        self.scg0.sosccfg().modify(|w| {
            w.set_erefs(eref);
            w.set_range(range);
        });

        // Disable lock
        self.scg0.sosccsr().modify(|w| w.set_lk(SosccsrLk::WRITE_ENABLED));

        // TODO: We could enable the SOSC clock monitor. There are some things to
        // figure out first:
        //
        // * This requires SIRC to be enabled, not sure which branch. Maybe fro12m_root?
        // * If SOSC needs to work in deep sleep, AND the monitor is enabled:
        //   * SIRC also need needs to be low power
        // * We need to decide if we need an interrupt or a reset if the monitor trips

        // Apply remaining config
        self.scg0.sosccsr().modify(|w| {
            // For now, just disable the monitor. See above.
            w.set_sosccm(false);

            // Set deep sleep mode
            match parts.power {
                PoweredClock::NormalEnabledDeepSleepDisabled => {
                    w.set_soscsten(false);
                }
                PoweredClock::AlwaysEnabled => {
                    w.set_soscsten(true);
                }
            }

            // Enable SOSC
            w.set_soscen(true)
        });

        // Wait for SOSC to be valid, check for errors
        while !self.scg0.sosccsr().read().soscvld() {}
        if self.scg0.sosccsr().read().soscerr() == Soscerr::ENABLED_AND_ERROR {
            return Err(ClockError::BadConfig {
                clock: "clk_in",
                reason: "soscerr is set",
            });
        }

        // Re-lock the sosc
        self.scg0.sosccsr().modify(|w| w.set_lk(SosccsrLk::WRITE_DISABLED));

        self.clocks.clk_in = Some(Clock {
            frequency: freq,
            power: parts.power,
        });

        Ok(())
    }

    fn configure_spll(&mut self) -> Result<(), ClockError> {
        // # Vocab
        //
        // | Name   | Meaning                                                     |
        // | :---   | :---                                                        |
        // | Fin    | Frequency of clkin                                          |
        // | clkout | Output clock of the PLL                                     |
        // | Fout   | Frequency of clkout (depends on mode)                       |
        // | clkref | PLL Reference clock, the input clock to the PFD             |
        // | Fref   | Frequency of clkref, Fref = Fin / N                         |
        // | Fcco   | Frequency of the output clock of the CCO, Fcco = M * Fref   |
        // | N      | Predivider value                                            |
        // | M      | Feedback divider value                                      |
        // | P      | Postdivider value                                           |
        // | Tpon   | PLL start-up time                                           |

        // No PLL? Nothing to do!
        let Some(cfg) = self.config.spll.as_ref() else {
            return Ok(());
        };

        // Ensure the LDO is active
        self.ensure_ldo_active();

        // match on the source, ensure it is active already
        let res = match cfg.source {
            #[cfg(not(feature = "sosc-as-gpio"))]
            config::SpllSource::Sosc => self
                .clocks
                .clk_in
                .as_ref()
                .map(|c| (c, Source::SOSC))
                .ok_or("sosc not active"),
            config::SpllSource::Firc => self
                .clocks
                .clk_45m
                .as_ref()
                .map(|c| (c, Source::FIRC))
                .ok_or("firc not active"),
            config::SpllSource::Sirc => self
                .clocks
                .fro_12m
                .as_ref()
                .map(|c| (c, Source::SIRC))
                .ok_or("sirc not active"),
        };
        // This checks if active
        let (clk, variant) = res.map_err(|s| ClockError::BadConfig {
            clock: "spll",
            reason: s,
        })?;
        // This checks the correct power reqs
        if !clk.power.meets_requirement_of(&cfg.power) {
            return Err(ClockError::BadConfig {
                clock: "spll",
                reason: "needs low power source",
            });
        }

        // Bandwidth calc
        //
        // > In normal applications, you must calculate the bandwidth manually by using the feedback divider M (ranging from 1 to (2^16)-1),
        // > Equation 1, and Equation 2. The PLL is automatically stable in such case. In normal applications, SPLLCTRL[BANDDIRECT] must
        // > be 0; in this case, the bandwidth changes as a function of M.
        if clk.frequency == 0 {
            return Err(ClockError::BadConfig {
                clock: "spll",
                reason: "internal error",
            });
        }

        // These are calculated differently depending on the mode.
        let f_in = clk.frequency;
        let bp_pre: bool;
        let bp_post: bool;
        let bp_post2: bool;
        let m: u16;
        let p: Option<u8>;
        let n: Option<u8>;

        // Calculate both Fout and Fcco so we can ensure they don't overflow
        // and are in range
        let fout: Option<u32>;
        let fcco: Option<u32>;

        let m_check = |m: u16| {
            if !(1..=u16::MAX).contains(&m) {
                Err(ClockError::BadConfig {
                    clock: "spll",
                    reason: "m_mult out of range",
                })
            } else {
                Ok(m)
            }
        };
        let p_check = |p: u8| {
            if !(1..=31).contains(&p) {
                Err(ClockError::BadConfig {
                    clock: "spll",
                    reason: "p_div out of range",
                })
            } else {
                Ok(p)
            }
        };
        let n_check = |n: u8| {
            if !(1..=u8::MAX).contains(&n) {
                Err(ClockError::BadConfig {
                    clock: "spll",
                    reason: "n_div out of range",
                })
            } else {
                Ok(n)
            }
        };

        match cfg.mode {
            // Fout = M x Fin
            config::SpllMode::Mode1a { m_mult } => {
                bp_pre = true;
                bp_post = true;
                bp_post2 = false;
                m = m_check(m_mult)?;
                p = None;
                n = None;
                fcco = f_in.checked_mul(m_mult as u32);
                fout = fcco;
            }
            // if !bypass_p2_div: Fout = (M / (2 x P)) x Fin
            // if  bypass_p2_div: Fout = (M /    P   ) x Fin
            config::SpllMode::Mode1b {
                m_mult,
                p_div,
                bypass_p2_div,
            } => {
                bp_pre = true;
                bp_post = false;
                bp_post2 = bypass_p2_div;
                m = m_check(m_mult)?;
                p = Some(p_check(p_div)?);
                n = None;
                let mut div = p_div as u32;
                if !bypass_p2_div {
                    div *= 2;
                }
                fcco = f_in.checked_mul(m_mult as u32);
                fout = (f_in / div).checked_mul(m_mult as u32);
            }
            // Fout = (M / N) x Fin
            config::SpllMode::Mode1c { m_mult, n_div } => {
                bp_pre = false;
                bp_post = true;
                bp_post2 = false;
                m = m_check(m_mult)?;
                p = None;
                n = Some(n_check(n_div)?);
                fcco = (f_in / (n_div as u32)).checked_mul(m_mult as u32);
                fout = fcco;
            }
            // if !bypass_p2_div: Fout = (M / (N x 2 x P)) x Fin
            // if  bypass_p2_div: Fout = (M / (  N x P  )) x Fin
            config::SpllMode::Mode1d {
                m_mult,
                n_div,
                p_div,
                bypass_p2_div,
            } => {
                bp_pre = false;
                bp_post = false;
                bp_post2 = bypass_p2_div;
                m = m_check(m_mult)?;
                p = Some(p_check(p_div)?);
                n = Some(n_check(n_div)?);
                // This can't overflow: u8 x u8 (x 2) always fits in u32
                let mut div = (p_div as u32) * (n_div as u32);
                if !bypass_p2_div {
                    div *= 2;
                }
                fcco = (f_in / (n_div as u32)).checked_mul(m_mult as u32);
                fout = (f_in / div).checked_mul(m_mult as u32);
            }
        };

        // Dump all the PLL calcs if needed for debugging
        #[cfg(feature = "defmt")]
        {
            defmt::debug!("f_in: {:?}", f_in);
            defmt::debug!("bp_pre: {:?}", bp_pre);
            defmt::debug!("bp_post: {:?}", bp_post);
            defmt::debug!("bp_post2: {:?}", bp_post2);
            defmt::debug!("m: {:?}", m);
            defmt::debug!("p: {:?}", p);
            defmt::debug!("n: {:?}", n);
            defmt::debug!("fout: {:?}", fout);
            defmt::debug!("fcco: {:?}", fcco);
        }

        // Ensure the Fcco and Fout calcs didn't overflow
        let fcco = fcco.ok_or(ClockError::BadConfig {
            clock: "spll",
            reason: "fcco invalid1",
        })?;
        let fout = fout.ok_or(ClockError::BadConfig {
            clock: "spll",
            reason: "fout invalid",
        })?;

        // Fcco: 275MHz to 550MHz
        if !(275_000_000..=550_000_000).contains(&fcco) {
            return Err(ClockError::BadConfig {
                clock: "spll",
                reason: "fcco invalid2",
            });
        }

        // Fout: 4.3MHz to 2x Max CPU Frequency
        let fmax = match self.config.vdd_power.active_mode.level {
            VddLevel::MidDriveMode => ClockLimits::MID_DRIVE.cpu_clk,
            VddLevel::OverDriveMode => ClockLimits::OVER_DRIVE.cpu_clk,
        };
        let spll_range_bad1 = !(4_300_000..=(2 * fmax)).contains(&fout);
        let spll_range_bad2 = fout > self.active_limits().pll1_clk;

        if spll_range_bad1 || spll_range_bad2 {
            return Err(ClockError::BadConfig {
                clock: "spll",
                reason: "fout invalid",
            });
        }

        // A = floor(m / 4) + 1
        let selp_a = (m / 4) + 1;
        // SELP = A  if A <  31
        //      = 31 if A >= 31
        let selp = selp_a.min(31);

        // A = 1                    if        M >= 8000
        //   = floor(8000 / M)      if 8000 > M >= 122
        //   = 2 x floor(M / 4) / 3 if 122  > M >= 1
        let seli_a = if m >= 8000 {
            1
        } else if m >= 122 {
            8000 / m
        } else {
            (2 * (m / 4)) / 3
        };
        // SELI = A  if A <  63
        //      = 63 if A >= 63
        let seli = seli_a.min(63);
        // SELR must be 0.
        let selr = 0;

        self.scg0.spllctrl().modify(|w| {
            w.set_source(variant);
            w.set_selp(selp as u8);
            w.set_seli(seli as u8);
            w.set_selr(selr);
        });

        if let Some(n) = n {
            self.scg0.spllndiv().modify(|w| w.set_ndiv(n));
        }
        if let Some(p) = p {
            self.scg0.spllpdiv().modify(|w| w.set_pdiv(p));
        }
        self.scg0.spllmdiv().modify(|w| w.set_mdiv(m));

        self.scg0.spllctrl().modify(|w| {
            w.set_bypassprediv(bp_pre);
            w.set_bypasspostdiv(bp_post);
            w.set_bypasspostdiv2(bp_post2);

            // TODO: support FRM?
            w.set_frm(false);
        });

        // Unlock
        self.scg0.spllcsr().modify(|w| w.set_lk(SpllcsrLk::WRITE_ENABLED));

        // TODO: Support clock monitors?
        // self.scg0.spllcsr().modify(|w| w.spllcm().?);

        self.scg0.trim_lock().write(|w| {
            w.set_trim_lock_key(0x5a5a);
            w.set_trim_unlock(TrimUnlock::NOT_LOCKED)
        });

        // SPLLLOCK_CNFG: The lock time programmed in this register must be
        // equal to meet the PLL 500μs lock time plus the 300 refclk count startup.
        //
        // LOCK_TIME = 500μs/T ref + 300, F ref = F in /N (input frequency divided by pre-divider ratio).
        //
        // 500us is 1/2000th of a second, therefore Fref / 2000 is the number of cycles in 500us.
        let f_ref = if let Some(n) = n { f_in / (n as u32) } else { f_in };
        let lock_time = f_ref.div_ceil(2000) + 300;
        self.scg0.splllock_cnfg().write(|w| w.set_lock_time(lock_time));

        // TODO: Support Spread spectrum?

        self.scg0.spllcsr().modify(|w| {
            w.set_spllclken(true);
            w.set_spllpwren(true);
            w.set_spllsten(if matches!(cfg.power, PoweredClock::AlwaysEnabled) {
                Spllsten::ENABLED_IN_STOP
            } else {
                Spllsten::DISABLED_IN_STOP
            });
        });

        // Wait for SPLL to set up
        loop {
            let csr = self.scg0.spllcsr().read();
            if csr.spll_lock() == SpllLock::ENABLED_AND_VALID {
                if csr.spllerr() == Spllerr::ENABLED_AND_ERROR {
                    return Err(ClockError::BadConfig {
                        clock: "spll",
                        reason: "spllerr is set",
                    });
                }
                break;
            }
        }

        // Re-lock SPLL CSR
        self.scg0.spllcsr().modify(|w| w.set_lk(SpllcsrLk::WRITE_DISABLED));

        // Store clock state
        self.clocks.pll1_clk = Some(Clock {
            frequency: fout,
            power: cfg.power,
        });

        // Do we enable the `pll1_clk_div` output?
        if let Some(d) = cfg.pll1_clk_div.as_ref() {
            // Halt and reset the div; then set our desired div.
            self.syscon.pll1clkdiv().write(|w| {
                w.set_halt(Pll1clkdivHalt::HALT);
                w.set_reset(Pll1clkdivReset::ASSERTED);
                w.set_div(d.into_bits());
            });
            // Then unhalt it, and reset it
            self.syscon.pll1clkdiv().write(|w| {
                w.set_halt(Pll1clkdivHalt::RUN);
                w.set_reset(Pll1clkdivReset::RELEASED);
            });

            // Wait for clock to stabilize
            while self.syscon.pll1clkdiv().read().unstab() == Pll1clkdivUnstab::ONGOING {}

            // Store off the clock info
            self.clocks.pll1_clk_div = Some(Clock {
                frequency: fout / d.into_divisor(),
                power: cfg.power,
            });
        }

        Ok(())
    }

    fn configure_main_clk(&mut self) -> Result<(), ClockError> {
        let (var, name, clk) = match self.config.main_clock.source {
            #[cfg(not(feature = "sosc-as-gpio"))]
            MainClockSource::SoscClkIn => (Scs::SOSC, "clk_in", self.clocks.clk_in.as_ref()),
            MainClockSource::SircFro12M => (Scs::SIRC, "fro_12m", self.clocks.fro_12m.as_ref()),
            MainClockSource::FircHfRoot => (Scs::FIRC, "fro_hf_root", self.clocks.fro_hf_root.as_ref()),
            MainClockSource::RoscFro16K => (Scs::ROSC, "fro16k", self.clocks.clk_16k_vdd_core.as_ref()),
            MainClockSource::SPll1 => (Scs::SPLL, "pll1_clk", self.clocks.pll1_clk.as_ref()),
        };
        let Some(main_clk_src) = clk else {
            return Err(ClockError::BadConfig {
                clock: name,
                reason: "Needed for main_clock but not enabled",
            });
        };

        if !main_clk_src.power.meets_requirement_of(&self.config.main_clock.power) {
            return Err(ClockError::BadConfig {
                clock: name,
                reason: "Needed for main_clock but not low power",
            });
        }

        let (levels, mclk_max, cpuclk_max, wsmax) = match self.config.vdd_power.active_mode.level {
            VddLevel::MidDriveMode => (
                VDD_CORE_MID_DRIVE_WAIT_STATE_LIMITS,
                ClockLimits::MID_DRIVE.main_clk,
                ClockLimits::MID_DRIVE.cpu_clk,
                VDD_CORE_MID_DRIVE_MAX_WAIT_STATES,
            ),
            VddLevel::OverDriveMode => (
                VDD_CORE_OVER_DRIVE_WAIT_STATE_LIMITS,
                ClockLimits::OVER_DRIVE.main_clk,
                ClockLimits::OVER_DRIVE.cpu_clk,
                VDD_CORE_OVER_DRIVE_MAX_WAIT_STATES,
            ),
        };

        // Is the main_clk source in range for main_clk?
        if main_clk_src.frequency > mclk_max {
            return Err(ClockError::BadConfig {
                clock: name,
                reason: "Exceeds main_clock frequency",
            });
        }

        // Calculate expected CPU frequency based on main_clk and AHB div
        let ahb_div = self.config.main_clock.ahb_clk_div;
        let cpu_freq = main_clk_src.frequency / ahb_div.into_divisor();

        // Is the expected CPU frequency in range for cpu_clk?
        if cpu_freq > cpuclk_max {
            return Err(ClockError::BadConfig {
                clock: name,
                reason: "Exceeds ahb max frequency",
            });
        }

        // BEFORE we switch, update the flash wait states to the appropriate levels
        //
        // NOTE: "cpu_clk" is the same as "system_clk". Table 22 is not clear exactly
        // WHICH source clock the limits apply to, but system/ahb/cpu is a fair bet.
        //
        // TODO: This calculation doesn't consider low power mode yet!
        let wait_states = levels
            .iter()
            .find(|(fmax, _ws)| cpu_freq <= *fmax)
            .map(|t| t.1)
            .unwrap_or(wsmax);
        self.fmu0.fctrl().modify(|w| w.set_rwsc(wait_states));

        // TODO: (Double) check if clock is actually valid before switching?
        // Are we already on the right clock?
        let now = self.scg0.csr().read().scs();
        if now != var {
            // Set RCCR
            self.scg0.rccr().modify(|w| w.set_scs(var));

            // Wait for match
            while self.scg0.csr().read().scs() != var {}
        }

        // The main_clk is now set to the selected input clock
        self.clocks.main_clk = Some(main_clk_src.clone());

        // Update AHB clock division, if necessary
        if ahb_div.into_bits() != 0 {
            // AHB has no halt/reset fields - it's different to other DIV8s!
            self.syscon.ahbclkdiv().modify(|w| w.set_div(ahb_div.into_bits()));
            // Wait for clock to stabilize
            while self.syscon.ahbclkdiv().read().unstab() == AhbclkdivUnstab::ONGOING {}
        }

        // Store off the clock info
        self.clocks.cpu_system_clk = Some(Clock {
            frequency: cpu_freq,
            power: main_clk_src.power,
        });

        Ok(())
    }

    fn configure_voltages(&mut self) -> Result<(), ClockError> {
        match self.config.vdd_power.active_mode.level {
            VddLevel::MidDriveMode => {
                // This is the default mode, I don't believe we need to do anything.
                //
                // "The LVDE and HVDE fields reset only with a POR.
                // All other fields reset only with a system reset."
            }
            VddLevel::OverDriveMode => {
                // You can change the core VDD levels for the LDO_CORE low power regulator only
                // when CORELDO_VDD_DS=1.
                //
                // When switching CORELDO_VDD_DS from low to normal drive strength, ensure the LDO_CORE high
                // VDD LVL setting is set to the same level that was set prior to switching to the LDO_CORE drive strength
                // (CORELDO_VDD_DS). Otherwise, if the LVDs are enabled, an unexpected LVD can occur.
                //
                // Ensure drive strength is normal (BEFORE shifting level)
                self.spc0
                    .active_cfg()
                    .modify(|w| w.set_coreldo_vdd_ds(ActiveCfgCoreldoVddDs::NORMAL));

                // ## DS 26.3.2:
                //
                // When increasing voltage and frequency in Active mode, you must perform the following steps:
                //
                // 1. Increase voltage to a new level (ACTIVE_CFG[CORELDO_VDD_LVL]).
                self.spc0
                    .active_cfg()
                    .modify(|w| w.set_coreldo_vdd_lvl(ActiveCfgCoreldoVddLvl::OVER));

                // 2. Wait for voltage change to complete (SC[BUSY] = 0).
                while self.spc0.sc().read().busy() {}

                // 3. Configure flash memory to support higher voltage level and frequency (FMU_FCTRL[RWSC].
                //
                // NOTE: This step skipped - we will update RWSC when we later apply main cpu clock
                // frequency changes.

                // 4. Configure SRAM to support higher voltage levels (SRAMCTL[VSM]).
                self.spc0.sramctl().modify(|w| w.set_vsm(Vsm::SRAM1V2));

                // 5. Request SRAM voltage update (write 1 to SRAMCTL[REQ]).
                self.spc0.sramctl().modify(|w| w.set_req(true));

                // 6. Wait for SRAM voltage change to complete (SRAMCTL[ACK] = 1).
                while !self.spc0.sramctl().read().ack() {}

                // 7. Clear request for SRAM voltage change (write 0 to SRAMCTL[REQ]).
                self.spc0.sramctl().modify(|w| w.set_req(false));

                // 8. Increase frequency to a new level (for example, SCG_RCCR).
                //
                // NOTE: This step skipped - we will update RCCR when we later apply main cpu clock
                // frequency changes.

                // 9. You can continue execution.
                // :)
            }
        }

        // If the CORELDO_VDD_DS fields are set to the same value in both the ACTIVE_CFG and LP_CFG registers,
        // the CORELDO_VDD_LVL's in the ACTIVE_CFG and LP_CFG register must be set to the same voltage
        // level settings.
        //
        // TODO(AJM): I don't really understand this! Enforce it literally for now I guess.
        let ds_match = self.config.vdd_power.active_mode.drive == self.config.vdd_power.low_power_mode.drive;
        let vdd_match = self.config.vdd_power.active_mode.level == self.config.vdd_power.low_power_mode.level;

        if ds_match && !vdd_match {
            return Err(ClockError::BadConfig {
                clock: "vdd_power",
                reason: "DS matches but LVL mismatches!",
            });
        }

        // You can change the core VDD levels for the LDO_CORE low power regulator only when
        // ACTIVE_CFG[CORELDO_VDD_DS] = 1. So, before entering any of the low-power states (DSLEEP,
        // PDOWN, DPDOWN) with LDO_CORE low power regulator selected (LP_CFG[CORELDO_VDD_DS] = 0),
        // you must use CORELDO_VDD_LVL to select the correct regulation level during ACTIVE run mode.
        //
        // NOTE(AJM): We've set drive strength to "normal" above, and do not (potentially) set it to
        // "low" until later below.

        // NOTE(AJM): The reference manual doesn't have any similar configuration requirements
        // for low power mode. We'll just configure it, I guess?
        //
        // NOTE(AJM): "LP_CFG: This register resets only after a POR or LVD event."
        let (ds, bgap) = match self.config.vdd_power.low_power_mode.drive {
            VddDriveStrength::Low { enable_bandgap } => {
                // If the bandgap is enabled, also enable the high/low voltage
                // detectors. if it is disabled, these must also be disabled.
                self.spc0.lp_cfg().modify(|w| {
                    w.set_sys_hvde(enable_bandgap);
                    w.set_sys_lvde(enable_bandgap);
                    w.set_core_lvde(enable_bandgap);
                });

                (pac::spc::vals::LpCfgCoreldoVddDs::LOW, enable_bandgap)
            }
            VddDriveStrength::Normal => {
                // "If you specify normal drive strength, you must write a value to LP[BGMODE] that enables the bandgap."
                (pac::spc::vals::LpCfgCoreldoVddDs::NORMAL, true)
            }
        };
        let lvl = match self.config.vdd_power.low_power_mode.level {
            VddLevel::MidDriveMode => LpCfgCoreldoVddLvl::MID,
            VddLevel::OverDriveMode => LpCfgCoreldoVddLvl::OVER,
        };
        self.spc0.lp_cfg().modify(|w| w.set_coreldo_vdd_ds(ds));

        // If we're enabling the bandgap, ensure we do it BEFORE changing the VDD level
        // If we're disabling the bandgap, ensure we do it AFTER changing the VDD level
        if bgap {
            self.spc0.lp_cfg().modify(|w| w.set_bgmode(LpCfgBgmode::BGMODE01));
            self.spc0.lp_cfg().modify(|w| w.set_coreldo_vdd_lvl(lvl));
        } else {
            self.spc0.lp_cfg().modify(|w| w.set_coreldo_vdd_lvl(lvl));
            self.spc0.lp_cfg().modify(|w| w.set_bgmode(LpCfgBgmode::BGMODE0));
        }

        // Updating CORELDO_VDD_LVL sets the SC[BUSY] flag. That flag remains set for at least the total time
        // delay that Active Voltage Trim Delay (ACTIVE_VDELAY) specifies.
        //
        // Before changing CORELDO_VDD_LVL, you must wait until the SC[BUSY] flag clears before entering the
        // selected low-power sleep
        //
        // NOTE(AJM): Let's just proactively wait now so we don't have to worry about it on subsequent sleeps
        while self.spc0.sc().read().busy() {}

        // NOTE(AJM): I don't really know if this is valid! I'm guessing in most cases you would want to
        // use the low drive strength for lp mode, and high drive strength for active mode?
        match self.config.vdd_power.active_mode.drive {
            VddDriveStrength::Low { enable_bandgap } => {
                // If the bandgap is enabled, also enable the high/low voltage
                // detectors. if it is disabled, these must also be disabled.
                self.spc0.active_cfg().modify(|w| {
                    w.set_sys_hvde(enable_bandgap);
                    w.set_sys_lvde(enable_bandgap);
                    w.set_core_lvde(enable_bandgap);
                });

                // optionally disable bandgap AFTER setting vdd strength to low
                self.spc0
                    .active_cfg()
                    .modify(|w| w.set_coreldo_vdd_ds(ActiveCfgCoreldoVddDs::LOW));
                self.spc0.active_cfg().modify(|w| {
                    if enable_bandgap {
                        w.set_bgmode(ActiveCfgBgmode::BGMODE01)
                    } else {
                        w.set_bgmode(ActiveCfgBgmode::BGMODE0)
                    }
                });
            }
            VddDriveStrength::Normal => {
                // Already set to normal above
            }
        }

        match self.config.vdd_power.core_sleep {
            CoreSleep::WfeUngated => {}
            CoreSleep::WfeGated => {
                // Allow automatic gating of the core when in LIGHT sleep
                self.cmc.ckctrl().modify(|w| w.set_ckmode(CkctrlCkmode::CKMODE0001));

                // Debug is disabled when core sleeps
                self.cmc.dbgctl().modify(|w| w.set_sod(true));

                // Allow the core to be gated - this WILL kill the debugging session!
                let mut cp = unsafe { cortex_m::Peripherals::steal() };
                cp.SCB.set_sleepdeep();
            }
        }

        // Allow automatic gating of the flash memory
        let (wake, doze) = match self.config.vdd_power.flash_sleep {
            config::FlashSleep::Never => (false, false),
            config::FlashSleep::FlashDoze => (false, true),
            config::FlashSleep::FlashDozeWithFlashWake => (true, true),
        };

        self.cmc.flashcr().modify(|w| {
            w.set_flashdoze(doze);
            w.set_flashwake(wake);
        });

        // At init, disable all analog peripherals. These can be re-enabled
        // if necessary for HAL drivers.
        self.spc0.active_cfg1().write(|w| w.0 = 0);
        self.spc0.lp_cfg1().write(|w| w.0 = 0);

        // Update status
        self.clocks.active_power = self.config.vdd_power.active_mode.level;
        self.clocks.lp_power = self.config.vdd_power.low_power_mode.level;

        Ok(())
    }
}

//
// Macros/macro impls
//

/// This macro is used to implement the [`Gate`] trait for a given peripheral
/// that is controlled by the MRCC peripheral.
macro_rules! impl_cc_gate {
    ($name:ident, $clk_reg:ident, $rst_reg:ident, $field:ident, $config:ty) => {
        impl Gate for crate::peripherals::$name {
            type MrccPeriphConfig = $config;

            paste! {
                #[inline]
                unsafe fn enable_clock() {
                    pac::MRCC0.$clk_reg().modify(|w| w.[<set_ $field>](true));
                }

                #[inline]
                unsafe fn disable_clock() {
                    pac::MRCC0.$clk_reg().modify(|w| w.[<set_ $field>](false));
                }

                #[inline]
                unsafe fn release_reset() {
                    pac::MRCC0.$rst_reg().modify(|w| w.[<set_ $field>](true));
                }

                #[inline]
                unsafe fn assert_reset() {
                    pac::MRCC0.$rst_reg().modify(|w| w.[<set_ $field>](false));
                }
            }

            #[inline]
            fn is_clock_enabled() -> bool {
                pac::MRCC0.$clk_reg().read().$field()
            }

            #[inline]
            fn is_reset_released() -> bool {
                pac::MRCC0.$rst_reg().read().$field()
            }
        }
    };
}

/// This module contains implementations of MRCC APIs, specifically of the [`Gate`] trait,
/// for various low level peripherals.
pub(crate) mod gate {
    use super::periph_helpers::{AdcConfig, I3cConfig, Lpi2cConfig, LpuartConfig, NoConfig, OsTimerConfig};
    use super::*;
    use crate::clocks::periph_helpers::CTimerConfig;

    // These peripherals have no additional upstream clocks or configuration required
    // other than enabling through the MRCC gate. Currently, these peripherals will
    // ALWAYS return `Ok(0)` when calling [`enable_and_reset()`] and/or
    // [`SPConfHelper::post_enable_config()`].
    impl_cc_gate!(PORT0, mrcc_glb_cc1, mrcc_glb_rst1, port0, NoConfig);
    impl_cc_gate!(PORT1, mrcc_glb_cc1, mrcc_glb_rst1, port1, NoConfig);
    impl_cc_gate!(PORT2, mrcc_glb_cc1, mrcc_glb_rst1, port2, NoConfig);
    impl_cc_gate!(PORT3, mrcc_glb_cc1, mrcc_glb_rst1, port3, NoConfig);
    impl_cc_gate!(PORT4, mrcc_glb_cc1, mrcc_glb_rst1, port4, NoConfig);

    impl_cc_gate!(CRC0, mrcc_glb_cc0, mrcc_glb_rst0, crc0, NoConfig);

    // These peripherals DO have meaningful configuration, and could fail if the system
    // clocks do not match their needs.
    impl_cc_gate!(ADC0, mrcc_glb_cc1, mrcc_glb_rst1, adc0, AdcConfig);
    impl_cc_gate!(ADC1, mrcc_glb_cc1, mrcc_glb_rst1, adc1, AdcConfig);
    impl_cc_gate!(ADC2, mrcc_glb_cc1, mrcc_glb_rst1, adc2, AdcConfig);
    impl_cc_gate!(ADC3, mrcc_glb_cc1, mrcc_glb_rst1, adc3, AdcConfig);

    impl_cc_gate!(I3C0, mrcc_glb_cc0, mrcc_glb_rst0, i3c0, I3cConfig);
    impl_cc_gate!(CTIMER0, mrcc_glb_cc0, mrcc_glb_rst0, ctimer0, CTimerConfig);
    impl_cc_gate!(CTIMER1, mrcc_glb_cc0, mrcc_glb_rst0, ctimer1, CTimerConfig);
    impl_cc_gate!(CTIMER2, mrcc_glb_cc0, mrcc_glb_rst0, ctimer2, CTimerConfig);
    impl_cc_gate!(CTIMER3, mrcc_glb_cc0, mrcc_glb_rst0, ctimer3, CTimerConfig);
    impl_cc_gate!(CTIMER4, mrcc_glb_cc0, mrcc_glb_rst0, ctimer4, CTimerConfig);
    impl_cc_gate!(OSTIMER0, mrcc_glb_cc1, mrcc_glb_rst1, ostimer0, OsTimerConfig);

    // TRNG peripheral - uses NoConfig since it has no selectable clock source
    impl_cc_gate!(TRNG0, mrcc_glb_cc1, mrcc_glb_rst1, trng0, NoConfig);

    // Peripherals that use ACC instead of CC!
    impl_cc_gate!(LPUART0, mrcc_glb_acc0, mrcc_glb_rst0, lpuart0, LpuartConfig);
    impl_cc_gate!(LPUART1, mrcc_glb_acc0, mrcc_glb_rst0, lpuart1, LpuartConfig);
    impl_cc_gate!(LPUART2, mrcc_glb_acc0, mrcc_glb_rst0, lpuart2, LpuartConfig);
    impl_cc_gate!(LPUART3, mrcc_glb_acc0, mrcc_glb_rst0, lpuart3, LpuartConfig);
    impl_cc_gate!(LPUART4, mrcc_glb_acc0, mrcc_glb_rst0, lpuart4, LpuartConfig);
    impl_cc_gate!(LPUART5, mrcc_glb_acc1, mrcc_glb_rst1, lpuart5, LpuartConfig);

    // DMA0 peripheral - uses NoConfig since it has no selectable clock source
    impl_cc_gate!(DMA0, mrcc_glb_acc0, mrcc_glb_rst0, dma0, NoConfig);

    impl_cc_gate!(GPIO0, mrcc_glb_acc2, mrcc_glb_rst2, gpio0, NoConfig);
    impl_cc_gate!(GPIO1, mrcc_glb_acc2, mrcc_glb_rst2, gpio1, NoConfig);
    impl_cc_gate!(GPIO2, mrcc_glb_acc2, mrcc_glb_rst2, gpio2, NoConfig);
    impl_cc_gate!(GPIO3, mrcc_glb_acc2, mrcc_glb_rst2, gpio3, NoConfig);
    impl_cc_gate!(GPIO4, mrcc_glb_acc2, mrcc_glb_rst2, gpio4, NoConfig);

    impl_cc_gate!(LPI2C0, mrcc_glb_acc0, mrcc_glb_rst0, lpi2c0, Lpi2cConfig);
    impl_cc_gate!(LPI2C1, mrcc_glb_acc0, mrcc_glb_rst0, lpi2c1, Lpi2cConfig);
    impl_cc_gate!(LPI2C2, mrcc_glb_acc1, mrcc_glb_rst1, lpi2c2, Lpi2cConfig);
    impl_cc_gate!(LPI2C3, mrcc_glb_acc1, mrcc_glb_rst1, lpi2c3, Lpi2cConfig);
}
