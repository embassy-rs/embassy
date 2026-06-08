#![no_std]
#![no_main]

/// LVGL Demo for Riverdi RVT50HQSNWC00-B board
///
/// This example demonstrates LVGL graphics library running on the
/// Riverdi RVT50HQSNWC00-B display module with STM32U5A9NJH6Q.
///
/// Features:
/// - LTDC display controller for RGB interface
/// - Double buffering for smooth animation
/// - Basic LVGL widgets (buttons, labels, etc.)
/// - Touch input support (I2C)
///
/// Display: 5.0" TFT LCD, 800x480 pixels
/// MCU: STM32U5A9NJH6Q (Cortex-M33, 160 MHz)
///
/// Note: This example requires the lvgl and lvgl-sys crates.

use defmt::{info, unwrap, Format};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed, Input, Pull};
use embassy_stm32::i2c::{I2c, Config as I2cConfig, Instance};
use embassy_stm32::ltdc::{self, Ltdc, LtdcConfiguration, LtdcLayer, LtdcLayerConfig, PolarityActive, PolarityEdge};
use embassy_stm32::{bind_interrupts, peripherals, Config as Stm32Config, rcc};
use embassy_time::{Duration, Timer};
use embedded_hal::blocking::i2c::{Write, WriteRead};
use {defmt_rtt as _, panic_probe as _};

// LVGL imports
use lvgl::{
    Align, Color, Display, DrawBuffer, HorizontalAlign, InputDevice, Point, Style, Widget,
    ui::Screen,
};
use lvgl::widgets::{Button, Label, Bar, Arc, Checkbox, Slider, Switch, TextArea, DropDown, Roller};

// Display constants for RVT50HQSNWC00-B
const DISPLAY_WIDTH: usize = 800;
const DISPLAY_HEIGHT: usize = 480;
const DISPLAY_HOR_RES: u16 = DISPLAY_WIDTH as u16;
const DISPLAY_VER_RES: u16 = DISPLAY_HEIGHT as u16;

// LVGL configuration
const LVGL_HOR_RES_MAX: u16 = DISPLAY_HOR_RES;
const LVGL_VER_RES_MAX: u16 = DISPLAY_VER_RES;
const LVGL_DPI: u8 = 160; // Approximate DPI for 5" display

// Frame buffer size - using RGB565 format (2 bytes per pixel)
// For double buffering, we need 2 * width * height * bytes_per_pixel
// With RGB565: 2 * 800 * 480 * 2 = 1,536,000 bytes (~1.5MB)
// The STM32U5A9NJH6Q has 2.5MB RAM, so this should fit
const FB_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

// Static frame buffers for double buffering
// Using RGB565 format (16 bits per pixel)
pub static mut FB1: [u16; FB_SIZE] = [0; FB_SIZE];
pub static mut FB2: [u16; FB_SIZE] = [0; FB_SIZE];

// LVGL draw buffer - smaller buffer for LVGL to draw into
// This is used by LVGL for rendering before copying to the main framebuffer
const LVGL_DRAW_BUF_SIZE: usize = DISPLAY_WIDTH * 40; // 40 lines at a time
pub static mut LVGL_DRAW_BUF: [u16; LVGL_DRAW_BUF_SIZE] = [0; LVGL_DRAW_BUF_SIZE];

// Touch screen I2C address (typical for FT5x06/GT911 touch controllers)
const TOUCH_I2C_ADDRESS: u8 = 0x38; // Common address for Riverdi touch controllers

// Bind interrupts
bind_interrupts!(struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
    I2C1 => embassy_stm32::i2c::InterruptHandler<peripherals::I2C1>;
});

/// Touch screen point structure
#[derive(Debug, Clone, Copy, Format)]
struct TouchPoint {
    x: u16,
    y: u16,
    pressed: bool,
}

/// Custom display driver for LVGL that uses LTDC
struct LtdcDisplay {
    ltdc: Ltdc<'static, peripherals::LTDC>,
    layer_config: LtdcLayerConfig,
    current_buffer: *mut u16,
    buffer_size: usize,
}

impl LtdcDisplay {
    pub fn new(
        ltdc: Ltdc<'static, peripherals::LTDC>,
        layer_config: LtdcLayerConfig,
        buffer: *mut u16,
        buffer_size: usize,
    ) -> Self {
        Self {
            ltdc,
            layer_config,
            current_buffer: buffer,
            buffer_size,
        }
    }

    /// Swap buffers and update LTDC
    pub async fn swap_buffer(&mut self, new_buffer: *mut u16) -> Result<(), ltdc::Error> {
        self.current_buffer = new_buffer;
        self.ltdc
            .set_buffer(self.layer_config.layer, self.current_buffer as *const _)
            .await
    }

    /// Get current buffer
    pub fn get_buffer(&self) -> *mut u16 {
        self.current_buffer
    }
}

/// LVGL display implementation for LTDC
struct LvglLtdcDisplay {
    display: LtdcDisplay,
}

impl Display for LvglLtdcDisplay {
    fn flush(&mut self, _display: &mut lvgl::Display, _area: &lvgl::Area, _color_map: &[u8]) {
        // This will be called by LVGL when it needs to flush the draw buffer
        // For now, we'll handle this in the main loop
        // In a real implementation, you'd copy from LVGL draw buffer to active framebuffer
        // and then swap buffers
    }
}

/// Touch input device for LVGL
struct LvglTouchInput {
    touch_point: TouchPoint,
}

impl InputDevice for LvglTouchInput {
    fn read(&mut self) -> Option<Point> {
        if self.touch_point.pressed {
            Some(Point {
                x: self.touch_point.x as i16,
                y: self.touch_point.y as i16,
            })
        } else {
            None
        }
    }
}

/// Initialize clocks for STM32U5A9NJH6Q
/// This configuration is optimized for display use with LTDC
fn init_clocks() -> Peripherals {
    let mut config = Stm32Config::default();
    
    // Configure HSE (High Speed External oscillator)
    // RVT50HQSNWC00-B typically uses 16 MHz external oscillator
    config.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(16_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    
    // Configure PLL1 for system clock (160 MHz)
    config.rcc.pll1 = Some(rcc::Pll {
        source: rcc::PllSource::HSE,
        prediv: rcc::PllPreDiv::DIV1,
        mul: rcc::PllMul::MUL10,  // 16 MHz * 10 = 160 MHz
        divp: None,
        divq: None,
        divr: Some(rcc::PllDiv::DIV1),
    });
    
    // System clock from PLL1_R
    config.rcc.sys = rcc::Sysclk::PLL1_R;
    
    // Configure PLL3 for LTDC clock (typically 25-30 MHz for 800x480 displays)
    // We want ~25 MHz pixel clock for 800x480 @ 60Hz
    // Pixel clock = (800 + hfp + hbp + hsync) * (480 + vfp + vbp + vsync) * refresh_rate
    // For typical timing: (800 + 8 + 8 + 5) * (480 + 8 + 8 + 5) * 60 = 1008 * 501 * 60 ≈ 30.3 MHz
    // So we need PLL3 to provide ~30 MHz
    config.rcc.pll3 = Some(rcc::Pll {
        source: rcc::PllSource::HSE,
        prediv: rcc::PllPreDiv::DIV4,     // 16 MHz / 4 = 4 MHz
        mul: rcc::PllMul::MUL75,          // 4 MHz * 75 = 300 MHz
        divp: None,
        divq: None,
        divr: Some(rcc::PllDiv::DIV10),   // 300 MHz / 10 = 30 MHz
    });
    
    // LTDC clock source from PLL3_R
    config.rcc.mux.ltdcsel = rcc::mux::Ltdcsel::PLL3_R;
    
    // Enable ICACHE for better performance
    config.rcc.icache = rcc::Icache::Enabled;
    
    // Enable DCACHE for better performance with display buffers
    config.rcc.dcache = rcc::Dcache::Enabled;
    
    embassy_stm32::init(config)
}

/// Initialize LTDC for RVT50HQSNWC00-B display
/// The display uses RGB interface with typical timing parameters
fn init_ltdc(
    p: &mut Peripherals,
) -> (
    Ltdc<'static, peripherals::LTDC>,
    Output<'static, peripherals::PD6>,
    Output<'static, peripherals::PE4>,
    Output<'static, peripherals::PE6>,
) {
    // Display timing parameters (typical for 800x480 displays)
    // These may need adjustment based on the specific display panel
    const H_SYNC: u16 = 5;      // Horizontal synchronization pulse width
    const H_BACK_PORCH: u16 = 40; // Horizontal back porch
    const H_FRONT_PORCH: u16 = 20; // Horizontal front porch
    const V_SYNC: u16 = 5;      // Vertical synchronization pulse width
    const V_BACK_PORCH: u16 = 10; // Vertical back porch
    const V_FRONT_PORCH: u16 = 20; // Vertical front porch

    let ltdc_config = LtdcConfiguration {
        active_width: DISPLAY_WIDTH as _,
        active_height: DISPLAY_HEIGHT as _,
        h_back_porch: H_BACK_PORCH,
        h_front_porch: H_FRONT_PORCH,
        v_back_porch: V_BACK_PORCH,
        v_front_porch: V_FRONT_PORCH,
        h_sync: H_SYNC,
        v_sync: V_SYNC,
        h_sync_polarity: PolarityActive::ActiveLow,  // Typically low for LCD panels
        v_sync_polarity: PolarityActive::ActiveLow,  // Typically low for LCD panels
        data_enable_polarity: PolarityActive::ActiveHigh,
        pixel_clock_polarity: PolarityEdge::RisingEdge,
    };

    info!("Initializing LTDC with config: {:?}", ltdc_config);

    // Initialize LTDC with RGB888 format (24 bits per pixel, but we'll use RGB565 in practice)
    // Note: We use RGB565 format to save memory, but configure as RGB888 for flexibility
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

    // Control pins
    let ltdc_de = Output::new(p.PD6, Level::Low, Speed::High);
    let ltdc_disp_ctrl = Output::new(p.PE4, Level::Low, Speed::High);
    let ltdc_bl_ctrl = Output::new(p.PE6, Level::Low, Speed::High);

    // Enable display
    ltdc_de.set_high();
    ltdc_disp_ctrl.set_high();
    ltdc_bl_ctrl.set_high();

    info!("LTDC initialized successfully");

    (ltdc, ltdc_de, ltdc_disp_ctrl, ltdc_bl_ctrl)
}

/// Initialize I2C for touch controller
fn init_touch_i2c(p: &mut Peripherals) -> I2c<'static, peripherals::I2C1> {
    // Typical I2C configuration for touch controllers
    // SCL and SDA pins - adjust based on your board configuration
    // Common pins for RVT50HQSNWC00-B: PB6 (SCL), PB7 (SDA)
    let scl = p.PB6;
    let sda = p.PB7;

    let config = I2cConfig::default();
    
    I2c::new(p.I2C1, Irqs, scl, sda, config)
}

/// Read touch point from touch controller
/// This is a placeholder - actual implementation depends on the touch controller chip
/// Common controllers: FT5x06, GT911, etc.
async fn read_touch_point(i2c: &mut I2c<'static, peripherals::I2C1>) -> Option<TouchPoint> {
    // Try to read from typical touch controller registers
    // This is a simplified implementation
    
    // Check if touch is pressed (register 0x02 for FT5x06)
    let mut touch_data = [0u8; 6]; // X high, X low, Y high, Y low, touch count, gesture
    
    // Read touch data - address depends on controller
    // For FT5x06: touch data starts at register 0x03
    match i2c.write_read(TOUCH_I2C_ADDRESS, &[0x02], &mut touch_data) {
        Ok(_) => {
            // Check if there's a touch (bit 6 of first byte)
            let touch_count = touch_data[0] & 0x0F;
            if touch_count > 0 {
                // Read X and Y coordinates
                // For FT5x06: X = (touch_data[1] << 8) | touch_data[2]
                //            Y = (touch_data[3] << 8) | touch_data[4]
                let x = ((touch_data[1] as u16) << 8) | touch_data[2] as u16;
                let y = ((touch_data[3] as u16) << 8) | touch_data[4] as u16;
                
                // Touch coordinates might need scaling to match display resolution
                // Many touch controllers return coordinates in their native resolution
                // which may differ from the display resolution
                
                Some(TouchPoint {
                    x: x.min(DISPLAY_WIDTH as u16 - 1),
                    y: y.min(DISPLAY_HEIGHT as u16 - 1),
                    pressed: true,
                })
            } else {
                Some(TouchPoint {
                    x: 0,
                    y: 0,
                    pressed: false,
                })
            }
        }
        Err(_) => {
            // I2C read failed - no touch detected
            Some(TouchPoint {
                x: 0,
                y: 0,
                pressed: false,
            })
        }
    }
}

/// Initialize LVGL
fn init_lvgl() -> (Screen, DrawBuffer) {
    // Create draw buffer for LVGL
    // Safety: We're using statically allocated memory
    let draw_buf = unsafe {
        DrawBuffer::new(
            LVGL_DRAW_BUF.as_mut_ptr(),
            LVGL_DRAW_BUF_SIZE,
            LVGL_HOR_RES_MAX,
        )
    };

    // Create display
    let mut display = lvgl::Display::new(LVGL_HOR_RES_MAX, LVGL_VER_RES_MAX);
    display.set_draw_buffer(&draw_buf);
    display.set_flush_callback(|_display, _area, _color_map| {
        // This callback is called when LVGL needs to flush the draw buffer
        // In our implementation, we'll handle this in the main loop
    });

    // Create screen
    let screen = Screen::new();
    screen.set_style(Style::screen_mut().set_bg_color(Color::from_rgb((0, 0, 0))));

    (screen, draw_buf)
}

/// Create demo UI
fn create_demo_ui(screen: &mut Screen) {
    info!("Creating LVGL demo UI");

    // Create a simple label
    let mut label = Label::new(screen);
    label.set_text("Riverdi RVT50HQSNWC00-B");
    label.set_align(Align::TopMiddle);
    label.set_y(20);
    label.set_style(Style::label_mut().set_text_color(Color::WHITE).set_font(&lvgl::Font::Montserrat26));

    // Create a subtitle
    let mut subtitle = Label::new(screen);
    subtitle.set_text("LVGL Demo with Embassy");
    subtitle.set_align(Align::TopMiddle);
    subtitle.set_y(50);
    subtitle.set_style(Style::label_mut().set_text_color(Color::from_rgb((200, 200, 200))).set_font(&lvgl::Font::Montserrat18));

    // Create a button
    let mut btn = Button::new(screen);
    btn.set_size(200, 50);
    btn.set_align(Align::TopMiddle);
    btn.set_y(100);
    
    let mut btn_label = Label::new(&mut btn);
    btn_label.set_text("Click Me!");
    btn_label.center();

    // Create a counter label
    let mut counter_label = Label::new(screen);
    counter_label.set_text("Counter: 0");
    counter_label.set_align(Align::TopMiddle);
    counter_label.set_y(180);
    counter_label.set_style(Style::label_mut().set_text_color(Color::WHITE).set_font(&lvgl::Font::Montserrat20));

    // Create a slider
    let mut slider = Slider::new(screen);
    slider.set_width(300);
    slider.set_align(Align::TopMiddle);
    slider.set_y(250);
    slider.set_range(0, 100);
    slider.set_value(50, false);

    // Create a bar
    let mut bar = Bar::new(screen);
    bar.set_size(300, 20);
    bar.set_align(Align::TopMiddle);
    bar.set_y(300);
    bar.set_range(0, 100);
    bar.set_value(50, false);

    // Create a checkbox
    let mut checkbox = Checkbox::new(screen);
    checkbox.set_text("Enable Feature");
    checkbox.set_align(Align::TopMiddle);
    checkbox.set_y(350);

    // Create an arc (circular progress)
    let mut arc = Arc::new(screen);
    arc.set_size(150, 150);
    arc.set_align(Align::TopRight);
    arc.set_x(-50);
    arc.set_y(200);
    arc.set_rotation(270);
    arc.set_range(0, 100);
    arc.set_value(75);
    arc.set_bg_angles(0, 360);

    info!("Demo UI created successfully");
}

/// Main LVGL task
#[embassy_executor::task]
async fn lvgl_task(
    mut ltdc: Ltdc<'static, peripherals::LTDC>,
    layer_config: LtdcLayerConfig,
    mut i2c: I2c<'static, peripherals::I2C1>,
) {
    info!("Starting LVGL task");

    // Initialize LVGL
    let (mut screen, draw_buf) = init_lvgl();

    // Create demo UI
    create_demo_ui(&mut screen);

    // Initialize double buffering
    let mut current_fb = unsafe { FB1.as_mut_ptr() };
    let mut next_fb = unsafe { FB2.as_mut_ptr() };
    let mut ltdc_display = LtdcDisplay::new(ltdc, layer_config, current_fb, FB_SIZE);

    // Initialize LVGL display driver
    let mut lvgl_display = LvglLtdcDisplay {
        display: ltdc_display,
    };

    // Initialize touch input
    let mut touch_input = LvglTouchInput {
        touch_point: TouchPoint {
            x: 0,
            y: 0,
            pressed: false,
        },
    };

    // Register display and input with LVGL
    lvgl::Display::register(&mut lvgl_display);
    lvgl::InputDevice::register(&mut touch_input);

    // Main LVGL loop
    let mut counter = 0;
    loop {
        // Read touch input
        if let Some(touch_point) = read_touch_point(&mut i2c).await {
            touch_input.touch_point = touch_point;
        }

        // Update LVGL
        lvgl::tick_inc(5); // Increment LVGL tick (5ms)
        
        // Handle LVGL tasks
        lvgl::handler();

        // Update counter label every second
        counter += 1;
        if counter % 200 == 0 { // ~1 second at 5ms tick
            // In a real implementation, we'd update the label text
            // For now, just log it
            info!("Counter: {}", counter / 200);
        }

        // Swap buffers periodically (e.g., every frame or every few frames)
        // In a real implementation, you'd swap when LVGL has finished drawing
        if counter % 4 == 0 { // Swap every 20ms (50 FPS)
            // Swap frame buffers
            core::mem::swap(&mut current_fb, &mut next_fb);
            
            // Update LTDC to use the new buffer
            if let Err(e) = ltdc_display.swap_buffer(current_fb).await {
                info!("Buffer swap error: {:?}", e);
            }
        }

        // Small delay to control frame rate
        Timer::after(Duration::from_millis(5)).await;
    }
}

/// LED blink task for visual feedback
#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
    let mut counter = 0;
    loop {
        info!("LED blink: {}", counter);
        counter += 1;

        // On
        led.set_low();
        Timer::after(Duration::from_millis(50)).await;

        // Off
        led.set_high();
        Timer::after(Duration::from_millis(450)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Riverdi RVT50HQSNWC00-B LVGL Demo");
    info!("MCU: STM32U5A9NJH6Q");
    info!("Display: 800x480 RGB LCD");

    // Initialize clocks
    let mut p = init_clocks();

    // Enable ICACHE for better performance
    embassy_stm32::pac::ICACHE.cr().write(|w| {
        w.set_en(true);
    });

    info!("Clocks initialized");

    // Initialize LTDC display
    let (ltdc, _ltdc_de, _ltdc_disp_ctrl, _ltdc_bl_ctrl) = init_ltdc(&mut p);

    // Configure LTDC layer for RGB565 format
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

    info!("LTDC layer configured");

    // Initialize I2C for touch controller
    let i2c = init_touch_i2c(&mut p);

    info!("I2C initialized for touch controller");

    // Create LED output for visual feedback
    let led = Output::new(p.PD2, Level::High, Speed::Low);

    // Spawn tasks
    unwrap!(spawner.spawn(led_task(led)));
    unwrap!(spawner.spawn(lvgl_task(ltdc, layer_config, i2c)));

    info!("Tasks spawned, entering idle loop");

    // Main loop - just idle
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
