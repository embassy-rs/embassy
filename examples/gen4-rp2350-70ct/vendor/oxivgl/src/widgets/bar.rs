// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{cell::Cell, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    LVGL_SCALE, WidgetError,
    obj::{AsLvHandle, Obj},
    to_lvgl,
};

/// LVGL bar mode.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarMode {
    /// Standard bar (indicator from min to value).
    Normal = oxivgl_sys::lv_bar_mode_t_LV_BAR_MODE_NORMAL,
    /// Indicator draws from zero point towards value (needs range with negative
    /// min).
    Symmetrical = oxivgl_sys::lv_bar_mode_t_LV_BAR_MODE_SYMMETRICAL,
    /// Indicator between start value and end value.
    Range = oxivgl_sys::lv_bar_mode_t_LV_BAR_MODE_RANGE,
}

/// Bar widget orientation.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarOrientation {
    /// Automatic based on dimensions.
    Auto = lv_bar_orientation_t_LV_BAR_ORIENTATION_AUTO,
    /// Force horizontal.
    Horizontal = lv_bar_orientation_t_LV_BAR_ORIENTATION_HORIZONTAL,
    /// Force vertical.
    Vertical = lv_bar_orientation_t_LV_BAR_ORIENTATION_VERTICAL,
}

/// LVGL bar (progress bar) widget with normalized f32 value API.
///
/// Call [`set_range`](Bar::set_range) to set the physical maximum, then
/// [`set_value`](Bar::set_value) with values in the same unit. Min is always 0.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Bar, Screen};
///
/// let screen = Screen::active().unwrap();
/// let bar = Bar::new(&screen).unwrap();
/// bar.set_range(100.0);
/// bar.set_value(42.0); // 42 %
/// ```
#[derive(Debug)]
pub struct Bar<'p> {
    obj: Obj<'p>,
    max: Cell<f32>,
}

impl<'p> AsLvHandle for Bar<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Bar<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Bar<'p> {
    /// Create a new bar (progress bar) widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_bar_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Bar { obj: Obj::from_raw(handle), max: Cell::new(0.0) })
        }
    }

    /// Set range maximum (min = 0). Must be called before
    /// [`set_value`](Bar::set_value).
    pub fn set_range(&self, max: f32) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        self.max.set(max);
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_bar_set_range(self.obj.handle(), 0, LVGL_SCALE) };
        self
    }

    /// Set current value in physical units.
    pub fn set_value(&self, value: f32) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_bar_set_value(self.obj.handle(), to_lvgl(value, self.max.get()), false) };
        self
    }

    /// Set raw LVGL range (bypasses f32 normalization).
    pub fn set_range_raw(&self, min: i32, max: i32) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_bar_set_range(self.obj.handle(), min, max) };
        self
    }

    /// Set raw LVGL value with optional animation (bypasses f32 normalization).
    pub fn set_value_raw(&self, value: i32, anim: bool) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_bar_set_value(self.obj.handle(), value, anim) };
        self
    }

    /// Get raw LVGL value (bypasses f32 normalization).
    pub fn get_value_raw(&self) -> i32 {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_bar_get_value(self.obj.handle()) }
    }

    /// Set bar mode (normal, symmetrical, or range).
    pub fn set_mode(&self, mode: BarMode) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_bar_set_mode(self.obj.handle(), mode as lv_bar_mode_t) };
        self
    }

    /// Set raw LVGL start value with optional animation (range mode only).
    pub fn set_start_value_raw(&self, value: i32, anim: bool) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_bar_set_start_value(self.obj.handle(), value, anim) };
        self
    }

    /// Get raw LVGL start value (range mode only, bypasses f32 normalization).
    pub fn get_start_value_raw(&self) -> i32 {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_bar_get_start_value(self.obj.handle()) }
    }

    /// Get the raw LVGL minimum value.
    pub fn get_min_value(&self) -> i32 {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_bar_get_min_value(self.obj.handle()) }
    }

    /// Get the raw LVGL maximum value.
    pub fn get_max_value(&self) -> i32 {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_bar_get_max_value(self.obj.handle()) }
    }

    /// Get the bar mode (normal, symmetrical, or range).
    pub fn get_mode(&self) -> BarMode {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        // lv_bar_mode_t values 0–2 are all covered by BarMode.
        unsafe { core::mem::transmute(lv_bar_get_mode(self.obj.handle())) }
    }

    /// Set the bar orientation.
    pub fn set_orientation(&self, orientation: BarOrientation) -> &Self {
        // SAFETY: lv_handle() is non-null (checked in new()).
        unsafe { lv_bar_set_orientation(self.lv_handle(), orientation as lv_bar_orientation_t) };
        self
    }

    /// Get the bar orientation.
    pub fn get_orientation(&self) -> BarOrientation {
        // SAFETY: lv_handle() is non-null (checked in new()).
        let raw = unsafe { lv_bar_get_orientation(self.lv_handle()) };
        match raw {
            x if x == lv_bar_orientation_t_LV_BAR_ORIENTATION_HORIZONTAL => BarOrientation::Horizontal,
            x if x == lv_bar_orientation_t_LV_BAR_ORIENTATION_VERTICAL => BarOrientation::Vertical,
            _ => BarOrientation::Auto,
        }
    }

    /// Get current value in physical units.
    pub fn get_value(&self) -> f32 {
        assert_ne!(self.obj.handle(), null_mut(), "Bar handle cannot be null");
        let max = self.max.get();
        if max == 0.0 {
            return 0.0;
        }
        // SAFETY: handle non-null (asserted above).
        let raw = unsafe { lv_bar_get_value(self.obj.handle()) };
        (raw as f32 / LVGL_SCALE as f32) * max
    }
}
