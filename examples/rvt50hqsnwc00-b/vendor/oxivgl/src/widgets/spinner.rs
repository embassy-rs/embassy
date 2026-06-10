// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL Spinner widget — an animated arc that rotates continuously.
/// Built on top of [`Arc`](super::Arc); wraps [`Obj`](super::obj::Obj) and
/// `Deref`s to it for style/positioning methods.
///
/// Requires `LV_USE_SPINNER = 1` and `LV_USE_ARC = 1` in `lv_conf.h`.
#[derive(Debug)]
pub struct Spinner<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Spinner<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Spinner<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Spinner<'p> {
    /// Create a spinner as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    ///
    /// Default animation: 1000 ms cycle, 200° arc length.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_spinner_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Spinner { obj: Obj::from_raw(handle) }) }
    }

    /// Set the animation cycle time and arc angle.
    ///
    /// `time_ms` — full rotation period in milliseconds.
    /// `arc_length` — visible arc length in degrees (0–360).
    pub fn set_anim_params(&self, time_ms: u32, arc_length: u32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinner_set_anim_params(self.obj.handle(), time_ms, arc_length) };
        self
    }

    /// Get the animation cycle duration in milliseconds.
    pub fn get_anim_duration(&self) -> u32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_spinner_get_anim_duration(self.lv_handle()) }
    }

    /// Get the visible arc sweep angle in degrees.
    pub fn get_arc_sweep(&self) -> u32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_spinner_get_arc_sweep(self.lv_handle()) }
    }
}
