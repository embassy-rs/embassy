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
    let _ = ACTIVE_USERS.fetch_update(Ordering::Release, Ordering::Acquire, |x| Some(x + 1));

    // Make sure the it's enabled
    pac::CRYPTOCELL.enable().write(|w| w.set_enable(true));

    CryptoCellActivationHandle::new()
}

fn release() {
    // If we released the last one, turn off CRYPTOCELL
    if let Ok(deps) = ACTIVE_USERS.fetch_update(Ordering::Release, Ordering::Acquire, |x| Some(x - 1))
        && deps == 1
    {
        pac::CRYPTOCELL.enable().write(|w| w.set_enable(false));
    }
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
