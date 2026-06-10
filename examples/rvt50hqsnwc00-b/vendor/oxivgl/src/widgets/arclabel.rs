// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL ArcLabel widget — text rendered along a circular arc.
//!
//! Requires `LV_USE_ARCLABEL = 1` in `lv_conf.h`.

use core::{ffi::CStr, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// Direction for arc-label text flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ArcLabelDir {
    /// Text flows clockwise.
    Clockwise = lv_arclabel_dir_t_LV_ARCLABEL_DIR_CLOCKWISE,
    /// Text flows counter-clockwise.
    CounterClockwise = lv_arclabel_dir_t_LV_ARCLABEL_DIR_COUNTER_CLOCKWISE,
}

/// LVGL ArcLabel widget. Wraps [`Obj`] and `Deref`s to it for style methods.
///
/// Displays text along a circular arc with configurable radius, start angle,
/// arc size, and direction.
#[derive(Debug)]
pub struct ArcLabel<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for ArcLabel<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for ArcLabel<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> ArcLabel<'p> {
    /// Create an ArcLabel widget as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_arclabel_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(ArcLabel { obj: Obj::from_raw(handle) })
        }
    }

    /// Set the displayed text from a `'static` C string.
    ///
    /// LVGL stores the pointer directly — the text must be `'static`.
    pub fn set_text_static(&self, s: &'static CStr) -> &Self {
        // SAFETY: handle non-null (constructor guarantees); s is 'static and
        // NUL-terminated. LVGL stores the pointer without copying
        // (lv_arclabel_set_text_static).
        unsafe { lv_arclabel_set_text_static(self.obj.handle(), s.as_ptr()) };
        self
    }

    /// Set the arc radius in pixels.
    pub fn set_radius(&self, r: u32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_arclabel_set_radius(self.obj.handle(), r) };
        self
    }

    /// Set the start angle of the arc (in degrees, 0 = top/12 o'clock).
    pub fn set_angle_start(&self, angle: f32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_arclabel_set_angle_start(self.obj.handle(), angle) };
        self
    }

    /// Set the angular size of the arc (in degrees).
    pub fn set_angle_size(&self, size: f32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_arclabel_set_angle_size(self.obj.handle(), size) };
        self
    }

    /// Set text flow direction (clockwise or counter-clockwise).
    pub fn set_dir(&self, dir: ArcLabelDir) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_arclabel_set_dir(self.obj.handle(), dir as lv_arclabel_dir_t) };
        self
    }
}
