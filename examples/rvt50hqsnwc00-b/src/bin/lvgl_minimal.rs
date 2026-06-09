#![no_std]
#![no_main]
#![allow(static_mut_refs)]

/// Display example for Riverdi RVT50HQSNWC00-B.
///
/// Without features: framebuffer gradient demo.
/// With `--features lvgl`: minimal LVGL label + touch input.
///
/// ```bash
/// cargo run --bin lvgl_minimal
/// cargo run --bin lvgl_minimal --features lvgl
/// ```

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig};
use embassy_stm32::peripherals;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

pub static mut FB1: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
pub static mut FB2: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

#[cfg(not(feature = "lvgl"))]
mod fb_demo {
    use super::*;

    pub async fn run(spawner: Spawner) -> ! {
        info!("Riverdi RVT50HQSNWC00-B - framebuffer minimal demo");

        let p = rvt50_board::init_clocks();
        rvt50_board::enable_icache();
        let rvt50_board::DisplayResources { ltdc, led, i2c: _ } = rvt50_board::init_display(p).await;

        let led = Output::new(led, Level::High, Speed::Low);
        spawner.spawn(unwrap!(super::led_task(led)));
        spawner.spawn(unwrap!(display_task(ltdc)));

        idle_loop().await
    }

    #[embassy_executor::task]
    async fn display_task(mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>) {
        info!("Starting display task");

        let layer_config = LtdcLayerConfig {
            pixel_format: ltdc::PixelFormat::Rgb565,
            layer: LtdcLayer::Layer1,
            window_x0: 0,
            window_x1: DISPLAY_WIDTH as _,
            window_y0: 0,
            window_y1: DISPLAY_HEIGHT as _,
        };

        ltdc.init_layer(&layer_config, None);
        ltdc.set_buffer(LtdcLayer::Layer1, unsafe { FB1.as_ptr() } as *const _)
            .await
            .unwrap();

        let mut frame = 0u32;
        let mut use_fb1 = true;

        loop {
            let buffer = if use_fb1 {
                unsafe { FB1.as_mut_ptr() }
            } else {
                unsafe { FB2.as_mut_ptr() }
            };

            fill_gradient(buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT, frame);
            draw_test_pattern(buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT, frame);

            use_fb1 = !use_fb1;
            let next_buffer = if use_fb1 {
                unsafe { FB1.as_ptr() }
            } else {
                unsafe { FB2.as_ptr() }
            };

            ltdc.set_buffer(LtdcLayer::Layer1, next_buffer as *const _)
                .await
                .unwrap();

            frame += 1;
            Timer::after(Duration::from_millis(16)).await;
        }
    }

    fn fill_gradient(buffer: *mut u16, width: usize, height: usize, frame: u32) {
        unsafe {
            let slice = core::slice::from_raw_parts_mut(buffer, width * height);
            for y in 0..height {
                for x in 0..width {
                    let index = y * width + x;
                    let r = ((x * 31 / width) as u16) << 11;
                    let g = ((y * 63 / height) as u16) << 5;
                    let b = (frame % 32) as u16;
                    slice[index] = r | g | b;
                }
            }
        }
    }

    fn draw_test_pattern(buffer: *mut u16, width: usize, height: usize, frame: u32) {
        unsafe {
            let slice = core::slice::from_raw_parts_mut(buffer, width * height);
            let rect_x = (frame as usize * 5) % (width - 100);
            let rect_y = (frame as usize * 3) % (height - 100);

            for y in rect_y..rect_y + 100 {
                for x in rect_x..rect_x + 100 {
                    if x < width && y < height {
                        slice[y * width + x] = 0xFFFF;
                    }
                }
            }

            for x in 0..width {
                slice[x] = 0xF800;
                slice[(height - 1) * width + x] = 0xF800;
            }
            for y in 0..height {
                slice[y * width] = 0xF800;
                slice[y * width + width - 1] = 0xF800;
            }

            draw_text(slice, width, 20, 20, "RVT50HQSNWC00-B", 0xFFFF);
            draw_text(slice, width, 20, 40, "LVGL Minimal", 0xAAAA);
        }
    }

    fn draw_text(buffer: &mut [u16], width: usize, x: usize, y: usize, text: &str, color: u16) {
        let mut cx = x;
        for _c in text.chars() {
            for dy in 0..12 {
                for dx in 0..8 {
                    let px = cx + dx;
                    let py = y + dy;
                    if px < width && py < buffer.len() / width {
                        buffer[py * width + px] = color;
                    }
                }
            }
            cx += 10;
        }
    }
}

#[cfg(feature = "lvgl")]
mod lvgl_ui {
    use super::*;
    use core::ffi::c_uint;

    // Force linking the LVGL static library built by lvgl-sys.
    use lvgl_sys as _;

    unsafe extern "C" {
        fn rvt50_lvgl_init(framebuffer: *mut u16, width: u16, height: u16);
        fn rvt50_lvgl_set_touch(x: u16, y: u16, pressed: bool);
        fn rvt50_lvgl_tick(ms: c_uint);
        fn rvt50_lvgl_handler();
    }

    pub async fn run(spawner: Spawner) -> ! {
        info!("Riverdi RVT50HQSNWC00-B - LVGL minimal demo");

        let p = rvt50_board::init_clocks();
        rvt50_board::enable_icache();
        let rvt50_board::DisplayResources { ltdc, led, i2c } = rvt50_board::init_display(p).await;

        let led = Output::new(led, Level::High, Speed::Low);
        spawner.spawn(unwrap!(super::led_task(led)));
        spawner.spawn(unwrap!(lvgl_task(ltdc, i2c)));

        idle_loop().await
    }

    #[embassy_executor::task]
    async fn lvgl_task(
        mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
        mut i2c: embassy_stm32::i2c::I2c<'static, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>,
    ) {
        info!("Starting LVGL task");

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
            rvt50_lvgl_init(FB1.as_mut_ptr(), DISPLAY_WIDTH as u16, DISPLAY_HEIGHT as u16);
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
}

async fn idle_loop() -> ! {
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
    let mut counter = 0;
    loop {
        led.set_low();
        Timer::after(Duration::from_millis(50)).await;
        led.set_high();
        Timer::after(Duration::from_millis(450)).await;
        info!("LED blink: {}", counter);
        counter += 1;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    #[cfg(not(feature = "lvgl"))]
    return fb_demo::run(spawner).await;

    #[cfg(feature = "lvgl")]
    return lvgl_ui::run(spawner).await;
}
