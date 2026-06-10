// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::vec::Vec;
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    subject::Subject,
    WidgetError,
};

/// LVGL roller (scroll-wheel picker) widget.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Roller, RollerMode, Screen};
///
/// let screen = Screen::active().unwrap();
/// let roller = Roller::new(&screen).unwrap();
/// roller.set_options("Jan\nFeb\nMar", RollerMode::Normal);
/// roller.set_visible_row_count(3);
/// roller.center();
/// ```
#[derive(Debug)]
pub struct Roller<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Roller<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Roller<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

/// Roller scrolling mode.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum RollerMode {
    /// Roller stops at the first and last option.
    Normal = lv_roller_mode_t_LV_ROLLER_MODE_NORMAL,
    /// Roller wraps around infinitely.
    Infinite = lv_roller_mode_t_LV_ROLLER_MODE_INFINITE,
}

impl<'p> Roller<'p> {
    /// Create a new roller widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_roller_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Roller {
                obj: Obj::from_raw(handle),
            })
        }
    }

    /// Set roller options as newline-separated string.
    pub fn set_options(&self, opts: &str, mode: RollerMode) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Roller handle cannot be null"
        );
        let mut buf = Vec::with_capacity(opts.len() + 1);
        buf.extend_from_slice(opts.as_bytes());
        buf.push(0);
        // SAFETY: handle non-null; buf NUL-terminated. LVGL copies internally.
        unsafe {
            lv_roller_set_options(
                self.obj.handle(),
                buf.as_ptr() as *const c_char,
                mode as u32,
            )
        };
        self
    }

    /// Set the number of visible rows.
    pub fn set_visible_row_count(&self, rows: u32) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Roller handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_roller_set_visible_row_count(self.obj.handle(), rows) };
        self
    }

    /// Set the selected item (0-based index).
    pub fn set_selected(&self, idx: u32, anim: bool) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Roller handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_roller_set_selected(self.obj.handle(), idx, anim) };
        self
    }

    /// Get the currently selected item index.
    pub fn get_selected(&self) -> u32 {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Roller handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_roller_get_selected(self.obj.handle()) }
    }

    /// Get the number of options in the roller.
    pub fn get_option_count(&self) -> u32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_roller_get_option_count(self.lv_handle()) }
    }

    /// Bind roller selected index to an integer subject (two-way).
    ///
    /// Roller changes update the subject and subject changes update the roller.
    pub fn bind_value(&self, subject: &Subject) -> &Self {
        // SAFETY: lv_handle() is non-null (checked in new()); subject pointer
        // is pinned and valid for the subject's lifetime.
        unsafe { lv_roller_bind_value(self.lv_handle(), subject.as_ptr()) };
        self
    }

    /// Copy the selected option text into `buf`. Returns `None` if `buf` is
    /// empty or the text is not valid UTF-8.
    pub fn get_selected_str<'b>(&self, buf: &'b mut [u8]) -> Option<&'b str> {
        if buf.is_empty() {
            return None;
        }
        // SAFETY: handle non-null (checked in new()); buf is valid writable memory.
        unsafe {
            lv_roller_get_selected_str(
                self.lv_handle(),
                buf.as_mut_ptr() as *mut c_char,
                buf.len() as u32,
            );
        }
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        core::str::from_utf8(&buf[..len]).ok()
    }
}
