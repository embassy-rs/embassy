// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    child::Child,
    dropdown::DdDir,
    obj::{AsLvHandle, Obj},
    WidgetError,
};

/// LVGL tabview widget — a container with a tab bar and multiple content panes.
///
/// Requires `LV_USE_TABVIEW = 1` in `lv_conf.h`.
///
/// Each tab content pane is a scrollable `Obj` owned by the tabview. Obtain
/// panes via [`add_tab`](Self::add_tab) and populate them with child widgets.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Screen, Tabview};
///
/// let screen = Screen::active().unwrap();
/// let tv = Tabview::new(&screen).unwrap();
/// let tab1 = tv.add_tab("First");
/// let tab2 = tv.add_tab("Second");
/// ```
#[derive(Debug)]
pub struct Tabview<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Tabview<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Tabview<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Tabview<'p> {
    /// Create a tabview as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted); lv_init() called via LvglDriver.
        let handle = unsafe { lv_tabview_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Tabview { obj: Obj::from_raw(handle) })
        }
    }

    /// Add a tab with the given name. Returns a non-owning handle to the tab
    /// content pane, which is owned by the tabview.
    ///
    /// `name` is truncated to 63 bytes.
    pub fn add_tab(&self, name: &str) -> Child<Obj<'p>> {
        let bytes = name.as_bytes();
        let len = bytes.len().min(63);
        let mut buf = [0u8; 64];
        buf[..len].copy_from_slice(&bytes[..len]);
        // SAFETY: handle non-null; buf is NUL-terminated; LVGL copies the name.
        // The returned pointer is a child obj owned by the tabview.
        let ptr =
            unsafe { lv_tabview_add_tab(self.lv_handle(), buf.as_ptr() as *const c_char) };
        assert!(!ptr.is_null(), "lv_tabview_add_tab returned NULL");
        Child::new(Obj::from_raw(ptr))
    }

    /// Show the tab at `idx`. `anim` enables slide animation.
    pub fn set_active(&self, idx: u32, anim: bool) -> &Self {
        // SAFETY: handle non-null; idx out of range is clamped by LVGL.
        unsafe { lv_tabview_set_active(self.lv_handle(), idx, anim) };
        self
    }

    /// Set the position of the tab bar (Top / Bottom / Left / Right).
    pub fn set_tab_bar_position(&self, dir: DdDir) -> &Self {
        // SAFETY: handle non-null; dir is a valid lv_dir_t value.
        unsafe { lv_tabview_set_tab_bar_position(self.lv_handle(), dir as lv_dir_t) };
        self
    }

    /// Set the pixel width (Left/Right bar) or height (Top/Bottom bar) of the
    /// tab bar.
    pub fn set_tab_bar_size(&self, size: i32) -> &Self {
        // SAFETY: handle non-null.
        unsafe { lv_tabview_set_tab_bar_size(self.lv_handle(), size) };
        self
    }

    /// Get the number of tabs.
    pub fn get_tab_count(&self) -> u32 {
        // SAFETY: handle non-null.
        unsafe { lv_tabview_get_tab_count(self.lv_handle()) }
    }

    /// Get the zero-based index of the currently active tab.
    pub fn get_tab_active(&self) -> u32 {
        // SAFETY: handle non-null.
        unsafe { lv_tabview_get_tab_active(self.lv_handle()) }
    }

    /// Get a non-owning handle to the content container (parent of all tab
    /// panes).
    pub fn get_content(&self) -> Child<Obj<'p>> {
        // SAFETY: handle non-null; returned pointer is owned by the tabview.
        let ptr = unsafe { lv_tabview_get_content(self.lv_handle()) };
        assert!(!ptr.is_null(), "lv_tabview_get_content returned NULL");
        Child::new(Obj::from_raw(ptr))
    }

    /// Get a non-owning handle to the tab bar (contains the tab buttons).
    pub fn get_tab_bar(&self) -> Child<Obj<'p>> {
        // SAFETY: handle non-null; returned pointer is owned by the tabview.
        let ptr = unsafe { lv_tabview_get_tab_bar(self.lv_handle()) };
        assert!(!ptr.is_null(), "lv_tabview_get_tab_bar returned NULL");
        Child::new(Obj::from_raw(ptr))
    }

    /// Get the current tab bar position.
    pub fn get_tab_bar_position(&self) -> DdDir {
        // SAFETY: handle non-null (checked in new()).
        let raw = unsafe { lv_tabview_get_tab_bar_position(self.lv_handle()) };
        match raw {
            x if x == lv_dir_t_LV_DIR_TOP => DdDir::Top,
            x if x == lv_dir_t_LV_DIR_BOTTOM => DdDir::Bottom,
            x if x == lv_dir_t_LV_DIR_LEFT => DdDir::Left,
            _ => DdDir::Right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::dropdown::DdDir;

    #[test]
    fn dddir_values_for_tabview() {
        // Verify the DdDir values match lv_dir_t used by lv_tabview_set_tab_bar_position.
        assert_eq!(DdDir::Top as u32, oxivgl_sys::lv_dir_t_LV_DIR_TOP);
        assert_eq!(DdDir::Bottom as u32, oxivgl_sys::lv_dir_t_LV_DIR_BOTTOM);
        assert_eq!(DdDir::Left as u32, oxivgl_sys::lv_dir_t_LV_DIR_LEFT);
        assert_eq!(DdDir::Right as u32, oxivgl_sys::lv_dir_t_LV_DIR_RIGHT);
    }
}
