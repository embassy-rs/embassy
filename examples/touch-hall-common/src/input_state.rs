//! Touch button input state (held / short / long press pulses).

use core::sync::atomic::{AtomicU8, Ordering};

const MAX_BUTTONS: usize = 64;

static HELD: [AtomicU8; MAX_BUTTONS] = [const { AtomicU8::new(0) }; MAX_BUTTONS];
static SHORT: [AtomicU8; MAX_BUTTONS] = [const { AtomicU8::new(0) }; MAX_BUTTONS];
static LONG: [AtomicU8; MAX_BUTTONS] = [const { AtomicU8::new(0) }; MAX_BUTTONS];
static RELEASE: [AtomicU8; MAX_BUTTONS] = [const { AtomicU8::new(0) }; MAX_BUTTONS];

fn slot(atom: Option<&AtomicU8>) -> Option<&AtomicU8> {
    atom
}

pub fn set_held(index: usize, held: bool) {
    if let Some(slot) = slot(HELD.get(index)) {
        slot.store(held as u8, Ordering::Relaxed);
    }
}

pub fn held(index: usize) -> bool {
    HELD.get(index)
        .map(|slot| slot.load(Ordering::Relaxed) != 0)
        .unwrap_or(false)
}

pub fn pulse_short(index: usize) {
    if let Some(slot) = slot(SHORT.get(index)) {
        slot.store(1, Ordering::Relaxed);
    }
}

pub fn pulse_long(index: usize) {
    if let Some(slot) = slot(LONG.get(index)) {
        slot.store(1, Ordering::Relaxed);
    }
}

pub fn pulse_release(index: usize) {
    if let Some(slot) = slot(RELEASE.get(index)) {
        slot.store(1, Ordering::Relaxed);
    }
}

/// Read short-press pulse and clear it.
pub fn take_short(index: usize) -> bool {
    SHORT
        .get(index)
        .map(|slot| slot.swap(0, Ordering::Relaxed) != 0)
        .unwrap_or(false)
}

/// Read long-press pulse and clear it.
pub fn take_long(index: usize) -> bool {
    LONG
        .get(index)
        .map(|slot| slot.swap(0, Ordering::Relaxed) != 0)
        .unwrap_or(false)
}

/// Read release pulse and clear it.
pub fn take_release(index: usize) -> bool {
    RELEASE
        .get(index)
        .map(|slot| slot.swap(0, Ordering::Relaxed) != 0)
        .unwrap_or(false)
}

/// True while any touch button is held down.
pub fn any_held() -> bool {
    HELD.iter().any(|slot| slot.load(Ordering::Relaxed) != 0)
}
