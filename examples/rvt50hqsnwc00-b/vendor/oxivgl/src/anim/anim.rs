// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_void, marker::PhantomData, sync::atomic::AtomicI32};

use oxivgl_sys::*;

use crate::widgets::AsLvHandle;

/// Handle to a running LVGL animation (the LVGL-owned copy).
///
/// Returned by [`Anim::start()`]. The handle stores enough information
/// to look up the animation via `lv_anim_get`, so operations like
/// [`pause_for`](Self::pause_for) are safe — they silently no-op if the
/// animation has already completed.
///
/// ```ignore
/// let mut a = Anim::new();
/// a.set_var(&obj).set_values(0, 100).set_duration(500)
///     .set_exec_cb(Some(anim_set_x));
/// let handle = a.start();
/// handle.pause_for(200);
/// ```
pub struct AnimHandle {
    /// Target variable pointer, used for `lv_anim_get` lookup.
    var: *mut c_void,
    /// Exec callback, used for `lv_anim_get` lookup.
    exec_cb: lv_anim_exec_xcb_t,
}

impl core::fmt::Debug for AnimHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AnimHandle").finish_non_exhaustive()
    }
}

impl AnimHandle {
    /// Pause the running animation for `ms` milliseconds.
    ///
    /// The animation resumes automatically after the pause expires.
    /// If the animation has already completed (or the target widget was
    /// deleted), this is a no-op.
    ///
    /// Must be called from the LVGL task (LVGL is not thread-safe).
    pub fn pause_for(&self, ms: u32) {
        // Look up the animation by (var, exec_cb). Returns null if the
        // animation has already completed or the widget was deleted.
        let ptr = unsafe { lv_anim_get(self.var, self.exec_cb) };
        if !ptr.is_null() {
            // SAFETY: lv_anim_get returned a valid pointer to a running
            // animation owned by LVGL's internal list.
            unsafe { lv_anim_pause_for(ptr, ms) };
        }
    }
}

/// Stack-local animation builder. LVGL copies the descriptor on `start()`,
/// so this can be dropped after starting.
///
/// The `'w` lifetime ties the animation to the target widget, ensuring the
/// widget is alive when [`start()`](Self::start) is called. After `start()`,
/// LVGL owns a copy and cancels it automatically when the widget is deleted
/// (`lv_obj_delete` calls `lv_anim_delete(obj, NULL)`, `lv_obj.c:525`).
pub struct Anim<'w> {
    pub(crate) inner: lv_anim_t,
    _widget: PhantomData<&'w ()>,
    /// Tracks whether `user_data` holds a bezier pair Box allocation
    /// that must be freed on repeated `set_bezier3_path` calls.
    has_bezier_user_data: bool,
}

impl<'w> core::fmt::Debug for Anim<'w> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Anim").finish_non_exhaustive()
    }
}

impl<'w> Anim<'w> {
    /// Create a new animation descriptor.
    pub fn new() -> Self {
        // SAFETY: lv_anim_t is a plain C struct with no invalid bit patterns;
        // zeroing is safe and matches LVGL's own initialisation pattern.
        let mut inner = unsafe { core::mem::zeroed::<lv_anim_t>() };
        // SAFETY: lv_anim_init writes default field values into the zeroed
        // struct. No other LVGL state is required.
        unsafe { lv_anim_init(&mut inner) };
        Self { inner, _widget: PhantomData, has_bezier_user_data: false }
    }

    /// Set the animated variable (the raw `lv_obj_t*` pointer).
    ///
    /// The `'w` lifetime ensures the widget outlives this `Anim` descriptor.
    pub fn set_var(&mut self, obj: &'w impl AsLvHandle) -> &mut Self {
        unsafe { lv_anim_set_var(&mut self.inner, obj.lv_handle() as *mut c_void) };
        self
    }

    /// Set start and end values.
    pub fn set_values(&mut self, start: i32, end: i32) -> &mut Self {
        unsafe { lv_anim_set_values(&mut self.inner, start, end) };
        self
    }

    /// Set animation duration in milliseconds.
    pub fn set_duration(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_duration(&mut self.inner, ms) };
        self
    }

    /// Set delay before animation starts in milliseconds.
    pub fn set_delay(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_delay(&mut self.inner, ms) };
        self
    }

    /// Set the value-setter callback.
    pub fn set_exec_cb(&mut self, cb: lv_anim_exec_xcb_t) -> &mut Self {
        unsafe { lv_anim_set_exec_cb(&mut self.inner, cb) };
        self
    }

    /// Set a custom exec callback (receives the full `lv_anim_t`).
    pub fn set_custom_exec_cb(&mut self, cb: lv_anim_custom_exec_cb_t) -> &mut Self {
        unsafe { lv_anim_set_custom_exec_cb(&mut self.inner, cb) };
        self
    }

    /// Set the animation easing/path function.
    pub fn set_path_cb(&mut self, cb: lv_anim_path_cb_t) -> &mut Self {
        unsafe { lv_anim_set_path_cb(&mut self.inner, cb) };
        self
    }

    /// Set reverse playback duration in milliseconds.
    pub fn set_reverse_duration(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_reverse_duration(&mut self.inner, ms) };
        self
    }

    /// Set delay before reverse playback.
    pub fn set_reverse_delay(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_reverse_delay(&mut self.inner, ms) };
        self
    }

    /// Set repeat count (use [`ANIM_REPEAT_INFINITE`] for looping).
    pub fn set_repeat_count(&mut self, cnt: u32) -> &mut Self {
        unsafe { lv_anim_set_repeat_count(&mut self.inner, cnt) };
        self
    }

    /// Set delay between repetitions in milliseconds.
    pub fn set_repeat_delay(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_repeat_delay(&mut self.inner, ms) };
        self
    }

    /// Set a cubic-bezier path function using two `AtomicI32` control points.
    ///
    /// The control points `p1` and `p2` are read each frame, so updating
    /// them changes the curve in real-time. Values are in `[0..1024]`.
    /// The atomics must be `'static` because LVGL copies the animation
    /// descriptor and may read the pointers after this `Anim` is dropped.
    ///
    /// **Leak:** Each call allocates 16 bytes via `Box::into_raw` (freed only
    /// if `set_bezier3_path` is called again on the same `Anim`). Acceptable
    /// on embedded where animations are typically long-lived.
    pub fn set_bezier3_path(
        &mut self,
        p1: &'static AtomicI32,
        p2: &'static AtomicI32,
    ) -> &mut Self {
        // Free any previous bezier pair allocation from a prior call.
        if self.has_bezier_user_data {
            // SAFETY: has_bezier_user_data is only set when user_data
            // points to a Box::into_raw allocation of [*const AtomicI32; 2].
            unsafe {
                drop(alloc::boxed::Box::from_raw(
                    self.inner.user_data as *mut [*const AtomicI32; 2],
                ));
            }
        }
        // Pack the two pointers into user_data. Since AtomicI32 refs are
        // 'static, the pointers remain valid for the animation lifetime.
        // We use a leaked Box to hold the pair because LVGL copies the
        // animation descriptor (including the user_data pointer) in
        // lv_anim_start(). Both the original and the LVGL-owned copy must
        // point to the same stable allocation.
        let pair = alloc::boxed::Box::into_raw(alloc::boxed::Box::new([
            p1 as *const AtomicI32,
            p2 as *const AtomicI32,
        ]));
        self.inner.user_data = pair as *mut c_void;
        self.has_bezier_user_data = true;
        unsafe { lv_anim_set_path_cb(&mut self.inner, Some(bezier3_path_cb)) };
        self
    }

    /// Start the animation. LVGL copies the descriptor internally.
    ///
    /// Returns a handle that can look up the running animation by
    /// `(var, exec_cb)`. The handle's operations are no-ops once the
    /// animation completes.
    ///
    /// # Panics
    ///
    /// Panics if [`set_var`](Self::set_var) was not called — LVGL would
    /// pass a null pointer to the exec callback, causing undefined behaviour.
    pub fn start(&self) -> AnimHandle {
        assert!(
            !self.inner.var.is_null(),
            "Anim::start() called without set_var() — would pass null to exec callback"
        );
        let ptr = unsafe { lv_anim_start(&self.inner) };
        assert!(!ptr.is_null(), "lv_anim_start returned NULL");
        AnimHandle {
            var: self.inner.var,
            exec_cb: self.inner.exec_cb,
        }
    }
}

/// Linear animation path — wraps `lv_anim_path_linear`.
pub unsafe extern "C" fn anim_path_linear(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_linear(a) }
}

/// Overshoot animation path.
pub unsafe extern "C" fn anim_path_overshoot(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_overshoot(a) }
}

/// Ease-in animation path.
pub unsafe extern "C" fn anim_path_ease_in(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_ease_in(a) }
}

/// Ease-out animation path.
pub unsafe extern "C" fn anim_path_ease_out(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_ease_out(a) }
}

/// Ease-in-out animation path.
pub unsafe extern "C" fn anim_path_ease_in_out(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_ease_in_out(a) }
}

/// Bounce animation path.
pub unsafe extern "C" fn anim_path_bounce(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_bounce(a) }
}

/// Built-in cubic-bezier3 path callback. Reads P1/P2 from user_data.
unsafe extern "C" fn bezier3_path_cb(a: *const lv_anim_t) -> i32 {
    let a = unsafe { &*a };
    let pair = a.user_data as *const [*const AtomicI32; 2];
    let [p1_ptr, p2_ptr] = unsafe { &*pair };
    let p1 = unsafe { &**p1_ptr }.load(core::sync::atomic::Ordering::Relaxed);
    let p2 = unsafe { &**p2_ptr }.load(core::sync::atomic::Ordering::Relaxed);
    let t = unsafe { lv_map(a.act_time, 0, a.duration, 0, 1024) };
    let step = unsafe { lv_bezier3(t, 0, p1 as u32, p2 as i32, 1024) };
    let new_value = (step as i64 * (a.end_value - a.start_value) as i64) >> 10;
    new_value as i32 + a.start_value
}

/// `LV_ANIM_REPEAT_INFINITE`
pub const ANIM_REPEAT_INFINITE: u32 = LV_ANIM_REPEAT_INFINITE;

/// Exec callback: `lv_obj_set_style_translate_x(var, v, 0)`.
pub unsafe extern "C" fn anim_set_translate_x(var: *mut c_void, v: i32) {
    // SAFETY: var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_obj_set_style_translate_x(var as *mut lv_obj_t, v, 0) };
}

// ── Common animation exec callbacks (lv_anim_exec_xcb_t) ──

/// Exec callback: `lv_obj_set_x(var, v)`.
pub unsafe extern "C" fn anim_set_x(var: *mut c_void, v: i32) {
    // SAFETY: var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_obj_set_x(var as *mut lv_obj_t, v) };
}

/// Exec callback: `lv_obj_set_y(var, v)`.
pub unsafe extern "C" fn anim_set_y(var: *mut c_void, v: i32) {
    // SAFETY: var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_obj_set_y(var as *mut lv_obj_t, v) };
}

/// Exec callback: `lv_obj_set_size(var, v, v)` — uniform width+height.
pub unsafe extern "C" fn anim_set_size(var: *mut c_void, v: i32) {
    // SAFETY: var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_obj_set_size(var as *mut lv_obj_t, v, v) };
}

// ── Common animation custom exec callbacks (lv_anim_custom_exec_cb_t) ──

/// Custom exec callback: `lv_obj_set_width(anim.var, v)`.
pub unsafe extern "C" fn anim_set_width(a: *mut lv_anim_t, v: i32) {
    // SAFETY: a.var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_obj_set_width((*a).var as *mut lv_obj_t, v) };
}

/// Custom exec callback: `lv_obj_set_height(anim.var, v)`.
pub unsafe extern "C" fn anim_set_height(a: *mut lv_anim_t, v: i32) {
    // SAFETY: a.var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_obj_set_height((*a).var as *mut lv_obj_t, v) };
}

/// Exec callback: `lv_obj_set_style_pad_row(var, v, 0)`.
pub unsafe extern "C" fn anim_set_pad_row(var: *mut c_void, v: i32) {
    // SAFETY: var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_obj_set_style_pad_row(var as *mut lv_obj_t, v, 0) };
}

/// Exec callback: `lv_obj_set_style_pad_column(var, v, 0)`.
pub unsafe extern "C" fn anim_set_pad_column(var: *mut c_void, v: i32) {
    // SAFETY: var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_obj_set_style_pad_column(var as *mut lv_obj_t, v, 0) };
}

/// Custom exec callback: `lv_slider_set_value(anim.var, v, false)`.
pub unsafe extern "C" fn anim_set_slider_value(a: *mut lv_anim_t, v: i32) {
    // SAFETY: a.var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_slider_set_value((*a).var as *mut lv_obj_t, v, false) };
}

/// Exec callback: `lv_arc_set_value(var, v)`.
pub unsafe extern "C" fn anim_set_arc_value(var: *mut c_void, v: i32) {
    // SAFETY: var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_arc_set_value(var as *mut lv_obj_t, v) };
}

/// Exec callback: `lv_bar_set_value(var, v, LV_ANIM_ON)`.
pub unsafe extern "C" fn anim_set_bar_value(var: *mut c_void, v: i32) {
    // SAFETY: var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_bar_set_value(var as *mut lv_obj_t, v, true) };
}

/// Exec callback: `lv_scale_set_rotation(var, v)`.
pub unsafe extern "C" fn anim_set_scale_rotation(var: *mut c_void, v: i32) {
    // SAFETY: var is a valid lv_obj_t pointer (guaranteed by LVGL anim framework).
    unsafe { lv_scale_set_rotation(var as *mut lv_obj_t, v) };
}
