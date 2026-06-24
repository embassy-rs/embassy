// SPDX-License-Identifier: MIT OR Apache-2.0
//! Draw task and borrowed draw descriptor types.
//!
//! All types in this module are **callback-scoped**: valid only during the
//! `DRAW_TASK_ADDED` event callback. Storing them beyond that scope is
//! undefined behaviour (LVGL frees the draw task after the callback returns).
//!
//! See `docs/spec-memory-lifetime.md` §2 for the lifetime table.

use oxivgl_sys::*;

use crate::widgets::Part;

use super::area::Area;
use super::layer::Layer;

/// Non-owning handle to an LVGL draw task.
///
/// Valid only during a `DRAW_TASK_ADDED` event callback — LVGL owns the
/// task and frees it after the callback returns.
pub struct DrawTask {
    ptr: *mut lv_draw_task_t,
}

impl core::fmt::Debug for DrawTask {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DrawTask").finish_non_exhaustive()
    }
}

impl DrawTask {
    /// Create from a raw pointer (called by `Event::draw_task()`).
    pub(crate) fn from_raw(ptr: *mut lv_draw_task_t) -> Self {
        Self { ptr }
    }

    /// Base draw descriptor (part, id1, id2). Returns a value copy.
    pub fn base(&self) -> DrawDscBase {
        // SAFETY: ptr valid during callback; lv_draw_task_get_draw_dsc returns
        // a pointer to the descriptor embedded in the task (lv_draw.c).
        let dsc = unsafe { lv_draw_task_get_draw_dsc(self.ptr) };
        let base = unsafe { &*(dsc as *const lv_draw_dsc_base_t) };
        DrawDscBase { part: Part::from_raw(base.part), id1: base.id1, id2: base.id2 }
    }

    /// Label draw descriptor, if this task draws a label.
    ///
    /// Returns `None` if the task is not a label draw operation.
    pub fn label_dsc(&self) -> Option<DrawLabelDsc> {
        // SAFETY: ptr valid during callback; returns null if not a label task.
        let dsc = unsafe { lv_draw_task_get_label_dsc(self.ptr) };
        if dsc.is_null() { None } else { Some(DrawLabelDsc { ptr: dsc }) }
    }

    /// Current draw area (copy).
    pub fn area(&self) -> Area {
        // SAFETY: ptr valid during callback.
        let task = unsafe { &*self.ptr };
        task.area.into()
    }

    /// Overwrite the draw area.
    pub fn set_area(&self, area: Area) {
        // SAFETY: ptr valid during callback; area is a plain value field.
        unsafe { (*self.ptr).area = area.into() };
    }

    /// Fill draw descriptor, if this task draws a filled rectangle.
    ///
    /// Returns `None` if the task is not a fill draw operation.
    pub fn fill_dsc(&self) -> Option<DrawFillDsc> {
        // SAFETY: ptr valid during callback; returns null if not a fill task.
        let dsc = unsafe { lv_draw_task_get_fill_dsc(self.ptr) };
        if dsc.is_null() { None } else { Some(DrawFillDsc { ptr: dsc }) }
    }

    /// Access the fill draw descriptor via a closure.
    ///
    /// The closure receives a reference to the descriptor, valid only
    /// for the duration of the call. Returns `None` if this task is not a
    /// fill operation, or `Some(R)` with the closure's return value.
    pub fn with_fill_dsc<R>(&self, f: impl FnOnce(&DrawFillDsc) -> R) -> Option<R> {
        self.fill_dsc().map(|dsc| f(&dsc))
    }

    /// Access the label draw descriptor via a closure.
    ///
    /// The closure receives a reference to the descriptor, valid only
    /// for the duration of the call. Returns `None` if this task is not a
    /// label operation, or `Some(R)` with the closure's return value.
    pub fn with_label_dsc<R>(&self, f: impl FnOnce(&DrawLabelDsc) -> R) -> Option<R> {
        self.label_dsc().map(|dsc| f(&dsc))
    }

    /// Box-shadow draw descriptor, if this task draws a box shadow.
    pub fn box_shadow_dsc(&self) -> Option<DrawBoxShadowDsc> {
        // SAFETY: ptr valid during callback; returns null if not a shadow task.
        let dsc = unsafe { lv_draw_task_get_box_shadow_dsc(self.ptr) };
        if dsc.is_null() { None } else { Some(DrawBoxShadowDsc { ptr: dsc }) }
    }

    /// Access the box-shadow draw descriptor via a closure.
    pub fn with_box_shadow_dsc<R>(&self, f: impl FnOnce(&DrawBoxShadowDsc) -> R) -> Option<R> {
        self.box_shadow_dsc().map(|dsc| f(&dsc))
    }

    /// Raw draw task type discriminant (`lv_draw_task_type_t` cast to `u32`).
    ///
    /// Use the `LV_DRAW_TASK_TYPE_*` constants from `oxivgl_sys` to
    /// distinguish fill, label, image, etc. tasks.
    pub fn task_type(&self) -> u32 {
        // SAFETY: ptr valid during callback.
        unsafe { lv_draw_task_get_type(self.ptr) as u32 }
    }

    /// Get the draw layer from this task's base descriptor.
    ///
    /// Required for `DRAW_TASK_ADDED` handlers that call [`Layer::draw_rect`] /
    /// [`Layer::draw_label`]. Valid only during the callback.
    pub fn layer(&self) -> Option<Layer> {
        // SAFETY: ptr valid during callback; lv_draw_task_get_draw_dsc returns
        // a pointer to the embedded base descriptor (lv_draw.c).
        let dsc = unsafe { lv_draw_task_get_draw_dsc(self.ptr) };
        if dsc.is_null() {
            return None;
        }
        // SAFETY: dsc is a valid lv_draw_dsc_base_t * (first field in every draw dsc).
        let base = unsafe { &*(dsc as *const lv_draw_dsc_base_t) };
        if base.layer.is_null() { None } else { Some(Layer::from_raw(base.layer)) }
    }
}

/// Base draw descriptor fields (value copy — no pointer risk).
#[derive(Clone, Copy, Debug)]
pub struct DrawDscBase {
    /// Which widget part this draw task belongs to.
    pub part: Part,
    /// First identifier (e.g. tick index for scales).
    pub id1: u32,
    /// Second identifier (e.g. tick value for scales).
    pub id2: u32,
}

/// Mutable handle to a label draw descriptor.
///
/// Valid only during the `DRAW_TASK_ADDED` callback. Modifications take
/// effect on the current draw operation.
pub struct DrawLabelDsc {
    ptr: *mut lv_draw_label_dsc_t,
}

impl core::fmt::Debug for DrawLabelDsc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DrawLabelDsc").finish_non_exhaustive()
    }
}

impl DrawLabelDsc {
    /// Current label color.
    pub fn color(&self) -> lv_color_t {
        // SAFETY: ptr valid during callback.
        unsafe { (*self.ptr).color }
    }

    /// Set the label color.
    pub fn set_color(&self, color: lv_color_t) {
        // SAFETY: ptr valid during callback; color is a plain value field.
        unsafe { (*self.ptr).color = color };
    }

    /// Current label text, or `None` if the pointer is null.
    ///
    /// Returns `None` for non-UTF-8 text (e.g. raw binary font glyphs).
    pub fn text(&self) -> Option<&str> {
        // SAFETY: ptr valid during callback; text pointer valid for callback duration.
        let text_ptr = unsafe { (*self.ptr).text };
        if text_ptr.is_null() {
            return None;
        }
        // SAFETY: LVGL label text is valid UTF-8 (ASCII subset).
        let cstr = unsafe { core::ffi::CStr::from_ptr(text_ptr) };
        cstr.to_str().ok()
    }

    /// Replace the label text with an LVGL-allocated copy.
    ///
    /// Handles `lv_free`/`lv_strdup`/`text_local` internally. The previous
    /// text is freed if it was locally allocated. LVGL will free the new
    /// text after the draw operation completes.
    ///
    /// **Note:** Text is truncated to 31 bytes (plus NUL terminator).
    pub fn set_text(&self, text: &str) {
        // SAFETY: ptr valid during callback.
        let dsc = unsafe { &mut *self.ptr };

        // Free previous text if locally allocated.
        if dsc.text_local() != 0 {
            unsafe { lv_free(dsc.text as *mut core::ffi::c_void) };
        }

        // Build null-terminated buffer on stack, then lv_strdup.
        let mut buf = [0u8; 32];
        let len = text.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        buf[len] = 0;

        // SAFETY: lv_strdup allocates via LVGL's allocator; LVGL frees it
        // when text_local is set.
        dsc.text = unsafe { lv_strdup(buf.as_ptr() as *const core::ffi::c_char) };
        dsc.set_text_local(1);
    }

    /// Set the text alignment (`lv_text_align_t`).
    pub fn set_align(&self, align: crate::widgets::TextAlign) {
        // SAFETY: ptr valid during callback; align is a plain integer field.
        unsafe { (*self.ptr).align = align as lv_text_align_t };
    }

    /// Current opacity (0 = transparent, 255 = opaque).
    pub fn opa(&self) -> u8 {
        // SAFETY: ptr valid during callback.
        unsafe { (*self.ptr).opa }
    }

    /// Set the label opacity. Use 0 to hide the label text.
    pub fn set_opa(&self, opa: u8) {
        // SAFETY: ptr valid during callback; opa is a plain integer field.
        unsafe { (*self.ptr).opa = opa };
    }

    /// Current font pointer.
    pub fn font(&self) -> *const lv_font_t {
        // SAFETY: ptr valid during callback.
        unsafe { (*self.ptr).font }
    }

    /// Measure the pixel size of a text string using this descriptor's font.
    ///
    /// Returns `(width, height)`. Uses letter_space=0, line_space=0,
    /// max_width=1000, no flags — matching the LVGL example pattern.
    pub fn text_size(&self, text: &str) -> (i32, i32) {
        let mut buf = [0u8; 32];
        let len = text.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        buf[len] = 0;
        let mut size: lv_point_t = unsafe { core::mem::zeroed() };
        // SAFETY: font pointer valid during callback; buf is null-terminated.
        unsafe {
            lv_text_get_size(
                &mut size,
                buf.as_ptr() as *const core::ffi::c_char,
                (*self.ptr).font,
                0,    // letter_space
                0,    // line_space
                1000, // max_width
                lv_text_flag_t_LV_TEXT_FLAG_NONE,
            );
        }
        (size.x, size.y)
    }
}

/// Mutable handle to a fill draw descriptor.
///
/// Valid only during the `DRAW_TASK_ADDED` callback. Modifications take
/// effect on the current draw operation.
pub struct DrawFillDsc {
    ptr: *mut lv_draw_fill_dsc_t,
}

impl core::fmt::Debug for DrawFillDsc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DrawFillDsc").finish_non_exhaustive()
    }
}

impl DrawFillDsc {
    /// Current fill color.
    pub fn color(&self) -> lv_color_t {
        // SAFETY: ptr valid during callback.
        unsafe { (*self.ptr).color }
    }

    /// Set the fill color.
    pub fn set_color(&self, color: lv_color_t) {
        // SAFETY: ptr valid during callback; color is a plain value field.
        unsafe { (*self.ptr).color = color };
    }

    /// Current opacity (0 = transparent, 255 = opaque).
    pub fn opa(&self) -> u8 {
        // SAFETY: ptr valid during callback.
        unsafe { (*self.ptr).opa }
    }

    /// Set the fill opacity.
    pub fn set_opa(&self, opa: u8) {
        // SAFETY: ptr valid during callback; opa is a plain integer field.
        unsafe { (*self.ptr).opa = opa };
    }

    /// Current corner radius.
    pub fn radius(&self) -> i32 {
        // SAFETY: ptr valid during callback.
        unsafe { (*self.ptr).radius }
    }

    /// Set the corner radius.
    pub fn set_radius(&self, radius: i32) {
        // SAFETY: ptr valid during callback; radius is a plain value field.
        unsafe { (*self.ptr).radius = radius };
    }
}

/// Mutable handle to a box shadow draw descriptor.
///
/// Valid only during the `DRAW_TASK_ADDED` callback. Modifications take
/// effect on the current draw operation.
pub struct DrawBoxShadowDsc {
    ptr: *mut lv_draw_box_shadow_dsc_t,
}

impl core::fmt::Debug for DrawBoxShadowDsc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DrawBoxShadowDsc").finish_non_exhaustive()
    }
}

impl DrawBoxShadowDsc {
    /// Set the shadow width (spread distance in pixels).
    pub fn set_width(&self, width: i32) {
        // SAFETY: ptr valid during callback; width is a plain value field.
        unsafe { (*self.ptr).width = width };
    }

    /// Set the shadow X offset.
    pub fn set_ofs_x(&self, x: i32) {
        // SAFETY: ptr valid during callback.
        unsafe { (*self.ptr).ofs_x = x };
    }

    /// Set the shadow Y offset.
    pub fn set_ofs_y(&self, y: i32) {
        // SAFETY: ptr valid during callback.
        unsafe { (*self.ptr).ofs_y = y };
    }

    /// Set the shadow corner radius.
    pub fn set_radius(&self, radius: i32) {
        // SAFETY: ptr valid during callback.
        unsafe { (*self.ptr).radius = radius };
    }
}
