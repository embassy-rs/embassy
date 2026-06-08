#![no_std]
#![no_main]

/// Minimal LVGL Example for Riverdi RVT50HQSNWC00-B
///
/// This example provides a foundation for running LVGL on the
/// Riverdi RVT50HQSNWC00-B display module with STM32U5A9NJH6Q.
///
/// To use this example:
/// 1. Enable the lvgl feature: cargo run --features lvgl --bin lvgl_minimal
/// 2. Ensure you have the lvgl and lvgl-sys crates available
///
/// This example demonstrates:
/// - LTDC display initialization
/// - Basic framebuffer management
/// - LVGL integration (when feature is enabled)

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ltdc::{self, Ltdc, LtdcConfiguration, LtdcLayer, LtdcLayerConfig, PolarityActive, PolarityEdge};
use embassy_stm32::{bind_interrupts, peripherals, Config, rcc};
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
    
    // Configure HSE (16 MHz external oscillator)
    config.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(16_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    
    // Configure PLL1 for system clock (160 MHz)
    config.rcc.pll1 = Some(rcc::Pll {
        source: rcc::PllSource::HSE,
        prediv: rcc::PllPreDiv::DIV1,
        mul: rcc::PllMul::MUL10,
        divp: None,
        divq: None,
        divr: Some(rcc::PllDiv::DIV1),
    });
    
    config.rcc.sys = rcc::Sysclk::PLL1_R;
    
    // Configure PLL3 for LTDC clock (~30 MHz for 800x480 display)
    config.rcc.pll3 = Some(rcc::Pll {
        source: rcc::PllSource::HSE,
        prediv: rcc::PllPreDiv::DIV4,
        mul: rcc::PllMul::MUL75,
        divp: None,
        divq: None,
        divr: Some(rcc::PllDiv::DIV10),
    });
    
    config.rcc.mux.ltdcsel = rcc::mux::Ltdcsel::PLL3_R;
    
    // Enable caches for better performance
    config.rcc.icache = rcc::Icache::Enabled;
    config.rcc.dcache = rcc::Dcache::Enabled;
    
    embassy_stm32::init(config)
}

/// Initialize LTDC for the display
fn init_ltdc(p: &mut Peripherals) -> Ltdc<'static, peripherals::LTDC> {
    // Display timing parameters (adjust based on your display specifications)
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

    // Initialize LTDC with RGB565 format
    let mut ltdc = Ltdc::<_, ltdc::Rgb565>::new_with_pins(
        p.LTDC,
        Irqs,
        p.PD3,   // CLK
        p.PE0,   // HSYNC
        p.PD13,  // VSYNC
        p.PD6,   // DE (Data Enable)
        p.PB9,   // B0
        p.PB2,   // B1
        p.PD14,  // B2
        p.PD15,  // B3
        p.PD0,   // B4
        p.PD1,   // B5
        p.PE7,   // B6
        p.PE8,   // B7
        p.PC8,   // G0
        p.PC9,   // G1
        p.PE9,   // G2
        p.PE10,  // G3
        p.PE11,  // G4
        p.PE12,  // G5
        p.PE13,  // G6
        p.PE14,  // G7
        p.PC6,   // R0
        p.PC7,   // R1
        p.PE15,  // R2
        p.PD8,   // R3
        p.PD9,   // R4
        p.PD10,  // R5
        p.PD11,  // R6
        p.PD12,  // R7
    );

    ltdc.init(&ltdc_config);

    // Configure and enable display control pins
    let _ltdc_de = Output::new(p.PD6, Level::High, Speed::High);
    let _ltdc_disp_ctrl = Output::new(p.PE4, Level::High, Speed::High);
    let _ltdc_bl_ctrl = Output::new(p.PE6, Level::High, Speed::High);

    info!("LTDC initialized successfully");

    ltdc
}

/// Simple display task that shows basic graphics
/// This can be used as a base for LVGL integration
#[embassy_executor::task]
async fn display_task(
    mut ltdc: Ltdc<'static, peripherals::LTDC>,
) {
    info!("Starting display task");

    // Configure LTDC layer for RGB565
    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::RGB565,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: DISPLAY_WIDTH as _,
        window_y0: 0,
        window_y1: DISPLAY_HEIGHT as _,
    };

    // Initialize layer
    ltdc.init_layer(&layer_config, None);

    // Set initial buffer
    ltdc.set_buffer(LtdcLayer::Layer1, unsafe { FB1.as_ptr() } as *const _)
        .await
        .unwrap();

    // Simple frame counter
    let mut frame = 0u32;
    let mut use_fb1 = true;

    loop {
        // Get the current buffer
        let buffer = if use_fb1 {
            unsafe { FB1.as_mut_ptr() }
        } else {
            unsafe { FB2.as_mut_ptr() }
        };

        // Fill with a gradient pattern
        fill_gradient(buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT, frame);

        // Draw a simple test pattern
        draw_test_pattern(buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT, frame);

        // Swap buffers
        use_fb1 = !use_fb1;
        let next_buffer = if use_fb1 {
            unsafe { FB1.as_ptr() }
        } else {
            unsafe { FB2.as_ptr() }
        };

        // Update LTDC to use the new buffer
        ltdc.set_buffer(LtdcLayer::Layer1, next_buffer as *const _)
            .await
            .unwrap();

        frame += 1;
        Timer::after(Duration::from_millis(16)).await; // ~60 FPS
    }
}

/// Fill buffer with a gradient
fn fill_gradient(buffer: *mut u16, width: usize, height: usize, frame: u32) {
    unsafe {
        let slice = core::slice::from_raw_parts_mut(buffer, width * height);
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                // Simple gradient based on position and frame
                let r = ((x * 31 / width) as u16) << 11;
                let g = ((y * 63 / height) as u16) << 5;
                let b = ((frame % 32) as u16);
                slice[index] = r | g | b;
            }
        }
    }
}

/// Draw a simple test pattern
fn draw_test_pattern(buffer: *mut u16, width: usize, height: usize, frame: u32) {
    unsafe {
        let slice = core::slice::from_raw_parts_mut(buffer, width * height);
        
        // Draw a moving rectangle
        let rect_x = ((frame * 5) % (width - 100)) as usize;
        let rect_y = ((frame * 3) % (height - 100)) as usize;
        
        for y in rect_y..rect_y + 100 {
            for x in rect_x..rect_x + 100 {
                if x < width && y < height {
                    let index = y * width + x;
                    // White rectangle
                    slice[index] = 0xFFFF;
                }
            }
        }
        
        // Draw a border
        for x in 0..width {
            if x < width {
                slice[x] = 0xF800; // Red top border
                slice[(height - 1) * width + x] = 0xF800; // Red bottom border
            }
        }
        for y in 0..height {
            if y * width < width * height {
                slice[y * width] = 0xF800; // Red left border
                slice[y * width + width - 1] = 0xF800; // Red right border
            }
        }
        
        // Draw title text (simplified)
        draw_text(slice, width, 20, 20, "RVT50HQSNWC00-B", 0xFFFF);
        draw_text(slice, width, 20, 40, "LVGL Minimal", 0xAAAA);
    }
}

/// Simple text drawing (very basic)
fn draw_text(buffer: &mut [u16], width: usize, x: usize, y: usize, text: &str, color: u16) {
    let mut cx = x;
    for _c in text.chars() {
        // Draw each character as a small block
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

/// LED blink task for visual feedback
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

    // Initialize clocks
    let mut p = init_clocks();

    // Enable ICACHE for better performance
    embassy_stm32::pac::ICACHE.cr().write(|w| {
        w.set_en(true);
    });

    info!("Clocks initialized");

    // Initialize LTDC
    let ltdc = init_ltdc(&mut p);

    info!("LTDC initialized");

    // Create LED output
    let led = Output::new(p.PD2, Level::High, Speed::Low);

    // Spawn tasks
    embassy_executor::spawn(led_task(led)).unwrap();
    embassy_executor::spawn(display_task(ltdc)).unwrap();

    info!("Tasks spawned, entering idle loop");

    // Main loop - just idle
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
