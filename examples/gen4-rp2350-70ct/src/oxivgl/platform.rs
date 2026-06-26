//! Embassy + PIO-RGB platform glue for the gen4-RP2350-70CT OxivGL demo.
//!
//! Much simpler than an LTDC port: the PIO/DMA scan-out engine continuously
//! refreshes the single PSRAM framebuffer for us, so there is **no buffer swap
//! and no `present()`** — LVGL just flushes dirty regions straight into the live
//! framebuffer. The UI task owns LVGL, drains the FT5446 touch queue into the
//! pointer indev, and drives `lv_timer_handler` on a fixed tick.
//!
//! The main loop mirrors [`rvt50hqsnwc00-b`]/[`rp2350-touch-lcd-7`]: touch input
//! triggers a multi-tick LVGL batch so press/release animations finish before the
//! next idle tick, instead of hammering partial PSRAM flushes on every sample.

extern crate alloc;

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_time::{Duration, Instant, Timer};
use oxivgl::display::{DISPLAY_READY, LvglBuffers};
use oxivgl::driver::LvglDriver;
use oxivgl::view::{View, register_view_events};
use oxivgl::widgets::{Obj, Screen};
use oxivgl_sys::LV_DEF_REFR_PERIOD;
use static_cell::StaticCell;

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::oxivgl::display::ScanOutDisplay;
use crate::oxivgl::indev::{TouchInput, TouchSample};
use crate::oxivgl::touch_dbg;
use crate::oxivgl::widget_view::WidgetView;
use crate::touch_feed::{self, TouchBoardSample};

/// LVGL timer tick — run the handler ~4× per LVGL refresh period.
const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;
const PRESENT_PERIOD_MS: u64 = 33;
const UI_TICK_MS: u64 = 5;
const PRESENT_LVGL_TICKS: usize = 4;

/// Number of display lines covered by each LVGL partial stripe buffer.
///
/// This is the OxivGL tile height and mirrors the rlvgl demo's
/// `render::TILE_LINES = 39`: LVGL renders each dirty area into one of these
/// SRAM stripe buffers (the "tile"), and the flush callback copies that tile
/// linearly into the live PSRAM framebuffer via `pio_rgb::blit_rgb565` — the
/// same tile-rendering strategy used by the hand-rolled rlvgl renderer, just
/// driven natively by LVGL's PARTIAL render mode instead of a custom
/// `TileRenderer`.
///
/// `800 × 39 × RGB565 ≈ 62.4 KiB` per buffer; two of them are allocated, which
/// still fits beside the LVGL pool and the PIO scan-out bounce buffers in the
/// RP2350B's 520 KiB SRAM while giving LVGL a wide tile to minimise flush
/// callback overhead and keep PSRAM writes sequential.
pub const COLOR_BUF_LINES: usize = 39;
/// Byte size of one LVGL partial stripe buffer (RGB565).
pub const LVGL_BUF_BYTES: usize = DISPLAY_WIDTH * COLOR_BUF_LINES * 2;

static VIEW: StaticCell<WidgetView> = StaticCell::new();
static FIRST_TOUCH_UI_LOGGED: AtomicBool = AtomicBool::new(false);

impl From<TouchBoardSample> for TouchSample {
    fn from(b: TouchBoardSample) -> Self {
        TouchSample {
            x: b.x,
            y: b.y,
            pressed: b.pressed,
        }
    }
}

fn drain_touch_queue(
    rx: &mut embassy_sync::channel::Receiver<
        'static,
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        TouchBoardSample,
        16,
    >,
    touch: &TouchInput,
) -> bool {
    let mut had_touch = false;
    while let Ok(board) = rx.try_receive() {
        had_touch = true;
        touch_dbg::bump_queued();
        let sample = TouchSample::from(board);
        if !FIRST_TOUCH_UI_LOGGED.swap(true, Ordering::Relaxed) {
            defmt::info!(
                "oxivgl first touch sample x={} y={} pressed={}",
                sample.x,
                sample.y,
                sample.pressed
            );
        }
        touch.feed(sample);
    }
    had_touch
}

async fn lvgl_present_batch(
    driver: &LvglDriver,
    view: &mut WidgetView,
    touch_rx: &mut embassy_sync::channel::Receiver<
        'static,
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        TouchBoardSample,
        16,
    >,
    touch: &TouchInput,
) {
    for _ in 0..PRESENT_LVGL_TICKS {
        let _ = drain_touch_queue(touch_rx, touch);
        driver.timer_handler();
        Timer::after(Duration::from_millis(LVGL_TICK_MS)).await;
    }
    let _ = view.update();
}

/// Run the OxivGL widget demo (LVGL UI task on the PIO-RGB framebuffer).
///
/// `fb` is the live PSRAM scan-out framebuffer already bound to
/// [`crate::pio_rgb::init_scanout`].
pub async fn run_widget_demo(fb: *mut u16, bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>) -> ! {
    let driver = LvglDriver::init(DISPLAY_WIDTH as i32, DISPLAY_HEIGHT as i32);
    let _display = ScanOutDisplay::init(fb, bufs);

    DISPLAY_READY.wait().await;

    let view = VIEW.init(WidgetView::default());
    let screen = Screen::active().expect("LVGL screen must exist after display init");
    let container = Obj::from_raw_non_owning(screen.handle());
    if view.create(&container).is_err() {
        defmt::warn!("oxivgl widget create failed");
        loop {
            Timer::after(Duration::from_secs(60)).await;
        }
    }
    register_view_events(view, &container);

    let touch = TouchInput::register();
    let mut touch_rx = touch_feed::receiver();
    let mut heartbeat_ticks: u32 = 0;

    defmt::info!("oxivgl UI loop starting");

    // Paint the first full frame before entering the interactive loop.
    lvgl_present_batch(&driver, view, &mut touch_rx, &touch).await;

    let mut next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);

    loop {
        Timer::after(Duration::from_millis(UI_TICK_MS)).await;

        let had_touch = drain_touch_queue(&mut touch_rx, &touch);

        if had_touch {
            // Let LVGL finish press/release transitions before going idle again.
            lvgl_present_batch(&driver, view, &mut touch_rx, &touch).await;
            next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);
        } else {
            driver.timer_handler();
            if Instant::now() >= next_present {
                lvgl_present_batch(&driver, view, &mut touch_rx, &touch).await;
                next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);
            }
        }

        heartbeat_ticks += 1;
        // ~1 s at UI_TICK_MS = 5 ms
        if heartbeat_ticks % 200 == 0 {
            defmt::info!(
                "oxivgl heartbeat queued={} fed={} read_cb={} clicks={}",
                touch_dbg::QUEUED.load(core::sync::atomic::Ordering::Relaxed),
                touch_dbg::FED.load(core::sync::atomic::Ordering::Relaxed),
                touch_dbg::READ_CB.load(core::sync::atomic::Ordering::Relaxed),
                touch_dbg::LVGL_CLICKS.load(core::sync::atomic::Ordering::Relaxed),
            );
        }
    }
}
