//! Board support for the [4D Systems gen4-RP2350-70CT](https://4dsystems.com.au/)
//! (RP2350B + 7" 800x480 RGB panel + FT5446 capacitive touch + APS6404 PSRAM).
//!
//! Pin assignments mirror the vendor board header `gen4_rp2350_70ct.h`.

use defmt::info;
use embassy_rp::clocks::ClockConfig;
use embassy_rp::config::Config;
use embassy_rp::gpio::{Flex, Level, Output};
use embassy_rp::i2c::{Blocking, Config as I2cConfig, I2c};
use embassy_rp::{Peri, Peripherals, peripherals};

use crate::ft5446::{self, TouchPoint};

pub const DISPLAY_WIDTH: usize = 800;
pub const DISPLAY_HEIGHT: usize = 480;

/// GPIO map taken from the 4D Systems `gen4_rp2350_70ct.h` board header.
pub mod pins {
    pub const LCD_BACKLIGHT: u8 = 17;
    pub const LCD_DE: u8 = 18;
    pub const LCD_VSYNC: u8 = 19;
    pub const LCD_HSYNC: u8 = 20;
    pub const LCD_PCLK: u8 = 21;
    /// DATA0 (blue LSB); 16 consecutive RGB565 lines GPIO22..=37.
    pub const LCD_DATA0: u8 = 22;
    pub const TOUCH_INT: u8 = 38;
    pub const TOUCH_SCL: u8 = 39;
    pub const TOUCH_SDA: u8 = 46;
    pub const TOUCH_RST: u8 = 47;
    /// PSRAM chip-select drives QMI CS1 (XIP_CS1).
    pub const PSRAM_CS: u8 = 0;
}

pub type BoardI2c = I2c<'static, peripherals::I2C1, Blocking>;

/// Initialise the RP2350 with a 230 MHz system clock to match the vendor
/// `Graphics4D` firmware (`set_sys_clock_khz(230000)`), from which the PIO RGB
/// scan-out dividers (sync = sys/36 MHz) are derived.
pub fn init() -> Peripherals {
    let clock = ClockConfig::system_freq(230_000_000).unwrap();
    embassy_rp::init(Config::new(clock))
}

/// Bring up the FT5446 touch I2C bus (I2C1, 400 kHz) on SCL=GPIO39 / SDA=GPIO46.
pub fn init_i2c(
    i2c1: Peri<'static, peripherals::I2C1>,
    scl: Peri<'static, impl embassy_rp::i2c::SclPin<peripherals::I2C1>>,
    sda: Peri<'static, impl embassy_rp::i2c::SdaPin<peripherals::I2C1>>,
) -> BoardI2c {
    let mut config = I2cConfig::default();
    config.frequency = 400_000;
    I2c::new_blocking(i2c1, scl, sda, config)
}

pub struct TouchPins {
    pub rst: Output<'static>,
    pub int: Flex<'static>,
}

pub fn init_touch_pins(
    rst: Peri<'static, impl embassy_rp::gpio::Pin>,
    int: Peri<'static, impl embassy_rp::gpio::Pin>,
) -> TouchPins {
    TouchPins {
        rst: Output::new(rst, Level::High),
        int: Flex::new(int),
    }
}

pub async fn init_ft5446(i2c: &mut BoardI2c, touch: &mut TouchPins) {
    ft5446::init(i2c, &mut touch.rst, &mut touch.int).await;
}

pub fn read_touch(i2c: &mut BoardI2c) -> TouchPoint {
    ft5446::read_touch(i2c)
}

/// LCD backlight control (active-high GPIO on the gen4-70CT).
pub struct LcdPins {
    bl: Output<'static>,
}

pub fn init_lcd_pins(bl: Peri<'static, impl embassy_rp::gpio::Pin>) -> LcdPins {
    LcdPins {
        bl: Output::new(bl, Level::Low),
    }
}

impl LcdPins {
    pub fn set_backlight(&mut self, on: bool) {
        if on {
            self.bl.set_high();
        } else {
            self.bl.set_low();
        }
        info!("LCD backlight {}", if on { "on" } else { "off" });
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
        "gen4-RP2350-70CT — {}x{} RGB panel + FT5446 touch",
        DISPLAY_WIDTH, DISPLAY_HEIGHT
    );
}
