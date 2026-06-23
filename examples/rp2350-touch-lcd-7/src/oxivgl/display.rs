//! LVGL display flush for the ST7262 / PIO RGB panel (Waveshare LVGL C port).

use core::ffi::c_void;

use oxivgl::display::DISPLAY_READY;
use oxivgl_sys::{
    lv_area_t, lv_display_create, lv_display_set_buffers, lv_display_set_color_format,
    lv_display_set_default, lv_display_set_flush_cb, lv_display_t,
    lv_color_format_t_LV_COLOR_FORMAT_RGB565,
    lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_FULL,
};

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::pio_rgb;

const FB_PIXELS: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
const FB_BYTES: usize = FB_PIXELS * 2;
const TRANSFER_BYTES: usize = DISPLAY_WIDTH * 80 * 2;

static mut LVGL_DISP: *mut lv_display_t = core::ptr::null_mut();

/// PSRAM layout for panel framebuffers and DMA staging (see Waveshare `lv_port_disp.c`).
#[derive(Clone, Copy)]
pub struct PanelMemory {
    pub fb0: *mut u16,
    pub fb1: *mut u16,
    pub transfer0: *mut u16,
    pub transfer1: *mut u16,
}

/// LVGL display token.
#[derive(Debug)]
pub struct PanelDisplay;

impl PanelDisplay {
    /// Register LVGL with full-screen PSRAM buffers and wire the PIO RGB flush callback.
    pub fn init(w: i32, h: i32, mem: &PanelMemory) -> Self {
        unsafe {
            lvgl_disp_init_panel(w, h, mem);
        }
        Self
    }
}

pub(crate) fn lvgl_display() -> *mut lv_display_t {
    unsafe { LVGL_DISP }
}
pub fn prefill_background() {
    let px = 0xFFFFu16;
    for fb in [pio_rgb::front_ptr(), pio_rgb::draw_ptr()] {
        if fb.is_null() {
            continue;
        }
        unsafe {
            for i in 0..FB_PIXELS {
                *fb.add(i) = px;
            }
        }
    }
}

unsafe fn lvgl_disp_init_panel(w: i32, h: i32, mem: &PanelMemory) -> *mut lv_display_t {
    unsafe {
        let disp = lv_display_create(w, h);
        assert!(!disp.is_null(), "lv_display_create returned NULL");

        lv_display_set_color_format(disp, lv_color_format_t_LV_COLOR_FORMAT_RGB565);
        // Match C port: draw into the back buffer, scan-out the front buffer.
        lv_display_set_buffers(
            disp,
            mem.fb1 as *mut c_void,
            mem.fb0 as *mut c_void,
            FB_BYTES as u32,
            lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_FULL,
        );
        lv_display_set_flush_cb(disp, Some(flush_callback));
        lv_display_set_default(disp);
        LVGL_DISP = disp;
        DISPLAY_READY.signal(());
        disp
    }
}

pub fn present_framebuffer() -> *const u16 {
    pio_rgb::front_ptr()
}

unsafe extern "C" fn flush_callback(
    disp: *mut lv_display_t,
    _area_p: *const lv_area_t,
    _px_map: *mut u8,
) {
    if disp.is_null() {
        return;
    }
    // Full refresh: LVGL already rendered into the inactive PSRAM buffer; swap scan-out.
    pio_rgb::request_swap(disp);
}

/// Allocate panel framebuffers + DMA staging inside PSRAM.
pub fn init_psram_memory(psram_base: *mut u8, psram_size: usize) -> Option<PanelMemory> {
    let need = FB_BYTES * 2 + TRANSFER_BYTES * 2;
    if psram_size < need {
        return None;
    }
    let fb0 = psram_base.cast::<u16>();
    let fb1 = unsafe { psram_base.add(FB_BYTES).cast::<u16>() };
    let transfer0 = unsafe { psram_base.add(FB_BYTES * 2).cast::<u16>() };
    let transfer1 = unsafe { psram_base.add(FB_BYTES * 2 + TRANSFER_BYTES).cast::<u16>() };

    pio_rgb::bind_framebuffers(fb0, fb1);
    pio_rgb::bind_transfer_buffers(transfer0, transfer1);

    Some(PanelMemory {
        fb0,
        fb1,
        transfer0,
        transfer1,
    })
}

/// Legacy helper kept for callers that only need the framebuffer pair.
pub fn init_psram_framebuffers(psram_base: *mut u8, psram_size: usize) -> Option<(*mut u16, *mut u16)> {
    init_psram_memory(psram_base, psram_size).map(|m| (m.fb0, m.fb1))
}

// Stubs retained for platform code that still syncs partial buffers on other targets.
pub fn draw_buffer_after_lvgl_create() {}
pub fn prepare_back_for_draw() {}
pub fn sync_back_from_front() {}

pub fn front_framebuffer() -> *const u16 {
    pio_rgb::front_ptr()
}
