// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    subject::Subject,
    WidgetError,
};

/// LVGL slider widget (native range 0–100 by default).
///
/// Use [`on_event`](Obj::on_event) with `EventCode::VALUE_CHANGED`
/// to react to slider movement.
#[derive(Debug)]
pub struct Slider<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Slider<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Slider<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

/// Slider widget orientation.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliderOrientation {
    /// Automatic based on dimensions.
    Auto = lv_slider_orientation_t_LV_SLIDER_ORIENTATION_AUTO,
    /// Force horizontal.
    Horizontal = lv_slider_orientation_t_LV_SLIDER_ORIENTATION_HORIZONTAL,
    /// Force vertical.
    Vertical = lv_slider_orientation_t_LV_SLIDER_ORIENTATION_VERTICAL,
}

/// Slider operating mode.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum SliderMode {
    /// Normal slider (single value).
    Normal = lv_slider_mode_t_LV_SLIDER_MODE_NORMAL,
    /// Symmetrical from center.
    Symmetrical = lv_slider_mode_t_LV_SLIDER_MODE_SYMMETRICAL,
    /// Range slider (two handles, start + end).
    Range = lv_slider_mode_t_LV_SLIDER_MODE_RANGE,
}

impl<'p> Slider<'p> {
    /// Create a new slider widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_slider_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Slider {
                obj: Obj::from_raw(handle),
            })
        }
    }

    /// Returns the current slider value (native LVGL integer range).
    pub fn get_value(&self) -> i32 {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_slider_get_value(self.obj.handle()) }
    }

    /// Sets the slider range (min and max values).
    pub fn set_range(&self, min: i32, max: i32) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        unsafe { lv_slider_set_range(self.obj.handle(), min, max) };
        self
    }

    /// Sets the slider value (native LVGL integer range).
    pub fn set_value(&self, val: i32) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_slider_set_value(self.obj.handle(), val, false) };
        self
    }

    /// Sets the slider value with optional slide animation.
    pub fn set_value_animated(&self, val: i32, anim: bool) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_slider_set_value(self.obj.handle(), val, anim) };
        self
    }

    /// Set slider mode (normal, symmetrical, or range).
    pub fn set_mode(&self, mode: SliderMode) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_slider_set_mode(self.obj.handle(), mode as lv_slider_mode_t) };
        self
    }

    /// Set the start value (left handle in range mode).
    pub fn set_start_value(&self, val: i32) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_slider_set_start_value(self.obj.handle(), val, false) };
        self
    }

    /// Get the left/start value (range mode).
    pub fn get_left_value(&self) -> i32 {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_slider_get_left_value(self.obj.handle()) }
    }

    /// Get the minimum value of the slider range.
    pub fn get_min_value(&self) -> i32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_slider_get_min_value(self.lv_handle()) }
    }

    /// Get the maximum value of the slider range.
    pub fn get_max_value(&self) -> i32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_slider_get_max_value(self.lv_handle()) }
    }

    /// Get the current slider mode.
    pub fn get_mode(&self) -> SliderMode {
        // SAFETY: handle non-null (checked in new()).
        let raw = unsafe { lv_slider_get_mode(self.lv_handle()) };
        match raw {
            x if x == lv_slider_mode_t_LV_SLIDER_MODE_SYMMETRICAL => SliderMode::Symmetrical,
            x if x == lv_slider_mode_t_LV_SLIDER_MODE_RANGE => SliderMode::Range,
            _ => SliderMode::Normal,
        }
    }

    /// Set the slider orientation.
    pub fn set_orientation(&self, orientation: SliderOrientation) -> &Self {
        // SAFETY: lv_handle() is non-null (checked in new()).
        unsafe {
            lv_slider_set_orientation(self.lv_handle(), orientation as lv_slider_orientation_t)
        };
        self
    }

    /// Get the slider orientation.
    pub fn get_orientation(&self) -> SliderOrientation {
        // SAFETY: lv_handle() is non-null (checked in new()).
        let raw = unsafe { lv_slider_get_orientation(self.lv_handle()) };
        match raw {
            x if x == lv_slider_orientation_t_LV_SLIDER_ORIENTATION_HORIZONTAL => {
                SliderOrientation::Horizontal
            }
            x if x == lv_slider_orientation_t_LV_SLIDER_ORIENTATION_VERTICAL => {
                SliderOrientation::Vertical
            }
            _ => SliderOrientation::Auto,
        }
    }

    /// Return `true` if the slider is currently being dragged.
    pub fn is_dragged(&self) -> bool {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_slider_is_dragged(self.lv_handle()) }
    }

    /// Bind slider value to an integer subject (two-way).
    ///
    /// Slider changes update the subject and subject changes update the slider.
    pub fn bind_value(&self, subject: &Subject) -> &Self {
        // SAFETY: lv_handle() is non-null (checked in new()); subject pointer
        // is pinned and valid for the subject's lifetime.
        unsafe { lv_slider_bind_value(self.lv_handle(), subject.as_ptr()) };
        self
    }
}
