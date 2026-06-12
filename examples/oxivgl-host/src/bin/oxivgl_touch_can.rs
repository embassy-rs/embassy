//! Host (SDL2) port of the JSON-driven hall lighting UI with Linux SocketCAN.
//!
//! ```bash
//! cd examples/oxivgl-host
//! sudo ip link set can0 up type can bitrate 500000
//! cargo run --bin oxivgl_touch_can
//! ```

use core::ffi::CStr;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use log::info;
use oxivgl::driver::LvglDriver;
use oxivgl::view::{register_view_events, View};
use oxivgl::widgets::{Obj, Screen};
use oxivgl_sys::LV_DEF_REFR_PERIOD;
use static_cell::StaticCell;
use touch_hall_common::{CAN_BAUD, CAN_CHANNEL, HALL_NAME};

use embassy_oxivgl_host_examples::hall_view::HallView;
use embassy_oxivgl_host_examples::touch_can;

const DISPLAY_WIDTH: i32 = touch_hall_common::DISPLAY_WIDTH as i32;
const DISPLAY_HEIGHT: i32 = touch_hall_common::DISPLAY_HEIGHT as i32;
const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;
const TICKS_PER_FRAME: usize = 4;

static VIEW: StaticCell<HallView> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_millis()
        .init();

    info!(
        "OxivGL hall CAN host demo (SDL {}x{}) — {} on {CAN_CHANNEL} @ {CAN_BAUD} bit/s",
        DISPLAY_WIDTH, DISPLAY_HEIGHT, HALL_NAME
    );

    touch_can::start();

    let title = CStr::from_bytes_with_nul(b"Sporthallen Tableau\0").unwrap();
    let driver = LvglDriver::sdl(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .title(title)
        .mouse(true)
        .build();

    let view = VIEW.init(HallView::default());
    let screen = Screen::active().expect("LVGL screen must exist after SDL init");
    let container = Obj::from_raw_non_owning(screen.handle());
    view.create(&container).expect("hall view create failed");
    register_view_events(view);

    info!("SDL mouse indev active — hold buttons to send CAN commands");

    loop {
        let _ = view.update();
        for _ in 0..TICKS_PER_FRAME {
            driver.timer_handler();
            Timer::after(Duration::from_millis(LVGL_TICK_MS)).await;
        }
    }
}
