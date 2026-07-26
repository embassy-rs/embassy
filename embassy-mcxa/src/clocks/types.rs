//! Clock system types

use core::sync::atomic::Ordering;

use super::LIVE_HP_TOKENS;
use crate::clocks::config::{CoreSleep, VddLevel};

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
/// These values are configured by providing
/// [ClocksConfig](crate::clocks::config::ClocksConfig) to the
/// [`init()`](super::init) function at boot time.
#[derive(Default, Debug, Clone)]
#[non_exhaustive]
pub struct Clocks {
    /// Active power config
    pub active_power: VddLevel,

    /// Low-power power config
    pub lp_power: VddLevel,

    /// Is the bandgap enabled in active mode?
    pub bandgap_active: bool,

    /// Is the bandgap enabled in deep sleep mode?
    pub bandgap_lowpower: bool,

    /// Lowest sleep level
    pub core_sleep: CoreSleep,

    /// The `clk_in` is a clock provided by an external oscillator
    /// AKA SOSC
    #[cfg(not(feature = "sosc-as-gpio"))]
    pub clk_in: Option<Clock>,

    // FRO180M/FRO192M stuff
    //
    /// `fro_hf_root` is the direct output of the `FRO180M`/`FRO192M` internal oscillator
    ///
    /// It is used to feed downstream clocks, such as `fro_hf`, `clk_45m`/`clk_48m`,
    /// and `fro_hf_div`.
    pub fro_hf_root: Option<Clock>,

    /// `fro_hf` is the same frequency as `fro_hf_root`, but behind a gate.
    pub fro_hf: Option<Clock>,

    /// `clk_45m` (2xx) or `clk_48` (5xx) is a 45MHz/48MHz clock, sourced from `fro_hf`.
    pub clk_hf_fundamental: Option<Clock>,

    /// `fro_hf_div` is a configurable frequency clock, sourced from `fro_hf`.
    pub fro_hf_div: Option<Clock>,

    //
    // End FRO180M/FRO192M

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
    /// `clk_16k_vsys` is one of two/three outputs of the `FRO16K` internal oscillator.
    ///
    /// Also referred to as `clk_16k[0]` in the datasheet, it feeds peripherals in
    /// the system domain, such as the CMP and RTC.
    pub clk_16k_vsys: Option<Clock>,

    /// `clk_16k_vdd_core` is one of two/three outputs of the `FRO16K` internal oscillator.
    ///
    /// Also referred to as `clk_16k[1]` in the datasheet, it feeds peripherals in
    /// the VDD Core domain, such as the OSTimer or LPUarts.
    pub clk_16k_vdd_core: Option<Clock>,

    /// `clk_16k_vbat` is one of three outputs of the `FRO16K` internal oscillator.
    ///
    /// Also referred to as `clk_16k[2]` in the datasheet.
    #[cfg(feature = "mcxa5xx")]
    pub clk_16k_vbat: Option<Clock>,

    /// `clk_32k_vsys` is one of two/three outputs of the `FRO16K` internal oscillator.
    ///
    /// Also referred to as `clk_32k[0]` in the datasheet, it feeds peripherals in
    /// the system domain, such as the CMP and RTC.
    #[cfg(all(feature = "mcxa5xx", not(feature = "rosc-32k-as-gpio")))]
    pub clk_32k_vsys: Option<Clock>,

    /// `clk_32k_vdd_core` is one of two/three outputs of the `FRO16K` internal oscillator.
    ///
    /// Also referred to as `clk_32k[1]` in the datasheet, it feeds peripherals in
    /// the VDD Core domain, such as the OSTimer or LPUarts.
    #[cfg(all(feature = "mcxa5xx", not(feature = "rosc-32k-as-gpio")))]
    pub clk_32k_vdd_core: Option<Clock>,

    /// `clk_32k_vbat` is one of three outputs of the `FRO16K` internal oscillator.
    ///
    /// Also referred to as `clk_32k[2]` in the datasheet.
    #[cfg(all(feature = "mcxa5xx", not(feature = "rosc-32k-as-gpio")))]
    pub clk_32k_vbat: Option<Clock>,

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
    /// The system clocks were never initialized by calling [`init()`](super::init)
    NeverInitialized,
    /// The [`init()`](super::init) function was called more than once
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PoweredClock {
    /// The given clock will NOT continue running in Deep Sleep mode
    NormalEnabledDeepSleepDisabled,
    /// The given clock WILL continue running in Deep Sleep mode
    AlwaysEnabled,
}

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

    /// Ensure the `clk_16k_vbat` clock is active and valid at the given power state.
    #[cfg(feature = "mcxa5xx")]
    pub fn ensure_clk_16k_vbat_active(&self, _at_level: &PoweredClock) -> Result<u32, ClockError> {
        // NOTE: clk_16k is always active in low power mode
        Ok(self
            .clk_16k_vbat
            .as_ref()
            .ok_or(ClockError::BadConfig {
                clock: "clk_16k_vbat",
                reason: "required but not active",
            })?
            .frequency)
    }

    /// Ensure the `clk_32k_vsys` clock is active and valid at the given power state.
    #[cfg(all(feature = "mcxa5xx", not(feature = "rosc-32k-as-gpio")))]
    #[inline]
    pub fn ensure_clk_32k_vsys_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        self.ensure_clock_active(&self.clk_32k_vsys, "clk_32k_vsys", at_level)
    }

    /// Ensure the `clk_32k_vdd_core` clock is active and valid at the given power state.
    #[cfg(all(feature = "mcxa5xx", not(feature = "rosc-32k-as-gpio")))]
    #[inline]
    pub fn ensure_clk_32k_vdd_core_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        self.ensure_clock_active(&self.clk_32k_vdd_core, "clk_32k_vdd_core", at_level)
    }

    /// Ensure the `clk_32k_vbat` clock is active and valid at the given power state.
    #[cfg(all(feature = "mcxa5xx", not(feature = "rosc-32k-as-gpio")))]
    #[inline]
    pub fn ensure_clk_32k_vbat_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        self.ensure_clock_active(&self.clk_32k_vbat, "clk_32k_vbat", at_level)
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
