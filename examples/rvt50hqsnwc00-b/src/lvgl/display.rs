//! RGB565 LTDC framebuffer display driver for LVGL.
//!
//! Wraps [`lvgl::Display::register`] from the vendored
//! [`lv_binding_rust`](https://github.com/lvgl/lv_binding_rust) master under
//! `vendor/lv_binding_rust/`. The flush callback walks the LVGL partial draw
//! buffer once per refresh and copies pixels into the panel framebuffer in
//! the same layout LTDC scans out.

use lvgl::{Color, Display, DisplayRefresh, DrawBuffer, LvResult, init};

use crate::touch_config;

/// Partial buffer height in scanlines (Riverdi C port uses `width * 10`).
const DRAW_LINES: usize = 10;
const DRAW_BUF_PIXELS: usize = touch_config::DISPLAY_WIDTH as usize * DRAW_LINES;

pub struct Rvt50Display {
    pub inner: Display,
}

impl Rvt50Display {
    /// Register the LTDC framebuffer with LVGL.
    pub fn register(framebuffer: *mut u16) -> LvResult<Self> {
        init();

        let width = u32::from(touch_config::DISPLAY_WIDTH);
        let height = u32::from(touch_config::DISPLAY_HEIGHT);
        let buffer = DrawBuffer::<DRAW_BUF_PIXELS>::default();

        let display = Display::register(buffer, width, height, move |refresh| {
            flush_rgb565(framebuffer, touch_config::DISPLAY_WIDTH, refresh);
        })?;

        Ok(Self { inner: display })
    }
}

fn flush_rgb565(fb: *mut u16, fb_width: u16, refresh: &DisplayRefresh<DRAW_BUF_PIXELS>) {
    let area = &refresh.area;
    let line_width = (area.x2 - area.x1 + 1) as usize;
    let count = line_width * (area.y2 - area.y1 + 1) as usize;

    for (i, color) in refresh.colors.iter().enumerate().take(count) {
        let x = area.x1 as usize + i % line_width;
        let y = area.y1 as usize + i / line_width;
        let idx = y * fb_width as usize + x;
        // SAFETY: `fb` is a `[u16; DISPLAY_WIDTH * DISPLAY_HEIGHT]` framebuffer
        // owned by the LTDC task; LVGL's flush area stays inside those bounds.
        unsafe {
            fb.add(idx).write(rgb565(*color));
        }
    }
}

/// Pack an LVGL [`Color`] into the RGB565 word LTDC scans out.
///
/// Matches `lv_color16_t.full` (LV_COLOR_DEPTH=16, LV_COLOR_16_SWAP=0) without
/// reading the C union: red in the high 5 bits, green 6, blue 5.
#[inline]
fn rgb565(color: Color) -> u16 {
    let r = u16::from(color.r()) & 0x1F;
    let g = u16::from(color.g()) & 0x3F;
    let b = u16::from(color.b()) & 0x1F;
    (r << 11) | (g << 5) | b
}
