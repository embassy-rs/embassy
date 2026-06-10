//! STM32U5 + Embassy LTDC platform glue for OxivGL.

extern crate alloc;

#[cfg(feature = "touch")]
use embassy_futures::select::select;
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig};
use embassy_stm32::peripherals;
use embassy_time::{Duration, Instant, Timer};
use oxivgl::display::{LvglBuffers, DISPLAY_READY};
use oxivgl::driver::LvglDriver;
use oxivgl::view::{register_view_events, View};
use oxivgl::widgets::{Obj, Screen};
use oxivgl_sys::LV_DEF_REFR_PERIOD;
use static_cell::StaticCell;

use crate::oxivgl::display::{
    front_framebuffer, prefill_background, present_framebuffer, sync_back_from_front, LtdcDisplay,
};
use crate::oxivgl::indev::{TouchInput, TouchSample};
use crate::oxivgl::widget_view::WidgetView;
use crate::rvt50_board::DISPLAY_WIDTH;

const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;
/// LTDC refresh cadence (~30 fps).
const PRESENT_PERIOD_MS: u64 = 33;
/// Extra LVGL timer passes per LTDC present to keep animations smooth.
const PRESENT_LVGL_TICKS: usize = 3;

/// OxivGL stripe buffer dimensions (lines × width × 2 bytes).
pub const COLOR_BUF_LINES: usize = 20;
pub const LVGL_BUF_BYTES: usize = crate::rvt50_board::DISPLAY_WIDTH * COLOR_BUF_LINES * 2;

static VIEW: StaticCell<WidgetView> = StaticCell::new();

// ---------------------------------------------------------------------------
// Touch poller (touch feature only)
// ---------------------------------------------------------------------------

/// Reads the I2C touch panel on demand and tracks press/release transitions.
///
/// Normally the caller waits for [`TouchPoller::wait_for_int`] (the
/// `CTP_INT` falling edge) before polling, so I2C traffic only occurs on
/// actual touch events instead of every 5 ms.
#[cfg(feature = "touch")]
struct TouchPoller {
    i2c: embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Blocking,
        embassy_stm32::i2c::Master,
    >,
    int_pin: embassy_stm32::exti::ExtiInput<'static, embassy_stm32::mode::Async>,
    was_pressed: bool,
}

#[cfg(feature = "touch")]
impl TouchPoller {
    fn new(
        i2c: embassy_stm32::i2c::I2c<
            'static,
            embassy_stm32::mode::Blocking,
            embassy_stm32::i2c::Master,
        >,
        int_pin: embassy_stm32::exti::ExtiInput<'static, embassy_stm32::mode::Async>,
    ) -> Self {
        Self { i2c, int_pin, was_pressed: false }
    }

    /// Block until `CTP_INT` falls (touch event) or `deadline` is reached.
    async fn wait_for_int(&mut self, deadline: Instant) {
        let _ = select(self.int_pin.wait_for_falling_edge(), Timer::at(deadline)).await;
    }

    /// Read the current touch state from I2C and log transitions.
    fn poll(&mut self) -> TouchSample {
        use defmt::info;
        let t = crate::rvt50_board::read_touch(&mut self.i2c);
        if t.pressed && !self.was_pressed {
            info!(
                "oxivgl touch down x={} y={} raw=0x{:02x}",
                t.x, t.y, t.raw_status
            );
        } else if !t.pressed && self.was_pressed {
            info!("oxivgl touch up");
        }
        self.was_pressed = t.pressed;
        TouchSample { x: t.x as i32, y: t.y as i32, pressed: t.pressed }
    }
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
/// controller, and `touch_int` is the `CTP_INT` EXTI input (PE6, active-low).
/// The main loop sleeps until the INT falls (touch event) or the 33 ms present
/// deadline, so no I2C traffic occurs while the screen is idle.
pub async fn run_widget_demo(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>,
    #[cfg(feature = "touch")] i2c: embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Blocking,
        embassy_stm32::i2c::Master,
    >,
    #[cfg(feature = "touch")] touch_int: embassy_stm32::exti::ExtiInput<
        'static,
        embassy_stm32::mode::Async,
    >,
) -> ! {
    init_ltdc_layer(&mut ltdc).await;

    let driver = LvglDriver::init(DISPLAY_WIDTH as i32, crate::rvt50_board::DISPLAY_HEIGHT as i32);
    let _display = LtdcDisplay::init(
        DISPLAY_WIDTH as i32,
        crate::rvt50_board::DISPLAY_HEIGHT as i32,
        bufs,
    );

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
    let mut poller = TouchPoller::new(i2c, touch_int);

    // Initial flush: settle the display before entering the main loop.
    #[cfg(feature = "touch")]
    lvgl_present_batch(&driver, view, &mut ltdc, &touch, &mut poller).await;
    #[cfg(not(feature = "touch"))]
    lvgl_present_batch(&driver, view, &mut ltdc).await;

    let mut next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);

    loop {
        // Sleep until CTP_INT asserts (touch event) or the present deadline —
        // whichever comes first. No I2C traffic while the screen is idle.
        #[cfg(feature = "touch")]
        poller.wait_for_int(next_present).await;
        #[cfg(not(feature = "touch"))]
        Timer::at(next_present).await;

        // Sample touch, tick LVGL, dispatch input events.
        #[cfg(feature = "touch")]
        touch.publish(poller.poll());
        driver.timer_handler();
        #[cfg(feature = "touch")]
        touch.sync_read();

        if Instant::now() >= next_present {
            #[cfg(feature = "touch")]
            lvgl_present_batch(&driver, view, &mut ltdc, &touch, &mut poller).await;
            #[cfg(not(feature = "touch"))]
            lvgl_present_batch(&driver, view, &mut ltdc).await;
            next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);
        }
    }
}
