// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL math utility wrappers.

use oxivgl_sys::*;

/// Cubic Bezier calculation with 4 control points.
///
/// `t` is the time parameter in `[0..1024]`.
/// `u0` must be 0, `u3` must be 1024 (fixed endpoints).
/// `u1` and `u2` are the control values in `[0..1024]`.
/// Returns a value in `[0..1024]`.
pub fn bezier3(t: i32, u0: i32, u1: u32, u2: i32, u3: i32) -> i32 {
    // SAFETY: lv_bezier3 is a pure math function with no side effects.
    unsafe { lv_bezier3(t, u0, u1, u2, u3) }
}

/// Linear mapping from input range to output range.
///
/// Maps `x` from `[min_in..max_in]` to `[min_out..max_out]`.
pub fn map(x: i32, min_in: i32, max_in: i32, min_out: i32, max_out: i32) -> i32 {
    // SAFETY: lv_map is a pure math function with no side effects.
    unsafe { lv_map(x, min_in, max_in, min_out, max_out) }
}

/// Maximum value for Bezier control points and time parameter (1024).
pub const BEZIER_VAL_MAX: i32 = 1024;

/// LVGL fixed-point cosine. Angle in degrees (0–359).
/// Returns a value in `[-32767..32767]` (`LV_TRIGO_SIN_MAX`).
pub fn trigo_cos(angle: i32) -> i32 {
    // SAFETY: lv_trigo_cos is a pure math function.
    unsafe { lv_trigo_cos(angle as i16) }
}

/// LVGL fixed-point sine. Angle in degrees (0–359).
/// Returns a value in `[-32767..32767]` (`LV_TRIGO_SIN_MAX`).
pub fn trigo_sin(angle: i32) -> i32 {
    // SAFETY: lv_trigo_sin is a pure math function.
    unsafe { lv_trigo_sin(angle as i16) }
}

/// Bit shift for LVGL trigonometry results (`LV_TRIGO_SHIFT = 15`).
pub const TRIGO_SHIFT: i32 = LV_TRIGO_SHIFT as i32;

/// Integer arctangent. Returns angle in tenths of a degree (0–3599).
///
/// Note: LVGL binding argument order is `(x, y)` (not the conventional `(y,
/// x)`).
pub fn atan2(y: i32, x: i32) -> u16 {
    // SAFETY: pure computation, no side effects.
    // Binding signature: lv_atan2(x, y) — pass x first, y second.
    unsafe { lv_atan2(x, y) }
}
