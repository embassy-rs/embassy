// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL switch (toggle) widget.
#[derive(Debug)]
pub struct Switch<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Switch<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Switch<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

/// Switch orientation.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum SwitchOrientation {
    /// Auto-detect from widget dimensions.
    Auto = lv_switch_orientation_t_LV_SWITCH_ORIENTATION_AUTO,
    /// Horizontal switch.
    Horizontal = lv_switch_orientation_t_LV_SWITCH_ORIENTATION_HORIZONTAL,
    /// Vertical switch.
    Vertical = lv_switch_orientation_t_LV_SWITCH_ORIENTATION_VERTICAL,
}

impl<'p> Switch<'p> {
    /// Create a new switch (toggle) widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        let handle = unsafe { lv_switch_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Switch { obj: Obj::from_raw(handle) }) }
    }

    /// Set switch orientation.
    pub fn set_orientation(&self, orientation: SwitchOrientation) -> &Self {
        // SAFETY: handle non-null (from Switch::new).
        unsafe { lv_switch_set_orientation(self.lv_handle(), orientation as lv_switch_orientation_t) };
        self
    }

    /// Get the current switch orientation.
    pub fn get_orientation(&self) -> SwitchOrientation {
        // SAFETY: handle non-null (checked in new()).
        let raw = unsafe { lv_switch_get_orientation(self.lv_handle()) };
        match raw {
            x if x == lv_switch_orientation_t_LV_SWITCH_ORIENTATION_HORIZONTAL => SwitchOrientation::Horizontal,
            x if x == lv_switch_orientation_t_LV_SWITCH_ORIENTATION_VERTICAL => SwitchOrientation::Vertical,
            _ => SwitchOrientation::Auto,
        }
    }
}
