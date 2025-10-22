//! This example shows how to use SPI (Serial Peripheral Interface) in the RP2040 chip.
//!
//! Example written for a display using the GC9A01 chip. Possibly the Waveshare RP2040-LCD-1.28
//! (https://www.waveshare.com/wiki/RP2040-LCD-1.28)

#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::*;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi;
use embassy_rp::spi::{Blocking, Spi};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use mipidsi::Builder;
use mipidsi::models::GC9A01;
use mipidsi::options::{ColorInversion, ColorOrder};
use {defmt_rtt as _, panic_probe as _};

const DISPLAY_FREQ: u32 = 64_000_000;
const LCD_X_RES: i32 = 240;
const LCD_Y_RES: i32 = 240;
const FERRIS_WIDTH: u32 = 86;
const FERRIS_HEIGHT: u32 = 64;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut rng = RoscRng;

    info!("Hello World!");

    let bl = p.PIN_25;
    let rst = p.PIN_12;
    let display_cs = p.PIN_9;
    let dcx = p.PIN_8;
    let mosi = p.PIN_11;
    let clk = p.PIN_10;

    // create SPI
    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;

    let spi: Spi<'_, _, Blocking> = Spi::new_blocking_txonly(p.SPI1, clk, mosi, display_config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(&spi_bus, Output::new(display_cs, Level::High), display_config);
    let dcx = Output::new(dcx, Level::Low);
    let rst = Output::new(rst, Level::Low);
    // dcx: 0 = command, 1 = data

    // Enable LCD backlight
    let _bl = Output::new(bl, Level::High);

    // display interface abstraction from SPI and DC
    let di = SPIInterface::new(display_spi, dcx);

    // Define the display from the display interface and initialize it
    let mut display = Builder::new(GC9A01, di)
        .display_size(240, 240)
        .reset_pin(rst)
        .color_order(ColorOrder::Bgr)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut Delay)
        .unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    let raw_image_data = ImageRawLE::new(include_bytes!("../../assets/ferris.raw"), FERRIS_WIDTH);
    let mut ferris = Image::new(&raw_image_data, Point::zero());

    let r = rng.next_u32();
    let mut delta = Point {
        x: ((r % 10) + 5) as i32,
        y: (((r >> 8) % 10) + 5) as i32,
    };
    loop {
        // Move Ferris
        let bb = ferris.bounding_box();
        let tl = bb.top_left;
        let br = bb.bottom_right().unwrap();
        if tl.x < 0 || br.x > LCD_X_RES {
            delta.x = -delta.x;
        }
        if tl.y < 0 || br.y > LCD_Y_RES {
            delta.y = -delta.y;
        }

        // Erase ghosting
        let style = PrimitiveStyleBuilder::new().fill_color(Rgb565::BLACK).build();
        let mut off = Point { x: 0, y: 0 };
        if delta.x < 0 {
            off.x = FERRIS_WIDTH as i32;
        }
        Rectangle::new(tl + off, Size::new(delta.x as u32, FERRIS_HEIGHT))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();
        off = Point { x: 0, y: 0 };
        if delta.y < 0 {
            off.y = FERRIS_HEIGHT as i32;
        }
        Rectangle::new(tl + off, Size::new(FERRIS_WIDTH, delta.y as u32))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();
        // Translate Ferris
        ferris.translate_mut(delta);
        // Display the image
        ferris.draw(&mut display).unwrap();
        Timer::after(Duration::from_millis(50)).await;
    }
}
