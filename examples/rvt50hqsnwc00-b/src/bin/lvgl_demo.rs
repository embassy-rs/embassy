#![no_std]
#![no_main]
#![allow(static_mut_refs)]

/// Display demo for Riverdi RVT50HQSNWC00-B board
///
/// Demonstrates LTDC output with double buffering and a simple widget-like UI
/// drawn directly to the framebuffer. Full LVGL integration requires the `lvgl`
/// feature plus an `lv_conf.h` configuration — see README.md.
///
/// Run with: `cargo run --bin lvgl_demo`

use core::fmt::Write;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ltdc::{self, Ltdc, LtdcConfiguration, LtdcLayer, LtdcLayerConfig, PolarityActive, PolarityEdge};
use embassy_stm32::{bind_interrupts, peripherals, Config, Peripherals, rcc};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

const DISPLAY_WIDTH: usize = 800;
const DISPLAY_HEIGHT: usize = 480;

pub static mut FB1: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
pub static mut FB2: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

bind_interrupts!(struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
});

fn init_clocks() -> Peripherals {
    let mut config = Config::default();

    config.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(16_000_000),
        mode: rcc::HseMode::Oscillator,
    });

    config.rcc.pll1 = Some(rcc::Pll {
        source: rcc::PllSource::Hse,
        prediv: rcc::PllPreDiv::Div1,
        mul: rcc::PllMul::Mul10,
        divp: None,
        divq: None,
        divr: Some(rcc::PllDiv::Div1),
    });

    config.rcc.sys = rcc::Sysclk::Pll1R;

    config.rcc.pll3 = Some(rcc::Pll {
        source: rcc::PllSource::Hse,
        prediv: rcc::PllPreDiv::Div4,
        mul: rcc::PllMul::Mul75,
        divp: None,
        divq: None,
        divr: Some(rcc::PllDiv::Div10),
    });

    config.rcc.mux.ltdcsel = rcc::mux::Ltdcsel::Pll3R;

    embassy_stm32::init(config)
}

fn init_ltdc(p: Peripherals) -> (Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>, embassy_stm32::Peri<'static, peripherals::PD2>) {
    let Peripherals {
        LTDC,
        PD2,
        PD3,
        PE0,
        PD13,
        PD6,
        PD15,
        PD0,
        PD1,
        PE7,
        PE8,
        PE9,
        PE10,
        PE11,
        PE12,
        PE13,
        PE14,
        PD8,
        PD9,
        PD10,
        PD11,
        PD12,
        PE4,
        PE6,
        ..
    } = p;

    const H_SYNC: u16 = 5;
    const H_BACK_PORCH: u16 = 40;
    const H_FRONT_PORCH: u16 = 20;
    const V_SYNC: u16 = 5;
    const V_BACK_PORCH: u16 = 10;
    const V_FRONT_PORCH: u16 = 20;

    let ltdc_config = LtdcConfiguration {
        active_width: DISPLAY_WIDTH as _,
        active_height: DISPLAY_HEIGHT as _,
        h_back_porch: H_BACK_PORCH,
        h_front_porch: H_FRONT_PORCH,
        v_back_porch: V_BACK_PORCH,
        v_front_porch: V_FRONT_PORCH,
        h_sync: H_SYNC,
        v_sync: V_SYNC,
        h_sync_polarity: PolarityActive::ActiveLow,
        v_sync_polarity: PolarityActive::ActiveLow,
        data_enable_polarity: PolarityActive::ActiveHigh,
        pixel_clock_polarity: PolarityEdge::RisingEdge,
    };

    info!("Initializing LTDC...");

    let mut ltdc = Ltdc::<_, ltdc::Rgb565>::new_with_pins(
        LTDC,
        Irqs,
        PD3,
        PE0,
        PD13,
        PD6,
        PD15,
        PD0,
        PD1,
        PE7,
        PE8,
        PE9,
        PE10,
        PE11,
        PE12,
        PE13,
        PE14,
        PD8,
        PD9,
        PD10,
        PD11,
        PD12,
    );

    ltdc.init(&ltdc_config);

    let mut ltdc_disp_ctrl = Output::new(PE4, Level::Low, Speed::High);
    let mut ltdc_bl_ctrl = Output::new(PE6, Level::Low, Speed::High);
    ltdc_disp_ctrl.set_high();
    ltdc_bl_ctrl.set_high();

    info!("LTDC initialized successfully");

    (ltdc, PD2)
}

fn fill_rect(buffer: &mut [u16], width: usize, x: usize, y: usize, w: usize, h: usize, color: u16) {
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            if px < width && py < buffer.len() / width {
                buffer[py * width + px] = color;
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

fn draw_demo_ui(buffer: &mut [u16], width: usize, height: usize, counter: u32) {
    fill_rect(buffer, width, 0, 0, width, height, 0x0000);

    draw_text(buffer, width, 300, 20, "Riverdi RVT50HQSNWC00-B", 0xFFFF);
    draw_text(buffer, width, 300, 40, "Display Demo", 0xAAAA);

    // Button
    fill_rect(buffer, width, 300, 100, 200, 50, 0x001F);
    fill_rect(buffer, width, 304, 104, 192, 42, 0x0410);
    draw_text(buffer, width, 370, 118, "Click Me!", 0xFFFF);

    // Slider track and thumb
    fill_rect(buffer, width, 250, 180, 300, 8, 0x4208);
    let thumb_x = 250 + ((counter * 3) % 280) as usize;
    fill_rect(buffer, width, thumb_x, 172, 20, 24, 0x07E0);

    // Progress bar
    fill_rect(buffer, width, 250, 230, 300, 20, 0x4208);
    let bar_w = ((counter * 2) % 300) as usize;
    fill_rect(buffer, width, 250, 230, bar_w, 20, 0xF800);

    // Checkbox
    fill_rect(buffer, width, 250, 280, 24, 24, 0xFFFF);
    if counter % 60 < 30 {
        fill_rect(buffer, width, 256, 286, 12, 12, 0x07E0);
    }
    draw_text(buffer, width, 284, 286, "Enable Feature", 0xFFFF);

    // Counter
    let mut counter_str = heapless::String::<24>::new();
    let _ = write!(counter_str, "Counter: {}", counter);
    draw_text(buffer, width, 250, 330, counter_str.as_str(), 0xFFFF);

    // Arc-like indicator (filled wedge approximation)
    let cx = width - 125;
    let cy = 275;
    for dy in 0..150 {
        for dx in 0..150 {
            let px = cx + dx;
            let py = cy + dy;
            let dist = ((dx as i32 - 75).pow(2) + (dy as i32 - 75).pow(2)) as u32;
            if dist < 75u32.pow(2) && dist > 55u32.pow(2) && dx > 75 {
                if px < width && py < height {
                    buffer[py * width + px] = 0xFFE0;
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn demo_task(mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>) {
    info!("Starting display demo task");

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

    let mut counter = 0u32;
    let mut use_fb1 = true;

    loop {
        let buffer = if use_fb1 {
            unsafe { FB1.as_mut_ptr() }
        } else {
            unsafe { FB2.as_mut_ptr() }
        };

        unsafe {
            let slice = core::slice::from_raw_parts_mut(buffer, DISPLAY_WIDTH * DISPLAY_HEIGHT);
            draw_demo_ui(slice, DISPLAY_WIDTH, DISPLAY_HEIGHT, counter);
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

        counter += 1;
        Timer::after(Duration::from_millis(16)).await;
    }
}

#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
    let mut counter = 0;
    loop {
        info!("LED blink: {}", counter);
        counter += 1;

        led.set_low();
        Timer::after(Duration::from_millis(50)).await;

        led.set_high();
        Timer::after(Duration::from_millis(450)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Riverdi RVT50HQSNWC00-B Display Demo");
    info!("MCU: STM32U5A9NJH6Q");
    info!("Display: 800x480 RGB LCD");

    let p = init_clocks();

    embassy_stm32::pac::ICACHE.cr().write(|w| {
        w.set_en(true);
    });

    info!("Clocks initialized");

    let (ltdc, pd2) = init_ltdc(p);

    info!("LTDC initialized");

    let led = Output::new(pd2, Level::High, Speed::Low);

    spawner.spawn(unwrap!(led_task(led)));
    spawner.spawn(unwrap!(demo_task(ltdc)));

    info!("Tasks spawned, entering idle loop");

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
