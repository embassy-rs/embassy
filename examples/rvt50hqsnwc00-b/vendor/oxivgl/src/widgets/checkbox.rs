// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
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

    /// Set checkbox label text. Truncates at 127 bytes.
    pub fn text(&self, s: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Checkbox handle cannot be null");
        let bytes = s.as_bytes();
        let len = bytes.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&bytes[..len]);
        // SAFETY: handle non-null (asserted above); buf is NUL-terminated.
        unsafe { lv_checkbox_set_text(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }
}
