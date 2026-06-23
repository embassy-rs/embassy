//! LVGL display flush into the PSRAM framebuffer (partial render mode).
//!
//! Matches the gen4 PIO LVGL C port: LVGL draws into SRAM stripe buffers and
//! each flush copies the dirty region into the single PSRAM framebuffer that
//! PIO scan-out streams continuously.

use core::ffi::c_void;
use core::{ptr, slice};

use oxivgl::display::{DISPLAY_READY, LvglBuffers};
use oxivgl_sys::{
    lv_area_t, lv_color_format_t_LV_COLOR_FORMAT_RGB565, lv_display_create, lv_display_flush_ready,
    lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL, lv_display_set_buffers, lv_display_set_color_format,
    lv_display_set_default, lv_display_set_flush_cb, lv_display_t,
};

use crate::board::DISPLAY_WIDTH;
use crate::pio_rgb;

static mut LVGL_DISP: *mut lv_display_t = ptr::null_mut();
static mut FRAMEBUFFER: *mut u16 = ptr::null_mut();

/// PSRAM-backed LVGL display token.
#[derive(Debug)]
pub struct PsramDisplay;

/// LVGL stripe buffer size: 40 lines (fits in SRAM alongside PIO bounce buffers).
pub const COLOR_BUF_LINES: usize = 40;
pub const LVGL_BUF_BYTES: usize = DISPLAY_WIDTH * COLOR_BUF_LINES * 2;

impl PsramDisplay {
    pub fn init<const BYTES: usize>(
        w: i32,
        h: i32,
        bufs: &'static mut LvglBuffers<BYTES>,
        fb: *mut u16,
    ) -> Self {
        unsafe {
            FRAMEBUFFER = fb;
            let buf1_ptr = ptr::addr_of_mut!(bufs.buf1) as *mut c_void;
            let buf2_ptr = ptr::addr_of_mut!(bufs.buf2) as *mut c_void;

            let disp = lv_display_create(w, h);
            assert!(!disp.is_null(), "lv_display_create returned NULL");

            lv_display_set_color_format(disp, lv_color_format_t_LV_COLOR_FORMAT_RGB565);
            lv_display_set_buffers(
                disp,
                buf1_ptr,
                buf2_ptr,
                BYTES as u32,
                lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
            );
            lv_display_set_flush_cb(disp, Some(flush_callback));
            lv_display_set_default(disp);
            LVGL_DISP = disp;
            DISPLAY_READY.signal(());
        }
        Self
    }
}

pub(crate) fn lvgl_display() -> *mut lv_display_t {
    unsafe { LVGL_DISP }
}

unsafe extern "C" fn flush_callback(disp: *mut lv_display_t, area_p: *const lv_area_t, px_map: *mut u8) {
    if disp.is_null() || area_p.is_null() || px_map.is_null() {
        return;
    }

    let area = unsafe { &*area_p };
    if area.x2 < area.x1 || area.y2 < area.y1 {
        return;
    }

    let w = (area.x2 - area.x1 + 1) as usize;
    let h = (area.y2 - area.y1 + 1) as usize;
    let row_bytes = w * 2;
    let src = unsafe { slice::from_raw_parts(px_map, row_bytes * h) };

    unsafe {
        let fb = FRAMEBUFFER;
        if !fb.is_null() {
            pio_rgb::blit_rgb565(fb, area.x1, area.y1, w, h, src);
        }
        lv_display_flush_ready(disp);
    }
}

pub fn prefill_background(colour: u16) {
    unsafe {
        if !FRAMEBUFFER.is_null() {
            pio_rgb::fill_framebuffer(FRAMEBUFFER, colour);
        }
    }
}
