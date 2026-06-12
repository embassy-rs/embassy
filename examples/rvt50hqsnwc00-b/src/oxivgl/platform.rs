//! STM32U5 + Embassy LTDC platform glue for OxivGL.
//!
//! **Two tasks:**
//! - [`super::touch_feed::run_touch_int_task`] — CTP_INT wake → I2C → channel queue
//! - `run_widget_demo` — sole LVGL/LTDC owner; drains every queued sample

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
    draw_buffer_after_lvgl_create, front_framebuffer, prefill_background, present_framebuffer,
    prepare_back_for_draw, LtdcDisplay,
};
use crate::oxivgl::indev::{TouchInput, TouchSample};
use crate::oxivgl::touch_feed::{self, TouchBoardSample};
use crate::oxivgl::widget_view::WidgetView;
use crate::rvt50_board::DISPLAY_WIDTH;

const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;
const PRESENT_PERIOD_MS: u64 = 33;
const UI_TICK_MS: u64 = 5;
const PRESENT_LVGL_TICKS: usize = 4;

pub const COLOR_BUF_LINES: usize = 20;
pub const LVGL_BUF_BYTES: usize = crate::rvt50_board::DISPLAY_WIDTH * COLOR_BUF_LINES * 2;

static VIEW: StaticCell<WidgetView> = StaticCell::new();

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
        touch.feed(TouchSample::from(board));
    }
    had_touch
}

/// Run one LVGL tick: prepare the draw buffer, ingest touch, then refresh.
///
/// Returns `true` when at least one touch sample was consumed.
fn lvgl_step(
    driver: &LvglDriver,
    touch_rx: &mut embassy_sync::channel::Receiver<
        'static,
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        TouchBoardSample,
        16,
    >,
    touch: &TouchInput,
) -> bool {
    prepare_back_for_draw();
    let had_touch = drain_touch_queue(touch_rx, touch);
    driver.timer_handler();
    had_touch
}

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

async fn lvgl_present_batch(
    driver: &LvglDriver,
    view: &mut WidgetView,
    ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    touch_rx: &mut embassy_sync::channel::Receiver<
        'static,
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        TouchBoardSample,
        16,
    >,
    touch: &TouchInput,
) {
    for _ in 0..PRESENT_LVGL_TICKS {
        lvgl_step(driver, touch_rx, touch);
        Timer::after(Duration::from_millis(LVGL_TICK_MS)).await;
    }

    let _ = view.update();
    present_to_ltdc(ltdc).await;
}

/// Run the OxivGL widget demo (LVGL + LTDC UI task).
pub async fn run_widget_demo(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>,
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
    draw_buffer_after_lvgl_create();

    let touch = TouchInput::register();
    let mut touch_rx = touch_feed::receiver();

    lvgl_present_batch(&driver, view, &mut ltdc, &mut touch_rx, &touch).await;

    let mut next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);

    loop {
        Timer::after(Duration::from_millis(UI_TICK_MS)).await;

        let had_touch = lvgl_step(&driver, &mut touch_rx, &touch);

        if had_touch {
            present_to_ltdc(&mut ltdc).await;
            next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);
        } else if Instant::now() >= next_present {
            lvgl_present_batch(&driver, view, &mut ltdc, &mut touch_rx, &touch).await;
            next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);
        }
    }
}
