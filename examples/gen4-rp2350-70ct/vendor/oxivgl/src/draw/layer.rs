// SPDX-License-Identifier: MIT OR Apache-2.0
//! Layer handle and related constants.

use oxivgl_sys::*;

use super::area::Area;
use super::dsc::*;

/// Draw task type: fill rectangle.
/// Equivalent to `LV_DRAW_TASK_TYPE_FILL` (1).
pub const DRAW_TASK_TYPE_FILL: u32 = 1;

/// Non-owning handle to an LVGL draw layer.
///
/// Valid only during a draw event callback. Obtain via
/// [`Event::layer`](crate::event::Event::layer).
pub struct Layer {
    ptr: *mut lv_layer_t,
}

impl core::fmt::Debug for Layer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Layer").finish_non_exhaustive()
    }
}

impl Layer {
    /// Create from a raw LVGL layer pointer (used by `Event::layer()`).
    pub(crate) fn from_raw(ptr: *mut lv_layer_t) -> Self {
        Self { ptr }
    }

    /// Draw a filled rectangle onto this layer.
    ///
    /// The descriptor is passed by reference; `area` is a copy.
    pub fn draw_rect(&self, dsc: &DrawRectDsc, area: Area) {
        let area_lv: lv_area_t = area.into();
        // SAFETY: ptr valid during callback; dsc and area_lv are stack values.
        unsafe { lv_draw_rect(self.ptr, &dsc.inner, &area_lv) };
    }

    /// Draw an image onto this layer.
    ///
    /// `dsc` describes the image source and transform; `area` is the
    /// destination rectangle.
    pub fn draw_image(&self, dsc: &DrawImageDsc<'_>, area: Area) {
        let area_lv: lv_area_t = area.into();
        // SAFETY: ptr valid during callback; dsc and area are stack values.
        unsafe { lv_draw_image(self.ptr, dsc.as_ptr(), &area_lv) };
    }

    /// Draw a text label onto this layer.
    ///
    /// `text` must fit in 63 bytes; longer strings are truncated.
    /// The text pointer is valid only for the duration of this call.
    pub fn draw_label(&self, dsc: &DrawLabelDscOwned, area: Area, text: &str) {
        let mut buf = [0u8; 64];
        let len = text.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        buf[len] = 0;
        // lv_draw_label_dsc_t derives Copy — simple copy is safe.
        let mut local_dsc = dsc.inner;
        local_dsc.text = buf.as_ptr() as *const _;
        local_dsc.set_text_local(1);
        let area_lv: lv_area_t = area.into();
        // SAFETY: ptr valid during callback; text_local=1 means lv_draw_label calls
        // lv_strndup when queuing the task, so buf need only live until this call
        // returns.
        unsafe { lv_draw_label(self.ptr, &local_dsc, &area_lv) };
    }
}

/// Query the size of an image without decoding it.
///
/// Returns `Some((width, height))` on success, `None` if the image
/// cannot be decoded.
pub fn image_header_info(src: &lv_image_dsc_t) -> Option<(i32, i32)> {
    let w = src.header.w() as i32;
    let h = src.header.h() as i32;
    if w > 0 && h > 0 { Some((w, h)) } else { None }
}
