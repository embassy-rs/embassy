//! Board support for the 4D Systems **gen4-RP2350-70CT** display module.
//!
//! Pin map from `boards/gen4_rp2350_70ct.h` in the gen4 PIO LVGL reference port.

use defmt::info;
use embassy_rp::clocks::{clk_sys_freq, ClockConfig};
use embassy_rp::config::Config;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::Blocking;
use embassy_rp::i2c::{Config as I2cConfig, I2c};
use embassy_rp::peripherals;
use embassy_rp::pwm::{ChannelBPin, Config as PwmConfig, Pwm};
use embassy_rp::{Peri, Peripherals};
use fixed::types::extra::U4;
use fixed::FixedU16;

use crate::ft5446::{self, TouchPoint};

pub const DISPLAY_WIDTH: usize = 800;
pub const DISPLAY_HEIGHT: usize = 480;

const LCD_PWM_FREQ: u32 = 5_000;
const LCD_PWM_TOP: u16 = 1_024;

pub mod pins {
    pub const PSRAM_CS: u8 = 0;
    pub const LCD_BACKLIGHT: u8 = 17;
    pub const LCD_DE: u8 = 18;
    pub const LCD_VSYNC: u8 = 19;
    pub const LCD_HSYNC: u8 = 20;
    pub const LCD_PCLK: u8 = 21;
    pub const LCD_DATA0: u8 = 22;
    pub const TOUCH_INT: u8 = 38;
    pub const TOUCH_SCL: u8 = 39;
    pub const TOUCH_SDA: u8 = 46;
    pub const TOUCH_RST: u8 = 47;
}

pub type BoardI2c = I2c<'static, peripherals::I2C1, Blocking>;

/// Match the Graphics4D reference: 230 MHz system clock.
pub fn init() -> Peripherals {
    let clock = ClockConfig::system_freq(230_000_000).unwrap();
    embassy_rp::init(Config::new(clock))
}

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
    pub int: embassy_rp::gpio::Flex<'static>,
}

pub fn init_touch_pins(
    rst: Peri<'static, impl embassy_rp::gpio::Pin>,
    int: Peri<'static, impl embassy_rp::gpio::Pin>,
) -> TouchPins {
    TouchPins {
        rst: Output::new(rst, Level::High),
        int: embassy_rp::gpio::Flex::new(int),
    }
}

pub async fn init_ft5446(i2c: &mut BoardI2c, touch: &mut TouchPins) {
    ft5446::init(i2c, &mut touch.rst, &mut touch.int).await;
}

pub fn read_touch(i2c: &mut BoardI2c) -> TouchPoint {
    ft5446::read_touch(i2c)
}

pub struct Backlight {
    pwm: Pwm<'static>,
}

fn backlight_pwm_config(level: u8) -> PwmConfig {
    let sys_clk = clk_sys_freq();
    let div = sys_clk / (LCD_PWM_FREQ * u32::from(LCD_PWM_TOP));
    let mut cfg = PwmConfig::default();
    cfg.top = LCD_PWM_TOP;
    cfg.divider = FixedU16::<U4>::from_num(div);
    // Graphics4D `Contrast(15)` ≈ full brightness (PWM 1023).
    let compare = u32::from(level.min(15)) * u32::from(LCD_PWM_TOP) / 15;
    cfg.compare_a = 0;
    cfg.compare_b = compare as u16;
    cfg.enable = level > 0;
    cfg
}

pub fn init_backlight(
    slice: Peri<'static, peripherals::PWM_SLICE0>,
    pin: Peri<'static, impl ChannelBPin<peripherals::PWM_SLICE0>>,
) -> Backlight {
    Backlight {
        pwm: Pwm::new_output_b(slice, pin, backlight_pwm_config(15)),
    }
}

impl Backlight {
    pub fn set_level(&mut self, level: u8) {
        self.pwm.set_config(&backlight_pwm_config(level));
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
        "gen4-RP2350-70CT — {}x{} RGB565 PIO + FT5446",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT
    );
}
