//! SSD1306 on STM32WBA65I-DK1 — 4-wire SPI3 (MB2143 schematic sheet OLED).
//!
//! SPI3_SCK=PA0, SPI3_MOSI=PB8, CS=PE1, D/C=PE0, RST=PE3.

use defmt::warn;
use display_interface::DisplayError;
use display_interface_spi::SPIInterface;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Peri;
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::{PA0, PB8, PE0, PE1, PE3, SPI3};
use embassy_stm32::spi::mode::Master;
use embassy_time::{Delay, Timer};
use embedded_graphics::{
    mono_font::{ascii::{FONT_5X8, FONT_10X20}, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, Ssd1306};

type OledSpi = Spi<'static, Blocking, Master>;
type OledSpiDev = ExclusiveDevice<OledSpi, Output<'static>, NoDelay>;
type OledInterface = SPIInterface<OledSpiDev, Output<'static>>;
type OledDisplay = Ssd1306<OledInterface, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

/// Rhai `print()` mirror line (8 lines × 8 px font on 64 px height).
pub const PRINT_LINE: u8 = 7;

/// Vertical pitch for 8 text rows on a 64 px-tall panel.
const LINE_PITCH: i32 = 8;

pub struct OledBus {
    pub display: OledDisplay,
    pub rst: Output<'static>,
}

impl OledBus {
    pub fn new(
        spi3: Peri<'static, SPI3>,
        sck: Peri<'static, PA0>,
        mosi: Peri<'static, PB8>,
        cs_pin: Peri<'static, PE1>,
        dc_pin: Peri<'static, PE0>,
        rst_pin: Peri<'static, PE3>,
    ) -> Self {
        let mut spi_cfg = spi::Config::default();
        spi_cfg.frequency = Hertz::mhz(4);

        let spi = Spi::new_blocking_txonly(spi3, sck, mosi, spi_cfg);
        let cs = Output::new(cs_pin, Level::High, Speed::Low);
        let dc = Output::new(dc_pin, Level::Low, Speed::Low);
        let rst = Output::new(rst_pin, Level::High, Speed::Low);

        let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
        let interface = SPIInterface::new(spi_dev, dc);
        let display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        Self { display, rst }
    }

    pub fn try_init(&mut self) -> Result<(), DisplayError> {
        if self.display.reset(&mut self.rst, &mut Delay {}).is_err() {
            return Err(DisplayError::RSError);
        }
        self.display.init()
    }

    /// Draw up to 8 centered lines (FONT_5X8, 8 px row pitch, top-aligned).
    pub fn render_lines(&mut self, lines: &[heapless::String<22>; 8]) -> Result<(), DisplayError> {
        self.display.clear_buffer();
        let style = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);
        let text_style = TextStyleBuilder::new()
            .alignment(Alignment::Center)
            .baseline(Baseline::Top)
            .build();
        for (i, l) in lines.iter().enumerate() {
            if l.is_empty() {
                continue;
            }
            let y = i as i32 * LINE_PITCH;
            let _ = Text::with_text_style(l.as_str(), Point::new(64, y), style, text_style)
                .draw(&mut self.display);
        }
        self.display.flush()
    }

    pub fn show_hello(&mut self) -> Result<(), DisplayError> {
        self.display.clear_buffer();
        let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
        Text::with_alignment("Hello", Point::new(64, 28), style, Alignment::Center)
            .draw(&mut self.display)?;
        Text::with_alignment("WBA65I-DK1", Point::new(64, 50), style, Alignment::Center)
            .draw(&mut self.display)?;
        self.display.flush()
    }
}

/// Retry OLED init until success, toggling `led` each attempt for visual feedback.
pub async fn init_loop(mut oled: OledBus, mut led: Output<'static>) -> OledBus {
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        led.set_level(if attempt % 2 == 0 {
            Level::Low
        } else {
            Level::High
        });

        match oled.try_init() {
            Ok(()) => match oled.show_hello() {
                Ok(()) => {
                    led.set_level(Level::Low);
                    return oled;
                }
                Err(_) => warn!("OLED hello draw failed, retrying…"),
            },
            Err(_) => warn!("OLED init failed, retrying…"),
        }

        Timer::after_millis(500).await;
    }
}
