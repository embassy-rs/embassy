// SPDX-License-Identifier: MIT OR Apache-2.0
//! Rectangle area type and related constants.

use oxivgl_sys::*;

/// Rectangle area (x1, y1, x2, y2) — value copy of `lv_area_t`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Area {
    /// Left edge.
    pub x1: i32,
    /// Top edge.
    pub y1: i32,
    /// Right edge.
    pub x2: i32,
    /// Bottom edge.
    pub y2: i32,
}

impl Area {
    /// Width of this area.
    pub fn width(&self) -> i32 {
        self.x2 - self.x1 + 1
    }

    /// Height of this area.
    pub fn height(&self) -> i32 {
        self.y2 - self.y1 + 1
    }

    /// Adjust width symmetrically around center. Distributes the delta
    /// equally on both sides (extra pixel goes to the right).
    pub fn set_width_centered(&mut self, new_w: i32) {
        let old_w = self.width();
        let delta = new_w - old_w;
        self.x1 -= delta / 2;
        self.x2 += (delta + 1) / 2;
    }

    /// Align this area relative to `base` by `align`, then offset by `(ofs_x,
    /// ofs_y)`.
    ///
    /// Equivalent to `lv_area_align(base, self, align, ofs_x, ofs_y)`.
    pub fn align_to_area(&mut self, base: Area, align: crate::widgets::Align, ofs_x: i32, ofs_y: i32) {
        let base_lv: lv_area_t = base.into();
        let mut self_lv: lv_area_t = (*self).into();
        // SAFETY: both areas are valid stack values; lv_area_align writes to self_lv.
        unsafe { lv_area_align(&base_lv, &mut self_lv, align as lv_align_t, ofs_x, ofs_y) };
        *self = self_lv.into();
    }

    /// Set width from the left edge (x2 = x1 + new_w - 1). x1 is unchanged.
    pub fn set_width(&mut self, new_w: i32) {
        self.x2 = self.x1 + new_w - 1;
    }
}

impl From<lv_area_t> for Area {
    fn from(a: lv_area_t) -> Self {
        Self { x1: a.x1, y1: a.y1, x2: a.x2, y2: a.y2 }
    }
}

impl From<Area> for lv_area_t {
    fn from(a: Area) -> Self {
        Self { x1: a.x1, y1: a.y1, x2: a.x2, y2: a.y2 }
    }
}

/// Maximum radius constant — produces a circle or fully-rounded corners.
/// Equivalent to `LV_RADIUS_CIRCLE` (0x7FFF).
pub const RADIUS_CIRCLE: i32 = 0x7FFF;
