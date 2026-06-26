//! Lightweight touch pipeline counters — no per-sample defmt in hot paths.
//!
//! Verbose logging inside `touch.feed` / `pointer_read_cb` blocks the sole LVGL
//! task and can starve scan-out refill → flicker or corrupted pixels.

use core::sync::atomic::{AtomicU32, Ordering};

// #region agent log
pub static QUEUED: AtomicU32 = AtomicU32::new(0);
pub static FED: AtomicU32 = AtomicU32::new(0);
pub static SYNC_OK: AtomicU32 = AtomicU32::new(0);
pub static SYNC_SKIP_TRANSITION: AtomicU32 = AtomicU32::new(0);
pub static SYNC_SKIP_NULL: AtomicU32 = AtomicU32::new(0);
pub static READ_CB: AtomicU32 = AtomicU32::new(0);
pub static LVGL_EVENTS: AtomicU32 = AtomicU32::new(0);
pub static LVGL_CLICKS: AtomicU32 = AtomicU32::new(0);
pub static LVGL_FLUSHES: AtomicU32 = AtomicU32::new(0);
// #endregion

pub fn bump_queued() {
    QUEUED.fetch_add(1, Ordering::Relaxed);
}

pub fn bump_fed() {
    FED.fetch_add(1, Ordering::Relaxed);
}

pub fn bump_sync_ok() {
    SYNC_OK.fetch_add(1, Ordering::Relaxed);
}

pub fn bump_sync_skip_transition() {
    SYNC_SKIP_TRANSITION.fetch_add(1, Ordering::Relaxed);
}

pub fn bump_sync_skip_null() {
    SYNC_SKIP_NULL.fetch_add(1, Ordering::Relaxed);
}

pub fn bump_read_cb() {
    READ_CB.fetch_add(1, Ordering::Relaxed);
}

pub fn bump_lvgl_event() {
    LVGL_EVENTS.fetch_add(1, Ordering::Relaxed);
}

pub fn bump_lvgl_click() {
    LVGL_CLICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn bump_lvgl_flush() {
    LVGL_FLUSHES.fetch_add(1, Ordering::Relaxed);
}
