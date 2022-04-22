#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::cell::RefCell;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::Delay;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_rp::Peripherals;
use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::Text;
use st7789::{Orientation, ST7789};

use crate::my_display_interface::SPIDeviceInterface;
use crate::shared_spi::SpiDeviceWithCs;
use crate::touch::Touch;

use defmt_rtt as _; // global logger
use panic_probe as _;

//const DISPLAY_FREQ: u32 = 64_000_000;
const TOUCH_FREQ: u32 = 200_000;

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
    config.frequency = TOUCH_FREQ; // use the lowest freq
    config.phase = spi::Phase::CaptureOnSecondTransition;
    config.polarity = spi::Polarity::IdleHigh;

    let spi_bus = RefCell::new(Spi::new(p.SPI1, clk, mosi, miso, config));

    let display_spi = SpiDeviceWithCs::new(&spi_bus, Output::new(display_cs, Level::High));
    let touch_spi = SpiDeviceWithCs::new(&spi_bus, Output::new(touch_cs, Level::High));

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

mod shared_spi {
    use core::cell::RefCell;
    use core::fmt::Debug;

    use embedded_hal_1::digital::blocking::OutputPin;
    use embedded_hal_1::spi;
    use embedded_hal_1::spi::blocking::SpiDevice;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub enum SpiDeviceWithCsError<BUS, CS> {
        #[allow(unused)] // will probably use in the future when adding a flush() to SpiBus
        Spi(BUS),
        Cs(CS),
    }

    impl<BUS, CS> spi::Error for SpiDeviceWithCsError<BUS, CS>
    where
        BUS: spi::Error + Debug,
        CS: Debug,
    {
        fn kind(&self) -> spi::ErrorKind {
            match self {
                Self::Spi(e) => e.kind(),
                Self::Cs(_) => spi::ErrorKind::Other,
            }
        }
    }

    pub struct SpiDeviceWithCs<'a, BUS, CS> {
        bus: &'a RefCell<BUS>,
        cs: CS,
    }

    impl<'a, BUS, CS> SpiDeviceWithCs<'a, BUS, CS> {
        pub fn new(bus: &'a RefCell<BUS>, cs: CS) -> Self {
            Self { bus, cs }
        }
    }

    impl<'a, BUS, CS> spi::ErrorType for SpiDeviceWithCs<'a, BUS, CS>
    where
        BUS: spi::ErrorType,
        CS: OutputPin,
    {
        type Error = SpiDeviceWithCsError<BUS::Error, CS::Error>;
    }

    impl<'a, BUS, CS> SpiDevice for SpiDeviceWithCs<'a, BUS, CS>
    where
        BUS: spi::blocking::SpiBusFlush,
        CS: OutputPin,
    {
        type Bus = BUS;

        fn transaction<R>(
            &mut self,
            f: impl FnOnce(&mut Self::Bus) -> Result<R, BUS::Error>,
        ) -> Result<R, Self::Error> {
            let mut bus = self.bus.borrow_mut();
            self.cs.set_low().map_err(SpiDeviceWithCsError::Cs)?;

            let f_res = f(&mut bus);

            // On failure, it's important to still flush and deassert CS.
            let flush_res = bus.flush();
            let cs_res = self.cs.set_high();

            let f_res = f_res.map_err(SpiDeviceWithCsError::Spi)?;
            flush_res.map_err(SpiDeviceWithCsError::Spi)?;
            cs_res.map_err(SpiDeviceWithCsError::Cs)?;

            Ok(f_res)
        }
    }
}

/// Driver for the XPT2046 resistive touchscreen sensor
mod touch {
    use embedded_hal_1::spi::blocking::{SpiBus, SpiBusRead, SpiBusWrite, SpiDevice};

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
        SPI::Bus: SpiBus,
    {
        pub fn new(spi: SPI) -> Self {
            Self { spi }
        }

        pub fn read(&mut self) -> Option<(i32, i32)> {
            let mut x = [0; 2];
            let mut y = [0; 2];
            self.spi
                .transaction(|bus| {
                    bus.write(&[0x90])?;
                    bus.read(&mut x)?;
                    bus.write(&[0xd0])?;
                    bus.read(&mut y)?;
                    Ok(())
                })
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
    use embedded_hal_1::digital::blocking::OutputPin;
    use embedded_hal_1::spi::blocking::{SpiBusWrite, SpiDevice};

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
        SPI::Bus: SpiBusWrite,
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
        SPI::Bus: SpiBusWrite,
        DC: OutputPin,
    {
        fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result<(), DisplayError> {
            let r = self.spi.transaction(|bus| {
                // 1 = data, 0 = command
                if let Err(_) = self.dc.set_low() {
                    return Ok(Err(DisplayError::DCError));
                }

                // Send words over SPI
                send_u8(bus, cmds)?;

                Ok(Ok(()))
            });
            r.map_err(|_| DisplayError::BusWriteError)?
        }

        fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
            let r = self.spi.transaction(|bus| {
                // 1 = data, 0 = command
                if let Err(_) = self.dc.set_high() {
                    return Ok(Err(DisplayError::DCError));
                }

                // Send words over SPI
                send_u8(bus, buf)?;

                Ok(Ok(()))
            });
            r.map_err(|_| DisplayError::BusWriteError)?
        }
    }

    fn send_u8<T: SpiBusWrite>(spi: &mut T, words: DataFormat<'_>) -> Result<(), T::Error> {
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
