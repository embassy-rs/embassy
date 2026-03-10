//! Clock gate trait and free functions for enabling/disabling peripheral clocks.

use super::CLOCKS;
use super::periph_helpers::{PreEnableParts, SPConfHelper};
use super::types::ClockError;

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
// Macros/macro impls
//

/// This macro is used to implement the [`Gate`] trait for a given peripheral
/// that is controlled by the MRCC peripheral.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_cc_gate {
    ($name:ident, $clk_reg:ident, $field:ident, $config:ty) => {
        impl Gate for $crate::peripherals::$name {
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
                unsafe fn release_reset() {}

                #[inline]
                unsafe fn assert_reset() {}
            }

            #[inline]
            fn is_clock_enabled() -> bool {
                pac::MRCC0.$clk_reg().read().$field()
            }

            #[inline]
            fn is_reset_released() -> bool {
                false
            }
        }
    };

    ($name:ident, $clk_reg:ident, $rst_reg:ident, $field:ident, $config:ty) => {
        impl Gate for $crate::peripherals::$name {
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
                    // Wait for reset to set
                    while !pac::MRCC0.$rst_reg().read().[<$field>]() {}
                }

                #[inline]
                unsafe fn assert_reset() {
                    pac::MRCC0.$rst_reg().modify(|w| w.[<set_ $field>](false));
                    // Wait for reset to clear
                    while pac::MRCC0.$rst_reg().read().[<$field>]() {}
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
