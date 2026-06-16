//! Board support for [Waveshare RP2350-Touch-LCD-7](https://www.waveshare.com/wiki/RP2350-Touch-LCD-7).

use defmt::info;
use embassy_rp::clocks::ClockConfig;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::i2c::{Config as I2cConfig, I2c};
use embassy_rp::i2c::Blocking;
use embassy_rp::peripherals;
use embassy_rp::config::Config;
use embassy_rp::{Peri, Peripherals};

use crate::gt911::{self, TouchPoint};

pub const DISPLAY_WIDTH: usize = 800;
pub const DISPLAY_HEIGHT: usize = 480;
pub const CAN_BITRATE: u32 = 500_000;

pub mod pins {
    pub const I2C_SDA: u8 = 6;
    pub const I2C_SCL: u8 = 7;
    pub const GT911_INT: u8 = 18;
    pub const GT911_RST: u8 = 19;
    pub const CAN_SPI_SCK: u8 = 2;
    pub const CAN_SPI_MOSI: u8 = 3;
    pub const CAN_SPI_MISO: u8 = 4;
    pub const CAN_SPI_CS: u8 = 5;
    pub const CAN_INT: u8 = 1;
    pub const LCD_DE: u8 = 20;
    pub const LCD_VSYNC: u8 = 21;
    pub const LCD_HSYNC: u8 = 22;
    pub const LCD_PCLK: u8 = 23;
    pub const LCD_DATA0: u8 = 24;
    pub const LCD_RST: u8 = 41;
    pub const LCD_BL: u8 = 44;
    pub const LCD_EN: u8 = 45;
    pub const PSRAM_CS: u8 = 0;
}

pub type BoardI2c = I2c<'static, peripherals::I2C1, Blocking>;

pub fn init() -> Peripherals {
    let clock = ClockConfig::system_freq(240_000_000).unwrap();
    embassy_rp::init(Config::new(clock))
}

pub fn init_i2c(
    i2c1: Peri<'static, peripherals::I2C1>,
    scl: Peri<'static, impl embassy_rp::i2c::SclPin<peripherals::I2C1>>,
    sda: Peri<'static, impl embassy_rp::i2c::SdaPin<peripherals::I2C1>>,
) -> BoardI2c {
    I2c::new_blocking(i2c1, scl, sda, I2cConfig::default())
}

pub struct TouchPins {
    pub rst: Output<'static>,
    pub int: Input<'static>,
}

pub fn init_touch_pins(
    rst: Peri<'static, impl embassy_rp::gpio::Pin>,
    int: Peri<'static, impl embassy_rp::gpio::Pin>,
) -> TouchPins {
    TouchPins {
        rst: Output::new(rst, Level::High),
        int: Input::new(int, Pull::Up),
    }
}

pub async fn init_gt911(i2c: &mut BoardI2c, touch: &mut TouchPins) {
    gt911::init(i2c, &mut touch.rst).await;
}

pub fn read_touch(i2c: &mut BoardI2c) -> TouchPoint {
    gt911::read_touch(i2c)
}

pub struct LcdPins {
    pub rst: Output<'static>,
    pub en: Output<'static>,
    pub bl: Output<'static>,
}

pub fn init_lcd_pins(
    rst: Peri<'static, impl embassy_rp::gpio::Pin>,
    en: Peri<'static, impl embassy_rp::gpio::Pin>,
    bl: Peri<'static, impl embassy_rp::gpio::Pin>,
) -> LcdPins {
    let mut lcd = LcdPins {
        rst: Output::new(rst, Level::High),
        en: Output::new(en, Level::High),
        bl: Output::new(bl, Level::Low),
    };
    lcd.reset();
    lcd
}

impl LcdPins {
    pub fn reset(&mut self) {
        self.en.set_high();
        self.rst.set_low();
        cortex_m::asm::delay(240_000_000 / 50);
        self.rst.set_high();
        cortex_m::asm::delay(240_000_000 / 5);
    }

    pub fn set_backlight(&mut self, on: bool) {
        if on {
            self.en.set_high();
            self.rst.set_high();
            self.bl.set_high();
        } else {
            self.bl.set_low();
            self.en.set_low();
            self.rst.set_low();
        }
    }
}

pub fn init_psram(
    qmi: Peri<'static, peripherals::QMI_CS1>,
    cs: Peri<'static, impl embassy_rp::qmi_cs1::QmiCs1Pin>,
) -> Option<embassy_rp::psram::Psram<'static>> {
    use embassy_rp::psram::{Config as PsramConfig, Psram};
    use embassy_rp::qmi_cs1::QmiCs1;

    match Psram::new(QmiCs1::new(qmi, cs), PsramConfig::aps6404l()) {
        Ok(psram) => {
            info!("PSRAM ready: {} KiB", psram.size() / 1024);
            Some(psram)
        }
        Err(e) => {
            defmt::warn!("PSRAM init failed: {:?}", e);
            None
        }
    }
}

pub fn log_board_info() {
    info!(
        "RP2350-Touch-LCD-7 — {}x{} ST7262 + GT911 + XL2515",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT
    );
}
