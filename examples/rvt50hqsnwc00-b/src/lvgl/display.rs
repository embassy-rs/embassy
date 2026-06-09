//! RGB565 LTDC framebuffer display driver for LVGL.
//!
//! Flush logic matches [Riverdi's LVGL port](https://github.com/riverdi/riverdi-50-stm32u5-lvgl)
//! (`disp_flush_cb` in `lvgl-port/port.c`): partial draw buffer, `lv_color.full` → framebuffer.

use lvgl::{init, Color, Display, DisplayRefresh, DrawBuffer, LvResult};
use lvgl_sys;

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
        let pixel = color_to_rgb565(*color);
        unsafe {
            fb.add(idx).write(pixel);
        }
    }
}

#[inline]
fn color_to_rgb565(color: Color) -> u16 {
    let raw: lvgl_sys::lv_color_t = color.into();
    unsafe { raw.full }
}
