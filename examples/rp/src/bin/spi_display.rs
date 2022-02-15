#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use core::cell::RefCell;
use core::fmt::Debug;

use defmt::*;
use display_interface_spi::SPIInterfaceNoCS;
use embassy::executor::Spawner;
use embassy::time::Delay;
use embassy_rp::peripherals;
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_rp::{gpio, Peripherals};
use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::Text;
use gpio::{Level, Output};
use st7789::{Orientation, ST7789};

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let bl = p.PIN_13;
    let rst = p.PIN_15;
    let display_cs = p.PIN_9;
    let dcx = p.PIN_8;
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let clk = p.PIN_10;
    let touch_cs = p.PIN_16;
    //let touch_irq = p.PIN_17;

    // create SPI
    let mut config = spi::Config::default();
    config.frequency = DISPLAY_FREQ;
    config.phase = spi::Phase::CaptureOnSecondTransition;
    config.polarity = spi::Polarity::IdleHigh;

    let spi = RefCell::new(SpiState {
        last_mode: SpiMode::Display,
        spi: Spi::new(p.SPI1, clk, mosi, miso, config),
        display_cs: Output::new(display_cs, Level::Low),
    });

    let mut touch = Touch::new(TouchSpi(&spi), Output::new(touch_cs, Level::High));

    let dcx = Output::new(dcx, Level::Low);
    let rst = Output::new(rst, Level::Low);
    // dcx: 0 = command, 1 = data

    // Enable LCD backlight
    let _bl = Output::new(bl, Level::High);

    // display interface abstraction from SPI and DC
    let di = SPIInterfaceNoCS::new(DisplaySpi(&spi), dcx);

    // create driver
    let mut display = ST7789::new(di, rst, 240, 320);

    // initialize
    display.init(&mut Delay).unwrap();

    // set default orientation
    display.set_orientation(Orientation::Landscape).unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    let raw_image_data = ImageRawLE::new(include_bytes!("../../assets/ferris.raw"), 86);
    let ferris = Image::new(&raw_image_data, Point::new(34, 68));

    // Display the image
    ferris.draw(&mut display).unwrap();

    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::GREEN);
    Text::new(
        "Hello embedded_graphics \n + embassy + RP2040!",
        Point::new(20, 200),
        style,
    )
    .draw(&mut display)
    .unwrap();

    loop {
        if let Some((x, y)) = touch.read() {
            let style = PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLUE)
                .build();

            Rectangle::new(Point::new(x - 1, y - 1), Size::new(3, 3))
                .into_styled(style)
                .draw(&mut display)
                .unwrap();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpiMode {
    Display,
    Touch,
}

struct SpiState {
    spi: Spi<'static, peripherals::SPI1>,
    display_cs: Output<'static, peripherals::PIN_9>,

    last_mode: SpiMode,
}

const DISPLAY_FREQ: u32 = 64_000_000;
const TOUCH_FREQ: u32 = 200_000;

struct DisplaySpi<'a>(&'a RefCell<SpiState>);
impl<'a> embedded_hal::blocking::spi::Write<u8> for DisplaySpi<'a> {
    type Error = core::convert::Infallible;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let this = &mut *self.0.borrow_mut();
        if this.last_mode != SpiMode::Display {
            this.spi.set_frequency(DISPLAY_FREQ);
            this.display_cs.set_low();
            this.last_mode = SpiMode::Display;
        }
        this.spi.write(words).unwrap();
        Ok(())
    }
}

struct TouchSpi<'a>(&'a RefCell<SpiState>);
impl<'a> embedded_hal::blocking::spi::Transfer<u8> for TouchSpi<'a> {
    type Error = core::convert::Infallible;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let this = &mut *self.0.borrow_mut();
        if this.last_mode != SpiMode::Touch {
            this.spi.set_frequency(TOUCH_FREQ);
            this.display_cs.set_high();
            this.last_mode = SpiMode::Touch;
        }
        this.spi.transfer(words).unwrap();
        Ok(words)
    }
}

struct Calibration {
    x1: i32,
    x2: i32,
    y1: i32,
    y2: i32,
    sx: i32,
    sy: i32,
}

const CALIBRATION: Calibration = Calibration {
    x1: 3880,
    x2: 340,
    y1: 262,
    y2: 3850,
    sx: 320,
    sy: 240,
};

struct Touch<
    SPI: embedded_hal::blocking::spi::Transfer<u8>,
    CS: embedded_hal::digital::v2::OutputPin,
> {
    spi: SPI,
    cs: CS,
}

impl<SPI: embedded_hal::blocking::spi::Transfer<u8>, CS: embedded_hal::digital::v2::OutputPin>
    Touch<SPI, CS>
where
    SPI::Error: Debug,
    CS::Error: Debug,
{
    pub fn new(spi: SPI, cs: CS) -> Self {
        Self { spi, cs }
    }

    pub fn read(&mut self) -> Option<(i32, i32)> {
        self.cs.set_low().unwrap();
        let mut buf = [0x90, 0x00, 0x00, 0xd0, 0x00, 0x00];
        self.spi.transfer(&mut buf).unwrap();
        self.cs.set_high().unwrap();

        let x = ((buf[1] as u32) << 5 | (buf[2] as u32) >> 3) as i32;
        let y = ((buf[4] as u32) << 5 | (buf[5] as u32) >> 3) as i32;

        let cal = &CALIBRATION;

        let x = ((x - cal.x1) * cal.sx / (cal.x2 - cal.x1)).clamp(0, cal.sx);
        let y = ((y - cal.y1) * cal.sy / (cal.y2 - cal.y1)).clamp(0, cal.sy);
        if x == 0 && y == 0 {
            None
        } else {
            Some((x, y))
        }
    }
}
