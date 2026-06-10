// SPDX-License-Identifier: MIT OR Apache-2.0
use core::cell::Cell;

use alloc::boxed::Box;
use oxivgl_sys::*;

/// LVGL periodic timer with polling-based trigger detection.
///
/// Creates an LVGL timer that fires at a fixed interval. Check
/// [`triggered()`](Timer::triggered) in your View's `update()` method
/// to react to each tick. Drop calls `lv_timer_delete` (spec §2).
///
/// # Example
///
/// ```ignore
/// use oxivgl::timer::Timer;
///
/// struct MyView { timer: Timer, counter: i32 }
///
/// fn update(&mut self) -> Result<(), WidgetError> {
///     if self.timer.triggered() {
///         self.counter += 1;
///         // update widgets…
///     }
///     Ok(())
/// }
/// ```
pub struct Timer {
    ptr: *mut lv_timer_t,
    /// Heap-allocated flag with stable address for LVGL user_data.
    flag: *mut Cell<bool>,
}

impl core::fmt::Debug for Timer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Timer").finish_non_exhaustive()
    }
}

impl Timer {
    /// Create a timer that fires every `period_ms` milliseconds.
    ///
    /// The timer starts immediately. Use [`pause()`](Timer::pause) to
    /// defer, or [`set_repeat_count(1)`](Timer::set_repeat_count) for
    /// one-shot behavior.
    pub fn new(period_ms: u32) -> Result<Self, crate::widgets::WidgetError> {
        let flag = Box::into_raw(Box::new(Cell::new(false)));

        unsafe extern "C" fn tick(t: *mut lv_timer_t) {
            unsafe {
                let flag = lv_timer_get_user_data(t) as *mut Cell<bool>;
                (*flag).set(true);
            }
        }

        // SAFETY: tick is a valid extern "C" fn; flag is heap-allocated
        // and lives until Timer::drop frees it.
        let ptr = unsafe {
            lv_timer_create(Some(tick), period_ms, flag as *mut core::ffi::c_void)
        };
        if ptr.is_null() {
            // Reclaim the flag — no timer was created.
            unsafe { drop(Box::from_raw(flag)) };
            return Err(crate::widgets::WidgetError::LvglNullPointer);
        }
        // Rust Timer owns the lifetime — prevent LVGL from auto-deleting
        // when repeat_count reaches 0 (our Drop handles cleanup).
        unsafe { lv_timer_set_auto_delete(ptr, false) };
        Ok(Timer { ptr, flag })
    }

    /// Returns `true` once per timer period, then resets the flag.
    ///
    /// Call this in `View::update()` to detect timer ticks. Only the first
    /// call per update cycle returns `true`; subsequent calls in the same
    /// cycle return `false`. This is intentional for single-poll usage.
    pub fn triggered(&self) -> bool {
        // SAFETY: flag is heap-allocated and valid for Timer's lifetime.
        // Cell<bool> is not Sync, but this is safe because LVGL's timer
        // callback (which writes to the flag) and this read both execute
        // on the single LVGL task — never concurrently.
        let flag = unsafe { &*self.flag };
        if flag.get() {
            flag.set(false);
            true
        } else {
            false
        }
    }

    /// Change the timer period.
    pub fn set_period(&self, period_ms: u32) -> &Self {
        // SAFETY: ptr valid for Timer's lifetime (created by lv_timer_create,
        // freed in Drop).
        unsafe { lv_timer_set_period(self.ptr, period_ms) };
        self
    }

    /// Set repeat count. Use `-1` for infinite, `1` for one-shot.
    pub fn set_repeat_count(&self, count: i32) -> &Self {
        // SAFETY: ptr valid for Timer's lifetime.
        unsafe { lv_timer_set_repeat_count(self.ptr, count) };
        self
    }

    /// Mark the timer ready — it will fire on the next LVGL tick
    /// regardless of remaining period.
    pub fn ready(&self) -> &Self {
        // SAFETY: ptr valid for Timer's lifetime.
        unsafe { lv_timer_ready(self.ptr) };
        self
    }

    /// Pause the timer (stops firing until [`resume()`](Timer::resume)).
    pub fn pause(&self) -> &Self {
        // SAFETY: ptr valid for Timer's lifetime.
        unsafe { lv_timer_pause(self.ptr) };
        self
    }

    /// Resume a paused timer.
    pub fn resume(&self) -> &Self {
        // SAFETY: ptr valid for Timer's lifetime.
        unsafe { lv_timer_resume(self.ptr) };
        self
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // SAFETY: ptr created by lv_timer_create; flag by Box::into_raw.
        // Delete timer first (removes LVGL's reference to flag), then
        // reclaim the flag allocation. Order matters: LVGL must not
        // reference the flag after it's freed. Safe because LVGL's
        // timer handler and drop both run on the single LVGL task.
        unsafe {
            lv_timer_delete(self.ptr);
            drop(Box::from_raw(self.flag));
        }
    }
}
