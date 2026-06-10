// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    child::Child,
    obj::{AsLvHandle, Obj},
};
use crate::symbols::Symbol;

/// LVGL window widget — a container with a header bar (title + buttons) and a
/// scrollable content area.
///
/// Requires `LV_USE_WIN = 1` in `lv_conf.h`.
///
/// The header is a flex-row container created automatically. Use
/// [`add_title`](Self::add_title) and [`add_button`](Self::add_button) to
/// populate it. The content area is obtained via
/// [`get_content`](Self::get_content) and can host arbitrary child widgets.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Label, Screen, Win};
///
/// let screen = Screen::active().unwrap();
/// let win = Win::new(&screen).unwrap();
/// let _title = win.add_title("My Window");
/// let content = win.get_content();
/// let _lbl = Label::new(&content).unwrap();
/// ```
#[derive(Debug)]
pub struct Win<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Win<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Win<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Win<'p> {
    /// Create a window as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_win_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Win { obj: Obj::from_raw(handle) })
        }
    }

    /// Add a title label to the header. The returned handle is a label widget
    /// owned by the window's header.
    ///
    /// `txt` is truncated to 127 bytes. LVGL copies the string internally.
    pub fn add_title(&self, txt: &str) -> Child<Obj<'p>> {
        let bytes = txt.as_bytes();
        let len = bytes.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&bytes[..len]);
        // SAFETY: handle non-null (constructor guarantees); buf is
        // NUL-terminated; LVGL copies the text into an internal label.
        let ptr =
            unsafe { lv_win_add_title(self.obj.handle(), buf.as_ptr() as *const c_char) };
        assert!(!ptr.is_null(), "lv_win_add_title returned NULL");
        Child::new(Obj::from_raw(ptr))
    }

    /// Add a button with a symbol icon to the header.
    ///
    /// `icon` — a [`Symbol`] constant (e.g. `symbols::CLOSE`).
    /// `btn_w` — button width in pixels.
    ///
    /// The returned handle is a button widget owned by the window's header.
    pub fn add_button(&self, icon: &Symbol, btn_w: i32) -> Child<Obj<'p>> {
        // SAFETY: handle non-null (constructor guarantees); icon.as_ptr()
        // points to a 'static NUL-terminated byte slice. LVGL creates an
        // image child using the icon pointer (lv_win.c:lv_win_add_button).
        let ptr = unsafe {
            lv_win_add_button(
                self.obj.handle(),
                icon.as_ptr() as *const core::ffi::c_void,
                btn_w,
            )
        };
        assert!(!ptr.is_null(), "lv_win_add_button returned NULL");
        Child::new(Obj::from_raw(ptr))
    }

    /// Get the header container of the window.
    pub fn get_header(&self) -> Child<Obj<'p>> {
        // SAFETY: handle non-null (constructor guarantees); LVGL returns the
        // first child (the header obj created by the constructor).
        let ptr = unsafe { lv_win_get_header(self.obj.handle()) };
        assert!(!ptr.is_null(), "lv_win_get_header returned NULL");
        Child::new(Obj::from_raw(ptr))
    }

    /// Get the content area of the window. Add child widgets here.
    pub fn get_content(&self) -> Child<Obj<'p>> {
        // SAFETY: handle non-null (constructor guarantees); LVGL returns the
        // second child (the content obj created by the constructor).
        let ptr = unsafe { lv_win_get_content(self.obj.handle()) };
        assert!(!ptr.is_null(), "lv_win_get_content returned NULL");
        Child::new(Obj::from_raw(ptr))
    }
}
