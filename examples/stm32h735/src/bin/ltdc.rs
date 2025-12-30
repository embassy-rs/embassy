#![no_std]
#![no_main]
#![macro_use]
#![allow(static_mut_refs)]

/// This example demonstrates the LTDC lcd display peripheral and was tested to run on an stm32h735g-dk (embassy-stm32 feature "stm32h735ig" and probe-rs chip "STM32H735IGKx")
/// Even though the dev kit has 16MB of attached PSRAM this example uses the 320KB of internal AXIS RAM found on the mcu itself to make the example more standalone and portable.
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
    let p = rcc_setup::stm32h735g_init();

    // blink the led on another task
    let led = Output::new(p.PC3, Level::High, Speed::Low);
    spawner.spawn(unwrap!(led_task(led)));

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
        p.LTDC, Irqs, p.PG7, p.PC6, p.PA4, p.PG14, p.PD0, p.PD6, p.PA8, p.PE12, p.PA3, p.PB8, p.PB9, p.PB1, p.PB0,
        p.PA6, p.PE11, p.PH15, p.PH4, p.PC7, p.PD3, p.PE0, p.PH3, p.PH8, p.PH9, p.PH10, p.PH11, p.PE1, p.PE15,
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

    use embassy_stm32::rcc::{Hse, HseMode, *};
    use embassy_stm32::time::Hertz;
    use embassy_stm32::{Config, Peripherals};

    /// Sets up clocks for the stm32h735g mcu
    /// change this if you plan to use a different microcontroller
    pub fn stm32h735g_init() -> Peripherals {
        /*
         https://github.com/STMicroelectronics/STM32CubeH7/blob/master/Projects/STM32H735G-DK/Examples/GPIO/GPIO_EXTI/Src/main.c
         @brief  System Clock Configuration
            The system Clock is configured as follow :
            System Clock source            = PLL (HSE)
            SYSCLK(Hz)                     = 520000000 (CPU Clock)
            HCLK(Hz)                       = 260000000 (AXI and AHBs Clock)
            AHB Prescaler                  = 2
            D1 APB3 Prescaler              = 2 (APB3 Clock  130MHz)
            D2 APB1 Prescaler              = 2 (APB1 Clock  130MHz)
            D2 APB2 Prescaler              = 2 (APB2 Clock  130MHz)
            D3 APB4 Prescaler              = 2 (APB4 Clock  130MHz)
            HSE Frequency(Hz)              = 25000000
            PLL_M                          = 5
            PLL_N                          = 104
            PLL_P                          = 1
            PLL_Q                          = 4
            PLL_R                          = 2
            VDD(V)                         = 3.3
            Flash Latency(WS)              = 3
        */

        // setup power and clocks for an stm32h735g-dk run from an external 25 Mhz external oscillator
        let mut config = Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(25),
            mode: HseMode::Oscillator,
        });
        config.rcc.hsi = None;
        config.rcc.csi = false;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV5, // PLL_M
            mul: PllMul::MUL104,     // PLL_N
            divp: Some(PllDiv::DIV1),
            divq: Some(PllDiv::DIV4),
            divr: Some(PllDiv::DIV2),
        });
        // numbers adapted from Drivers/BSP/STM32H735G-DK/stm32h735g_discovery_ospi.c
        // MX_OSPI_ClockConfig
        config.rcc.pll2 = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV5, // PLL_M
            mul: PllMul::MUL80,      // PLL_N
            divp: Some(PllDiv::DIV5),
            divq: Some(PllDiv::DIV2),
            divr: Some(PllDiv::DIV2),
        });
        // numbers adapted from Drivers/BSP/STM32H735G-DK/stm32h735g_discovery_lcd.c
        // MX_LTDC_ClockConfig
        config.rcc.pll3 = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV5, // PLL_M
            mul: PllMul::MUL160,     // PLL_N
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV2),
            divr: Some(PllDiv::DIV83),
        });
        config.rcc.voltage_scale = VoltageScale::Scale0;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV2;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.apb3_pre = APBPrescaler::DIV2;
        config.rcc.apb4_pre = APBPrescaler::DIV2;
        embassy_stm32::init(config)
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
