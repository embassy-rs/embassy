#![no_std]
#![no_main]
#![allow(static_mut_refs)]

/// Minimal LVGL Example for Riverdi RVT50HQSNWC00-B
///
/// This example provides a foundation for running LVGL on the
/// Riverdi RVT50HQSNWC00-B display module with STM32U5A9NJH6Q.
///
/// This example demonstrates:
/// - LTDC display initialization
/// - Basic framebuffer management
/// - Double-buffered test graphics
///
/// Run with: `cargo run --bin lvgl_minimal`

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ltdc::{self, Ltdc, LtdcConfiguration, LtdcLayer, LtdcLayerConfig, PolarityActive, PolarityEdge};
use embassy_stm32::{bind_interrupts, peripherals, Config, Peripherals, rcc};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

// Display constants
const DISPLAY_WIDTH: usize = 800;
const DISPLAY_HEIGHT: usize = 480;

// Frame buffers for double buffering (RGB565 format)
pub static mut FB1: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
pub static mut FB2: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

// Bind interrupts
bind_interrupts!(struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
});

/// Initialize clocks for STM32U5A9NJH6Q
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

/// Initialize LTDC for the display
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
        PD3,  // CLK
        PE0,  // HSYNC
        PD13, // VSYNC
        PD6,  // DE
        PD15, // B3
        PD0,  // B4
        PD1,  // B5
        PE7,  // B6
        PE8,  // B7
        PE9,  // G2
        PE10, // G3
        PE11, // G4
        PE12, // G5
        PE13, // G6
        PE14, // G7
        PD8,  // R3
        PD9,  // R4
        PD10, // R5
        PD11, // R6
        PD12, // R7
    );

    ltdc.init(&ltdc_config);

    let mut ltdc_disp_ctrl = Output::new(PE4, Level::Low, Speed::High);
    let mut ltdc_bl_ctrl = Output::new(PE6, Level::Low, Speed::High);
    ltdc_disp_ctrl.set_high();
    ltdc_bl_ctrl.set_high();

    info!("LTDC initialized successfully");

    (ltdc, PD2)
}

/// Simple display task that shows basic graphics
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
    info!("Riverdi RVT50HQSNWC00-B - LVGL Minimal Example");
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
    spawner.spawn(unwrap!(display_task(ltdc)));

    info!("Tasks spawned, entering idle loop");

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
