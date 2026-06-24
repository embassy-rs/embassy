// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL line widget. Wraps [`Obj`](super::obj::Obj) and `Deref`s to it for
/// style methods.
#[derive(Debug)]
pub struct Line<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Line<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Line<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Line<'p> {
    /// Create a line widget as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_line_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Line { obj: Obj::from_raw(handle) }) }
    }

    /// Set the line points. LVGL stores the raw pointer — the slice must
    /// be `'static` (e.g. a `static` array or `Box::leak`ed allocation).
    pub fn set_points(&self, points: &'static [lv_point_precise_t]) -> &Self {
        // SAFETY: handle non-null (from Line::new); points is 'static so
        // the pointer LVGL stores will remain valid.
        unsafe { lv_line_set_points(self.lv_handle(), points.as_ptr(), points.len() as u32) };
        self
    }

    /// Get the number of points in the line.
    pub fn get_point_count(&self) -> u32 {
        // SAFETY: handle non-null (from Line::new).
        unsafe { lv_line_get_point_count(self.lv_handle()) }
    }

    /// Get whether the Y-axis is inverted (origin at bottom-left).
    pub fn get_y_invert(&self) -> bool {
        // SAFETY: handle non-null (from Line::new).
        unsafe { lv_line_get_y_invert(self.lv_handle()) }
    }
}
