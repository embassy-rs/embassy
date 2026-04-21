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

use config::ClocksConfig;
use critical_section::CriticalSection;

use crate::pac;

pub mod config;
mod gate;
mod operator;
pub mod periph_helpers;
mod sleep;
mod types;

// Re-exports
pub use config::VddLevel;
pub use gate::{Gate, assert_reset, disable, enable, enable_and_reset, is_reset_released, release_reset};
pub use sleep::deep_sleep_if_possible;
pub use types::{Clock, ClockError, Clocks, PoweredClock, WakeGuard};

//
// Statics/Consts
//

/// The state of system core clocks.
///
/// Initialized by [`init()`], and then unchanged for the remainder of the program.
pub(super) static CLOCKS: critical_section::Mutex<RefCell<Option<Clocks>>> =
    critical_section::Mutex::new(RefCell::new(None));
pub(super) static LIVE_HP_TOKENS: AtomicUsize = AtomicUsize::new(0);

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
    let mut operator = operator::ClockOperator {
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

    operator.unlock_mrcc();

    // Before applying any requested clocks, apply the requested VDD_CORE
    // voltage level
    operator.configure_voltages()?;

    // Enable SIRC clocks FIRST, in case we need to use SIRC as main_clk for
    // a short while.
    operator.configure_sirc_clocks_early()?;
    operator.configure_firc_clocks()?;
    operator.configure_fro16k_clocks()?;

    // NOTE: OSC32K must be configured AFTER FRO16K.
    #[cfg(all(feature = "mcxa5xx", feature = "unstable-osc32k", not(feature = "rosc-32k-as-gpio")))]
    operator.configure_osc32k_clocks()?;

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

/// Are there active `WakeGuard`s?
///
/// Requires a critical section to ensure this doesn't race between getting the guard
/// count and performing some action like setting up deep sleep
#[inline(always)]
pub fn active_wake_guards(_cs: &CriticalSection) -> bool {
    // Relaxed is okay: we are in a critical section
    LIVE_HP_TOKENS.load(Ordering::Relaxed) != 0
}
