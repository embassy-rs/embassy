//! STM32U5 + Embassy LTDC platform glue for OxivGL.

extern crate alloc;

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
/// Poll touch + drive LVGL timers between LTDC presents (matches rlvgl touch loop).
const INPUT_POLL_MS: u64 = 5;
/// LTDC refresh cadence (~30 fps).
const PRESENT_PERIOD_MS: u64 = 33;
/// Extra LVGL timer passes after sync, before each LTDC swap.
const PRESENT_LVGL_TICKS: usize = 3;

/// OxivGL stripe buffer height (lines × width × 2 bytes per stripe buffer).
pub const COLOR_BUF_LINES: usize = 20;
pub const LVGL_BUF_BYTES: usize = crate::rvt50_board::DISPLAY_WIDTH * COLOR_BUF_LINES * 2;

static VIEW: StaticCell<WidgetView> = StaticCell::new();

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

#[cfg(feature = "touch")]
use defmt::info;

#[cfg(feature = "touch")]
static mut TOUCH_WAS_PRESSED: bool = false;

#[cfg(feature = "touch")]
fn sample_board_touch(
    i2c: &mut embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Blocking,
        embassy_stm32::i2c::Master,
    >,
) -> TouchSample {
    let touch = crate::rvt50_board::read_touch(i2c);
    // SAFETY: UI task only.
    let was_pressed = unsafe { TOUCH_WAS_PRESSED };
    if touch.pressed && !was_pressed {
        info!(
            "oxivgl touch down x={} y={} raw=0x{:02x}",
            touch.x,
            touch.y,
            touch.raw_status
        );
    } else if !touch.pressed && was_pressed {
        info!("oxivgl touch up");
    }
    // SAFETY: UI task only.
    unsafe {
        TOUCH_WAS_PRESSED = touch.pressed;
    }
    TouchSample {
        x: touch.x as i32,
        y: touch.y as i32,
        pressed: touch.pressed,
    }
}

async fn present_to_ltdc(ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>) {
    let fb_ptr = present_framebuffer();
    ltdc.init_buffer(LtdcLayer::Layer1, fb_ptr as *const _);
    let _ = ltdc.reload().await;
}

async fn lvgl_present_batch(
    driver: &LvglDriver,
    view: &mut WidgetView,
    ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    #[cfg(feature = "touch")] touch: &TouchInput,
    #[cfg(feature = "touch")] i2c: &mut Option<
        embassy_stm32::i2c::I2c<
            'static,
            embassy_stm32::mode::Blocking,
            embassy_stm32::i2c::Master,
        >,
    >,
) {
    sync_back_from_front();

    for _ in 0..PRESENT_LVGL_TICKS {
        #[cfg(feature = "touch")]
        if let Some(i2c) = i2c.as_mut() {
            touch.publish(sample_board_touch(i2c));
        }
        driver.timer_handler();
        #[cfg(feature = "touch")]
        touch.sync_read();
        Timer::after(Duration::from_millis(LVGL_TICK_MS)).await;
    }

    let _ = view.update();
    present_to_ltdc(ltdc).await;
}

/// Run the OxivGL widget demo (touch optional via `touch` feature).
pub async fn run_widget_demo(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>,
    #[cfg(feature = "touch")] mut i2c: Option<
        embassy_stm32::i2c::I2c<
            'static,
            embassy_stm32::mode::Blocking,
            embassy_stm32::i2c::Master,
        >,
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
    lvgl_present_batch(&driver, view, &mut ltdc, &touch, &mut i2c).await;
    #[cfg(not(feature = "touch"))]
    lvgl_present_batch(&driver, view, &mut ltdc).await;

    let mut last_present = Instant::now();

    loop {
        #[cfg(feature = "touch")]
        if let Some(i2c) = i2c.as_mut() {
            touch.publish(sample_board_touch(i2c));
        }
        driver.timer_handler();
        #[cfg(feature = "touch")]
        touch.sync_read();

        if last_present.elapsed() >= Duration::from_millis(PRESENT_PERIOD_MS) {
            #[cfg(feature = "touch")]
            lvgl_present_batch(&driver, view, &mut ltdc, &touch, &mut i2c).await;
            #[cfg(not(feature = "touch"))]
            lvgl_present_batch(&driver, view, &mut ltdc).await;
            last_present = Instant::now();
        } else {
            Timer::after(Duration::from_millis(INPUT_POLL_MS)).await;
        }
    }
}
