//! Board support for [Waveshare RP2350-Touch-LCD-7](https://www.waveshare.com/wiki/RP2350-Touch-LCD-7).

use defmt::info;
use embassy_rp::clocks::{clk_sys_freq, ClockConfig};
use embassy_rp::gpio::{Flex, Level, Output};
use embassy_rp::i2c::{Config as I2cConfig, I2c};
use embassy_rp::i2c::Blocking;
use embassy_rp::peripherals;
use embassy_rp::pwm::{ChannelAPin, Config as PwmConfig, Pwm};
use embassy_rp::config::Config;
use embassy_rp::{Peri, Peripherals};
use fixed::types::extra::U4;
use fixed::FixedU16;

use crate::gt911::{self, TouchPoint};

pub const DISPLAY_WIDTH: usize = 800;
pub const DISPLAY_HEIGHT: usize = 480;
pub const CAN_BITRATE: u32 = 500_000;
const LCD_PWM_FREQ: u32 = 5_000;
const LCD_PWM_TOP: u16 = 1_000;

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

pub async fn init_gt911(i2c: &mut BoardI2c, touch: &mut TouchPins) {
    gt911::init(i2c, &mut touch.rst, &mut touch.int).await;
}

pub fn read_touch(i2c: &mut BoardI2c) -> TouchPoint {
    gt911::read_touch(i2c)
}

pub struct LcdPins {
    pub rst: Output<'static>,
    pub en: Output<'static>,
    bl_pwm: Pwm<'static>,
}

fn lcd_pwm_config(percent: u8) -> PwmConfig {
    let sys_clk = clk_sys_freq();
    let div = sys_clk / (LCD_PWM_FREQ * u32::from(LCD_PWM_TOP));
    let mut cfg = PwmConfig::default();
    cfg.top = LCD_PWM_TOP;
    cfg.divider = FixedU16::<U4>::from_num(div);
    // Waveshare `bsp_lcd_set_brightness`: pwm level 0 = brightest (active-low BL).
    cfg.compare_a = (u32::from(LCD_PWM_TOP) * (100 - u32::from(percent.min(100))) / 100) as u16;
    cfg.compare_b = 0;
    cfg.enable = percent > 0;
    cfg
}

pub fn init_lcd_pins(
    rst: Peri<'static, impl embassy_rp::gpio::Pin>,
    en: Peri<'static, impl embassy_rp::gpio::Pin>,
    bl_slice: Peri<'static, peripherals::PWM_SLICE10>,
    bl: Peri<'static, impl ChannelAPin<peripherals::PWM_SLICE10>>,
) -> LcdPins {
    let mut lcd = LcdPins {
        rst: Output::new(rst, Level::High),
        en: Output::new(en, Level::High),
        bl_pwm: Pwm::new_output_a(bl_slice, bl, {
            let mut cfg = lcd_pwm_config(0);
            cfg.enable = false;
            cfg
        }),
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
        self.set_brightness(if on { 100 } else { 0 });
    }

    pub fn set_brightness(&mut self, percent: u8) {
        if percent == 0 {
            self.en.set_low();
            self.rst.set_low();
            let mut cfg = lcd_pwm_config(0);
            cfg.enable = false;
            self.bl_pwm.set_config(&cfg);
            info!("LCD backlight off");
            return;
        }

        self.en.set_high();
        self.rst.set_high();

        // Use set_config (channel A only). Avoid set_duty_cycle() — it also writes
        // compare B on GPIO 45 (LCD EN) because both share PWM slice 10.
        let cfg = lcd_pwm_config(percent);
        self.bl_pwm.set_config(&cfg);
        info!(
            "LCD backlight {}% (PWM compare_a={})",
            percent, cfg.compare_a
        );
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
