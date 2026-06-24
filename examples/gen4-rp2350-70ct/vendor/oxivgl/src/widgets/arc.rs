// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{cell::Cell, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    obj::{Align, AsLvHandle, Obj},
    subject::Subject,
    to_lvgl, WidgetError, LVGL_SCALE,
};

/// Arc operating mode.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ArcMode {
    /// Normal: indicator fills from start_angle to value.
    Normal = lv_arc_mode_t_LV_ARC_MODE_NORMAL,
    /// Symmetrical: indicator expands from mid-point.
    Symmetrical = lv_arc_mode_t_LV_ARC_MODE_SYMMETRICAL,
    /// Reverse: indicator fills counter-clockwise.
    Reverse = lv_arc_mode_t_LV_ARC_MODE_REVERSE,
}

/// LVGL arc widget. Value range is normalized: call
/// [`set_range`](Arc::set_range) with a physical maximum, then
/// [`set_value`](Arc::set_value) with the physical value in the same unit.
///
/// For a ready-made gauge ring see [`Arc::gauge_ring`].
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Arc, Screen};
///
/// let screen = Screen::active().unwrap();
/// let arc = Arc::new(&screen).unwrap();
/// arc.set_range(150.0); // 0–150 A
/// arc.set_value(75.0);  // 50 %
/// ```
#[derive(Debug)]
pub struct Arc<'p> {
    obj: Obj<'p>,
    max: Cell<f32>,
}

impl<'p> AsLvHandle for Arc<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Arc<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Arc<'p> {
    /// Create a new arc widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_arc_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Arc {
                obj: Obj::from_raw(handle),
                max: Cell::new(0.0),
            })
        }
    }

    /// Set range maximum. Min is always 0. Must be called before `set_value`.
    pub fn set_range(&self, max: f32) -> &Self {
        self.max.set(max);
        // SAFETY: handle non-null (from Arc::new/gauge_ring, both null-check).
        unsafe { lv_arc_set_range(self.obj.handle(), 0, LVGL_SCALE) };
        self
    }

    /// Set current value in physical units (mapped via `max` set by
    /// [`set_range`](Arc::set_range)).
    pub fn set_value(&self, v: f32) -> &Self {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_set_value(self.obj.handle(), to_lvgl(v, self.max.get())) };
        self
    }

    /// Get current value in physical units.
    pub fn get_value(&self) -> f32 {
        let max = self.max.get();
        if max == 0.0 {
            return 0.0;
        }
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        let raw = unsafe { lv_arc_get_value(self.obj.handle()) };
        (raw as f32 / LVGL_SCALE as f32) * max
    }

    /// Set the arc start angle rotation in degrees.
    pub fn set_rotation(&self, rotation: i32) -> &Self {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_set_rotation(self.obj.handle(), rotation) };
        self
    }

    /// Set background arc start and end angles (degrees).
    pub fn set_bg_angles(&self, start: i32, end: i32) -> &Self {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_set_bg_angles(self.obj.handle(), start as f32, end as f32) };
        self
    }

    /// Set raw LVGL range (bypasses f32 normalization).
    pub fn set_range_raw(&self, min: i32, max: i32) -> &Self {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_set_range(self.obj.handle(), min, max) };
        self
    }

    /// Set raw LVGL value (bypasses f32 normalization).
    pub fn set_value_raw(&self, v: i32) -> &Self {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_set_value(self.obj.handle(), v) };
        self
    }

    /// Get raw LVGL value (bypasses f32 normalization).
    pub fn get_value_raw(&self) -> i32 {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_get_value(self.obj.handle()) }
    }

    /// Get the indicator arc start angle in degrees.
    pub fn get_angle_start(&self) -> f32 {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_get_angle_start(self.obj.handle()) }
    }

    /// Get the indicator arc end angle in degrees.
    pub fn get_angle_end(&self) -> f32 {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_get_angle_end(self.obj.handle()) }
    }

    /// Get the background arc start angle in degrees.
    pub fn get_bg_angle_start(&self) -> f32 {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_get_bg_angle_start(self.obj.handle()) }
    }

    /// Get the background arc end angle in degrees.
    pub fn get_bg_angle_end(&self) -> f32 {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_get_bg_angle_end(self.obj.handle()) }
    }

    /// Get the arc start angle rotation in degrees.
    pub fn get_rotation(&self) -> i32 {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_get_rotation(self.obj.handle()) }
    }

    /// Get the arc mode (normal, symmetrical, or reverse).
    pub fn get_mode(&self) -> ArcMode {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        // lv_arc_mode_t values 0–2 are all covered by ArcMode.
        unsafe { core::mem::transmute(lv_arc_get_mode(self.obj.handle())) }
    }

    /// Get the knob drag rate in degrees per second.
    pub fn get_change_rate(&self) -> u32 {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_get_change_rate(self.obj.handle()) }
    }

    /// Align a child object to the arc's current indicator end angle.
    ///
    /// Positions `obj` on the arc's edge at `r_offset` pixels from the
    /// indicator midline. No rotation transform — works with partial rendering.
    pub fn align_obj_to_angle(&self, obj: &impl AsLvHandle, r_offset: i32) -> &Self {
        // SAFETY: both handles non-null.
        unsafe { lv_arc_align_obj_to_angle(self.obj.handle(), obj.lv_handle(), r_offset) };
        self
    }

    /// Rotate a child object to follow the arc's current angle.
    ///
    /// Positions `obj` on the arc's edge and applies `transform_rotation` so
    /// the object visually follows the arc curvature.
    ///
    /// Requires `LV_DRAW_SW_SUPPORT_ARGB8888` and `LV_DRAW_SW_SUPPORT_RGB565A8`
    /// enabled in `lv_conf.h`.
    pub fn rotate_obj_to_angle(&self, obj: &impl AsLvHandle, r_offset: i32) -> &Self {
        // SAFETY: both handles non-null.
        unsafe { lv_arc_rotate_obj_to_angle(self.obj.handle(), obj.lv_handle(), r_offset) };
        self
    }

    /// Set the arc mode (normal, symmetrical, or reverse).
    pub fn set_mode(&self, mode: ArcMode) -> &Self {
        unsafe { lv_arc_set_mode(self.lv_handle(), mode as lv_arc_mode_t) };
        self
    }

    /// Set the background arc start angle in degrees (0° = right, 90° =
    /// bottom).
    pub fn set_bg_start_angle(&self, start: i32) -> &Self {
        unsafe { lv_arc_set_bg_start_angle(self.lv_handle(), start as f32) };
        self
    }

    /// Set the background arc end angle in degrees.
    pub fn set_bg_end_angle(&self, end: i32) -> &Self {
        unsafe { lv_arc_set_bg_end_angle(self.lv_handle(), end as f32) };
        self
    }

    /// Bind arc value to an integer subject (two-way).
    ///
    /// Arc changes update the subject and subject changes update the arc.
    pub fn bind_value(&self, subject: &Subject) -> &Self {
        // SAFETY: lv_handle() is non-null (checked in new()); subject pointer
        // is pinned and valid for the subject's lifetime.
        unsafe { lv_arc_bind_value(self.lv_handle(), subject.as_ptr()) };
        self
    }

    /// Create an arc pre-configured as a display-only gauge ring (no knob, not
    /// clickable).
    ///
    /// - `size`: diameter in px (arc is centered in parent).
    /// - `arc_width`: ring thickness in px.
    /// - `range_max`: physical maximum (e.g. `150.0` for 0–150 A).
    /// - `track_color` / `indicator_color`: RGB hex colors.
    /// - `rotation`: start angle in degrees (0 = 3 o'clock, CW; 150 ≈ 7
    ///   o'clock).
    /// - `sweep`: arc extent in degrees (e.g. `200` for a 200° arc).
    pub fn gauge_ring(
        parent: &impl AsLvHandle,
        size: i32,
        arc_width: i32,
        range_max: f32,
        track_color: u32,
        indicator_color: u32,
        rotation: i32,
        sweep: i32,
    ) -> Result<Self, WidgetError> {
        let arc = Arc::new(parent)?;
        arc.max.set(range_max);
        let h = arc.obj.handle();
        // SAFETY: h non-null (Arc::new null-checks); all LVGL style/arc fns safe with
        // valid ptr.
        unsafe {
            lv_obj_set_size(h, size, size);
            lv_obj_align(h, Align::Center as lv_align_t, 0, 0);
            lv_arc_set_rotation(h, rotation);
            lv_arc_set_bg_angles(h, 0.0, sweep as f32);
            lv_arc_set_range(h, 0, LVGL_SCALE);
            lv_arc_set_value(h, 0);
            // Background track
            lv_obj_set_style_arc_width(h, arc_width, lv_part_t_LV_PART_MAIN as u32);
            lv_obj_set_style_arc_color(h, lv_color_hex(track_color), lv_part_t_LV_PART_MAIN as u32);
            // Indicator
            lv_obj_set_style_arc_width(h, arc_width, lv_part_t_LV_PART_INDICATOR as u32);
            lv_obj_set_style_arc_color(
                h,
                lv_color_hex(indicator_color),
                lv_part_t_LV_PART_INDICATOR as u32,
            );
            // Hide knob
            lv_obj_set_style_pad_all(h, 0, lv_part_t_LV_PART_KNOB as u32);
            lv_obj_set_style_opa(
                h,
                crate::enums::Opa::TRANSP.0 as lv_opa_t,
                lv_part_t_LV_PART_KNOB as u32,
            );
            // Not interactive
            lv_obj_remove_flag(h, crate::enums::ObjFlag::CLICKABLE.0);
        }
        Ok(arc)
    }
}
