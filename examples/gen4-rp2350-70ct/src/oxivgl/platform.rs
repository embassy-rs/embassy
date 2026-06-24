//! Embassy + PIO-RGB platform glue for the gen4-RP2350-70CT OxivGL demo.
//!
//! Much simpler than an LTDC port: the PIO/DMA scan-out engine continuously
//! refreshes the single PSRAM framebuffer for us, so there is **no buffer swap
//! and no `present()`** — LVGL just flushes dirty regions straight into the live
//! framebuffer. The UI task owns LVGL, drains the FT5446 touch queue into the
//! pointer indev, and drives `lv_timer_handler` on a fixed tick.

extern crate alloc;

use embassy_time::{Duration, Timer};
use oxivgl::display::{DISPLAY_READY, LvglBuffers};
use oxivgl::driver::LvglDriver;
use oxivgl::view::{View, register_view_events};
use oxivgl::widgets::{Obj, Screen};
use oxivgl_sys::LV_DEF_REFR_PERIOD;
use static_cell::StaticCell;

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::oxivgl::display::ScanOutDisplay;
use crate::oxivgl::indev::{TouchInput, TouchSample};
use crate::oxivgl::widget_view::WidgetView;
use crate::touch_feed::{self, TouchBoardSample};

/// LVGL timer tick — run the handler ~4× per LVGL refresh period.
const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;

/// Number of display lines covered by each LVGL partial stripe buffer.
///
/// Two of these stripe buffers are allocated; 16 lines (≈25 KiB each) keeps the
/// pair small enough to fit beside the LVGL pool and the PIO scan-out bounce
/// buffers in the RP2350B's 520 KiB SRAM, while still giving LVGL a decent
/// partial-render chunk.
pub const COLOR_BUF_LINES: usize = 16;
/// Byte size of one LVGL partial stripe buffer (RGB565).
pub const LVGL_BUF_BYTES: usize = DISPLAY_WIDTH * COLOR_BUF_LINES * 2;

static VIEW: StaticCell<WidgetView> = StaticCell::new();

impl From<TouchBoardSample> for TouchSample {
    fn from(b: TouchBoardSample) -> Self {
        TouchSample {
            x: b.x,
            y: b.y,
            pressed: b.pressed,
        }
    }
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
    let touch_rx = touch_feed::receiver();

    // Paint the first full frame, then keep servicing LVGL: PARTIAL render only
    // touches changed widgets, so each subsequent flush writes a few KiB into
    // the persistent framebuffer instead of the whole 768 KiB screen.
    loop {
        while let Ok(board) = touch_rx.try_receive() {
            touch.feed(TouchSample::from(board));
        }

        driver.timer_handler();
        let _ = view.update();

        Timer::after(Duration::from_millis(LVGL_TICK_MS)).await;
    }
}
