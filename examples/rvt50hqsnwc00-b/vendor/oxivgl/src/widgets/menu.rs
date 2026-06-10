// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    child::Child,
    obj::{AsLvHandle, Obj},
};

/// LVGL menu widget — a navigation container with pages, sub-pages, headers,
/// and optional sidebar.
///
/// Requires `LV_USE_MENU = 1` in `lv_conf.h`.
///
/// Pages created via [`page_create`](Self::page_create) are owned by the menu.
/// Containers, sections, and separators created via the associated functions
/// are owned by their parent page/section.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Label, Menu, Screen};
///
/// let screen = Screen::active().unwrap();
/// let menu = Menu::new(&screen).unwrap();
/// let page = menu.page_create(None);
/// let cont = Menu::cont_create(&page);
/// let _lbl = Label::new(&cont).unwrap();
/// menu.set_page(&page);
/// ```
#[derive(Debug)]
pub struct Menu<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Menu<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Menu<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

/// Menu header position mode.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MenuHeaderMode {
    /// Header fixed at top.
    TopFixed = lv_menu_mode_header_t_LV_MENU_HEADER_TOP_FIXED,
    /// Header at top, scrolls out of view.
    TopUnfixed = lv_menu_mode_header_t_LV_MENU_HEADER_TOP_UNFIXED,
    /// Header fixed at bottom.
    BottomFixed = lv_menu_mode_header_t_LV_MENU_HEADER_BOTTOM_FIXED,
}

impl<'p> Menu<'p> {
    /// Create a menu as a child of `parent`.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_menu_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Menu { obj: Obj::from_raw(handle) })
        }
    }

    /// Create a menu page. `title` is shown in the header when navigating to
    /// this page. Pass `None` for untitled pages.
    ///
    /// LVGL copies the title string internally. The returned page is owned by
    /// the menu — wrap in [`Child`](super::Child) if you need to keep a handle.
    pub fn page_create(&self, title: Option<&str>) -> Child<Obj<'p>> {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // `_buf` must remain live until after `lv_menu_page_create` returns —
        // `ptr` points into it. Do NOT rename `_buf` to `_` (that would drop
        // it immediately, making `ptr` a dangling pointer).
        let (ptr, _buf) = match title {
            Some(t) => {
                let bytes = t.as_bytes();
                let len = bytes.len().min(127);
                let mut buf = [0u8; 128];
                buf[..len].copy_from_slice(&bytes[..len]);
                (buf.as_ptr() as *const c_char, Some(buf))
            }
            None => (core::ptr::null(), None),
        };
        // SAFETY: menu handle non-null (asserted above); ptr is NULL or points
        // to a NUL-terminated stack buffer live until end of this block.
        // LVGL copies the title (lv_menu.c creates a label child with the text).
        let page = unsafe { lv_menu_page_create(self.obj.handle(), ptr) };
        assert!(!page.is_null(), "lv_menu_page_create returned NULL");
        Child::new(Obj::from_raw(page))
    }

    /// Create a menu container on `parent` (typically a page or section).
    ///
    /// The container is owned by its parent — do not drop separately.
    pub fn cont_create(parent: &impl AsLvHandle) -> Child<Obj<'_>> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent cannot be null");
        // SAFETY: parent_ptr non-null (asserted above).
        let cont = unsafe { lv_menu_cont_create(parent_ptr) };
        assert!(!cont.is_null(), "lv_menu_cont_create returned NULL");
        Child::new(Obj::from_raw(cont))
    }

    /// Create a menu section on `parent` (typically a page).
    ///
    /// Sections group items with visual separation.
    pub fn section_create(parent: &impl AsLvHandle) -> Child<Obj<'_>> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent cannot be null");
        // SAFETY: parent_ptr non-null (asserted above).
        let sec = unsafe { lv_menu_section_create(parent_ptr) };
        assert!(!sec.is_null(), "lv_menu_section_create returned NULL");
        Child::new(Obj::from_raw(sec))
    }

    /// Create a menu separator on `parent` (typically a page).
    pub fn separator_create(parent: &impl AsLvHandle) -> Child<Obj<'_>> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent cannot be null");
        // SAFETY: parent_ptr non-null (asserted above).
        let sep = unsafe { lv_menu_separator_create(parent_ptr) };
        assert!(!sep.is_null(), "lv_menu_separator_create returned NULL");
        Child::new(Obj::from_raw(sep))
    }

    /// Set the page to display in the main area.
    /// Pass a NULL-handled page (via `None` pattern) to clear history.
    pub fn set_page(&self, page: &impl AsLvHandle) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: both handles valid. LVGL navigates to the page.
        unsafe { lv_menu_set_page(self.obj.handle(), page.lv_handle()) };
        self
    }

    /// Clear the main page (and menu history).
    pub fn clear_page(&self) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: handle non-null. Passing NULL clears main and history.
        unsafe { lv_menu_set_page(self.obj.handle(), null_mut()) };
        self
    }

    /// Set a page to display in the sidebar.
    pub fn set_sidebar_page(&self, page: &impl AsLvHandle) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: both handles valid.
        unsafe { lv_menu_set_sidebar_page(self.obj.handle(), page.lv_handle()) };
        self
    }

    /// Clear the sidebar.
    pub fn clear_sidebar(&self) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: handle non-null. Passing NULL clears sidebar.
        unsafe { lv_menu_set_sidebar_page(self.obj.handle(), null_mut()) };
        self
    }

    /// Configure a container to navigate to `page` when clicked.
    pub fn set_load_page_event(&self, cont: &impl AsLvHandle, page: &impl AsLvHandle) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: all handles non-null. LVGL registers a CLICKED event on cont
        // that navigates to page.
        unsafe { lv_menu_set_load_page_event(self.obj.handle(), cont.lv_handle(), page.lv_handle()) };
        self
    }

    /// Enable or disable the root back button.
    pub fn set_mode_root_back_button(&self, enabled: bool) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        let mode = if enabled {
            lv_menu_mode_root_back_button_t_LV_MENU_ROOT_BACK_BUTTON_ENABLED
        } else {
            lv_menu_mode_root_back_button_t_LV_MENU_ROOT_BACK_BUTTON_DISABLED
        };
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_menu_set_mode_root_back_button(self.obj.handle(), mode) };
        self
    }

    /// Set the header position mode.
    pub fn set_mode_header(&self, mode: MenuHeaderMode) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_menu_set_mode_header(self.obj.handle(), mode as lv_menu_mode_header_t) };
        self
    }

    /// Check if `obj` is the root back button of this menu.
    pub fn back_button_is_root(&self, obj: &impl AsLvHandle) -> bool {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: both handles non-null.
        unsafe { lv_menu_back_button_is_root(self.obj.handle(), obj.lv_handle()) }
    }

    /// Get the main header container.
    pub fn get_main_header(&self) -> Child<Obj<'_>> {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: handle non-null. LVGL returns the internal header obj.
        let ptr = unsafe { lv_menu_get_main_header(self.obj.handle()) };
        assert!(!ptr.is_null(), "lv_menu_get_main_header returned NULL");
        Child::new(Obj::from_raw(ptr))
    }

    /// Get the main header back button.
    pub fn get_main_header_back_button(&self) -> Child<Obj<'_>> {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: handle non-null.
        let ptr = unsafe { lv_menu_get_main_header_back_button(self.obj.handle()) };
        assert!(!ptr.is_null(), "lv_menu_get_main_header_back_button returned NULL");
        Child::new(Obj::from_raw(ptr))
    }

    /// Get the currently displayed sidebar page.
    pub fn get_cur_sidebar_page(&self) -> Option<Child<Obj<'_>>> {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: handle non-null.
        let ptr = unsafe { lv_menu_get_cur_sidebar_page(self.obj.handle()) };
        if ptr.is_null() { None } else { Some(Child::new(Obj::from_raw(ptr))) }
    }

    /// Get the currently displayed main page.
    pub fn get_cur_main_page(&self) -> Option<Child<Obj<'_>>> {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: handle non-null.
        let ptr = unsafe { lv_menu_get_cur_main_page(self.obj.handle()) };
        if ptr.is_null() { None } else { Some(Child::new(Obj::from_raw(ptr))) }
    }

    /// Clear menu navigation history.
    pub fn clear_history(&self) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Menu handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_menu_clear_history(self.obj.handle()) };
        self
    }

    /// Get the current header position mode.
    pub fn get_mode_header(&self) -> MenuHeaderMode {
        // SAFETY: handle non-null (checked in new()).
        let raw = unsafe { lv_menu_get_mode_header(self.lv_handle()) };
        match raw {
            x if x == lv_menu_mode_header_t_LV_MENU_HEADER_TOP_UNFIXED => MenuHeaderMode::TopUnfixed,
            x if x == lv_menu_mode_header_t_LV_MENU_HEADER_BOTTOM_FIXED => MenuHeaderMode::BottomFixed,
            _ => MenuHeaderMode::TopFixed,
        }
    }

    /// Get the sidebar header container.
    pub fn get_sidebar_header(&self) -> Child<Obj<'_>> {
        // SAFETY: handle non-null (checked in new()).
        let ptr = unsafe { lv_menu_get_sidebar_header(self.lv_handle()) };
        assert!(!ptr.is_null(), "lv_menu_get_sidebar_header returned NULL");
        Child::new(Obj::from_raw(ptr))
    }
}
