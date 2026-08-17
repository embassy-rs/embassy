//! CryptoCell drivers
//!
//! If you want to interact with the CryptoCell, you can activate it using [`activate()`].
//! This function will return a handle that will keep the CryptoCell active until it is dropped.

#![macro_use]

use core::sync::atomic::{AtomicU16, Ordering};

use crate::pac;

pub mod rng;

static ACTIVE_USERS: AtomicU16 = AtomicU16::new(0);

/// Use this function to activate the CryptoCell subsystem.
///
/// CryptoCell is active until the handle is dropped.
#[must_use]
pub fn activate() -> CryptoCellActivationHandle {
    ACTIVE_USERS.fetch_add(1, Ordering::Relaxed);

    // Make sure the CryptoCell is enabled.
    pac::CRYPTOCELL.enable().write(|w| w.set_enable(true));

    CryptoCellActivationHandle::new()
}

fn release() {
    // We need to make sure an activation doesn't happen between us checking the value and disabling
    // the CryptoCell.
    critical_section::with(|_cs| {
        // If we released the last one, turn off CRYPTOCELL.
        // We can use the Relaxed ordering as this happens in a critical section.
        if ACTIVE_USERS.fetch_sub(1, Ordering::Relaxed) == 1 {
            pac::CRYPTOCELL.enable().write(|w| w.set_enable(false));
        }
    })
}

/// Cryptocell is active until this is dropped.
pub struct CryptoCellActivationHandle {
    _private: (),
}

impl CryptoCellActivationHandle {
    fn new() -> Self {
        Self { _private: () }
    }
}

impl Drop for CryptoCellActivationHandle {
    fn drop(&mut self) {
        release()
    }
}
