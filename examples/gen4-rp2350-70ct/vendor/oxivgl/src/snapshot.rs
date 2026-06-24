// SPDX-License-Identifier: MIT OR Apache-2.0
//! Screen capture API (host-only).

use oxivgl_sys::*;

use crate::driver::LvglDriver;

/// Captured screen snapshot. Owns the LVGL draw buffer.
///
/// `!Send + !Sync` by design — LVGL is single-task.
pub struct Snapshot {
    buf: *mut lv_draw_buf_t,
    width: u32,
    height: u32,
}

impl Snapshot {
    /// Capture any widget as ARGB8888.
    ///
    /// Returns `None` if the snapshot allocation fails.
    /// The caller must ensure LVGL has rendered the widget at least once
    /// (call after `lv_refr_now` or one render cycle).
    pub fn take_widget(obj: &impl crate::widgets::AsLvHandle) -> Option<Self> {
        // SAFETY: obj.lv_handle() is non-null (widget constructor guarantees).
        // lv_snapshot_take allocates a new draw buffer; returns NULL on failure.
        let buf = unsafe {
            lv_snapshot_take(obj.lv_handle(), lv_color_format_t_LV_COLOR_FORMAT_ARGB8888)
        };
        if buf.is_null() {
            return None;
        }
        // SAFETY: buf is non-null (checked above), header is valid after allocation.
        let header = unsafe { &(*buf).header };
        Some(Self { buf, width: header.w(), height: header.h() })
    }

    /// Raw draw buffer pointer. Used by [`Image::set_src_snapshot`](crate::widgets::Image::set_src_snapshot).
    pub(crate) fn draw_buf_ptr(&self) -> *mut lv_draw_buf_t {
        self.buf
    }

    /// Capture the active screen as RGB565.
    ///
    /// Requires `&LvglDriver` to prove LVGL is initialised.
    /// Returns `None` if the snapshot allocation fails.
    pub fn take(_driver: &LvglDriver) -> Option<Self> {
        let buf = unsafe { lv_snapshot_take(lv_screen_active(), lv_color_format_t_LV_COLOR_FORMAT_RGB565) };
        if buf.is_null() {
            return None;
        }
        let header = unsafe { &(*buf).header };
        Some(Self { buf, width: header.w(), height: header.h() })
    }

    /// Snapshot width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Snapshot height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Raw RGB565 pixel data. Row stride may include alignment padding.
    pub fn data(&self) -> &[u8] {
        let buf = unsafe { &*self.buf };
        unsafe { std::slice::from_raw_parts(buf.data, buf.data_size as usize) }
    }

    /// Row stride in bytes (may differ from `width * 2` due to alignment).
    #[cfg(feature = "png")]
    fn stride(&self) -> usize {
        let buf = unsafe { &*self.buf };
        buf.header.stride() as usize
    }

    /// Write snapshot as PNG. Converts RGB565 to RGB8.
    ///
    /// Requires the `png` feature.
    #[cfg(feature = "png")]
    pub fn write_png(&self, path: &std::path::Path) -> std::io::Result<()> {
        let w = self.width as usize;
        let h = self.height as usize;
        let stride = self.stride();
        let data = self.data();

        assert!(data.len() >= (h - 1) * stride + w * 2, "draw buffer too small for dimensions");
        let mut rgb = Vec::with_capacity(w * h * 3);
        for row in 0..h {
            for col in 0..w {
                let off = row * stride + col * 2;
                let p = u16::from_le_bytes([data[off], data[off + 1]]);
                let r = ((p >> 11) & 0x1F) as u8;
                let g = ((p >> 5) & 0x3F) as u8;
                let b = (p & 0x1F) as u8;
                rgb.push((r << 3) | (r >> 2));
                rgb.push((g << 2) | (g >> 4));
                rgb.push((b << 3) | (b >> 2));
            }
        }

        let file = std::fs::File::create(path)?;
        let buf = std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(buf, self.width, self.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| std::io::Error::other(e))?;
        writer.write_image_data(&rgb).map_err(|e| std::io::Error::other(e))?;
        Ok(())
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe { lv_draw_buf_destroy(self.buf) };
    }
}
