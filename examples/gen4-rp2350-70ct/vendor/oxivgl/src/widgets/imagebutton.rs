// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    WidgetError,
};

/// Imagebutton state — determines which set of images is displayed.
///
/// Each state can have its own left/mid/right image sources set via
/// [`Imagebutton::set_src`].
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImagebuttonState {
    /// Normal (released) state.
    Released = lv_imagebutton_state_t_LV_IMAGEBUTTON_STATE_RELEASED,
    /// Pressed state.
    Pressed = lv_imagebutton_state_t_LV_IMAGEBUTTON_STATE_PRESSED,
    /// Disabled state.
    Disabled = lv_imagebutton_state_t_LV_IMAGEBUTTON_STATE_DISABLED,
    /// Checked + released state.
    CheckedReleased = lv_imagebutton_state_t_LV_IMAGEBUTTON_STATE_CHECKED_RELEASED,
    /// Checked + pressed state.
    CheckedPressed = lv_imagebutton_state_t_LV_IMAGEBUTTON_STATE_CHECKED_PRESSED,
    /// Checked + disabled state.
    CheckedDisabled = lv_imagebutton_state_t_LV_IMAGEBUTTON_STATE_CHECKED_DISABLED,
}

/// LVGL image button widget — a button whose appearance is defined by
/// left/mid/right image sources for each state.
///
/// Wraps [`Obj`](super::obj::Obj) and `Deref`s to it for style/positioning
/// methods.
///
/// Requires `LV_USE_IMAGEBUTTON = 1` in `lv_conf.h`.
#[derive(Debug)]
pub struct Imagebutton<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Imagebutton<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Imagebutton<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Imagebutton<'p> {
    /// Create an image button as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_imagebutton_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Imagebutton {
                obj: Obj::from_raw(handle),
            })
        }
    }

    /// Set the left/mid/right image sources for a given state.
    ///
    /// LVGL stores the raw pointers (`lv_imagebutton_t.img_src_left/mid/right`),
    /// so the descriptors must be `'static` (spec §3.1). Use
    /// [`image_declare!`](crate::image_declare) to obtain safe `&'static
    /// lv_image_dsc_t` references.
    ///
    /// Any of the three sources may be `None` to leave that part blank.
    pub fn set_src(
        &self,
        state: ImagebuttonState,
        src_left: Option<&'static lv_image_dsc_t>,
        src_mid: Option<&'static lv_image_dsc_t>,
        src_right: Option<&'static lv_image_dsc_t>,
    ) -> &Self {
        let left = src_left
            .map(|d| d as *const lv_image_dsc_t as *const core::ffi::c_void)
            .unwrap_or(core::ptr::null());
        let mid = src_mid
            .map(|d| d as *const lv_image_dsc_t as *const core::ffi::c_void)
            .unwrap_or(core::ptr::null());
        let right = src_right
            .map(|d| d as *const lv_image_dsc_t as *const core::ffi::c_void)
            .unwrap_or(core::ptr::null());
        // SAFETY: handle non-null (from constructor); image descriptors are
        // 'static, satisfying LVGL's stored-pointer requirement (spec §3.1).
        // Null pointers are valid for unused slots.
        unsafe {
            lv_imagebutton_set_src(
                self.lv_handle(),
                state as lv_imagebutton_state_t,
                left,
                mid,
                right,
            )
        };
        self
    }

    /// Set the widget state manually.
    ///
    /// Use this instead of `lv_obj_add_state`/`lv_obj_remove_state` to
    /// switch between imagebutton-specific states.
    pub fn set_state(&self, state: ImagebuttonState) -> &Self {
        // SAFETY: handle non-null (from constructor).
        unsafe { lv_imagebutton_set_state(self.lv_handle(), state as lv_imagebutton_state_t) };
        self
    }
}
