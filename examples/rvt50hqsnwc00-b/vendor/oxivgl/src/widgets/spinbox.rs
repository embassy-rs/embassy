// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL Spinbox widget — integer input with digit-by-digit editing.
/// Built on top of [`Textarea`](super::Textarea); wraps [`Obj`](super::obj::Obj)
/// and `Deref`s to it for style/positioning methods.
///
/// Requires `LV_USE_SPINBOX = 1` and `LV_USE_TEXTAREA = 1` in `lv_conf.h`.
#[derive(Debug)]
pub struct Spinbox<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Spinbox<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Spinbox<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Spinbox<'p> {
    /// Create a spinbox as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    ///
    /// Defaults: value 0, range −99999..99999, 5 digits, step 1.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_spinbox_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Spinbox { obj: Obj::from_raw(handle) }) }
    }

    /// Set the current value (clamped to the configured range).
    pub fn set_value(&self, value: i32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_set_value(self.obj.handle(), value) };
        self
    }

    /// Get the current integer value.
    pub fn get_value(&self) -> i32 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_get_value(self.obj.handle()) }
    }

    /// Set the value range (inclusive).
    pub fn set_range(&self, min: i32, max: i32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_set_range(self.obj.handle(), min, max) };
        self
    }

    /// Set digit format: total `digit_count` (max 10) and decimal separator
    /// position. E.g. `set_digit_format(5, 2)` displays value 1234 as "012.34".
    pub fn set_digit_format(&self, digit_count: u32, separator_pos: u32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_set_digit_format(self.obj.handle(), digit_count, separator_pos) };
        self
    }

    /// Set the increment/decrement step size.
    pub fn set_step(&self, step: u32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_set_step(self.obj.handle(), step) };
        self
    }

    /// Get the current step size.
    pub fn get_step(&self) -> i32 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_get_step(self.obj.handle()) }
    }

    /// Enable or disable rollover (wrap from max→min or min→max).
    pub fn set_rollover(&self, enabled: bool) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_set_rollover(self.obj.handle(), enabled) };
        self
    }

    /// Increment the value by one step.
    pub fn increment(&self) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_increment(self.obj.handle()) };
        self
    }

    /// Decrement the value by one step.
    pub fn decrement(&self) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_decrement(self.obj.handle()) };
        self
    }

    /// Move cursor to the next lower digit (step ÷ 10).
    pub fn step_next(&self) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_step_next(self.obj.handle()) };
        self
    }

    /// Move cursor to the next higher digit (step × 10).
    pub fn step_prev(&self) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_step_prev(self.obj.handle()) };
        self
    }

    /// Set the cursor position (digit index).
    pub fn set_cursor_pos(&self, pos: u32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spinbox_set_cursor_pos(self.obj.handle(), pos) };
        self
    }

    /// Get whether rollover (wrap-around) is enabled.
    pub fn get_rollover(&self) -> bool {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_spinbox_get_rollover(self.lv_handle()) }
    }

    /// Get the total digit count.
    pub fn get_digit_count(&self) -> u32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_spinbox_get_digit_count(self.lv_handle()) }
    }

    /// Get the decimal point position.
    pub fn get_dec_point_pos(&self) -> u32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_spinbox_get_dec_point_pos(self.lv_handle()) }
    }

    /// Get the minimum allowed value.
    pub fn get_min_value(&self) -> i32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_spinbox_get_min_value(self.lv_handle()) }
    }

    /// Get the maximum allowed value.
    pub fn get_max_value(&self) -> i32 {
        // SAFETY: handle non-null (checked in new()).
        unsafe { lv_spinbox_get_max_value(self.lv_handle()) }
    }
}
