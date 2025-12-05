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

use config::{ClocksConfig, FircConfig, FircFreqSel, Fro16KConfig, SircConfig};
use mcxa_pac::scg0::firccsr::{FircFclkPeriphEn, FircSclkPeriphEn, Fircsten};
use mcxa_pac::scg0::sirccsr::{SircClkPeriphEn, Sircsten};
use periph_helpers::SPConfHelper;

use crate::pac;
pub mod config;
pub mod periph_helpers;

//
// Statics/Consts
//

/// The state of system core clocks.
///
/// Initialized by [`init()`], and then unchanged for the remainder of the program.
static CLOCKS: critical_section::Mutex<RefCell<Option<Clocks>>> = critical_section::Mutex::new(RefCell::new(None));

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

        _mrcc0: unsafe { pac::Mrcc0::steal() },
        scg0: unsafe { pac::Scg0::steal() },
        syscon: unsafe { pac::Syscon::steal() },
        vbat0: unsafe { pac::Vbat0::steal() },
    };

    operator.configure_firc_clocks()?;
    operator.configure_sirc_clocks()?;
    operator.configure_fro16k_clocks()?;

    // For now, just use FIRC as the main/cpu clock, which should already be
    // the case on reset
    assert!(operator.scg0.rccr().read().scs().is_firc());
    let input = operator.clocks.fro_hf_root.clone().unwrap();
    operator.clocks.main_clk = Some(input.clone());
    // We can also assume cpu/system clk == fro_hf because div is /1.
    assert_eq!(operator.syscon.ahbclkdiv().read().div().bits(), 0);
    operator.clocks.cpu_system_clk = Some(input);

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

/// The `Clocks` structure contains the initialized state of the core system clocks
///
/// These values are configured by providing [`config::ClocksConfig`] to the [`init()`] function
/// at boot time.
#[derive(Default, Debug, Clone)]
#[non_exhaustive]
pub struct Clocks {
    /// The `clk_in` is a clock provided by an external oscillator
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

    /// `main_clk` is the main clock used by the CPU, AHB, APB, IPS bus, and some
    /// peripherals.
    pub main_clk: Option<Clock>,

    /// `CPU_CLK` or `SYSTEM_CLK` is the output of `main_clk`, run through the `AHBCLKDIV`
    pub cpu_system_clk: Option<Clock>,

    /// `pll1_clk` is the output of the main system PLL, `pll1`.
    pub pll1_clk: Option<Clock>,
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

    // We hold on to stolen peripherals
    _mrcc0: pac::Mrcc0,
    scg0: pac::Scg0,
    syscon: pac::Syscon,
    vbat0: pac::Vbat0,
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
pub unsafe fn enable_and_reset<G: Gate>(cfg: &G::MrccPeriphConfig) -> Result<u32, ClockError> {
    let freq = enable::<G>(cfg).inspect_err(|_| disable::<G>())?;
    pulse_reset::<G>();
    Ok(freq)
}

/// Enable the clock gate for the given peripheral.
///
/// Prefer [`enable_and_reset`] unless you are specifically avoiding a pulse of the reset, or need
/// to control the duration of the pulse more directly.
///
/// # SAFETY
///
/// This peripheral must not yet be in use prior to calling `enable`.
#[inline]
pub unsafe fn enable<G: Gate>(cfg: &G::MrccPeriphConfig) -> Result<u32, ClockError> {
    G::enable_clock();
    while !G::is_clock_enabled() {}
    core::arch::asm!("dsb sy; isb sy", options(nomem, nostack, preserves_flags));

    let freq = critical_section::with(|cs| {
        let clocks = CLOCKS.borrow_ref(cs);
        let clocks = clocks.as_ref().ok_or(ClockError::NeverInitialized)?;
        cfg.post_enable_config(clocks)
    });

    freq.inspect_err(|_e| {
        G::disable_clock();
    })
}

/// Disable the clock gate for the given peripheral.
///
/// # SAFETY
///
/// This peripheral must no longer be in use prior to calling `enable`.
#[allow(dead_code)]
#[inline]
pub unsafe fn disable<G: Gate>() {
    G::disable_clock();
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
    G::release_reset();
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
    G::assert_reset();
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
    G::assert_reset();
    cortex_m::asm::nop();
    cortex_m::asm::nop();
    G::release_reset();
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
    /// Ensure the `fro_lf_div` clock is active and valid at the given power state.
    pub fn ensure_fro_lf_div_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        let Some(clk) = self.fro_lf_div.as_ref() else {
            return Err(ClockError::BadConfig {
                clock: "fro_lf_div",
                reason: "required but not active",
            });
        };
        if !clk.power.meets_requirement_of(at_level) {
            return Err(ClockError::BadConfig {
                clock: "fro_lf_div",
                reason: "not low power active",
            });
        }
        Ok(clk.frequency)
    }

    /// Ensure the `fro_hf` clock is active and valid at the given power state.
    pub fn ensure_fro_hf_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        let Some(clk) = self.fro_hf.as_ref() else {
            return Err(ClockError::BadConfig {
                clock: "fro_hf",
                reason: "required but not active",
            });
        };
        if !clk.power.meets_requirement_of(at_level) {
            return Err(ClockError::BadConfig {
                clock: "fro_hf",
                reason: "not low power active",
            });
        }
        Ok(clk.frequency)
    }

    /// Ensure the `fro_hf_div` clock is active and valid at the given power state.
    pub fn ensure_fro_hf_div_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        let Some(clk) = self.fro_hf_div.as_ref() else {
            return Err(ClockError::BadConfig {
                clock: "fro_hf_div",
                reason: "required but not active",
            });
        };
        if !clk.power.meets_requirement_of(at_level) {
            return Err(ClockError::BadConfig {
                clock: "fro_hf_div",
                reason: "not low power active",
            });
        }
        Ok(clk.frequency)
    }

    /// Ensure the `clk_in` clock is active and valid at the given power state.
    pub fn ensure_clk_in_active(&self, _at_level: &PoweredClock) -> Result<u32, ClockError> {
        Err(ClockError::NotImplemented { clock: "clk_in" })
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
    pub fn ensure_clk_1m_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        let Some(clk) = self.clk_1m.as_ref() else {
            return Err(ClockError::BadConfig {
                clock: "clk_1m",
                reason: "required but not active",
            });
        };
        if !clk.power.meets_requirement_of(at_level) {
            return Err(ClockError::BadConfig {
                clock: "clk_1m",
                reason: "not low power active",
            });
        }
        Ok(clk.frequency)
    }

    /// Ensure the `pll1_clk` clock is active and valid at the given power state.
    pub fn ensure_pll1_clk_active(&self, _at_level: &PoweredClock) -> Result<u32, ClockError> {
        Err(ClockError::NotImplemented { clock: "pll1_clk" })
    }

    /// Ensure the `pll1_clk_div` clock is active and valid at the given power state.
    pub fn ensure_pll1_clk_div_active(&self, _at_level: &PoweredClock) -> Result<u32, ClockError> {
        Err(ClockError::NotImplemented { clock: "pll1_clk_div" })
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
    /// Configure the FIRC/FRO180M clock family
    ///
    /// NOTE: Currently we require this to be a fairly hardcoded value, as this clock is used
    /// as the main clock used for the CPU, AHB, APB, etc.
    fn configure_firc_clocks(&mut self) -> Result<(), ClockError> {
        const HARDCODED_ERR: Result<(), ClockError> = Err(ClockError::BadConfig {
            clock: "firc",
            reason: "For now, FIRC must be enabled and in default state!",
        });

        // Did the user give us a FIRC config?
        let Some(firc) = self.config.firc.as_ref() else {
            return HARDCODED_ERR;
        };
        // Is the FIRC set to 45MHz (should be reset default)
        if !matches!(firc.frequency, FircFreqSel::Mhz45) {
            return HARDCODED_ERR;
        }
        let base_freq = 45_000_000;

        // Now, check if the FIRC as expected for our hardcoded value
        let mut firc_ok = true;

        // Is the hardware currently set to the default 45MHz?
        //
        // NOTE: the SVD currently has the wrong(?) values for these:
        // 45 -> 48
        // 60 -> 64
        // 90 -> 96
        // 180 -> 192
        // Probably correct-ish, but for a different trim value?
        firc_ok &= self.scg0.firccfg().read().freq_sel().is_firc_48mhz_192s();

        // Check some values in the CSR
        let csr = self.scg0.firccsr().read();
        // Is it enabled?
        firc_ok &= csr.fircen().is_enabled();
        // Is it accurate?
        firc_ok &= csr.fircacc().is_enabled_and_valid();
        // Is there no error?
        firc_ok &= csr.fircerr().is_error_not_detected();
        // Is the FIRC the system clock?
        firc_ok &= csr.fircsel().is_firc();
        // Is it valid?
        firc_ok &= csr.fircvld().is_enabled_and_valid();

        // Are we happy with the current (hardcoded) state?
        if !firc_ok {
            return HARDCODED_ERR;
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
            PoweredClock::NormalEnabledDeepSleepDisabled => Fircsten::DisabledInStopModes,
            PoweredClock::AlwaysEnabled => Fircsten::EnabledInStopModes,
        };

        // Do we enable the `fro_hf` output?
        let fro_hf_set = if *fro_hf_enabled {
            self.clocks.fro_hf = Some(Clock {
                frequency: base_freq,
                power: *power,
            });
            FircFclkPeriphEn::Enabled
        } else {
            FircFclkPeriphEn::Disabled
        };

        // Do we enable the `clk_45m` output?
        let clk_45m_set = if *clk_45m_enabled {
            self.clocks.clk_45m = Some(Clock {
                frequency: 45_000_000,
                power: *power,
            });
            FircSclkPeriphEn::Enabled
        } else {
            FircSclkPeriphEn::Disabled
        };

        self.scg0.firccsr().modify(|_r, w| {
            w.fircsten().variant(pow_set);
            w.firc_fclk_periph_en().variant(fro_hf_set);
            w.firc_sclk_periph_en().variant(clk_45m_set);
            w
        });

        // Do we enable the `fro_hf_div` output?
        if let Some(d) = fro_hf_div.as_ref() {
            // We need `fro_hf` to be enabled
            if !*fro_hf_enabled {
                return Err(ClockError::BadConfig {
                    clock: "fro_hf_div",
                    reason: "fro_hf not enabled",
                });
            }

            // Halt and reset the div; then set our desired div.
            self.syscon.frohfdiv().write(|w| {
                w.halt().halt();
                w.reset().asserted();
                unsafe { w.div().bits(d.into_bits()) };
                w
            });
            // Then unhalt it, and reset it
            self.syscon.frohfdiv().write(|w| {
                w.halt().run();
                w.reset().released();
                w
            });

            // Wait for clock to stabilize
            while self.syscon.frohfdiv().read().unstab().is_ongoing() {}

            // Store off the clock info
            self.clocks.fro_hf_div = Some(Clock {
                frequency: base_freq / d.into_divisor(),
                power: *power,
            });
        }

        Ok(())
    }

    /// Configure the SIRC/FRO12M clock family
    fn configure_sirc_clocks(&mut self) -> Result<(), ClockError> {
        let SircConfig {
            power,
            fro_12m_enabled,
            fro_lf_div,
        } = &self.config.sirc;
        let base_freq = 12_000_000;

        // Allow writes
        self.scg0.sirccsr().modify(|_r, w| w.lk().write_enabled());
        self.clocks.fro_12m_root = Some(Clock {
            frequency: base_freq,
            power: *power,
        });

        let deep = match power {
            PoweredClock::NormalEnabledDeepSleepDisabled => Sircsten::Disabled,
            PoweredClock::AlwaysEnabled => Sircsten::Enabled,
        };
        let pclk = if *fro_12m_enabled {
            self.clocks.fro_12m = Some(Clock {
                frequency: base_freq,
                power: *power,
            });
            self.clocks.clk_1m = Some(Clock {
                frequency: base_freq / 12,
                power: *power,
            });
            SircClkPeriphEn::Enabled
        } else {
            SircClkPeriphEn::Disabled
        };

        // Set sleep/peripheral usage
        self.scg0.sirccsr().modify(|_r, w| {
            w.sircsten().variant(deep);
            w.sirc_clk_periph_en().variant(pclk);
            w
        });

        while self.scg0.sirccsr().read().sircvld().is_disabled_or_not_valid() {}
        if self.scg0.sirccsr().read().sircerr().is_error_detected() {
            return Err(ClockError::BadConfig {
                clock: "sirc",
                reason: "error set",
            });
        }

        // reset lock
        self.scg0.sirccsr().modify(|_r, w| w.lk().write_disabled());

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
                w.halt().halt();
                w.reset().asserted();
                unsafe { w.div().bits(d.into_bits()) };
                w
            });
            // Then unhalt it, and reset it
            self.syscon.frolfdiv().modify(|_r, w| {
                w.halt().run();
                w.reset().released();
                w
            });

            // Wait for clock to stabilize
            while self.syscon.frolfdiv().read().unstab().is_ongoing() {}

            // Store off the clock info
            self.clocks.fro_lf_div = Some(Clock {
                frequency: base_freq / d.into_divisor(),
                power: *power,
            });
        }

        Ok(())
    }

    /// Configure the FRO16K/clk_16k clock family
    fn configure_fro16k_clocks(&mut self) -> Result<(), ClockError> {
        let Some(fro16k) = self.config.fro16k.as_ref() else {
            return Ok(());
        };
        // Enable FRO16K oscillator
        self.vbat0.froctla().modify(|_, w| w.fro_en().set_bit());

        // Lock the control register
        self.vbat0.frolcka().modify(|_, w| w.lock().set_bit());

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
        self.vbat0.froclke().modify(|_r, w| unsafe { w.clke().bits(bits) });

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

            #[inline]
            unsafe fn enable_clock() {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$clk_reg().modify(|_, w| w.$field().enabled());
            }

            #[inline]
            unsafe fn disable_clock() {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$clk_reg().modify(|_r, w| w.$field().disabled());
            }

            #[inline]
            fn is_clock_enabled() -> bool {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$clk_reg().read().$field().is_enabled()
            }

            #[inline]
            unsafe fn release_reset() {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$rst_reg().modify(|_, w| w.$field().enabled());
            }

            #[inline]
            unsafe fn assert_reset() {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$rst_reg().modify(|_, w| w.$field().disabled());
            }

            #[inline]
            fn is_reset_released() -> bool {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$rst_reg().read().$field().is_enabled()
            }
        }
    };
}

/// This module contains implementations of MRCC APIs, specifically of the [`Gate`] trait,
/// for various low level peripherals.
pub(crate) mod gate {
    #[cfg(not(feature = "time"))]
    use super::periph_helpers::OsTimerConfig;
    use super::periph_helpers::{AdcConfig, Lpi2cConfig, LpuartConfig, NoConfig};
    use super::*;

    // These peripherals have no additional upstream clocks or configuration required
    // other than enabling through the MRCC gate. Currently, these peripherals will
    // ALWAYS return `Ok(0)` when calling [`enable_and_reset()`] and/or
    // [`SPConfHelper::post_enable_config()`].
    impl_cc_gate!(PORT0, mrcc_glb_cc1, mrcc_glb_rst1, port0, NoConfig);
    impl_cc_gate!(PORT1, mrcc_glb_cc1, mrcc_glb_rst1, port1, NoConfig);
    impl_cc_gate!(PORT2, mrcc_glb_cc1, mrcc_glb_rst1, port2, NoConfig);
    impl_cc_gate!(PORT3, mrcc_glb_cc1, mrcc_glb_rst1, port3, NoConfig);
    impl_cc_gate!(PORT4, mrcc_glb_cc1, mrcc_glb_rst1, port4, NoConfig);

    impl_cc_gate!(GPIO0, mrcc_glb_cc2, mrcc_glb_rst2, gpio0, NoConfig);
    impl_cc_gate!(GPIO1, mrcc_glb_cc2, mrcc_glb_rst2, gpio1, NoConfig);
    impl_cc_gate!(GPIO2, mrcc_glb_cc2, mrcc_glb_rst2, gpio2, NoConfig);
    impl_cc_gate!(GPIO3, mrcc_glb_cc2, mrcc_glb_rst2, gpio3, NoConfig);
    impl_cc_gate!(GPIO4, mrcc_glb_cc2, mrcc_glb_rst2, gpio4, NoConfig);

    impl_cc_gate!(CRC0, mrcc_glb_cc0, mrcc_glb_rst0, crc0, NoConfig);

    // These peripherals DO have meaningful configuration, and could fail if the system
    // clocks do not match their needs.
    #[cfg(not(feature = "time"))]
    impl_cc_gate!(OSTIMER0, mrcc_glb_cc1, mrcc_glb_rst1, ostimer0, OsTimerConfig);

    impl_cc_gate!(LPI2C0, mrcc_glb_cc0, mrcc_glb_rst0, lpi2c0, Lpi2cConfig);
    impl_cc_gate!(LPI2C1, mrcc_glb_cc0, mrcc_glb_rst0, lpi2c1, Lpi2cConfig);
    impl_cc_gate!(LPI2C2, mrcc_glb_cc1, mrcc_glb_rst1, lpi2c2, Lpi2cConfig);
    impl_cc_gate!(LPI2C3, mrcc_glb_cc1, mrcc_glb_rst1, lpi2c3, Lpi2cConfig);

    impl_cc_gate!(LPUART0, mrcc_glb_cc0, mrcc_glb_rst0, lpuart0, LpuartConfig);
    impl_cc_gate!(LPUART1, mrcc_glb_cc0, mrcc_glb_rst0, lpuart1, LpuartConfig);
    impl_cc_gate!(LPUART2, mrcc_glb_cc0, mrcc_glb_rst0, lpuart2, LpuartConfig);
    impl_cc_gate!(LPUART3, mrcc_glb_cc0, mrcc_glb_rst0, lpuart3, LpuartConfig);
    impl_cc_gate!(LPUART4, mrcc_glb_cc0, mrcc_glb_rst0, lpuart4, LpuartConfig);
    impl_cc_gate!(LPUART5, mrcc_glb_cc1, mrcc_glb_rst1, lpuart5, LpuartConfig);
    impl_cc_gate!(ADC0, mrcc_glb_cc1, mrcc_glb_rst1, adc0, AdcConfig);
    impl_cc_gate!(ADC1, mrcc_glb_cc1, mrcc_glb_rst1, adc1, AdcConfig);
    impl_cc_gate!(ADC2, mrcc_glb_cc1, mrcc_glb_rst1, adc2, AdcConfig);
    impl_cc_gate!(ADC3, mrcc_glb_cc1, mrcc_glb_rst1, adc3, AdcConfig);

    // DMA0 peripheral - uses NoConfig since it has no selectable clock source
    impl_cc_gate!(DMA0, mrcc_glb_cc0, mrcc_glb_rst0, dma0, NoConfig);
}
