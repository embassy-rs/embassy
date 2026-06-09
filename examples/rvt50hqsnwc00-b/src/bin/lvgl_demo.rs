#![no_std]
#![no_main]
#![allow(static_mut_refs)]

/// Display demo for Riverdi RVT50HQSNWN00, aligned with
/// [lv_port_riverdi_stm32u5](https://github.com/lvgl/lv_port_riverdi_stm32u5).
///
/// Uses correct panel timing, LTDC polarities, touch I2C, and a widget-style
/// framebuffer UI. Full LVGL (`lv_demo_benchmark`, etc.) can be added later
/// via the optional `lvgl` feature once `lv_conf.h` is configured.
///
/// Run with: `cargo run --bin lvgl_demo`

use core::fmt::Write;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{self, TouchPoint, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_stm32::ltdc::{LtdcLayer, LtdcLayerConfig};
use embassy_stm32::ltdc::{self, Ltdc};
use embassy_stm32::peripherals;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

pub static mut FB1: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
pub static mut FB2: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

struct Rgb565;

impl Rgb565 {
    const WHITE: u16 = 0xFFFF;
    const BLUE: u16 = 0x001F;
    const GREEN: u16 = 0x07E0;
    const RED: u16 = 0xF800;
    const GRAY: u16 = 0x8410;
    const DARK: u16 = 0x1082;
    const ACCENT: u16 = 0x4A69;
    const PANEL: u16 = 0x2104;
    const BTN: u16 = 0x0410;
    const BTN_BORDER: u16 = 0x001F;
}

fn fill_rect(buffer: &mut [u16], width: usize, x: usize, y: usize, w: usize, h: usize, color: u16) {
    if w == 0 || h == 0 {
        return;
    }
    for dy in 0..h {
        let py = y + dy;
        if py >= DISPLAY_HEIGHT {
            break;
        }
        let row = py * width;
        for dx in 0..w {
            let px = x + dx;
            if px < width {
                buffer[row + px] = color;
            }
        }
    }
}

fn draw_text(buffer: &mut [u16], width: usize, x: usize, y: usize, text: &str, color: u16) {
    let mut cx = x;
    for _c in text.chars() {
        fill_rect(buffer, width, cx, y, 8, 12, color);
        cx += 10;
    }
}

fn draw_hline(buffer: &mut [u16], width: usize, x: usize, y: usize, len: usize, color: u16) {
    fill_rect(buffer, width, x, y, len, 1, color);
}

fn draw_touch_crosshair(buffer: &mut [u16], width: usize, touch: TouchPoint) {
    if !touch.pressed {
        return;
    }
    let x = touch.x as usize;
    let y = touch.y as usize;
    fill_rect(buffer, width, x.saturating_sub(15), y, 31, 2, Rgb565::WHITE);
    fill_rect(buffer, width, x, y.saturating_sub(15), 2, 31, Rgb565::WHITE);
    fill_rect(buffer, width, x.saturating_sub(3), y.saturating_sub(3), 7, 7, Rgb565::RED);
}

fn draw_demo_ui(buffer: &mut [u16], width: usize, frame: u32, touch: TouchPoint) {
    fill_rect(buffer, width, 0, 0, width, DISPLAY_HEIGHT, Rgb565::DARK);

    // Title bar (LVGL-style header)
    fill_rect(buffer, width, 0, 0, width, 48, Rgb565::ACCENT);
    draw_text(buffer, width, 16, 16, "RVT50HQSNWC00-B", Rgb565::WHITE);
    draw_text(buffer, width, 280, 16, "Embassy Demo", Rgb565::WHITE);

    // Widget panel
    fill_rect(buffer, width, 16, 64, 768, 400, Rgb565::PANEL);
    draw_hline(buffer, width, 16, 64, 768, Rgb565::GRAY);

    draw_text(buffer, width, 32, 80, "Inspired by lv_port_riverdi_stm32u5", Rgb565::WHITE);

    // Button
    fill_rect(buffer, width, 32, 120, 200, 52, Rgb565::BTN_BORDER);
    fill_rect(buffer, width, 36, 124, 192, 44, Rgb565::BTN);
    draw_text(buffer, width, 88, 140, "Button", Rgb565::WHITE);

    // Slider
    fill_rect(buffer, width, 32, 200, 320, 10, Rgb565::GRAY);
    let thumb = 32 + ((frame * 4) % 300) as usize;
    fill_rect(buffer, width, thumb, 192, 24, 28, Rgb565::GREEN);

    // Progress bar
    fill_rect(buffer, width, 32, 260, 320, 24, Rgb565::GRAY);
    let progress = ((frame * 2) % 320) as usize;
    fill_rect(buffer, width, 32, 260, progress, 24, Rgb565::RED);

    // Checkbox
    fill_rect(buffer, width, 32, 310, 28, 28, Rgb565::WHITE);
    if frame % 90 < 45 {
        fill_rect(buffer, width, 38, 316, 16, 16, Rgb565::GREEN);
    }
    draw_text(buffer, width, 68, 318, "Toggle", Rgb565::WHITE);

    // Color swatches (like LVGL theme preview)
    const SWATCHES: [u16; 6] = [Rgb565::RED, Rgb565::GREEN, Rgb565::BLUE, 0xFFE0, 0xF81F, Rgb565::WHITE];
    for (i, color) in SWATCHES.iter().enumerate() {
        fill_rect(buffer, width, 400 + i * 56, 120, 48, 48, *color);
    }

    // Arc gauge (ring segment, integer math)
    let cx = 680usize;
    let cy = 280usize;
    let value = (frame * 2) % 360;
    for dy in 0..120 {
        for dx in 0..120 {
            let px = cx + dx;
            let py = cy + dy;
            let dist = ((dx as i32 - 60).pow(2) + (dy as i32 - 60).pow(2)) as u32;
            if dist < 60u32.pow(2) && dist > 50u32.pow(2) && dx > 60 {
                let angle = (dy as u32 * 360) / 120;
                if angle <= value {
                    fill_rect(buffer, width, px, py, 2, 2, 0xFFE0);
                }
            }
        }
    }
    fill_rect(buffer, width, cx + 30, cy + 52, 60, 16, Rgb565::PANEL);
    let mut pct = heapless::String::<8>::new();
    let _ = write!(pct, "{}%", (value * 100 / 360));
    draw_text(buffer, width, cx + 40, cy + 54, pct.as_str(), Rgb565::WHITE);

    // FPS / frame counter
    let mut stats = heapless::String::<32>::new();
    let _ = write!(stats, "Frame: {}", frame);
    draw_text(buffer, width, 32, 360, stats.as_str(), Rgb565::WHITE);

    if touch.pressed {
        let mut pos = heapless::String::<32>::new();
        let _ = write!(pos, "Touch: {}, {}", touch.x, touch.y);
        draw_text(buffer, width, 32, 380, pos.as_str(), Rgb565::GREEN);
        draw_touch_crosshair(buffer, width, touch);
    } else {
        draw_text(buffer, width, 32, 380, "No touch panel", Rgb565::GRAY);
    }
}

#[embassy_executor::task]
async fn demo_task(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    #[cfg(feature = "touch")] mut i2c: embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Blocking,
        embassy_stm32::i2c::Master,
    >,
) {
    info!("Starting Riverdi display demo");

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
        #[cfg(feature = "touch")]
        let touch = rvt50_board::read_touch(&mut i2c);
        #[cfg(not(feature = "touch"))]
        let touch = TouchPoint::default();

        let buffer = if use_fb1 {
            unsafe { FB1.as_mut_ptr() }
        } else {
            unsafe { FB2.as_mut_ptr() }
        };

        unsafe {
            let slice = core::slice::from_raw_parts_mut(buffer, DISPLAY_WIDTH * DISPLAY_HEIGHT);
            draw_demo_ui(slice, DISPLAY_WIDTH, frame, touch);
        }

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

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Riverdi RVT50HQSNWN00 Display Demo");
    info!("Based on lv_port_riverdi_stm32u5 hardware config");

    let p = rvt50_board::init_clocks();
    rvt50_board::enable_icache();

    #[cfg(feature = "touch")]
    let rvt50_board::DisplayResources { ltdc, i2c } = rvt50_board::init_display(p).await;
    #[cfg(not(feature = "touch"))]
    let rvt50_board::DisplayResources { ltdc } = rvt50_board::init_display(p).await;

    #[cfg(feature = "touch")]
    spawner.spawn(unwrap!(demo_task(ltdc, i2c)));
    #[cfg(not(feature = "touch"))]
    spawner.spawn(unwrap!(demo_task(ltdc)));

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
