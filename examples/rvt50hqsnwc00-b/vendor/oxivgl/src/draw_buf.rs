// SPDX-License-Identifier: GPL-3.0-only
//! Owned LVGL draw buffer — wraps `lv_draw_buf_t`.

use core::marker::PhantomData;
use oxivgl_sys::*;

/// LVGL pixel color format.
///
/// Passed to [`DrawBuf::create`] to specify the pixel layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorFormat(pub u32);

impl ColorFormat {
    /// 16-bit RGB (5-6-5), no alpha channel.
    pub const RGB565: Self = Self(lv_color_format_t_LV_COLOR_FORMAT_RGB565 as u32);
    /// 32-bit ARGB (8-8-8-8), with full alpha.
    pub const ARGB8888: Self = Self(lv_color_format_t_LV_COLOR_FORMAT_ARGB8888 as u32);
    /// 8-bit luminance (grayscale). Used for bitmap masks.
    pub const L8: Self = Self(lv_color_format_t_LV_COLOR_FORMAT_L8 as u32);
}

/// Owned LVGL draw buffer. Allocated by LVGL on [`create`](DrawBuf::create) and freed on `Drop`.
///
/// Pass to [`Canvas::new`](crate::widgets::Canvas::new) — `Canvas` takes ownership
/// and ensures the buffer outlives the LVGL canvas object.
pub struct DrawBuf {
    ptr: *mut lv_draw_buf_t,
}

impl DrawBuf {
    /// Allocate a draw buffer of the given dimensions and color format.
    ///
    /// Returns `None` if LVGL allocation fails (OOM).
    pub fn create(w: u32, h: u32, cf: ColorFormat) -> Option<Self> {
        // SAFETY: lv_draw_buf_create allocates and zero-initialises the buffer;
        // returns null on allocation failure. We check before storing.
        let ptr = unsafe { lv_draw_buf_create(w, h, cf.0, 0) };
        if ptr.is_null() { None } else { Some(Self { ptr }) }
    }

    /// Raw LVGL pointer. Valid for the lifetime of this `DrawBuf`.
    pub(crate) fn as_ptr(&self) -> *mut lv_draw_buf_t {
        self.ptr
    }

    /// Buffer width in pixels.
    pub fn width(&self) -> i32 {
        // SAFETY: ptr is a valid lv_draw_buf_t allocated by lv_draw_buf_create.
        unsafe { (*self.ptr).header.w() as i32 }
    }

    /// Buffer height in pixels.
    pub fn height(&self) -> i32 {
        // SAFETY: ptr is a valid lv_draw_buf_t allocated by lv_draw_buf_create.
        unsafe { (*self.ptr).header.h() as i32 }
    }

    /// Obtain an image descriptor view into this buffer's pixel data.
    ///
    /// The returned [`ImageDsc`] borrows from `self` — the borrow checker
    /// prevents the `DrawBuf` from being dropped while the descriptor is in use.
    /// Pass to [`DrawImageDsc::from_image_dsc`](crate::draw::DrawImageDsc::from_image_dsc).
    pub fn image_dsc(&self) -> ImageDsc<'_> {
        // SAFETY: ptr is a valid lv_draw_buf_t; lv_draw_buf_to_image fills the
        // image descriptor with a pointer into the buffer's pixel data.
        let mut inner = unsafe { core::mem::zeroed::<lv_image_dsc_t>() };
        unsafe { lv_draw_buf_to_image(self.ptr, &mut inner) };
        ImageDsc { inner, _buf: PhantomData }
    }
}

/// Image descriptor view into a [`DrawBuf`]'s pixel data.
///
/// Obtained via [`DrawBuf::image_dsc`]. The `'buf` lifetime ensures the
/// backing pixel buffer remains valid for the duration of this descriptor.
/// Pass to [`DrawImageDsc::from_image_dsc`](crate::draw::DrawImageDsc::from_image_dsc).
pub struct ImageDsc<'buf> {
    pub(crate) inner: lv_image_dsc_t,
    _buf: PhantomData<&'buf DrawBuf>,
}

impl Drop for DrawBuf {
    fn drop(&mut self) {
        // SAFETY: ptr was allocated by lv_draw_buf_create and has not been freed.
        // Canvas registers an LV_EVENT_DELETE callback that calls Box::from_raw
        // on the heap-allocated DrawBuf, triggering this Drop exactly once after
        // lv_obj_delete has completed (LVGL no longer holds the pointer).
        unsafe { lv_draw_buf_destroy(self.ptr) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_format_rgb565_value() {
        // Sanity-check that the constant matches the raw binding value.
        assert_eq!(
            ColorFormat::RGB565.0,
            lv_color_format_t_LV_COLOR_FORMAT_RGB565 as u32
        );
    }

    #[test]
    fn color_format_argb8888_value() {
        assert_eq!(
            ColorFormat::ARGB8888.0,
            lv_color_format_t_LV_COLOR_FORMAT_ARGB8888 as u32
        );
    }
}
