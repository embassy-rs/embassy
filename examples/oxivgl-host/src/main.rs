//! Host (SDL2) port of the protronic OxivGL widget demo.
//!
//! Runs the same lighting-scene UI as `examples/rvt50hqsnwc00-b` on a PC so you
//! can verify LVGL event delivery with the SDL mouse indev before debugging I2C
//! touch on hardware.
//!
//! ```bash
//! cd examples/oxivgl-host
//! cargo run
//! ```

mod widget_view;

use core::ffi::CStr;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use log::info;
use oxivgl::driver::LvglDriver;
use oxivgl::view::{register_view_events, View};
use oxivgl::widgets::{Obj, Screen};
use oxivgl_sys::LV_DEF_REFR_PERIOD;
use static_cell::StaticCell;

use crate::widget_view::WidgetView;

const DISPLAY_WIDTH: i32 = 800;
const DISPLAY_HEIGHT: i32 = 480;
const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;
/// Matches `oxivgl::run_app` and the RVT50 `platform` loop.
const TICKS_PER_FRAME: usize = 4;

static VIEW: StaticCell<WidgetView> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_millis()
        .init();

    info!(
        "OxivGL protronic host demo (SDL {}x{}) — click scene buttons to test events",
        DISPLAY_WIDTH, DISPLAY_HEIGHT
    );

    let title = CStr::from_bytes_with_nul(b"protronic OxivGL host demo\0").unwrap();
    let driver = LvglDriver::sdl(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .title(title)
        .mouse(true)
        .build();

    let view = VIEW.init(WidgetView::default());
    let screen = Screen::active().expect("LVGL screen must exist after SDL init");
    let container = Obj::from_raw_non_owning(screen.handle());
    view.create(&container).expect("widget tree create failed");
    register_view_events(view);
    view.log_layout();

    info!("SDL mouse indev active — watch stderr for PRESSED/CLICKED events");

    loop {
        let _ = view.update();
        for _ in 0..TICKS_PER_FRAME {
            driver.timer_handler();
            Timer::after(Duration::from_millis(LVGL_TICK_MS)).await;
        }
    }
}
