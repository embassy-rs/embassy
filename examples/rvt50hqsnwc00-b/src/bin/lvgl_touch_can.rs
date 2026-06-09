#![no_std]
#![no_main]
#![allow(static_mut_refs)]

//! JSON-driven LVGL hall lighting UI with CAN press/hold/repeat for Riverdi RVT50.
//!
//! - UI: [lv_binding_rust](https://github.com/lvgl/lv_binding_rust) + `src/lvgl/`
//! - Board: [riverdi-50-stm32u5-lvgl](https://github.com/riverdi/riverdi-50-stm32u5-lvgl) display/touch patterns
//! - Config: `touch-projects/SporthalleLudwigsfelde/*.json`
//!
//! ```bash
//! ./scripts/cargo-lvgl.sh run --bin lvgl_touch_can --features lvgl,touch
//! ```

#[cfg(not(all(feature = "lvgl", feature = "touch")))]
compile_error!("lvgl_touch_can requires --features lvgl,touch");

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::lvgl::{self, HallUi};
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{self, CAN_BITRATE};
use embassy_rvt50hqsnwc00_b_examples::touch_can::{self, on_button_press, on_button_release};
use embassy_rvt50hqsnwc00_b_examples::touch_config::{self, BUTTON_COUNT};
use embassy_stm32::can::{CanRx, CanTx};
use embassy_stm32::i2c::I2c;
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig};
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals;
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const FB_PIXELS: usize =
    touch_config::DISPLAY_WIDTH as usize * touch_config::DISPLAY_HEIGHT as usize;

static FB1: StaticCell<[u16; FB_PIXELS]> = StaticCell::new();
static CAN_TX: StaticCell<CanTx<'static>> = StaticCell::new();
static CAN_RX: StaticCell<CanRx<'static>> = StaticCell::new();
static HALL_UI: StaticCell<HallUi> = StaticCell::new();

#[embassy_executor::task]
async fn ui_task(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    mut i2c: I2c<'static, Blocking, embassy_stm32::i2c::Master>,
) {
    info!(
        "Hall UI {}x{} — {}",
        touch_config::DISPLAY_WIDTH,
        touch_config::DISPLAY_HEIGHT,
        touch_config::HALL_NAME,
    );

    ltdc.init_layer(
        &LtdcLayerConfig {
            pixel_format: ltdc::PixelFormat::Rgb565,
            layer: LtdcLayer::Layer1,
            window_x0: 0,
            window_x1: touch_config::DISPLAY_WIDTH as _,
            window_y0: 0,
            window_y1: touch_config::DISPLAY_HEIGHT as _,
        },
        None,
    );

    let fb = FB1.init([0; FB_PIXELS]);
    let fb_ptr = fb.as_mut_ptr();
    let ui = HALL_UI.init(
        HallUi::build(fb_ptr, on_button_press, on_button_release).expect("hall UI build"),
    );

    ltdc.set_buffer(LtdcLayer::Layer1, fb_ptr as *const _)
        .await
        .unwrap();

    loop {
        let touch = rvt50_board::read_touch(&mut i2c);
        ui.set_touch(touch.x, touch.y, touch.pressed);
        lvgl::tick_and_run(5);

        for i in 0..BUTTON_COUNT {
            ui.set_button_active(i, touch_can::button_status(i));
        }

        ltdc.set_buffer(LtdcLayer::Layer1, fb_ptr as *const _)
            .await
            .unwrap();

        Timer::after(Duration::from_millis(5)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Riverdi RVT50 — LVGL touch CAN");

    let p = rvt50_board::init_clocks();
    rvt50_board::enable_icache();

    let rvt50_board::TouchCanResources {
        ltdc,
        i2c,
        fdcan,
        can_rx_pin,
        can_tx_pin,
        can_stb,
    } = rvt50_board::init_touch_can(p).await;

    let mut can = rvt50_board::init_can(fdcan, can_rx_pin, can_tx_pin, can_stb);
    can.set_bitrate(CAN_BITRATE);
    let (tx, rx, _) = can.into_normal_mode().split();

    spawner.spawn(unwrap!(ui_task(ltdc, i2c)));
    spawner.spawn(unwrap!(touch_can::tx_task(CAN_TX.init(tx))));
    spawner.spawn(unwrap!(touch_can::rx_task(CAN_RX.init(rx))));

    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}
