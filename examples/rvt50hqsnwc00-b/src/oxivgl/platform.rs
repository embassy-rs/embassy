//! STM32U5 + Embassy LTDC platform glue for OxivGL.

extern crate alloc;

use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig};
use embassy_stm32::peripherals;
use embassy_time::{Duration, Instant, Timer};
use oxivgl::display::{DISPLAY_READY, LvglBuffers};
use oxivgl::driver::LvglDriver;
use oxivgl::view::{View, register_view_events};
use oxivgl::widgets::{Obj, Screen};
use oxivgl_sys::LV_DEF_REFR_PERIOD;
use static_cell::StaticCell;

use crate::oxivgl::display::{
    LtdcDisplay, front_framebuffer, prefill_background, present_framebuffer, sync_back_from_front,
};
#[cfg(feature = "touch")]
use crate::oxivgl::indev::{TouchInput, TouchSample};
use crate::oxivgl::widget_view::WidgetView;
use crate::rvt50_board::DISPLAY_WIDTH;

const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;
/// LTDC refresh cadence (~30 fps).
const PRESENT_PERIOD_MS: u64 = 33;
/// Touch sample cadence. Matches the rlvgl touch demos and avoids missing
/// short presses when the controller does not emit a separate release edge.
#[cfg(feature = "touch")]
const TOUCH_POLL_MS: u64 = 5;
/// Extra LVGL timer passes per LTDC present to keep animations smooth.
const PRESENT_LVGL_TICKS: usize = 3;

/// OxivGL stripe buffer dimensions (lines × width × 2 bytes).
pub const COLOR_BUF_LINES: usize = 20;
pub const LVGL_BUF_BYTES: usize = crate::rvt50_board::DISPLAY_WIDTH * COLOR_BUF_LINES * 2;

static VIEW: StaticCell<WidgetView> = StaticCell::new();

// ---------------------------------------------------------------------------
// Touch poller (touch feature only)
// ---------------------------------------------------------------------------

/// Reads the I2C touch panel and tracks press/release transitions.
#[cfg(feature = "touch")]
struct TouchPoller {
    i2c: embassy_stm32::i2c::I2c<'static, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>,
    was_pressed: bool,
    last_x: i32,
    last_y: i32,
}

#[cfg(feature = "touch")]
impl TouchPoller {
    fn new(i2c: embassy_stm32::i2c::I2c<'static, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>) -> Self {
        Self {
            i2c,
            was_pressed: false,
            last_x: 0,
            last_y: 0,
        }
    }

    /// Read the current touch state from I2C and log transitions.
    fn poll(&mut self) -> TouchSample {
        use defmt::info;
        let t = crate::rvt50_board::read_touch(&mut self.i2c);

        if t.pressed {
            self.last_x = t.x as i32;
            self.last_y = t.y as i32;
        }
        // Idle reads park at the panel edge; keep releases on the last contact
        // point so LVGL can finish click hit-testing on the pressed widget.
        let sample = TouchSample {
            x: self.last_x,
            y: self.last_y,
            pressed: t.pressed,
        };

        if t.pressed && !self.was_pressed {
            info!(
                "oxivgl touch down x={} y={} raw=0x{:02x}",
                sample.x, sample.y, t.raw_status
            );
        } else if !t.pressed && self.was_pressed {
            info!("oxivgl touch up x={} y={}", sample.x, sample.y);
        }
        self.was_pressed = t.pressed;
        sample
    }
}

#[cfg(feature = "touch")]
fn pump_touch(driver: &LvglDriver, touch: &TouchInput, poller: &mut TouchPoller) {
    touch.publish(poller.poll());
    driver.timer_handler();
    touch.sync_read();
}

// ---------------------------------------------------------------------------
// LTDC helpers
// ---------------------------------------------------------------------------

async fn init_ltdc_layer(ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>) {
    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::Rgb565,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: DISPLAY_WIDTH as _,
        window_y0: 0,
        window_y1: crate::rvt50_board::DISPLAY_HEIGHT as _,
    };
    ltdc.init_layer(&layer_config, None);
}

async fn present_to_ltdc(ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>) {
    let fb_ptr = present_framebuffer();
    ltdc.init_buffer(LtdcLayer::Layer1, fb_ptr as *const _);
    let _ = ltdc.reload().await;
}

// ---------------------------------------------------------------------------
// LVGL tick batch
// ---------------------------------------------------------------------------

/// Copy the front buffer, run PRESENT_LVGL_TICKS LVGL timer passes (each with
/// a touch sample in EVENT mode), then present the result to LTDC.
///
/// Touch is sampled **before** `timer_handler` and fed into LVGL **after** so
/// that `prev_scr` refresh is settled before new press events are dispatched.
async fn lvgl_present_batch(
    driver: &LvglDriver,
    view: &mut WidgetView,
    ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    #[cfg(feature = "touch")] touch: &TouchInput,
    #[cfg(feature = "touch")] poller: &mut TouchPoller,
) {
    sync_back_from_front();

    for _ in 0..PRESENT_LVGL_TICKS {
        #[cfg(feature = "touch")]
        touch.publish(poller.poll());
        driver.timer_handler();
        #[cfg(feature = "touch")]
        touch.sync_read();
        Timer::after(Duration::from_millis(LVGL_TICK_MS)).await;
    }

    let _ = view.update();
    present_to_ltdc(ltdc).await;
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the OxivGL widget demo.
///
/// With the `touch` feature: `i2c` is the blocking I2C bus to the touch
/// controller. Touch is polled at a small fixed cadence so press/release
/// transitions are delivered to LVGL even when the controller interrupt line
/// does not produce a distinct edge for every state change.
pub async fn run_widget_demo(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>,
    #[cfg(feature = "touch")] i2c: embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Blocking,
        embassy_stm32::i2c::Master,
    >,
) -> ! {
    init_ltdc_layer(&mut ltdc).await;

    let driver = LvglDriver::init(DISPLAY_WIDTH as i32, crate::rvt50_board::DISPLAY_HEIGHT as i32);
    let _display = LtdcDisplay::init(DISPLAY_WIDTH as i32, crate::rvt50_board::DISPLAY_HEIGHT as i32, bufs);

    DISPLAY_READY.wait().await;
    prefill_background();

    ltdc.init_buffer(LtdcLayer::Layer1, front_framebuffer() as *const _);
    let _ = ltdc.reload().await;

    let view = VIEW.init(WidgetView::default());
    let screen = Screen::active().expect("LVGL screen must exist after display init");
    let container = Obj::from_raw_non_owning(screen.handle());
    if view.create(&container).is_err() {
        defmt::warn!("oxivgl widget create failed");
        loop {
            Timer::after(Duration::from_secs(60)).await;
        }
    }
    register_view_events(view);
    view.log_layout();

    #[cfg(feature = "touch")]
    let touch = TouchInput::register();
    #[cfg(feature = "touch")]
    let mut poller = TouchPoller::new(i2c);

    // Initial flush: settle the display before entering the main loop.
    #[cfg(feature = "touch")]
    lvgl_present_batch(&driver, view, &mut ltdc, &touch, &mut poller).await;
    #[cfg(not(feature = "touch"))]
    lvgl_present_batch(&driver, view, &mut ltdc).await;

    let mut next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);

    loop {
        #[cfg(feature = "touch")]
        Timer::after(Duration::from_millis(TOUCH_POLL_MS)).await;
        #[cfg(not(feature = "touch"))]
        Timer::at(next_present).await;

        #[cfg(feature = "touch")]
        pump_touch(&driver, &touch, &mut poller);
        #[cfg(not(feature = "touch"))]
        driver.timer_handler();

        if Instant::now() >= next_present {
            #[cfg(feature = "touch")]
            lvgl_present_batch(&driver, view, &mut ltdc, &touch, &mut poller).await;
            #[cfg(not(feature = "touch"))]
            lvgl_present_batch(&driver, view, &mut ltdc).await;
            next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);
        }
    }
}
