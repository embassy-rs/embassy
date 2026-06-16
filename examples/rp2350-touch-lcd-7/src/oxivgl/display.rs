//! LVGL display flush into PSRAM framebuffers for the ST7262 / PIO RGB panel.

use core::ffi::c_void;
use core::ptr;
use core::slice;

use oxivgl::display::{LvglBuffers, DISPLAY_READY};
use oxivgl_sys::{
    lv_area_t, lv_display_create, lv_display_flush_ready, lv_display_set_buffers,
    lv_display_set_color_format, lv_display_set_default, lv_display_set_flush_cb,
    lv_display_t, lv_color_format_t_LV_COLOR_FORMAT_RGB565,
    lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
};

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::pio_rgb;

const FB_PIXELS: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
const FB_BYTES: usize = FB_PIXELS * 2;

static mut LVGL_DISP: *mut lv_display_t = core::ptr::null_mut();

#[derive(Clone, Copy, PartialEq, Eq)]
enum DrawBufferState {
    InitialBack,
    Ready,
    NeedsSync,
}

static mut DRAW_BUF: DrawBufferState = DrawBufferState::NeedsSync;

/// LVGL display token.
#[derive(Debug)]
pub struct PanelDisplay;

impl PanelDisplay {
    pub fn init<const BYTES: usize>(
        w: i32,
        h: i32,
        bufs: &'static mut LvglBuffers<BYTES>,
    ) -> Self {
        unsafe {
            lvgl_disp_init_panel(w, h, bufs);
        }
        Self
    }
}

pub(crate) fn lvgl_display() -> *mut lv_display_t {
    unsafe { LVGL_DISP }
}

fn rgb565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3)
}

pub fn prefill_background() {
    let px = rgb565(16, 32, 48).to_le_bytes();
    for fb in [pio_rgb::front_ptr(), pio_rgb::back_ptr()] {
        if fb.is_null() {
            continue;
        }
        unsafe {
            for i in 0..FB_PIXELS {
                ptr::copy_nonoverlapping(px.as_ptr(), fb.add(i).cast::<u8>(), 2);
            }
        }
    }
}

pub unsafe fn lvgl_disp_init_panel<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
) -> *mut lv_display_t {
    unsafe {
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
        disp
    }
}

pub fn sync_back_from_front() {
    let front = pio_rgb::front_ptr();
    let back = pio_rgb::back_ptr();
    if front.is_null() || back.is_null() {
        return;
    }
    unsafe {
        ptr::copy_nonoverlapping(front, back, FB_PIXELS);
    }
}

pub fn draw_buffer_after_lvgl_create() {
    unsafe {
        DRAW_BUF = DrawBufferState::InitialBack;
    }
}

pub fn prepare_back_for_draw() {
    unsafe {
        if DRAW_BUF == DrawBufferState::NeedsSync {
            sync_back_from_front();
            DRAW_BUF = DrawBufferState::Ready;
        }
    }
}

pub fn front_framebuffer() -> *const u16 {
    pio_rgb::front_ptr()
}

pub fn present_framebuffer() -> *const u16 {
    pio_rgb::request_swap();
    pio_rgb::front_ptr()
}

unsafe extern "C" fn flush_callback(
    disp: *mut lv_display_t,
    area_p: *const lv_area_t,
    px_map: *mut u8,
) {
    if disp.is_null() || area_p.is_null() || px_map.is_null() {
        return;
    }

    let area = unsafe { &*area_p };
    if area.x2 < area.x1 || area.y2 < area.y1 {
        return;
    }

    let w = (area.x2 - area.x1 + 1) as usize;
    let h = (area.y2 - area.y1 + 1) as usize;
    let src = unsafe { slice::from_raw_parts(px_map, w * h * 2) };

    pio_rgb::blit_rgb565(
        pio_rgb::back_ptr(),
        area.x1,
        area.y1,
        w,
        h,
        DISPLAY_WIDTH,
        src,
    );

    unsafe {
        lv_display_flush_ready(disp);
    }
}

/// Allocate two full-screen RGB565 buffers inside PSRAM and wire them to [`pio_rgb`].
pub fn init_psram_framebuffers(psram_base: *mut u8, psram_size: usize) -> Option<(*mut u16, *mut u16)> {
    let need = FB_BYTES * 2;
    if psram_size < need {
        return None;
    }
    let fb0 = psram_base.cast::<u16>();
    let fb1 = unsafe { psram_base.add(FB_BYTES).cast::<u16>() };
    pio_rgb::bind_framebuffers(fb0, fb1);
    Some((fb0, fb1))
}
