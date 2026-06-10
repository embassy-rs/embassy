// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    child::Child,
    obj::{AsLvHandle, Obj},
    WidgetError,
};
use crate::enums::ScrollDir;

/// LVGL tileview widget — a scrollable grid of full-screen tile panes.
///
/// Requires `LV_USE_TILEVIEW = 1` in `lv_conf.h`.
///
/// Each tile is a scrollable `Obj` owned by the tileview. Obtain panes via
/// [`add_tile`](Self::add_tile) and populate them with child widgets. Tiles
/// are arranged in a column/row grid and the user can swipe between them in
/// the allowed directions.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Screen, Tileview};
/// use oxivgl::enums::ScrollDir;
///
/// let screen = Screen::active().unwrap();
/// let tv = Tileview::new(&screen).unwrap();
/// let tile1 = tv.add_tile(0, 0, ScrollDir::BOTTOM);
/// let tile2 = tv.add_tile(0, 1, ScrollDir::TOP | ScrollDir::RIGHT);
/// ```
#[derive(Debug)]
pub struct Tileview<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Tileview<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Tileview<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Tileview<'p> {
    /// Create a tileview as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted); lv_init() called via LvglDriver.
        let handle = unsafe { lv_tileview_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Tileview { obj: Obj::from_raw(handle) })
        }
    }

    /// Add a tile at grid position (`col`, `row`) with the given allowed scroll
    /// directions. Returns a non-owning handle to the tile content pane, which
    /// is owned by the tileview.
    ///
    /// `dir` specifies which directions the user may swipe from this tile.
    /// Combine directions with `|`, e.g. `ScrollDir::TOP | ScrollDir::RIGHT`.
    pub fn add_tile(&self, col: u8, row: u8, dir: ScrollDir) -> Child<Obj<'p>> {
        // SAFETY: handle non-null (constructor guarantees); returned pointer is
        // a child obj owned by the tileview.
        let ptr =
            unsafe { lv_tileview_add_tile(self.lv_handle(), col, row, dir.0 as lv_dir_t) };
        assert!(!ptr.is_null(), "lv_tileview_add_tile returned NULL");
        Child::new(Obj::from_raw(ptr))
    }

    /// Set the active tile by its object handle. `anim` enables slide animation.
    pub fn set_tile(&self, tile: &impl AsLvHandle, anim: bool) -> &Self {
        // SAFETY: both handles non-null (constructor/add_tile guarantee).
        unsafe { lv_tileview_set_tile(self.lv_handle(), tile.lv_handle(), anim) };
        self
    }

    /// Set the active tile by grid index. `anim` enables slide animation.
    pub fn set_tile_by_index(&self, col: u32, row: u32, anim: bool) -> &Self {
        // SAFETY: handle non-null; out-of-range indices are handled by LVGL.
        unsafe { lv_tileview_set_tile_by_index(self.lv_handle(), col, row, anim) };
        self
    }

    /// Get a non-owning handle to the currently active tile, or `None` if no
    /// tile has been set as active yet.
    pub fn get_tile_active(&self) -> Option<Child<Obj<'p>>> {
        // SAFETY: handle non-null; returned pointer (if non-null) is owned by
        // the tileview.
        let ptr = unsafe { lv_tileview_get_tile_active(self.lv_handle()) };
        if ptr.is_null() {
            None
        } else {
            Some(Child::new(Obj::from_raw(ptr)))
        }
    }
}
