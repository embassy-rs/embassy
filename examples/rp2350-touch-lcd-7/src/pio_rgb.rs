//! PIO RGB scan-out for the ST7262 panel (Waveshare `pio_rgb.c` port).
//!
//! Full DMA + multi-PIO scan-out is board-specific; this module tracks the active
//! framebuffer pair and signals when the visible buffer should swap after LVGL flush.

use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use defmt::warn;

static FB0: AtomicPtr<u16> = AtomicPtr::new(core::ptr::null_mut());
static FB1: AtomicPtr<u16> = AtomicPtr::new(core::ptr::null_mut());
static PRESENT_IDX: AtomicBool = AtomicBool::new(false);
static SWAP_PENDING: AtomicBool = AtomicBool::new(false);
static SCANOUT_READY: AtomicBool = AtomicBool::new(false);

/// Register two full-screen RGB565 buffers (typically in PSRAM).
pub fn bind_framebuffers(fb0: *mut u16, fb1: *mut u16) {
    FB0.store(fb0, Ordering::Release);
    FB1.store(fb1, Ordering::Release);
}

/// Panel + PIO RGB bring-up.
///
/// TODO: Port Waveshare `pio_rgb.pio` state machines (PIO1 sync + PIO2 data/DMA).
/// Until then LVGL renders into PSRAM; connect a logic analyzer to verify flushes.
pub fn init_scanout() {
    if SCANOUT_READY.swap(true, Ordering::AcqRel) {
        return;
    }
    warn!("PIO RGB scan-out stub — framebuffers update in PSRAM only");
}

/// Pointer to the buffer LVGL should treat as the hidden draw target.
pub fn back_ptr() -> *mut u16 {
    let front_is_one = PRESENT_IDX.load(Ordering::Acquire);
    if front_is_one {
        FB0.load(Ordering::Acquire)
    } else {
        FB1.load(Ordering::Acquire)
    }
}

/// Pointer to the buffer currently designated for display.
pub fn front_ptr() -> *mut u16 {
    let front_is_one = PRESENT_IDX.load(Ordering::Acquire);
    if front_is_one {
        FB1.load(Ordering::Acquire)
    } else {
        FB0.load(Ordering::Acquire)
    }
}

/// Swap front/back after LVGL finished a frame.
pub fn request_swap() {
    SWAP_PENDING.store(true, Ordering::Release);
    PRESENT_IDX.fetch_xor(true, Ordering::AcqRel);
    SWAP_PENDING.store(false, Ordering::Release);
}

/// Partial blit from LVGL flush into the back buffer.
pub fn blit_rgb565(back: *mut u16, x1: i32, y1: i32, w: usize, h: usize, stride: usize, src: &[u8]) {
    if back.is_null() {
        return;
    }
    let row_bytes = w * 2;
    for row in 0..h {
        let y = y1 as usize + row;
        let dst_off = y * stride + x1 as usize;
        let src_off = row * row_bytes;
        if src_off + row_bytes > src.len() {
            break;
        }
        unsafe {
            core::ptr::copy_nonoverlapping(
                src.as_ptr().add(src_off),
                back.add(dst_off).cast::<u8>(),
                row_bytes,
            );
        }
    }
}
