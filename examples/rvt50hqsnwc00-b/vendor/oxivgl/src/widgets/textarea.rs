// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL textarea widget (single- or multi-line text input).
///
/// Requires `LV_USE_TEXTAREA = 1` in `lv_conf.h`.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Align, Screen, Textarea};
///
/// let screen = Screen::active().unwrap();
/// let ta = Textarea::new(&screen).unwrap();
/// ta.set_one_line(true);
/// ta.set_text("Hello");
/// ta.align(Align::Center, 0, 0);
/// ```
#[derive(Debug)]
pub struct Textarea<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Textarea<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Textarea<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Textarea<'p> {
    /// Create a new textarea widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_textarea_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Textarea { obj: Obj::from_raw(handle) }) }
    }

    /// Set the textarea text. LVGL copies the string internally.
    pub fn set_text(&self, text: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Textarea handle cannot be null");
        let mut buf = [0u8; 128];
        let len = text.len().min(127);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        // SAFETY: handle non-null; buf is NUL-terminated. LVGL copies internally.
        unsafe { lv_textarea_set_text(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Get the current textarea text. Returns `None` if the internal buffer
    /// is null or not valid UTF-8.
    ///
    /// The returned `&str` borrows from LVGL's internal buffer and is valid
    /// until the next mutation of the textarea content.
    pub fn get_text(&self) -> Option<&str> {
        assert_ne!(self.obj.handle(), null_mut(), "Textarea handle cannot be null");
        // SAFETY: handle non-null; LVGL returns pointer to internal buffer.
        let ptr = unsafe { lv_textarea_get_text(self.obj.handle()) };
        if ptr.is_null() {
            return None;
        }
        // SAFETY: ptr is valid C string from LVGL's internal buffer.
        let cstr = unsafe { core::ffi::CStr::from_ptr(ptr) };
        cstr.to_str().ok()
    }

    /// Append text at the cursor position. LVGL copies the string internally.
    pub fn add_text(&self, text: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Textarea handle cannot be null");
        let mut buf = [0u8; 128];
        let len = text.len().min(127);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        // SAFETY: handle non-null; buf is NUL-terminated. LVGL copies internally.
        unsafe { lv_textarea_add_text(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Insert a single character at the cursor position.
    pub fn add_char(&self, c: char) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Textarea handle cannot be null");
        // SAFETY: handle non-null; c is a valid Unicode code point.
        unsafe { lv_textarea_add_char(self.obj.handle(), c as u32) };
        self
    }

    /// Delete the character before the cursor.
    pub fn delete_char(&self) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Textarea handle cannot be null");
        // SAFETY: handle non-null.
        unsafe { lv_textarea_delete_char(self.obj.handle()) };
        self
    }

    /// Set placeholder text shown when the textarea is empty.
    /// LVGL copies the string internally.
    pub fn set_placeholder_text(&self, text: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Textarea handle cannot be null");
        let mut buf = [0u8; 128];
        let len = text.len().min(127);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        // SAFETY: handle non-null; buf is NUL-terminated. LVGL copies internally.
        unsafe { lv_textarea_set_placeholder_text(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Enable or disable password mode (characters replaced with bullets).
    pub fn set_password_mode(&self, en: bool) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Textarea handle cannot be null");
        // SAFETY: handle non-null.
        unsafe { lv_textarea_set_password_mode(self.obj.handle(), en) };
        self
    }

    /// Enable or disable one-line mode (no line breaks).
    pub fn set_one_line(&self, en: bool) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Textarea handle cannot be null");
        // SAFETY: handle non-null.
        unsafe { lv_textarea_set_one_line(self.obj.handle(), en) };
        self
    }

    /// Set the cursor position. Use `i32::MAX` (`LV_TEXTAREA_CURSOR_LAST`)
    /// for the end.
    pub fn set_cursor_pos(&self, pos: i32) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Textarea handle cannot be null");
        // SAFETY: handle non-null.
        unsafe { lv_textarea_set_cursor_pos(self.obj.handle(), pos) };
        self
    }

    /// Restrict accepted characters.
    ///
    /// LVGL stores the raw pointer (`lv_textarea.c`); the string MUST be
    /// `'static` per spec-memory-lifetime §1/§3.
    pub fn set_accepted_chars(&self, chars: &'static core::ffi::CStr) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Textarea handle cannot be null");
        // SAFETY: handle non-null; chars is 'static and NUL-terminated.
        // LVGL stores the raw pointer; 'static satisfies the lifetime
        // requirement (spec-memory-lifetime §1/§3).
        unsafe { lv_textarea_set_accepted_chars(self.obj.handle(), chars.as_ptr() as *const c_char) };
        self
    }

    /// Set the maximum number of characters.
    pub fn set_max_length(&self, len: u32) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Textarea handle cannot be null");
        // SAFETY: handle non-null.
        unsafe { lv_textarea_set_max_length(self.obj.handle(), len) };
        self
    }

    /// Get the current cursor position (character index).
    pub fn get_cursor_pos(&self) -> u32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_textarea_get_cursor_pos(self.lv_handle()) }
    }

    /// Get whether cursor click-to-position is enabled.
    pub fn get_cursor_click_pos(&self) -> bool {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_textarea_get_cursor_click_pos(self.lv_handle()) }
    }

    /// Get whether password mode is active.
    pub fn get_password_mode(&self) -> bool {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_textarea_get_password_mode(self.lv_handle()) }
    }

    /// Get whether one-line mode is active.
    pub fn get_one_line(&self) -> bool {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_textarea_get_one_line(self.lv_handle()) }
    }

    /// Get the maximum number of characters allowed.
    pub fn get_max_length(&self) -> u32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_textarea_get_max_length(self.lv_handle()) }
    }

    /// Get whether text selection is enabled.
    pub fn get_text_selection(&self) -> bool {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_textarea_get_text_selection(self.lv_handle()) }
    }

    /// Get the password character show time in milliseconds.
    pub fn get_password_show_time(&self) -> u32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_textarea_get_password_show_time(self.lv_handle()) }
    }
}
