//! RGB565 LTDC framebuffer display driver for LVGL.
//!
//! Mirrors the flush logic of the C
//! [Riverdi LVGL port](https://github.com/riverdi/riverdi-50-stm32u5-lvgl)
//! (`disp_flush_cb` in `lvgl-port/port.c`) using the safe
//! [`Display::register`](lvgl::Display::register) API from
//! [lv_binding_rust](https://github.com/lvgl/lv_binding_rust): a partial draw
//! buffer is rendered into and copied into the panel framebuffer one rectangle
//! at a time.

use lvgl::{Color, Display, DisplayRefresh, DrawBuffer, LvResult, init};

use crate::touch_config;

/// Partial buffer height in scanlines (Riverdi uses `width * 10`).
const DRAW_LINES: usize = 10;
const DRAW_BUF_PIXELS: usize = touch_config::DISPLAY_WIDTH as usize * DRAW_LINES;

pub struct Rvt50Display {
    pub inner: Display,
}

impl Rvt50Display {
    /// Register the LTDC framebuffer with LVGL.
    pub fn register(framebuffer: *mut u16) -> LvResult<Self> {
        init();

        let width = touch_config::DISPLAY_WIDTH;
        let height = touch_config::DISPLAY_HEIGHT;
        let buffer = DrawBuffer::<DRAW_BUF_PIXELS>::default();

        let display = Display::register(buffer, width.into(), height.into(), move |refresh| {
            flush_rgb565(framebuffer, width, refresh);
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
        // SAFETY: `fb` points to a `[u16; DISPLAY_WIDTH * DISPLAY_HEIGHT]`
        // framebuffer owned by the LTDC task; LVGL's flush area is clipped to
        // those bounds so `idx < DISPLAY_WIDTH * DISPLAY_HEIGHT`.
        unsafe {
            fb.add(idx).write(rgb565(*color));
        }
    }
}

/// Pack an LVGL [`Color`] into the RGB565 word expected by the LTDC framebuffer.
///
/// Matches `lv_color16_t.full` (LV_COLOR_DEPTH=16, LV_COLOR_16_SWAP=0):
/// red is stored in the high 5 bits, green in the middle 6 bits, blue in the
/// low 5 bits. Avoids reading `lv_color_t` as a C union.
#[inline]
fn rgb565(color: Color) -> u16 {
    let r = u16::from(color.r()) & 0x1F;
    let g = u16::from(color.g()) & 0x3F;
    let b = u16::from(color.b()) & 0x1F;
    (r << 11) | (g << 5) | b
}
