// SPDX-License-Identifier: MIT OR Apache-2.0
//! Input device queries — non-owning wrappers for LVGL indev functions.

use oxivgl_sys::*;

/// 2D point (mirrors `lv_point_t`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    /// Horizontal coordinate.
    pub x: i32,
    /// Vertical coordinate.
    pub y: i32,
}

/// Non-owning handle to an LVGL input device.
///
/// LVGL owns the indev lifecycle — this wrapper only provides read access.
/// Obtain via [`Indev::active()`] inside an event handler.
pub struct Indev {
    ptr: *mut lv_indev_t,
}

impl core::fmt::Debug for Indev {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Indev").finish_non_exhaustive()
    }
}

impl Indev {
    /// Currently active input device (valid only inside an event handler).
    ///
    /// Returns `None` when no indev is being processed.
    pub fn active() -> Option<Self> {
        let ptr = unsafe { lv_indev_active() };
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    /// Pointer movement vector since last read.
    pub fn get_vect(&self) -> Point {
        let mut pt: lv_point_t = unsafe { core::mem::zeroed() };
        unsafe { lv_indev_get_vect(self.ptr, &mut pt) };
        Point { x: pt.x, y: pt.y }
    }

    /// Consecutive short-click count.
    ///
    /// Updated before `SHORT_CLICKED` fires. Resets after timeout or
    /// movement beyond the short-click distance threshold.
    pub fn short_click_streak(&self) -> u8 {
        unsafe { lv_indev_get_short_click_streak(self.ptr) }
    }
}
