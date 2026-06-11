//! Cross-task touch / LVGL input debug probes (RTT via `touch_info_task`).
//!
//! Atomics let a low-priority heartbeat task print the latest touch sample and
//! LVGL indev feedback without spamming the UI loop.

use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicU32, Ordering};

/// Latest touch X coordinate (display pixels).
pub static X: AtomicI32 = AtomicI32::new(0);
/// Latest touch Y coordinate (display pixels).
pub static Y: AtomicI32 = AtomicI32::new(0);
/// `true` while the panel reports active contact.
pub static PRESSED: AtomicBool = AtomicBool::new(false);
/// `true` when the last I2C touch read succeeded.
pub static I2C_OK: AtomicBool = AtomicBool::new(false);
/// First byte from the touch controller status register.
pub static RAW_STATUS: AtomicU8 = AtomicU8::new(0);
/// Last `lv_indev_get_active_obj()` pointer (lower 32 bits on Cortex-M).
pub static ACTIVE_OBJ: AtomicU32 = AtomicU32::new(0);
/// Scene button index under the last sample (`-1` = none), from layout bounds.
pub static HIT_BTN: AtomicI32 = AtomicI32::new(-1);
/// Number of widget events handled by [`super::widget_view::WidgetView::on_event`].
pub static EVENT_COUNT: AtomicU32 = AtomicU32::new(0);
/// Number of `CTP_INT` wake-ups handled by the interrupt-driven touch task.
pub static INT_WAKEUPS: AtomicU32 = AtomicU32::new(0);

/// Store the latest board touch sample for the heartbeat task.
pub fn publish_touch(x: i32, y: i32, pressed: bool, i2c_ok: bool, raw_status: u8) {
    X.store(x, Ordering::Relaxed);
    Y.store(y, Ordering::Relaxed);
    PRESSED.store(pressed, Ordering::Relaxed);
    I2C_OK.store(i2c_ok, Ordering::Relaxed);
    RAW_STATUS.store(raw_status, Ordering::Relaxed);
}

/// Store LVGL indev feedback after a `timer_handler()` pass with touch active.
pub fn publish_indev(active_obj: *mut oxivgl_sys::lv_obj_t, hit_btn: Option<usize>) {
    ACTIVE_OBJ.store(active_obj as u32, Ordering::Relaxed);
    HIT_BTN.store(hit_btn.map(|i| i as i32).unwrap_or(-1), Ordering::Relaxed);
}

/// Increment the widget event counter (called from `WidgetView::on_event`).
pub fn bump_event_count() {
    EVENT_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Increment the `CTP_INT` wake-up counter (called from the touch task).
pub fn bump_int_wakeups() {
    INT_WAKEUPS.fetch_add(1, Ordering::Relaxed);
}
