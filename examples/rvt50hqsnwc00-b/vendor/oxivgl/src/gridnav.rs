// SPDX-License-Identifier: MIT OR Apache-2.0
//! Grid navigation (`LV_USE_GRIDNAV`) — keyboard-driven focus inside containers.

use oxivgl_sys::*;

use crate::widgets::AsLvHandle;

/// Control flags for grid navigation behaviour.
///
/// Corresponds to `lv_gridnav_ctrl_t` in lvgl/src/indev/lv_gridnav.h.
/// Combine flags with `|` for compound behaviour.
///
/// ```
/// use oxivgl::gridnav::GridnavCtrl;
///
/// let ctrl = GridnavCtrl::ROLLOVER | GridnavCtrl::SCROLL_FIRST;
/// assert_eq!(ctrl.0, 0x3);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridnavCtrl(pub u32);

impl GridnavCtrl {
    /// No special navigation behaviour.
    pub const NONE: Self = Self(0x0);
    /// When the edge is reached, wrap to the next/previous row or to the
    /// first/last row on up/down.
    pub const ROLLOVER: Self = Self(0x1);
    /// Scroll the focused object before moving to the next one.
    pub const SCROLL_FIRST: Self = Self(0x2);
    /// Only use left/right keys; up/down are forwarded to the focused object.
    pub const HORIZONTAL_MOVE_ONLY: Self = Self(0x4);
    /// Only use up/down keys; left/right are forwarded to the focused object.
    pub const VERTICAL_MOVE_ONLY: Self = Self(0x8);
}

impl core::ops::BitOr for GridnavCtrl {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Add grid navigation to a container object.
///
/// After this call, arrow keys move focus between the direct children of `obj`.
/// The container itself must be added to a group to receive key events.
///
/// Only `obj` needs to be in the group — children should be removed with
/// [`crate::group::group_remove_obj`] so that group navigation does not
/// interfere with gridnav navigation.
///
/// See lvgl/src/indev/lv_gridnav.h — `lv_gridnav_add`.
pub fn gridnav_add(obj: &impl AsLvHandle, ctrl: GridnavCtrl) {
    // SAFETY: obj.lv_handle() is non-null (enforced by AsLvHandle contract).
    // lv_gridnav_add attaches an event handler to obj and stores ctrl in
    // user data; it does not retain obj beyond the call.
    // ctrl.0 is a valid lv_gridnav_ctrl_t bitmask (u32).
    // See lvgl/src/indev/lv_gridnav.c — lv_gridnav_add.
    unsafe { lv_gridnav_add(obj.lv_handle(), ctrl.0 as lv_gridnav_ctrl_t) };
}

/// Remove grid navigation from a container object.
///
/// See lvgl/src/indev/lv_gridnav.h — `lv_gridnav_remove`.
pub fn gridnav_remove(obj: &impl AsLvHandle) {
    // SAFETY: obj.lv_handle() is non-null. lv_gridnav_remove detaches the
    // event handler previously installed by lv_gridnav_add.
    // See lvgl/src/indev/lv_gridnav.c — lv_gridnav_remove.
    unsafe { lv_gridnav_remove(obj.lv_handle()) };
}
