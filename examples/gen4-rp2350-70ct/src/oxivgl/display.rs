//! OxivGL / LVGL v9.5 display glue for the gen4-RP2350-70CT PIO RGB panel.
//!
//! Unlike an LTDC panel (which scans out of a CPU-addressable framebuffer that
//! the controller refreshes for us), this board drives the 800×480 RGB panel
//! with a PIO + DMA scan-out engine (see [`crate::pio_rgb`]) that streams a
//! **single persistent RGB565 framebuffer** living in PSRAM. We therefore run
//! LVGL in `PARTIAL` render mode and let the flush callback copy each dirty
//! region straight into that live framebuffer via [`pio_rgb::blit_rgb565`].
//!
//! This is the OxivGL counterpart of the hand-rolled `rlvgl` demo and follows
//! the exact same anti-flicker strategy distilled from the C reference
//! (`gen4_rp2350_lvgl`): one framebuffer, only the changed pixels rewritten per
//! frame, so the CPU never floods the shared QMI/PSRAM bus and starves the
//! scan-out refill.

use core::ffi::c_void;
use core::sync::atomic::{AtomicPtr, Ordering};
use core::{ptr, slice};

use oxivgl::display::{DISPLAY_READY, LvglBuffers};
use oxivgl_sys::{
    lv_area_t, lv_color_format_t_LV_COLOR_FORMAT_RGB565, lv_display_create, lv_display_flush_ready,
    lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL, lv_display_set_buffers, lv_display_set_color_format,
    lv_display_set_default, lv_display_set_flush_cb, lv_display_t,
};

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::pio_rgb;

/// LVGL display handle, set once in [`ScanOutDisplay::init`].
static LVGL_DISP: AtomicPtr<lv_display_t> = AtomicPtr::new(ptr::null_mut());

/// The single PSRAM framebuffer that PIO/DMA scans out; LVGL flushes into it.
static FRAMEBUFFER: AtomicPtr<u16> = AtomicPtr::new(ptr::null_mut());

/// LVGL display token — proves [`ScanOutDisplay::init`] completed.
#[derive(Debug)]
pub struct ScanOutDisplay;

impl ScanOutDisplay {
    /// Register the LVGL display and wire the PIO-RGB flush callback.
    ///
    /// `fb` is the live PSRAM scan-out framebuffer; `bufs` are the LVGL partial
    /// stripe buffers (small SRAM working buffers, not full screens).
    pub fn init<const BYTES: usize>(fb: *mut u16, bufs: &'static mut LvglBuffers<BYTES>) -> Self {
        FRAMEBUFFER.store(fb, Ordering::Release);
        // SAFETY: `lv_init()` ran in `LvglDriver::init`; single init; `'static` bufs.
        unsafe {
            init_display(fb, bufs);
        }
        Self
    }
}

/// Return the LVGL display created by [`ScanOutDisplay::init`].
pub(crate) fn lvgl_display() -> *mut lv_display_t {
    LVGL_DISP.load(Ordering::Acquire)
}

/// # Safety
/// `lv_init()` must have been called and `bufs` must outlive the display.
unsafe fn init_display<const BYTES: usize>(_fb: *mut u16, bufs: &'static mut LvglBuffers<BYTES>) {
    // SAFETY: single init; buffer pointers come from static-mut LVGL stripes.
    unsafe {
        let buf1 = ptr::addr_of_mut!(bufs.buf1) as *mut c_void;
        let buf2 = ptr::addr_of_mut!(bufs.buf2) as *mut c_void;

        let disp = lv_display_create(DISPLAY_WIDTH as i32, DISPLAY_HEIGHT as i32);
        assert!(!disp.is_null(), "lv_display_create returned NULL");

        lv_display_set_color_format(disp, lv_color_format_t_LV_COLOR_FORMAT_RGB565);
        // PARTIAL mode: LVGL renders dirty areas into the small stripe buffers,
        // then calls `flush_callback`, which blits them into the live PSRAM FB.
        lv_display_set_buffers(
            disp,
            buf1,
            buf2,
            BYTES as u32,
            lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
        );
        lv_display_set_flush_cb(disp, Some(flush_callback));
        lv_display_set_default(disp);

        LVGL_DISP.store(disp, Ordering::Release);
        DISPLAY_READY.signal(());
    }
}

unsafe extern "C" fn flush_callback(disp: *mut lv_display_t, area_p: *const lv_area_t, px_map: *mut u8) {
    if disp.is_null() || area_p.is_null() || px_map.is_null() {
        return;
    }

    // SAFETY: LVGL guarantees `area` and `px_map` valid for this callback.
    let area = unsafe { &*area_p };
    if area.x2 < area.x1 || area.y2 < area.y1 {
        // SAFETY: signalling readiness on a valid display is always sound.
        unsafe { lv_display_flush_ready(disp) };
        return;
    }

    let fb = FRAMEBUFFER.load(Ordering::Acquire);
    let w = (area.x2 - area.x1 + 1) as usize;
    let h = (area.y2 - area.y1 + 1) as usize;

    // SAFETY: `px_map` points at `w * h` RGB565 pixels supplied by LVGL.
    let src = unsafe { slice::from_raw_parts(px_map, w * h * 2) };
    // Copy just the dirty rectangle into the live PSRAM framebuffer. Writing
    // only the changed region (instead of a full 768 KiB frame) keeps the QMI
    // bus free for the scan-out refill DMA → no roll / flicker.
    pio_rgb::blit_rgb565(fb, area.x1, area.y1, w, h, src);

    // SAFETY: `disp` is the valid LVGL display created during init.
    unsafe { lv_display_flush_ready(disp) };
}
