//! STM32U5 LTDC display flush for OxivGL / LVGL v9.5 on the Riverdi RVT50.
//!
//! Uses two full-screen RGB565 buffers instead of a
//! separate shadow framebuffer, keeping RAM within the STM32U5A9 2.5 MiB budget.

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

use crate::rvt50_board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

const FB_BYTES: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT * 2;

/// LVGL display handle (set once in [`lvgl_disp_init_ltdc`]).
static mut LVGL_DISP: *mut lv_display_t = core::ptr::null_mut();

/// Which LTDC buffer is currently shown (0 or 1).
static mut PRESENT_IDX: u8 = 0;

/// Double full-screen buffers presented to LTDC (RGB565 byte pairs).
static mut PRESENT_FB0: [u8; FB_BYTES] = [0; FB_BYTES];
static mut PRESENT_FB1: [u8; FB_BYTES] = [0; FB_BYTES];

fn front_ptr() -> *mut u8 {
    unsafe {
        if PRESENT_IDX == 0 {
            ptr::addr_of_mut!(PRESENT_FB0)
        } else {
            ptr::addr_of_mut!(PRESENT_FB1)
        }
    }
    .cast::<u8>()
}

fn back_ptr() -> *mut u8 {
    unsafe {
        if PRESENT_IDX == 0 {
            ptr::addr_of_mut!(PRESENT_FB1)
        } else {
            ptr::addr_of_mut!(PRESENT_FB0)
        }
    }
    .cast::<u8>()
}

/// LTDC display token — proves LVGL display init completed.
#[derive(Debug)]
pub struct LtdcDisplay;

impl LtdcDisplay {
    /// Register LVGL display buffers and wire the LTDC flush callback.
    pub fn init<const BYTES: usize>(
        w: i32,
        h: i32,
        bufs: &'static mut LvglBuffers<BYTES>,
    ) -> Self {
        // SAFETY: `lv_init()` completed in `LvglDriver::init`; single init; `'static` bufs.
        unsafe {
            lvgl_disp_init_ltdc(w, h, bufs);
        }
        Self
    }
}

/// Return the LVGL display created by [`LtdcDisplay::init`].
pub(crate) fn lvgl_display() -> *mut lv_display_t {
    // SAFETY: written once during init before the UI loop runs.
    unsafe { LVGL_DISP }
}

fn rgb565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3)
}

/// Fill both LTDC framebuffers with the demo panel background.
pub fn prefill_background() {
    let px = rgb565(16, 32, 48).to_le_bytes();
    // SAFETY: only the UI task touches these static mut buffers.
    unsafe {
        for fb in [ptr::addr_of_mut!(PRESENT_FB0), ptr::addr_of_mut!(PRESENT_FB1)] {
            let base = fb.cast::<u8>();
            for i in (0..FB_BYTES).step_by(2) {
                ptr::copy_nonoverlapping(px.as_ptr(), base.add(i), 2);
            }
        }
    }
}

/// Register LVGL display buffers and wire the LTDC flush callback.
///
/// # Safety
/// `lv_init()` must have been called. `bufs` must remain valid for the
/// display lifetime.
pub unsafe fn lvgl_disp_init_ltdc<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
) -> *mut lv_display_t {
    // SAFETY: single init; pointers come from static mut framebuffers.
    unsafe {
        let buf1_ptr = ptr::addr_of_mut!(bufs.buf1) as *mut c_void;
        let buf2_ptr = ptr::addr_of_mut!(bufs.buf2) as *mut c_void;

        let disp = lv_display_create(w, h);
        assert!(!disp.is_null(), "lv_display_create returned NULL");

        // Embassy LTDC RGB565 matches standard (non-swapped) channel order.
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

/// Copy the visible buffer to the back buffer so partial LVGL flushes retain
/// pixels outside the dirty regions.
pub fn sync_back_from_front() {
    // SAFETY: only the UI task touches these static mut buffers.
    unsafe {
        ptr::copy_nonoverlapping(front_ptr(), back_ptr(), FB_BYTES);
    }
}

/// Pointer to the framebuffer currently shown on LTDC (before the next swap).
pub fn front_framebuffer() -> *const u16 {
    // SAFETY: only the UI task touches these static mut buffers.
    front_ptr() as *const u16
}

/// Swap LTDC buffers and return the pointer to the newly visible framebuffer.
pub fn present_framebuffer() -> *const u16 {
    // SAFETY: only the UI task touches these static mut buffers.
    unsafe {
        PRESENT_IDX = 1 - PRESENT_IDX;
        front_ptr() as *const u16
    }
}

unsafe extern "C" fn flush_callback(
    disp: *mut lv_display_t,
    area_p: *const lv_area_t,
    px_map: *mut u8,
) {
    if disp.is_null() || area_p.is_null() || px_map.is_null() {
        return;
    }

    // SAFETY: LVGL provides valid area and pixel map for the duration of this callback.
    let area = unsafe { &*area_p };
    if area.x2 < area.x1 || area.y2 < area.y1 {
        return;
    }

    let w = (area.x2 - area.x1 + 1) as usize;
    let h = (area.y2 - area.y1 + 1) as usize;
    let row_bytes = w * 2;
    let stride = DISPLAY_WIDTH * 2;

    // SAFETY: px_map points at `w*h` RGB565 pixels supplied by LVGL.
    let src = unsafe { slice::from_raw_parts(px_map, row_bytes * h) };

    // SAFETY: back buffer is only written from this LVGL flush callback.
    unsafe {
        let back = back_ptr();
        for row in 0..h {
            let y = area.y1 as usize + row;
            if y >= DISPLAY_HEIGHT {
                break;
            }
            let dst_off = y * stride + area.x1 as usize * 2;
            let src_off = row * row_bytes;
            let end = dst_off + row_bytes;
            if end <= FB_BYTES {
                ptr::copy_nonoverlapping(
                    src[src_off..].as_ptr(),
                    back.add(dst_off),
                    row_bytes,
                );
            }
        }
        lv_display_flush_ready(disp);
    }
}
