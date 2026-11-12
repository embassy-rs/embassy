//! Core clock state and peripheral clock gating.
//!
//! [`init()`] does not change the main clock source. It checks that `main_clk`
//! is still driven from its reset default, `fro_hf` (SCG's FIRC; RM "Chapter 33
//! Clocking"), and records the resulting `main_clk` and `system_clk`
//! frequencies for [`with_clocks`]. It returns
//! [`ClockError::UnsupportedMainClockSource`] if something before it has moved
//! `main_clk` to another source (SOSC, SIRC, a PLL, ...).
//!
//! Peripheral clock gating (SYSCON `AHBCLKCTRLn`/`PRESETCTRLn`) is done through
//! the [`Gate`] trait and the free functions alongside it, and does not depend
//! on the above.

use core::cell::RefCell;

use config::ClocksConfig;

use crate::pac;

pub mod config;
mod gate;
mod types;

pub use gate::{
    Gate, assert_reset, disable, enable, enable_and_reset, is_clock_enabled, is_reset_released, pulse_reset,
    release_reset,
};
pub use types::{ClockError, Clocks};

/// `SCG0.CSR.SCS` value for FIRC (RM, SCG chapter, "Clock Status Register").
const SCS_FIRC: u8 = 3;

/// `SCG0.FIRCCFG.RANGE` value for 48 MHz; the other encoding is 144 MHz.
const FIRC_RANGE_48M: u8 = 0;

/// The state of system core clocks.
///
/// Initialized by [`init()`], and then unchanged for the remainder of the program.
static CLOCKS: critical_section::Mutex<RefCell<Option<Clocks>>> = critical_section::Mutex::new(RefCell::new(None));

/// Initialize the core system clocks with the given [`ClocksConfig`].
///
/// This function should be called exactly once at start-up, usually via a
/// call to [`embassy_mcxn::init()`](crate::init()). Subsequent calls return
/// [`ClockError::AlreadyInitialized`].
pub fn init(_settings: ClocksConfig) -> Result<(), ClockError> {
    critical_section::with(|cs| {
        if CLOCKS.borrow_ref(cs).is_some() {
            Err(ClockError::AlreadyInitialized)
        } else {
            Ok(())
        }
    })?;

    // `CSR.SCS` reports the source actually in use, not the one requested.
    if pac::SCG0.csr().read().scs().to_bits() != SCS_FIRC {
        return Err(ClockError::UnsupportedMainClockSource);
    }

    let main_clk_hz = if pac::SCG0.firccfg().read().range().to_bits() == FIRC_RANGE_48M {
        48_000_000
    } else {
        144_000_000
    };

    // AHBCLKDIV.DIV divides by DIV + 1.
    let ahb_div = pac::SYSCON0.ahbclkdiv().read().div() as u32 + 1;
    let system_clk_hz = main_clk_hz / ahb_div;

    let clocks = Clocks {
        main_clk_hz,
        system_clk_hz,
    };

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
/// Returns `None` until [`init()`] has succeeded.
pub fn with_clocks<R: 'static, F: FnOnce(&Clocks) -> R>(f: F) -> Option<R> {
    critical_section::with(|cs| {
        let c = CLOCKS.borrow_ref(cs);
        let c = c.as_ref()?;
        Some(f(c))
    })
}
