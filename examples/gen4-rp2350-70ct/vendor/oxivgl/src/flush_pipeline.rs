// SPDX-License-Identifier: MIT OR Apache-2.0
//! ESP32 flush pipeline: async DMA transfer between LVGL and the display driver.
//!
//! LVGL's `flush_callback` (called from the render task) sends pixel data through
//! [`DRAW_OPERATION`] to [`flush_frame_buffer`] (running on a high-priority
//! interrupt executor), which forwards it to the board's [`DisplayOutput`]
//! implementation. Completion is signalled back via [`FLUSH_OPERATION`].

use core::slice::from_raw_parts;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use oxivgl_sys::{lv_area_t, lv_display_flush_ready, lv_display_t};

/// Error type for display output operations.
#[derive(Debug)]
pub enum UiError {
    /// Display output failed.
    Display,
}

/// Trait abstracting the raw pixel-data display output.
/// Defined in ui; implemented by the board layer.
#[allow(async_fn_in_trait)]
pub trait DisplayOutput {
    /// Write raw pixel data to the display.
    async fn show_raw_data(
        &mut self,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        data: &[u8],
    ) -> Result<(), UiError>;
}

/// Wrapper around an LVGL display pointer, allowing `Send`/`Sync` for
/// cross-executor channel transfer between the render task and flush task.
#[derive(Debug)]
pub struct LvDispDrv(pub(crate) *mut lv_display_t);
// SAFETY: LvDispDrv is only sent through a channel from the LVGL render task
// to the flush task; no concurrent access occurs.
unsafe impl Send for LvDispDrv {}
unsafe impl Sync for LvDispDrv {}

/// Pixel data produced by LVGL's flush callback, consumed by [`flush_frame_buffer`].
#[derive(Debug)]
pub struct DrawOperation {
    disp_drv: LvDispDrv,
    /// Points into a `static mut LvglBuf` — the `'static` lifetime is truthful.
    /// Aliasing safety: LVGL's `flushing` flag (see `wait_for_flushing` in
    /// lv_refr.c) prevents buffer reuse until `lv_display_flush_ready()` clears it.
    /// `flush_frame_buffer` consumes this ref before signaling `FLUSH_OPERATION`,
    /// which unblocks `wait_callback` → `flush_ready`. Do not store outside the
    /// flush pipeline.
    pub data: &'static [u8],
    /// X offset of the area in pixels.
    pub x: u16,
    /// Y offset of the area in pixels.
    pub y: u16,
    /// Width of the area in pixels.
    pub w: u16,
    /// Height of the area in pixels.
    pub h: u16,
}

// NOTE: single-display limit — these statics couple LVGL's flush pipeline to one
// display. A second simultaneous LVGL display is not supported.

/// Channel carrying rendered pixel stripes from LVGL to the flush task.
pub static DRAW_OPERATION: Channel<CriticalSectionRawMutex, DrawOperation, 1> = Channel::new();
/// Channel carrying flush-complete acknowledgements back to the LVGL render task.
pub static FLUSH_OPERATION: Channel<CriticalSectionRawMutex, LvDispDrv, 1> = Channel::new();

/// Async flush task: receives pixel data from LVGL, forwards to [`DisplayOutput`].
///
/// Spawn this on a high-priority interrupt executor. Signals [`DISPLAY_READY`](super::display::DISPLAY_READY)
/// once ready, then loops forever consuming [`DRAW_OPERATION`] and writing to the display.
#[esp_hal::ram]
pub async fn flush_frame_buffer(mut display_driver: impl DisplayOutput) -> ! {
    debug!("Starting flush task");
    super::display::DISPLAY_READY.signal(());
    let flush_sender = FLUSH_OPERATION.sender();
    loop {
        debug!("Flushing frame buffer");
        let draw_operation = DRAW_OPERATION.receive().await;
        let DrawOperation { disp_drv, data, x, y, w, h } = draw_operation;
        if let Err(_e) = display_driver.show_raw_data(x, y, w, h, data).await {
            error!("show_raw_data failed");
        }

        // DO NOT call LVGL from interrupt context.
        // Just notify the LVGL thread to call lv_display_flush_ready().
        flush_sender.send(disp_drv).await;
        debug!("Flush done");
    }
}

/// LVGL wait callback: blocks (via `waiti 0`) until flush completes.
#[esp_hal::ram]
pub(crate) unsafe extern "C" fn wait_callback(_disp: *mut lv_display_t) {
    // Wait for flush_frame_buffer (interrupt executor) to complete the SPI
    // transfer. Use `waiti 0` to sleep until the next interrupt instead of
    // busy-spinning — this lets the DMA/SPI completion interrupt wake us
    // and avoids wasting CPU cycles.
    // Use try_receive (non-blocking) + waiti rather than receive().await:
    // wait_callback runs on the LVGL task stack, not in an async context.
    // A blocking .await here would deadlock; try_receive + waiti 0 suspends
    // the core until the next interrupt without holding a critical section.
    loop {
        if let Ok(drv) = FLUSH_OPERATION.try_receive() {
            // SAFETY: drv.0 is the lv_display_t pointer originally supplied by LVGL to
            // flush_callback and stored in LvDispDrv; it remains valid for the display lifetime.
            unsafe {
                lv_display_flush_ready(drv.0);
            }
            return;
        }
        // SAFETY: `waiti 0` is a valid Xtensa instruction; executing it inside an
        // interrupt executor is safe — it just suspends the core until the next interrupt.
        #[cfg(target_os = "none")]
        unsafe {
            core::arch::asm!("waiti 0")
        };
    }
}

/// LVGL flush callback: packages pixel data and sends to the flush task.
#[esp_hal::ram]
pub(crate) unsafe extern "C" fn flush_callback(
    disp: *mut lv_display_t,
    area_p: *const lv_area_t,
    px_map: *mut u8,
) {
    if disp.is_null() || area_p.is_null() || px_map.is_null() {
        error!("flush_callback: null disp, area_p, or px_map");
        return;
    }
    // SAFETY: area_p is non-null (checked above); LVGL guarantees the lv_area_t
    // reference is valid for the duration of this callback.
    let area = unsafe { &*area_p };
    if area.x2 < area.x1 || area.y2 < area.y1 {
        error!("flush_callback: invalid area");
        return;
    }

    let w = (area.x2 - area.x1 + 1) as u16;
    let h = (area.y2 - area.y1 + 1) as u16;

    debug!("Flushing {} x {} ({};{} .. {};{})", w, h, area.x1, area.y1, area.x2, area.y2);

    let Some(len_pixels) = (w as usize).checked_mul(h as usize) else {
        error!("flush_callback: w*h overflowed");
        return;
    };

    // px_map is already byte-swapped by LVGL (RGB565_SWAPPED format).
    // Interpret as RGB565 bytes (2 per pixel).
    let data_bytes = len_pixels * 2;
    let op = DrawOperation {
        disp_drv: LvDispDrv(disp),
        // SAFETY: px_map is non-null (checked above); points into one of the `static mut
        // LvglBuf` buffers registered via `lv_display_set_buffers` in `lvgl_disp_init` —
        // this is what makes the `'static` lifetime truthful. The aliasing invariant
        // (no concurrent LVGL writes) is upheld by the `flushing` flag; see the
        // `DrawOperation::data` doc comment.
        data: unsafe { from_raw_parts(px_map, data_bytes) },
        x: area.x1 as u16,
        y: area.y1 as u16,
        w,
        h,
    };
    // Believed unreachable: wait_callback blocks until flush_frame_buffer
    // drains the channel, so it is always empty when LVGL calls flush_callback.
    // If this fires, it indicates a protocol violation (e.g. flush task not
    // spawned). No recovery: calling flush_ready would lie (data not flushed),
    // not calling it deadlocks wait_callback. Log and let it hang — the error
    // message will be visible in the log output.
    if let Err(_e) = DRAW_OPERATION.try_send(op) {
        error!("DRAW_OPERATION channel full — should be unreachable");
    }
}
