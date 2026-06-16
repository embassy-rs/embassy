//! Shared PLC UI button highlight state (Rhai `ui[]` only — touch via [`crate::touch_feedback`]).

use core::sync::atomic::{AtomicU8, Ordering};

use crate::BUTTON_COUNT;

const MAX_BUTTONS: usize = 64;

static BUTTON_STATUS: [AtomicU8; MAX_BUTTONS] = [const { AtomicU8::new(0) }; MAX_BUTTONS];

/// Highlight from Rhai `ui[]` (CAN/minp state). Touch uses [`crate::touch_feedback`].
pub fn plc_active(index: usize) -> bool {
    BUTTON_STATUS
        .get(index)
        .map(|slot| slot.load(Ordering::Relaxed) != 0)
        .unwrap_or(false)
}

/// True when any Rhai `ui[]` slot is active.
pub fn any_plc_active() -> bool {
    BUTTON_STATUS
        .iter()
        .take(BUTTON_COUNT)
        .any(|slot| slot.load(Ordering::Relaxed) != 0)
}

/// Copy current Rhai-driven highlights into `out`.
pub fn snapshot(out: &mut [u8], count: usize) {
    let count = count.min(BUTTON_COUNT).min(out.len());
    for (i, slot) in out.iter_mut().enumerate().take(count) {
        *slot = BUTTON_STATUS[i].load(Ordering::Relaxed);
    }
}

/// Store one highlight slot (legacy RX path).
pub fn store(index: usize, active: u8) {
    if let Some(slot) = BUTTON_STATUS.get(index) {
        slot.store(active, Ordering::Relaxed);
    }
}

#[cfg(feature = "rhai")]
pub fn apply_from_plc(plc: &crate::rhai_state::Plc, scratch: &mut [u8]) {
    plc.read_ui(scratch);
    for (i, value) in scratch.iter().enumerate().take(BUTTON_COUNT) {
        BUTTON_STATUS[i].store(*value, Ordering::Relaxed);
    }
}
