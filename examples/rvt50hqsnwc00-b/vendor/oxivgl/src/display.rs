// SPDX-License-Identifier: MIT OR Apache-2.0
// Formerly `lvgl_buffers` — renamed for clarity.
//! DMA-aligned render buffers and embedded display initialisation.
//!
//! Buffer types (`LvglBuf`, `LvglBuffers`) are target-independent.
//! `lvgl_disp_init` registers them with LVGL and (on ESP32) wires up the
//! flush pipeline from the `flush_pipeline` module.

use core::ffi::c_void;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use oxivgl_sys::{
    lv_display_create, lv_display_set_buffers, lv_display_set_color_format,
    lv_color_format_t_LV_COLOR_FORMAT_RGB565_SWAPPED,
    lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
};
#[cfg(feature = "esp-hal")]
use oxivgl_sys::{lv_display_set_flush_cb, lv_display_set_flush_wait_cb};

/// Number of pixel rows per render stripe. Large value trades stack RAM for fewer flush calls.
// NOTE: this is a lot of buffer — reduces available stack RAM intentionally; easy to shrink later.
pub const COLOR_BUF_LINES: usize = 40;

/// Aligned render buffer; `BYTES` = `screen_w × COLOR_BUF_LINES × 2` (RGB565).
/// Caller allocates as a `static mut` so the pointer is valid for the LVGL display lifetime.
#[repr(align(16))]
pub struct LvglBuf<const BYTES: usize>(pub [u8; BYTES]);

impl<const BYTES: usize> core::fmt::Debug for LvglBuf<BYTES> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LvglBuf").finish_non_exhaustive()
    }
}

impl<const BYTES: usize> LvglBuf<BYTES> {
    /// Create a zeroed render buffer.
    pub const fn new() -> Self { Self([0; BYTES]) }
}

/// Pair of DMA-aligned render buffers. Parameterised by byte size so the caller
/// controls allocation using the actual screen width:
/// `LvglBuffers::<{SCREEN_W as usize * COLOR_BUF_LINES * 2}>`
pub struct LvglBuffers<const BYTES: usize> {
    /// First render buffer.
    pub buf1: LvglBuf<BYTES>,
    /// Second render buffer (double-buffering).
    pub buf2: LvglBuf<BYTES>,
}

impl<const BYTES: usize> core::fmt::Debug for LvglBuffers<BYTES> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LvglBuffers").finish_non_exhaustive()
    }
}

impl<const BYTES: usize> LvglBuffers<BYTES> {
    /// Create zeroed double-buffered render buffers.
    pub const fn new() -> Self { Self { buf1: LvglBuf::new(), buf2: LvglBuf::new() } }
}

/// Signalled by the flush task (ESP32) or immediately (host) once the display
/// driver is ready. [`crate::view::run_app`] waits on this before entering
/// the render loop.
pub static DISPLAY_READY: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// Register render buffers with LVGL and wire up the flush pipeline.
///
/// # Safety
/// `lv_init()` must have been called. Call at most once. `bufs` must be
/// `'static` so the pointers remain valid for the LVGL display lifetime.
/// `w` and `h` are the display resolution in pixels.
pub unsafe fn lvgl_disp_init<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
) {
    // SAFETY: addr_of_mut! obtains raw pointers without creating &mut references.
    // Caller guarantees single-call, lv_init() precondition, and 'static lifetime.
    unsafe {
        let buf1_ptr = core::ptr::addr_of_mut!(bufs.buf1) as *mut c_void;
        let buf2_ptr = core::ptr::addr_of_mut!(bufs.buf2) as *mut c_void;

        assert_eq!(buf1_ptr as usize % 4, 0, "DMA buffer must be 4-byte aligned");

        let disp = lv_display_create(w, h);
        assert!(!disp.is_null(), "lv_display_create returned NULL");

        lv_display_set_color_format(disp, lv_color_format_t_LV_COLOR_FORMAT_RGB565_SWAPPED);
        lv_display_set_buffers(
            disp,
            buf1_ptr,
            buf2_ptr,
            BYTES as u32,
            lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
        );
        #[cfg(feature = "esp-hal")]
        {
            use crate::flush_pipeline::{flush_callback, wait_callback};
            lv_display_set_flush_cb(disp, Some(flush_callback));
            lv_display_set_flush_wait_cb(disp, Some(wait_callback));
        }
        // On non-esp-hal targets the flush task never runs; signal ready immediately.
        #[cfg(not(feature = "esp-hal"))]
        DISPLAY_READY.signal(());
    }
}
