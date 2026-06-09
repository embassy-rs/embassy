#![no_std]
#![no_main]
#![allow(static_mut_refs)]

//! LVGL touch demo for Riverdi capacitive-touch variants (e.g. RVT50HQSNWC00).
//!
//! ```bash
//! cargo run --bin lvgl_touch --features lvgl,touch
//! ```

#[cfg(not(all(feature = "lvgl", feature = "touch")))]
compile_error!("lvgl_touch requires --features lvgl,touch");

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_stm32::i2c::I2c;
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig};
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

// Force linking the LVGL static library built by lvgl-sys.
use lvgl_sys as _;

static mut FB1: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

unsafe extern "C" {
    fn rvt50_lvgl_touch_demo_init(framebuffer: *mut u16, width: u16, height: u16);
    fn rvt50_lvgl_set_touch(x: u16, y: u16, pressed: bool);
    fn rvt50_lvgl_tick(ms: u32);
    fn rvt50_lvgl_handler();
}

#[embassy_executor::task]
async fn lvgl_touch_task(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    mut i2c: I2c<'static, Blocking, embassy_stm32::i2c::Master>,
) {
    info!("Starting LVGL touch demo");

    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::Rgb565,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: DISPLAY_WIDTH as _,
        window_y0: 0,
        window_y1: DISPLAY_HEIGHT as _,
    };
    ltdc.init_layer(&layer_config, None);

    unsafe {
        FB1.fill(0);
        rvt50_lvgl_touch_demo_init(FB1.as_mut_ptr(), DISPLAY_WIDTH as u16, DISPLAY_HEIGHT as u16);
    }

    ltdc.set_buffer(LtdcLayer::Layer1, unsafe { FB1.as_ptr() } as *const _)
        .await
        .unwrap();

    loop {
        let touch = rvt50_board::read_touch(&mut i2c);
        unsafe {
            rvt50_lvgl_set_touch(touch.x, touch.y, touch.pressed);
            rvt50_lvgl_tick(5);
            rvt50_lvgl_handler();
        }

        ltdc.set_buffer(LtdcLayer::Layer1, unsafe { FB1.as_ptr() } as *const _)
            .await
            .unwrap();

        Timer::after(Duration::from_millis(5)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Riverdi RVT50 - LVGL touch demo");

    let p = rvt50_board::init_clocks();
    rvt50_board::enable_icache();

    let rvt50_board::DisplayResources { ltdc, i2c } = rvt50_board::init_display(p).await;

    spawner.spawn(unwrap!(lvgl_touch_task(ltdc, i2c)));

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
