//! Touch-hold latch — blocks CAN release (`00`) until a confirmed finger lift.

use core::sync::atomic::{AtomicU8, Ordering};

const NONE: u8 = 255;

static LATCH: AtomicU8 = AtomicU8::new(NONE);

/// Finger is down on this button; no release frames until [`latch_clear`].
pub fn latch_press(index: u8) {
    LATCH.store(index, Ordering::Relaxed);
}

/// Confirmed lift — release frames may be sent again.
pub fn latch_clear() {
    LATCH.store(NONE, Ordering::Relaxed);
}

pub fn is_latched() -> bool {
    LATCH.load(Ordering::Relaxed) != NONE
}

pub fn latched() -> Option<u8> {
    match LATCH.load(Ordering::Relaxed) {
        NONE => None,
        index => Some(index),
    }
}
