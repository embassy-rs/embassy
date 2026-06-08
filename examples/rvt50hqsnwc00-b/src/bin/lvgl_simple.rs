#![no_std]
#![no_main]

/// Simple LVGL Demo for Riverdi RVT50HQSNWC00-B board
///
/// This is a simplified LVGL example that demonstrates the basics of
/// running LVGL on the Riverdi RVT50HQSNWC00-B display module.
///
/// This example uses:
/// - LTDC for display output
/// - Double buffering for smooth rendering
/// - Basic LVGL widgets
///
/// Note: This example requires the lvgl crate and may need adjustments
/// based on your specific hardware configuration.

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

// Frame buffer for LTDC - using RGB565 format (2 bytes per pixel)
// Total size: 800 * 480 * 2 = 768,000 bytes per buffer
// With double buffering: 1,536,000 bytes (~1.5MB)
pub static mut FB1: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
pub static mut FB2: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

// Bind interrupts
bind_interrupts!(struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
});

/// Simple framebuffer-based display for testing
/// This can be used to test display output before integrating LVGL
struct SimpleDisplay {
    buffer: *mut u16,
    width: usize,
    height: usize,
}

impl SimpleDisplay {
    pub fn new(buffer: *mut u16, width: usize, height: usize) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    /// Fill the display with a solid color
    pub fn fill(&mut self, color: u16) {
        unsafe {
            let slice = core::slice::from_raw_parts_mut(self.buffer, self.width * self.height);
            for pixel in slice.iter_mut() {
                *pixel = color;
            }
        }
    }

    /// Draw a pixel
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u16) {
        if x < self.width && y < self.height {
            unsafe {
                let offset = y * self.width + x;
                let ptr = self.buffer.add(offset);
                *ptr = color;
            }
        }
    }

    /// Draw a rectangle
    pub fn draw_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u16) {
        for dy in 0..h {
            for dx in 0..w {
                self.draw_pixel(x + dx, y + dy, color);
            }
        }
    }

    /// Draw text (very simple implementation)
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, color: u16) {
        // Very simple text rendering - just draw each character as a small block
        let mut cx = x;
        for c in text.chars() {
            // Draw a small rectangle for each character
            self.draw_rect(cx, y, 8, 12, color);
            cx += 10;
        }
    }
}

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
    
    // Configure PLL3 for LTDC clock (~30 MHz)
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
    // Display timing parameters
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
        p.PD6,   // DE
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

/// Display task - handles LTDC and framebuffer management
#[embassy_executor::task]
async fn display_task(
    mut ltdc: Ltdc<'static, peripherals::LTDC>,
) {
    info!("Starting display task");

    // Configure LTDC layer
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

    // Create simple display
    let mut display = SimpleDisplay::new(unsafe { FB1.as_mut_ptr() }, DISPLAY_WIDTH, DISPLAY_HEIGHT);

    // Set initial buffer
    ltdc.set_buffer(LtdcLayer::Layer1, unsafe { FB1.as_ptr() } as *const _)
        .await
        .unwrap();

    // Test pattern: draw some colors and shapes
    let mut counter = 0;
    loop {
        // Fill with black
        display.fill(0x0000);

        // Draw a title
        display.draw_text(20, 20, "Riverdi RVT50HQSNWC00-B", 0xFFFF);
        display.draw_text(20, 40, "LVGL Simple Demo", 0xAAAA);

        // Draw some colored rectangles
        display.draw_rect(20, 60, 100, 60, 0xF800); // Red
        display.draw_rect(140, 60, 100, 60, 0x07E0); // Green
        display.draw_rect(260, 60, 100, 60, 0x001F); // Blue

        // Draw a counter
        let counter_str = format!("Counter: {}", counter);
        display.draw_text(20, 140, &counter_str, 0xFFFF);

        // Draw a bouncing box
        let x = ((counter * 7) % (DISPLAY_WIDTH - 100)) as usize;
        let y = ((counter * 11) % (DISPLAY_HEIGHT - 100)) as usize;
        display.draw_rect(x, y, 100, 100, 0xFFFF);

        // Swap buffers
        let next_buffer = if counter % 2 == 0 {
            unsafe { FB2.as_mut_ptr() }
        } else {
            unsafe { FB1.as_mut_ptr() }
        };

        // Update display
        display.buffer = next_buffer;
        ltdc.set_buffer(LtdcLayer::Layer1, next_buffer as *const _)
            .await
            .unwrap();

        counter += 1;
        Timer::after(Duration::from_millis(16)).await; // ~60 FPS
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
    info!("Riverdi RVT50HQSNWC00-B - Simple Display Demo");
    info!("MCU: STM32U5A9NJH6Q");
    info!("Display: 800x480 RGB LCD");

    // Initialize clocks
    let mut p = init_clocks();

    // Enable ICACHE
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
