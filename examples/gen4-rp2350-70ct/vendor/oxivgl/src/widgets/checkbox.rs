// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError, with_cstr,
    obj::{AsLvHandle, Obj},
};

/// LVGL checkbox widget (label + tick box).
#[derive(Debug)]
pub struct Checkbox<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Checkbox<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Checkbox<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Checkbox<'p> {
    /// Create a new checkbox widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_checkbox_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Checkbox { obj: Obj::from_raw(handle) }) }
    }

    /// Get the checkbox label text. Returns `None` if the pointer is null or
    /// the text is not valid UTF-8.
    pub fn get_text(&self) -> Option<&str> {
        assert_ne!(self.obj.handle(), null_mut(), "Checkbox handle cannot be null");
        // SAFETY: handle non-null (asserted above); lv_checkbox_get_text returns
        // a pointer to LVGL-managed memory valid for the widget's lifetime.
        let ptr = unsafe { lv_checkbox_get_text(self.obj.handle()) };
        if ptr.is_null() {
            return None;
        }
        // SAFETY: ptr is non-null and NUL-terminated (guaranteed by LVGL).
        let cstr = unsafe { core::ffi::CStr::from_ptr(ptr) };
        cstr.to_str().ok()
    }

    /// Set checkbox label text. Accepts any `&str` (no length cap); LVGL copies
    /// the string internally.
    pub fn text(&self, s: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Checkbox handle cannot be null");
        // SAFETY: handle non-null (asserted above); with_cstr supplies a
        // NUL-terminated buffer valid for the call. LVGL copies internally.
        with_cstr(s, |p| unsafe { lv_checkbox_set_text(self.obj.handle(), p) });
        self
    }
}
