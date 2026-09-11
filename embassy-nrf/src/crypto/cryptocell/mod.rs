//! CryptoCell power management.

use core::sync::atomic::{AtomicU16, Ordering};

use crate::pac;

pub(crate) mod dma;

static ACTIVE_USERS: AtomicU16 = AtomicU16::new(0);

/// Powers the CryptoCell up if it is not already, and keeps it powered until the returned
/// handle is dropped.
pub(crate) fn activate() -> ActivationHandle {
    // The count and the enable bit have to change together, so that a release racing with
    // this activation cannot switch the block off underneath it. A few instructions.
    critical_section::with(|_cs| {
        if ACTIVE_USERS.fetch_add(1, Ordering::Relaxed) == 0 {
            pac::CRYPTOCELL.enable().write(|w| w.set_enable(true));
        }
    });
    ActivationHandle { _private: () }
}

fn release() {
    critical_section::with(|_cs| {
        if ACTIVE_USERS.fetch_sub(1, Ordering::Relaxed) == 1 {
            pac::CRYPTOCELL.enable().write(|w| w.set_enable(false));
        }
    })
}

/// Keeps the CryptoCell powered while it exists.
pub(crate) struct ActivationHandle {
    _private: (),
}

impl Drop for ActivationHandle {
    fn drop(&mut self) {
        release()
    }
}
