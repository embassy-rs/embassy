#![no_std]
#![no_main]
#![macro_use]
#![allow(static_mut_refs)]

/// This example demonstrates the LTDC lcd display peripheral and was tested to run on the CM7 core of an stm32h745I-disco (embassy-stm32 feature "stm32h745xi-cm7" and probe-rs chip "STM32H745XI")
/// Even though the dev kit has 16MB of attached PSRAM this example uses the internal RAM found on the mcu itself to make the example more standalone and portable.
/// For this reason a 256 color lookup table had to be used to keep the memory requirement down to an acceptable level.
/// The example bounces a ferris crab bitmap around the screen while blinking an led on another task
///
use bouncy_box::BouncyBox;
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ltdc::{self, Ltdc, LtdcConfiguration, LtdcLayer, LtdcLayerConfig, PolarityActive, PolarityEdge};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::{Duration, Timer};
use embedded_graphics::Pixel;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::pixelcolor::raw::RawU24;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use heapless::{Entry, FnvIndexMap};
use tinybmp::Bmp;
use {defmt_rtt as _, panic_probe as _};

const DISPLAY_WIDTH: usize = 480;
const DISPLAY_HEIGHT: usize = 272;
const MY_TASK_POOL_SIZE: usize = 2;

// the following two display buffers consume 261120 bytes that just about fits into axis ram found on the mcu
pub static mut FB1: [TargetPixelType; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
pub static mut FB2: [TargetPixelType; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

bind_interrupts!(struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
});

const NUM_COLORS: usize = 256;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = rcc_setup::stm32h745_cm7_init();

    // blink the led on another task
    let led = Output::new(p.PI13, Level::High, Speed::Low);
    spawner.spawn(unwrap!(led_task(led)));

    // At a minimum we need pull 'display mode' (PD7) high and enable the LCD backlight (PK0).
    // The BSP code from ST also sets both the reset pin (PB12) and display_enable (PK7) high,
    //  but in my testing this didn't actually change anything either way.
    let _lcd_display_mode = Output::new(p.PD7, Level::High, Speed::Low);
    let _lcd_bl_ctrl = Output::new(p.PK0, Level::High, Speed::Low);
    // let _lcd_disp_enable = Output::new(p.PK7, Level::High, Speed::Low);
    // let _lcd_reset = Output::new(p.PB12, Level::High, Speed::Low);

    // numbers from STMicroelectronics/STM32CubeH7 STM32H735G-DK C-based example
    const RK043FN48H_HSYNC: u16 = 41; // Horizontal synchronization
    const RK043FN48H_HBP: u16 = 13; // Horizontal back porch
    const RK043FN48H_HFP: u16 = 32; // Horizontal front porch
    const RK043FN48H_VSYNC: u16 = 10; // Vertical synchronization
    const RK043FN48H_VBP: u16 = 2; // Vertical back porch
    const RK043FN48H_VFP: u16 = 2; // Vertical front porch

    let ltdc_config = LtdcConfiguration {
        active_width: DISPLAY_WIDTH as _,
        active_height: DISPLAY_HEIGHT as _,
        h_back_porch: RK043FN48H_HBP - 11, // -11 from MX_LTDC_Init
        h_front_porch: RK043FN48H_HFP,
        v_back_porch: RK043FN48H_VBP,
        v_front_porch: RK043FN48H_VFP,
        h_sync: RK043FN48H_HSYNC,
        v_sync: RK043FN48H_VSYNC,
        h_sync_polarity: PolarityActive::ActiveLow,
        v_sync_polarity: PolarityActive::ActiveLow,
        data_enable_polarity: PolarityActive::ActiveHigh,
        pixel_clock_polarity: PolarityEdge::FallingEdge,
    };

    info!("init ltdc");
    let mut ltdc = Ltdc::new_with_pins(
        p.LTDC, Irqs, p.PI14, p.PI12, p.PI9, p.PJ12, p.PJ13, p.PJ14, p.PJ15, p.PK3, p.PK4, p.PK5, p.PK6, p.PJ7, p.PJ8,
        p.PJ9, p.PJ10, p.PJ11, p.PI0, p.PI1, p.PK2, p.PI15, p.PJ0, p.PJ1, p.PH9, p.PJ3, p.PJ4, p.PJ5, p.PJ6,
    );
    ltdc.init(&ltdc_config);

    // we only need to draw on one layer for this example (not to be confused with the double buffer)
    info!("enable bottom layer");
    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::L8, // 1 byte per pixel
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: DISPLAY_WIDTH as _,
        window_y0: 0,
        window_y1: DISPLAY_HEIGHT as _,
    };

    let ferris_bmp: Bmp<Rgb888> = Bmp::from_slice(include_bytes!("./ferris.bmp")).unwrap();
    let color_map = build_color_lookup_map(&ferris_bmp);
    let clut = build_clut(&color_map);

    // enable the bottom layer with a 256 color lookup table
    ltdc.init_layer(&layer_config, Some(&clut));

    // Safety: the DoubleBuffer controls access to the statically allocated frame buffers
    // and it is the only thing that mutates their content
    let mut double_buffer = DoubleBuffer::new(
        unsafe { FB1.as_mut() },
        unsafe { FB2.as_mut() },
        layer_config,
        color_map,
    );

    // this allows us to perform some simple animation for every frame
    let mut bouncy_box = BouncyBox::new(
        ferris_bmp.bounding_box(),
        Rectangle::new(Point::zero(), Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)),
        2,
    );

    loop {
        // cpu intensive drawing to the buffer that is NOT currently being copied to the LCD screen
        double_buffer.clear();
        let position = bouncy_box.next_point();
        let ferris = Image::new(&ferris_bmp, position);
        unwrap!(ferris.draw(&mut double_buffer));

        // perform async dma data transfer to the lcd screen
        unwrap!(double_buffer.swap(&mut ltdc).await);
    }
}

/// builds the color look-up table from all unique colors found in the bitmap. This should be a 256 color indexed bitmap to work.
fn build_color_lookup_map(bmp: &Bmp<Rgb888>) -> FnvIndexMap<u32, u8, NUM_COLORS> {
    let mut color_map: FnvIndexMap<u32, u8, NUM_COLORS> = heapless::FnvIndexMap::new();
    let mut counter: u8 = 0;

    // add black to position 0
    color_map.insert(Rgb888::new(0, 0, 0).into_storage(), counter).unwrap();
    counter += 1;

    for Pixel(_point, color) in bmp.pixels() {
        let raw = color.into_storage();
        if let Entry::Vacant(v) = color_map.entry(raw) {
            v.insert(counter).expect("more than 256 colors detected");
            counter += 1;
        }
    }
    color_map
}

/// builds the color look-up table from the color map provided
fn build_clut(color_map: &FnvIndexMap<u32, u8, NUM_COLORS>) -> [ltdc::RgbColor; NUM_COLORS] {
    let mut clut = [ltdc::RgbColor::default(); NUM_COLORS];
    for (color, index) in color_map.iter() {
        let color = Rgb888::from(RawU24::new(*color));
        clut[*index as usize] = ltdc::RgbColor {
            red: color.r(),
            green: color.g(),
            blue: color.b(),
        };
    }

    clut
}

#[embassy_executor::task(pool_size = MY_TASK_POOL_SIZE)]
async fn led_task(mut led: Output<'static>) {
    let mut counter = 0;
    loop {
        info!("blink: {}", counter);
        counter += 1;

        // on
        led.set_low();
        Timer::after(Duration::from_millis(50)).await;

        // off
        led.set_high();
        Timer::after(Duration::from_millis(450)).await;
    }
}

pub type TargetPixelType = u8;

// A simple double buffer
pub struct DoubleBuffer {
    buf0: &'static mut [TargetPixelType],
    buf1: &'static mut [TargetPixelType],
    is_buf0: bool,
    layer_config: LtdcLayerConfig,
    color_map: FnvIndexMap<u32, u8, NUM_COLORS>,
}

impl DoubleBuffer {
    pub fn new(
        buf0: &'static mut [TargetPixelType],
        buf1: &'static mut [TargetPixelType],
        layer_config: LtdcLayerConfig,
        color_map: FnvIndexMap<u32, u8, NUM_COLORS>,
    ) -> Self {
        Self {
            buf0,
            buf1,
            is_buf0: true,
            layer_config,
            color_map,
        }
    }

    pub fn current(&mut self) -> (&FnvIndexMap<u32, u8, NUM_COLORS>, &mut [TargetPixelType]) {
        if self.is_buf0 {
            (&self.color_map, self.buf0)
        } else {
            (&self.color_map, self.buf1)
        }
    }

    pub async fn swap<T: ltdc::Instance>(&mut self, ltdc: &mut Ltdc<'_, T>) -> Result<(), ltdc::Error> {
        let (_, buf) = self.current();
        let frame_buffer = buf.as_ptr();
        self.is_buf0 = !self.is_buf0;
        ltdc.set_buffer(self.layer_config.layer, frame_buffer as *const _).await
    }

    /// Clears the buffer
    pub fn clear(&mut self) {
        let (color_map, buf) = self.current();
        let black = Rgb888::new(0, 0, 0).into_storage();
        let color_index = color_map.get(&black).expect("no black found in the color map");

        for a in buf.iter_mut() {
            *a = *color_index; // solid black
        }
    }
}

// Implement DrawTarget for
impl DrawTarget for DoubleBuffer {
    type Color = Rgb888;
    type Error = ();

    /// Draw a pixel
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let size = self.size();
        let width = size.width as i32;
        let height = size.height as i32;
        let (color_map, buf) = self.current();

        for pixel in pixels {
            let Pixel(point, color) = pixel;

            if point.x >= 0 && point.y >= 0 && point.x < width && point.y < height {
                let index = point.y * width + point.x;
                let raw_color = color.into_storage();

                match color_map.get(&raw_color) {
                    Some(x) => {
                        buf[index as usize] = *x;
                    }
                    None => panic!("color not found in color map: {}", raw_color),
                };
            } else {
                // Ignore invalid points
            }
        }

        Ok(())
    }
}

impl OriginDimensions for DoubleBuffer {
    /// Return the size of the display
    fn size(&self) -> Size {
        Size::new(
            (self.layer_config.window_x1 - self.layer_config.window_x0) as _,
            (self.layer_config.window_y1 - self.layer_config.window_y0) as _,
        )
    }
}

mod rcc_setup {

    use core::mem::MaybeUninit;

    use embassy_stm32::rcc::{Hse, HseMode, *};
    use embassy_stm32::time::Hertz;
    use embassy_stm32::{Config, Peripherals, SharedData};

    // even if we only use one core, we need to call 'init_primary' and pass a `SharedData` reference
    #[unsafe(link_section = ".ram_d3.shared_data")]
    static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

    /// Sets up clocks for the stm32h745 cm7 mcu,
    /// change this if you plan to use a different microcontroller
    pub fn stm32h745_cm7_init() -> Peripherals {
        let mut config = Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz(25_000_000),
            mode: HseMode::Oscillator,
        });

        config.rcc.pll3 = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV5, // 25MHz / 5 = 5MHz
            mul: PllMul::MUL160,     // 5MHz * 160 = 800MHz
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV2),
            divr: Some(PllDiv::DIV83), // 800MHz / 83 = 9.63MHz for LTDC
        });
        embassy_stm32::init_primary(config, &SHARED_DATA)
    }
}

mod bouncy_box {
    use embedded_graphics::geometry::Point;
    use embedded_graphics::primitives::Rectangle;

    enum Direction {
        DownLeft,
        DownRight,
        UpLeft,
        UpRight,
    }

    pub struct BouncyBox {
        direction: Direction,
        child_rect: Rectangle,
        parent_rect: Rectangle,
        current_point: Point,
        move_by: usize,
    }

    // This calculates the coordinates of a chile rectangle bounced around inside a parent bounded box
    impl BouncyBox {
        pub fn new(child_rect: Rectangle, parent_rect: Rectangle, move_by: usize) -> Self {
            let center_box = parent_rect.center();
            let center_img = child_rect.center();
            let current_point = Point::new(center_box.x - center_img.x / 2, center_box.y - center_img.y / 2);
            Self {
                direction: Direction::DownRight,
                child_rect,
                parent_rect,
                current_point,
                move_by,
            }
        }

        pub fn next_point(&mut self) -> Point {
            let direction = &self.direction;
            let img_height = self.child_rect.size.height as i32;
            let box_height = self.parent_rect.size.height as i32;
            let img_width = self.child_rect.size.width as i32;
            let box_width = self.parent_rect.size.width as i32;
            let move_by = self.move_by as i32;

            match direction {
                Direction::DownLeft => {
                    self.current_point.x -= move_by;
                    self.current_point.y += move_by;

                    let x_out_of_bounds = self.current_point.x < 0;
                    let y_out_of_bounds = (self.current_point.y + img_height) > box_height;

                    if x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::UpRight
                    } else if x_out_of_bounds && !y_out_of_bounds {
                        self.direction = Direction::DownRight
                    } else if !x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::UpLeft
                    }
                }
                Direction::DownRight => {
                    self.current_point.x += move_by;
                    self.current_point.y += move_by;

                    let x_out_of_bounds = (self.current_point.x + img_width) > box_width;
                    let y_out_of_bounds = (self.current_point.y + img_height) > box_height;

                    if x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::UpLeft
                    } else if x_out_of_bounds && !y_out_of_bounds {
                        self.direction = Direction::DownLeft
                    } else if !x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::UpRight
                    }
                }
                Direction::UpLeft => {
                    self.current_point.x -= move_by;
                    self.current_point.y -= move_by;

                    let x_out_of_bounds = self.current_point.x < 0;
                    let y_out_of_bounds = self.current_point.y < 0;

                    if x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::DownRight
                    } else if x_out_of_bounds && !y_out_of_bounds {
                        self.direction = Direction::UpRight
                    } else if !x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::DownLeft
                    }
                }
                Direction::UpRight => {
                    self.current_point.x += move_by;
                    self.current_point.y -= move_by;

                    let x_out_of_bounds = (self.current_point.x + img_width) > box_width;
                    let y_out_of_bounds = self.current_point.y < 0;

                    if x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::DownLeft
                    } else if x_out_of_bounds && !y_out_of_bounds {
                        self.direction = Direction::UpLeft
                    } else if !x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::DownRight
                    }
                }
            }

            self.current_point
        }
    }
}
