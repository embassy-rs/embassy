// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::vec::Vec;
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    subject::Subject,
    WidgetError,
};

/// LVGL drop-down list widget.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Align, Dropdown, Screen};
///
/// let screen = Screen::active().unwrap();
/// let dd = Dropdown::new(&screen).unwrap();
/// dd.set_options("Apple\nBanana\nOrange");
/// dd.align(Align::Center, 0, 0);
/// ```
#[derive(Debug)]
pub struct Dropdown<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Dropdown<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Dropdown<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

/// Drop-down list open direction.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum DdDir {
    /// Open below (default).
    Bottom = lv_dir_t_LV_DIR_BOTTOM,
    /// Open above.
    Top = lv_dir_t_LV_DIR_TOP,
    /// Open to the left.
    Left = lv_dir_t_LV_DIR_LEFT,
    /// Open to the right.
    Right = lv_dir_t_LV_DIR_RIGHT,
}

impl<'p> Dropdown<'p> {
    /// Create a new dropdown widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_dropdown_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Dropdown {
                obj: Obj::from_raw(handle),
            })
        }
    }

    /// Set dropdown options as newline-separated string.
    /// LVGL copies the string internally.
    pub fn set_options(&self, opts: &str) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        let mut buf = Vec::with_capacity(opts.len() + 1);
        buf.extend_from_slice(opts.as_bytes());
        buf.push(0);
        // SAFETY: handle non-null; buf is NUL-terminated. LVGL copies internally.
        unsafe { lv_dropdown_set_options(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Set the open direction.
    pub fn set_dir(&self, dir: DdDir) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_dropdown_set_dir(self.obj.handle(), dir as lv_dir_t) };
        self
    }

    /// Set the dropdown symbol (typically an arrow icon string).
    ///
    /// LVGL stores the raw pointer (`dropdown->symbol = symbol`,
    /// `lv_dropdown.c:373`). The string must be `'static` (spec §3.3).
    /// Use a `c"..."` literal: `dropdown.set_symbol(c"\u{f078}")`.
    pub fn set_symbol(&self, symbol: &'static core::ffi::CStr) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        // SAFETY: handle non-null; symbol is 'static and NUL-terminated.
        // LVGL stores the raw pointer (lv_dropdown.c:373); 'static satisfies
        // the lifetime requirement (spec §3.3).
        unsafe {
            lv_dropdown_set_symbol(
                self.obj.handle(),
                symbol.as_ptr() as *const core::ffi::c_void,
            )
        };
        self
    }

    /// Set the selected item index.
    pub fn set_selected(&self, idx: u32) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_dropdown_set_selected(self.obj.handle(), idx) };
        self
    }

    /// Set a fixed text to display on the button (instead of the selected
    /// option). Useful for menu-style dropdowns.
    ///
    /// LVGL stores the raw pointer directly; the string must be `'static`.
    pub fn set_text(&self, text: &'static core::ffi::CStr) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        // SAFETY: handle non-null; text is 'static and NUL-terminated.
        // LVGL stores the raw pointer (lv_dropdown.c:174); 'static satisfies
        // the lifetime requirement (spec §12.5).
        unsafe { lv_dropdown_set_text(self.obj.handle(), text.as_ptr() as *const c_char) };
        self
    }

    /// Enable/disable highlighting of the selected item in the list.
    /// Set to `false` for menu-style dropdowns where no item stays selected.
    pub fn set_selected_highlight(&self, en: bool) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        unsafe { lv_dropdown_set_selected_highlight(self.obj.handle(), en) };
        self
    }

    /// Bind dropdown selected index to an integer subject (two-way).
    ///
    /// Dropdown changes update the subject and subject changes update the dropdown.
    pub fn bind_value(&self, subject: &Subject) -> &Self {
        // SAFETY: lv_handle() is non-null (checked in new()); subject pointer
        // is pinned and valid for the subject's lifetime.
        unsafe { lv_dropdown_bind_value(self.lv_handle(), subject.as_ptr()) };
        self
    }

    /// Get the number of options in the dropdown list.
    pub fn get_option_count(&self) -> u32 {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_dropdown_get_option_count(self.obj.handle()) }
    }

    /// Get whether the selected item is highlighted in the list.
    pub fn get_selected_highlight(&self) -> bool {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_dropdown_get_selected_highlight(self.obj.handle()) }
    }

    /// Get the open direction as raw `u32`.
    ///
    /// Returns the raw `lv_dir_t` value because `lv_dir_t` includes combined
    /// direction values (HOR, VER, ALL) not covered by [`DdDir`].
    pub fn get_dir(&self) -> u32 {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_dropdown_get_dir(self.obj.handle()) }
    }

    /// Get the popup list widget of the dropdown.
    ///
    /// The returned [`Child`](super::Child) is a non-owning handle; the list is owned
    /// by the dropdown and lives as long as the dropdown exists.
    pub fn get_list(&self) -> super::Child<super::Obj<'p>> {
        // SAFETY: handle non-null (checked in new()); lv_dropdown_get_list
        // always returns a valid child pointer for an existing dropdown.
        let ptr = unsafe { lv_dropdown_get_list(self.obj.handle()) };
        assert!(!ptr.is_null(), "lv_dropdown_get_list returned NULL");
        super::Child::new(super::Obj::from_raw(ptr))
    }

    /// Get the currently selected item index.
    pub fn get_selected(&self) -> u32 {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_dropdown_get_selected(self.obj.handle()) }
    }

    /// Copy the selected item text into `buf`. Returns the written slice
    /// (without null terminator), or `None` if the handle is null.
    pub fn get_selected_str<'b>(&self, buf: &'b mut [u8]) -> Option<&'b str> {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        if buf.is_empty() {
            return None;
        }
        // SAFETY: handle non-null; buf is valid writable memory.
        unsafe {
            lv_dropdown_get_selected_str(
                self.obj.handle(),
                buf.as_mut_ptr() as *mut core::ffi::c_char,
                buf.len() as u32,
            );
        }
        // Find null terminator written by LVGL.
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        core::str::from_utf8(&buf[..len]).ok()
    }
}
