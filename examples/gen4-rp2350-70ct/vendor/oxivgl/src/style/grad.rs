// SPDX-License-Identifier: MIT OR Apache-2.0
use oxivgl_sys::*;

/// Gradient extend mode (`lv_grad_extend_t`).
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum GradExtend {
    /// Extend by padding with the edge color.
    Pad = lv_grad_extend_t_LV_GRAD_EXTEND_PAD,
    /// Repeat the gradient pattern.
    Repeat = lv_grad_extend_t_LV_GRAD_EXTEND_REPEAT,
    /// Reflect (mirror) the gradient pattern.
    Reflect = lv_grad_extend_t_LV_GRAD_EXTEND_REFLECT,
}

/// Safe wrapper around `lv_grad_dsc_t`.
///
/// Build with setter methods, then move into
/// [`StyleBuilder::bg_grad`](super::StyleBuilder::bg_grad).
/// The style takes ownership — no external lifetime management needed.
pub struct GradDsc {
    pub(crate) inner: lv_grad_dsc_t,
}

impl core::fmt::Debug for GradDsc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GradDsc").finish_non_exhaustive()
    }
}

impl GradDsc {
    /// Create a zeroed gradient descriptor.
    pub fn new() -> Self {
        Self { inner: unsafe { core::mem::zeroed::<lv_grad_dsc_t>() } }
    }

    /// Initialize gradient color stops.
    /// `colors`: array of stop colors.
    /// `opas`: per-stop opacity (or empty for all-255).
    /// `fracs`: per-stop position 0–255 (or empty for even spacing).
    pub fn init_stops(&mut self, colors: &[lv_color_t], opas: &[u8], fracs: &[u8]) -> &mut Self {
        let count = colors.len() as i32;
        let opas_ptr = if opas.is_empty() { core::ptr::null() } else { opas.as_ptr() as *const lv_opa_t };
        let fracs_ptr = if fracs.is_empty() { core::ptr::null() } else { fracs.as_ptr() };
        unsafe { lv_grad_init_stops(&mut self.inner, colors.as_ptr(), opas_ptr, fracs_ptr, count) };
        self
    }

    /// Set a simple two-stop vertical/horizontal gradient manually.
    pub fn set_dir(&mut self, dir: super::GradDir) -> &mut Self {
        self.inner.set_dir(dir as u32);
        self
    }

    /// Set the number of stops (for manual stop configuration).
    pub fn set_stops_count(&mut self, count: u8) -> &mut Self {
        self.inner.stops_count = count;
        self
    }

    /// Configure an individual stop by index.
    pub fn set_stop(&mut self, idx: usize, color: lv_color_t, opa: u8, frac: u8) -> &mut Self {
        self.inner.stops[idx].color = color;
        self.inner.stops[idx].opa = opa;
        self.inner.stops[idx].frac = frac;
        self
    }

    /// Configure as a linear gradient.
    pub fn linear(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, extend: GradExtend) -> &mut Self {
        unsafe { lv_grad_linear_init(&mut self.inner, x1, y1, x2, y2, extend as u32) };
        self
    }

    /// Configure as a radial gradient.
    pub fn radial(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, extend: GradExtend) -> &mut Self {
        unsafe { lv_grad_radial_init(&mut self.inner, cx, cy, rx, ry, extend as u32) };
        self
    }

    /// Configure as a conical gradient.
    pub fn conical(&mut self, cx: i32, cy: i32, start_angle: i32, end_angle: i32, extend: GradExtend) -> &mut Self {
        unsafe { lv_grad_conical_init(&mut self.inner, cx, cy, start_angle, end_angle, extend as u32) };
        self
    }

    /// Configure as a simple horizontal gradient (left-to-right).
    pub fn horizontal(&mut self) -> &mut Self {
        unsafe { lv_grad_horizontal_init(&mut self.inner) };
        self
    }

    /// Set the focal point of a radial gradient.
    ///
    /// Call after [`radial`](Self::radial). `fx`/`fy` are the focal center
    /// coords; `fr` is the focal circle radius.
    pub fn radial_set_focal(&mut self, fx: i32, fy: i32, fr: i32) -> &mut Self {
        unsafe { lv_grad_radial_set_focal(&mut self.inner, fx, fy, fr) };
        self
    }
}
