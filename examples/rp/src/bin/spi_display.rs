//! This example shows how to use SPI (Serial Peripheral Interface) in the RP2040 chip.
//!
//! Example written for a display using the ST7789 chip. Possibly the Waveshare Pico-ResTouch
//! (https://www.waveshare.com/wiki/Pico-ResTouch-LCD-2.8)

#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::*;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi;
use embassy_rp::spi::{Blocking, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::Delay;
use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::Text;
use st7789::{Orientation, ST7789};
use {defmt_rtt as _, panic_probe as _};

use crate::my_display_interface::SPIDeviceInterface;
use crate::touch::Touch;

const DISPLAY_FREQ: u32 = 64_000_000;
const TOUCH_FREQ: u32 = 200_000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
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
    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;
    let mut touch_config = spi::Config::default();
    touch_config.frequency = TOUCH_FREQ;
    touch_config.phase = spi::Phase::CaptureOnSecondTransition;
    touch_config.polarity = spi::Polarity::IdleHigh;

    let spi: Spi<'_, _, Blocking> = Spi::new_blocking(p.SPI1, clk, mosi, miso, touch_config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(&spi_bus, Output::new(display_cs, Level::High), display_config);
    let touch_spi = SpiDeviceWithConfig::new(&spi_bus, Output::new(touch_cs, Level::High), touch_config);

    let mut touch = Touch::new(touch_spi);

    let dcx = Output::new(dcx, Level::Low);
    let rst = Output::new(rst, Level::Low);
    // dcx: 0 = command, 1 = data

    // Enable LCD backlight
    let _bl = Output::new(bl, Level::High);

    // display interface abstraction from SPI and DC
    let di = SPIDeviceInterface::new(display_spi, dcx);

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
            let style = PrimitiveStyleBuilder::new().fill_color(Rgb565::BLUE).build();

            Rectangle::new(Point::new(x - 1, y - 1), Size::new(3, 3))
                .into_styled(style)
                .draw(&mut display)
                .unwrap();
        }
    }
}

/// Driver for the XPT2046 resistive touchscreen sensor
mod touch {
    use embedded_hal_1::spi::{Operation, SpiDevice};

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

    pub struct Touch<SPI: SpiDevice> {
        spi: SPI,
    }

    impl<SPI> Touch<SPI>
    where
        SPI: SpiDevice,
    {
        pub fn new(spi: SPI) -> Self {
            Self { spi }
        }

        pub fn read(&mut self) -> Option<(i32, i32)> {
            let mut x = [0; 2];
            let mut y = [0; 2];
            self.spi
                .transaction(&mut [
                    Operation::Write(&[0x90]),
                    Operation::Read(&mut x),
                    Operation::Write(&[0xd0]),
                    Operation::Read(&mut y),
                ])
                .unwrap();

            let x = (u16::from_be_bytes(x) >> 3) as i32;
            let y = (u16::from_be_bytes(y) >> 3) as i32;

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
}

mod my_display_interface {
    use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
    use embedded_hal_1::digital::OutputPin;
    use embedded_hal_1::spi::SpiDevice;

    /// SPI display interface.
    ///
    /// This combines the SPI peripheral and a data/command pin
    pub struct SPIDeviceInterface<SPI, DC> {
        spi: SPI,
        dc: DC,
    }

    impl<SPI, DC> SPIDeviceInterface<SPI, DC>
    where
        SPI: SpiDevice,
        DC: OutputPin,
    {
        /// Create new SPI interface for communciation with a display driver
        pub fn new(spi: SPI, dc: DC) -> Self {
            Self { spi, dc }
        }
    }

    impl<SPI, DC> WriteOnlyDataCommand for SPIDeviceInterface<SPI, DC>
    where
        SPI: SpiDevice,
        DC: OutputPin,
    {
        fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result<(), DisplayError> {
            // 1 = data, 0 = command
            self.dc.set_low().map_err(|_| DisplayError::DCError)?;

            send_u8(&mut self.spi, cmds).map_err(|_| DisplayError::BusWriteError)?;
            Ok(())
        }

        fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
            // 1 = data, 0 = command
            self.dc.set_high().map_err(|_| DisplayError::DCError)?;

            send_u8(&mut self.spi, buf).map_err(|_| DisplayError::BusWriteError)?;
            Ok(())
        }
    }

    fn send_u8<T: SpiDevice>(spi: &mut T, words: DataFormat<'_>) -> Result<(), T::Error> {
        match words {
            DataFormat::U8(slice) => spi.write(slice),
            DataFormat::U16(slice) => {
                use byte_slice_cast::*;
                spi.write(slice.as_byte_slice())
            }
            DataFormat::U16LE(slice) => {
                use byte_slice_cast::*;
                for v in slice.as_mut() {
                    *v = v.to_le();
                }
                spi.write(slice.as_byte_slice())
            }
            DataFormat::U16BE(slice) => {
                use byte_slice_cast::*;
                for v in slice.as_mut() {
                    *v = v.to_be();
                }
                spi.write(slice.as_byte_slice())
            }
            DataFormat::U8Iter(iter) => {
                let mut buf = [0; 32];
                let mut i = 0;

                for v in iter.into_iter() {
                    buf[i] = v;
                    i += 1;

                    if i == buf.len() {
                        spi.write(&buf)?;
                        i = 0;
                    }
                }

                if i > 0 {
                    spi.write(&buf[..i])?;
                }

                Ok(())
            }
            DataFormat::U16LEIter(iter) => {
                use byte_slice_cast::*;
                let mut buf = [0; 32];
                let mut i = 0;

                for v in iter.map(u16::to_le) {
                    buf[i] = v;
                    i += 1;

                    if i == buf.len() {
                        spi.write(&buf.as_byte_slice())?;
                        i = 0;
                    }
                }

                if i > 0 {
                    spi.write(&buf[..i].as_byte_slice())?;
                }

                Ok(())
            }
            DataFormat::U16BEIter(iter) => {
                use byte_slice_cast::*;
                let mut buf = [0; 64];
                let mut i = 0;
                let len = buf.len();

                for v in iter.map(u16::to_be) {
                    buf[i] = v;
                    i += 1;

                    if i == len {
                        spi.write(&buf.as_byte_slice())?;
                        i = 0;
                    }
                }

                if i > 0 {
                    spi.write(&buf[..i].as_byte_slice())?;
                }

                Ok(())
            }
            _ => unimplemented!(),
        }
    }
}
