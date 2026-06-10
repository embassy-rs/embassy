// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    button::Button,
    child::Child,
    obj::{AsLvHandle, Obj},
};
use crate::symbols::Symbol;

/// LVGL list widget — a vertically-scrollable container with text headers and
/// icon buttons.
///
/// Requires `LV_USE_LIST = 1` in `lv_conf.h`.
///
/// Children created via [`add_text`](Self::add_text) and
/// [`add_button`](Self::add_button) are owned by the list (LVGL frees them
/// when the list is deleted).
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{List, Screen};
/// use oxivgl::symbols;
///
/// let screen = Screen::active().unwrap();
/// let list = List::new(&screen).unwrap();
/// list.add_text("Section");
/// let btn = list.add_button(Some(&symbols::FILE), "Open");
/// ```
#[derive(Debug)]
pub struct List<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for List<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for List<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> List<'p> {
    /// Create a list as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_list_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(List { obj: Obj::from_raw(handle) }) }
    }

    /// Add a text header (section label) to the list.
    ///
    /// LVGL copies the string internally. The returned label is owned by the
    /// list — do not store or drop it separately.
    pub fn add_text(&self, text: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "List handle cannot be null");
        let bytes = text.as_bytes();
        let len = bytes.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&bytes[..len]);
        // SAFETY: handle non-null (asserted above); buf is NUL-terminated
        // (zero-initialized, len <= 127). LVGL copies the text (lv_list.c:81).
        unsafe { lv_list_add_text(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Add a button with an optional icon and text.
    ///
    /// LVGL copies both the icon symbol and text internally
    /// (`lv_list.c:94` creates an image from the icon pointer,
    /// `lv_list.c:100` creates a label from the text pointer).
    ///
    /// The returned [`Child<Button>`] is a non-owning handle — the button is
    /// owned by the list widget. Use it for event registration or styling.
    pub fn add_button(&self, icon: Option<&Symbol>, text: &str) -> Child<Button<'p>> {
        assert_ne!(self.obj.handle(), null_mut(), "List handle cannot be null");
        let bytes = text.as_bytes();
        let len = bytes.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&bytes[..len]);
        let icon_ptr = match icon {
            Some(sym) => sym.as_ptr() as *const core::ffi::c_void,
            None => core::ptr::null(),
        };
        // SAFETY: handle non-null (asserted above); icon_ptr is NULL or a valid
        // symbol string pointer; buf is NUL-terminated. LVGL creates child
        // widgets that are owned by the list (lv_list.c:88-106).
        let btn_ptr = unsafe { lv_list_add_button(self.obj.handle(), icon_ptr, buf.as_ptr() as *const c_char) };
        assert!(!btn_ptr.is_null(), "lv_list_add_button returned NULL");
        // The button is a child of the list — wrap as Child to suppress Drop.
        Child::new(Button::from_raw(btn_ptr))
    }

    /// Get the text of a button in this list.
    ///
    /// Returns `None` if the button has no label child.
    pub fn get_button_text(&self, btn: &impl AsLvHandle) -> Option<&str> {
        assert_ne!(self.obj.handle(), null_mut(), "List handle cannot be null");
        // SAFETY: handle and btn handle non-null; LVGL returns "" if no label
        // found (lv_list.c:122).
        let ptr = unsafe { lv_list_get_button_text(self.obj.handle(), btn.lv_handle()) };
        if ptr.is_null() {
            return None;
        }
        // SAFETY: LVGL returns a valid C string pointer into the label's text
        // buffer. The label is owned by the list, so the pointer is valid as
        // long as the list lives.
        let cstr = unsafe { core::ffi::CStr::from_ptr(ptr) };
        cstr.to_str().ok()
    }
}
