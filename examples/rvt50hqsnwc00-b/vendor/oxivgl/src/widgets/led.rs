// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL LED widget. Wraps [`Obj`](super::obj::Obj) and `Deref`s to it for style
/// methods.
#[derive(Debug)]
pub struct Led<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Led<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Led<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Led<'p> {
    /// Create an LED widget as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_led_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Led { obj: Obj::from_raw(handle) }) }
    }

    /// Turn the LED on (full brightness).
    pub fn on(&self) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_led_on(self.obj.handle()) };
        self
    }

    /// Turn the LED off (lowest brightness).
    pub fn off(&self) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_led_off(self.obj.handle()) };
        self
    }

    /// Set LED brightness (0–255).
    pub fn set_brightness(&self, bright: u8) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_led_set_brightness(self.obj.handle(), bright) };
        self
    }

    /// Set the LED color.
    pub fn set_color(&self, color: lv_color_t) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_led_set_color(self.obj.handle(), color) };
        self
    }

    /// Get the LED brightness (0–255).
    pub fn get_brightness(&self) -> u8 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_led_get_brightness(self.obj.handle()) }
    }

    /// Get the LED color.
    pub fn get_color(&self) -> lv_color_t {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_led_get_color(self.obj.handle()) }
    }
}
