// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    child::Child,
    obj::{AsLvHandle, Obj},
};

/// LVGL message box widget — a modal dialog with optional title, text,
/// buttons, and close button.
///
/// Requires `LV_USE_MSGBOX = 1` in `lv_conf.h`.
///
/// When created with `parent = None`, LVGL creates a full-screen backdrop
/// and centers the message box on it (modal mode).
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Msgbox, Obj};
///
/// let mbox = Msgbox::new(None::<&Obj<'_>>).unwrap();
/// mbox.add_title("Hello");
/// mbox.add_text("This is a message.");
/// mbox.add_close_button();
/// ```
#[derive(Debug)]
pub struct Msgbox<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Msgbox<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Msgbox<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Msgbox<'p> {
    /// Create a message box.
    ///
    /// Pass `None` for a modal (full-screen backdrop) message box.
    /// Pass `Some(&parent)` to embed it in a specific container.
    pub fn new(parent: Option<&impl AsLvHandle>) -> Result<Self, WidgetError> {
        let parent_ptr = match parent {
            Some(p) => {
                let ptr = p.lv_handle();
                assert_ne!(ptr, null_mut(), "Parent widget cannot be null");
                ptr
            }
            None => null_mut(),
        };
        // SAFETY: parent_ptr is either NULL (modal mode) or a valid non-null
        // handle. lv_init() called via LvglDriver.
        let handle = unsafe { lv_msgbox_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Msgbox { obj: Obj::from_raw(handle) })
        }
    }

    /// Add a title to the message box (also creates a header if absent).
    ///
    /// LVGL copies the string internally.
    pub fn add_title(&self, title: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Msgbox handle cannot be null");
        let bytes = title.as_bytes();
        let len = bytes.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&bytes[..len]);
        // SAFETY: handle non-null (asserted above); buf is NUL-terminated.
        // LVGL copies the text (lv_msgbox.c creates a label child).
        unsafe { lv_msgbox_add_title(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Add a text paragraph to the content area.
    ///
    /// LVGL copies the string internally. Multiple calls add multiple text
    /// labels stacked vertically.
    pub fn add_text(&self, text: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Msgbox handle cannot be null");
        let bytes = text.as_bytes();
        let len = bytes.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&bytes[..len]);
        // SAFETY: handle non-null (asserted above); buf is NUL-terminated.
        // LVGL copies the text.
        unsafe { lv_msgbox_add_text(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Add a close button (×) to the header.
    pub fn add_close_button(&self) -> Child<Obj<'_>> {
        assert_ne!(self.obj.handle(), null_mut(), "Msgbox handle cannot be null");
        // SAFETY: handle non-null (asserted above). Creates a close button
        // in the header (lv_msgbox.c).
        let btn = unsafe { lv_msgbox_add_close_button(self.obj.handle()) };
        assert!(!btn.is_null(), "lv_msgbox_add_close_button returned NULL");
        Child::new(Obj::from_raw(btn))
    }

    /// Add a button to the footer with the given text.
    ///
    /// LVGL copies the string internally.
    pub fn add_footer_button(&self, text: &str) -> Child<Obj<'_>> {
        assert_ne!(self.obj.handle(), null_mut(), "Msgbox handle cannot be null");
        let bytes = text.as_bytes();
        let len = bytes.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&bytes[..len]);
        // SAFETY: handle non-null; buf is NUL-terminated. LVGL copies the text.
        let btn = unsafe { lv_msgbox_add_footer_button(self.obj.handle(), buf.as_ptr() as *const c_char) };
        assert!(!btn.is_null(), "lv_msgbox_add_footer_button returned NULL");
        Child::new(Obj::from_raw(btn))
    }

    /// Add a button to the header with the given icon symbol.
    ///
    /// Returns the button object as a non-owning `Child<Obj>`.
    pub fn add_header_button(&self, icon: &crate::symbols::Symbol) -> Child<Obj<'_>> {
        assert_ne!(self.obj.handle(), null_mut(), "Msgbox handle cannot be null");
        // SAFETY: handle non-null; icon.as_ptr() is a valid 'static C string.
        // LVGL stores the icon pointer as an image source.
        let btn = unsafe {
            lv_msgbox_add_header_button(self.obj.handle(), icon.as_ptr() as *const core::ffi::c_void)
        };
        assert!(!btn.is_null(), "lv_msgbox_add_header_button returned NULL");
        Child::new(Obj::from_raw(btn))
    }

    /// Get the content area of the message box (for adding custom widgets).
    pub fn get_content(&self) -> Child<Obj<'_>> {
        assert_ne!(self.obj.handle(), null_mut(), "Msgbox handle cannot be null");
        // SAFETY: handle non-null; returns the internal content container.
        let content = unsafe { lv_msgbox_get_content(self.obj.handle()) };
        assert!(!content.is_null(), "lv_msgbox_get_content returned NULL");
        Child::new(Obj::from_raw(content))
    }

    /// Get the footer area of the message box, if one exists.
    pub fn get_footer(&self) -> Option<Child<Obj<'_>>> {
        assert_ne!(self.obj.handle(), null_mut(), "Msgbox handle cannot be null");
        // SAFETY: handle non-null; returns the internal footer or NULL.
        let footer = unsafe { lv_msgbox_get_footer(self.obj.handle()) };
        if footer.is_null() { None } else { Some(Child::new(Obj::from_raw(footer))) }
    }

    /// Get the header area of the message box, if one exists.
    pub fn get_header(&self) -> Option<Child<Obj<'_>>> {
        assert_ne!(self.obj.handle(), null_mut(), "Msgbox handle cannot be null");
        // SAFETY: handle non-null; returns the internal header or NULL.
        let header = unsafe { lv_msgbox_get_header(self.obj.handle()) };
        if header.is_null() { None } else { Some(Child::new(Obj::from_raw(header))) }
    }

    /// Close the message box immediately.
    ///
    /// Consumes `self` because `lv_msgbox_close` calls `lv_obj_delete`
    /// internally — the LVGL object is freed by the call. The `lv_obj_is_valid`
    /// guard in `Obj::drop` prevents a double-free when `self` is dropped.
    pub fn close(self) {
        let handle = self.lv_handle();
        // SAFETY: lv_msgbox_close deletes the LVGL object. The lv_obj_is_valid
        // guard in Obj::drop prevents double-free when self is dropped after this.
        unsafe { lv_msgbox_close(handle) };
    }
}
