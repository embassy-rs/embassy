#![no_std]
#![no_main]
#![allow(static_mut_refs)]

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

use core::fmt::Write;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig};
use embassy_stm32::peripherals;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

// Frame buffer for LTDC - using RGB565 format (2 bytes per pixel)
// Total size: 800 * 480 * 2 = 768,000 bytes per buffer
// With double buffering: 1,536,000 bytes (~1.5MB)
pub static mut FB1: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
pub static mut FB2: [u16; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

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
        for _c in text.chars() {
            // Draw a small rectangle for each character
            self.draw_rect(cx, y, 8, 12, color);
            cx += 10;
        }
    }
}

/// Display task - handles LTDC and framebuffer management
#[embassy_executor::task]
async fn display_task(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
) {
    info!("Starting display task");

    // Configure LTDC layer
    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::Rgb565,
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
        let mut counter_str = heapless::String::<24>::new();
        let _ = write!(counter_str, "Counter: {}", counter);
        display.draw_text(20, 140, counter_str.as_str(), 0xFFFF);

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

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Riverdi RVT50HQSNWN00 - Simple Display Demo");

    let p = rvt50_board::init_clocks();
    rvt50_board::enable_icache();

    let rvt50_board::DisplayResources { ltdc } = rvt50_board::init_display(p).await;

    spawner.spawn(unwrap!(display_task(ltdc)));

    info!("Tasks spawned, entering idle loop");

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
