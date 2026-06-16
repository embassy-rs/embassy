//! Platform glue for the JSON-driven hall lighting OxivGL demo.

extern crate alloc;

use embassy_time::{Duration, Instant, Timer};
use oxivgl::display::{DISPLAY_READY, LvglBuffers};
use oxivgl::driver::LvglDriver;
use oxivgl::view::{register_view_events, View};
use oxivgl::widgets::{Obj, Screen};
use oxivgl_sys::LV_DEF_REFR_PERIOD;
use static_cell::StaticCell;

use crate::oxivgl::display::{
    draw_buffer_after_lvgl_create, front_framebuffer, prefill_background, present_framebuffer,
    prepare_back_for_draw, PanelDisplay,
};
use crate::oxivgl::hall_view::HallView;
use crate::oxivgl::indev::{TouchInput, TouchSample};
use crate::oxivgl::touch_feed::{self, TouchBoardSample};

const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;
const PRESENT_PERIOD_MS: u64 = 33;
const UI_TICK_MS: u64 = 5;
const PRESENT_LVGL_TICKS: usize = 4;

static VIEW: StaticCell<HallView> = StaticCell::new();

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

async fn present_to_panel() {
    let _ = present_framebuffer();
}

async fn lvgl_present_batch(
    driver: &LvglDriver,
    view: &mut HallView,
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
    present_to_panel().await;
}

/// Run the hall lighting UI + CAN integration demo.
pub async fn run_hall_demo(bufs: &'static mut LvglBuffers<{ super::platform::LVGL_BUF_BYTES }>) -> ! {
    let driver = LvglDriver::init(
        crate::board::DISPLAY_WIDTH as i32,
        crate::board::DISPLAY_HEIGHT as i32,
    );
    let _display = PanelDisplay::init(
        crate::board::DISPLAY_WIDTH as i32,
        crate::board::DISPLAY_HEIGHT as i32,
        bufs,
    );

    DISPLAY_READY.wait().await;
    prefill_background();

    let _ = front_framebuffer();

    let view = VIEW.init(HallView::default());
    let screen = Screen::active().expect("LVGL screen must exist after display init");
    let container = Obj::from_raw_non_owning(screen.handle());
    if view.create(&container).is_err() {
        defmt::warn!("oxivgl hall create failed");
        loop {
            Timer::after(Duration::from_secs(60)).await;
        }
    }
    register_view_events(view);
    draw_buffer_after_lvgl_create();

    let touch = TouchInput::register();
    let mut touch_rx = touch_feed::receiver();

    lvgl_present_batch(&driver, view, &mut touch_rx, &touch).await;

    let mut next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);

    loop {
        Timer::after(Duration::from_millis(UI_TICK_MS)).await;

        prepare_back_for_draw();
        let had_touch = drain_touch_queue(&mut touch_rx, &touch);

        if had_touch {
            lvgl_present_batch(&driver, view, &mut touch_rx, &touch).await;
            next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);
        } else {
            driver.timer_handler();
            if Instant::now() >= next_present {
                lvgl_present_batch(&driver, view, &mut touch_rx, &touch).await;
                next_present = Instant::now() + Duration::from_millis(PRESENT_PERIOD_MS);
            }
        }
    }
}
