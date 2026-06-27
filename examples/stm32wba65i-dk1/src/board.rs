//! STM32WBA65I-DK1 board pinout (MB2130 RF + MB2143 mezzanine).
//!
//! Peripherals exposed to the Rhai playground:
//! - LD6 green (PD8), LD5 red (PD9), LD3 blue (PB10) — active-low
//! - Joystick via ADC4 IN6 (PA3)
//! - 128×64 SSD1306 OLED on SPI3 (PA0=SCK, PB8=MOSI, CS=PE1, D/C=PE0, RST=PE3)

use embassy_stm32::gpio::{Level, Output};

pub const BOARD_NAME: &str = "STM32WBA65I-DK1";
pub const BLE_ADV_NAME: &str = "RhaiPlay";
/// User LED index (active-low on the board).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LedId {
    Green = 0,
    Red = 1,
    Blue = 2,
}

impl LedId {
    pub fn from_i32(n: i32) -> Option<Self> {
        match n {
            0 => Some(Self::Green),
            1 => Some(Self::Red),
            2 => Some(Self::Blue),
            _ => None,
        }
    }
}

/// Joystick direction decoded from the ADC threshold ladder on PA3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum JoyDir {
    None = 0,
    Select = 1,
    Left = 2,
    Down = 3,
    Up = 4,
    Right = 5,
}

impl JoyDir {
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// Decode a 12-bit ADC sample (3300 mV reference) into a direction.
    ///
    /// Thresholds from the board devicetree (ST Zephyr BSP):
    /// select=0, left=670, down=1320, up=2010, right=2650, released=3300 mV.
    pub fn from_raw(raw: u16, max: u16) -> Self {
        let mv = raw as u32 * 3300 / max as u32;
        if mv >= 2975 {
            Self::None
        } else if mv < 335 {
            Self::Select
        } else if mv < 995 {
            Self::Left
        } else if mv < 1665 {
            Self::Down
        } else if mv < 2330 {
            Self::Up
        } else {
            Self::Right
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Select => "select",
            Self::Left => "left",
            Self::Down => "down",
            Self::Up => "up",
            Self::Right => "right",
        }
    }
}

/// Three active-low user LEDs on the Discovery kit.
pub struct LedBank {
    green: Output<'static>,
    red: Output<'static>,
    blue: Output<'static>,
    /// Logical on/off (not pin level).
    state: [bool; 3],
}

impl LedBank {
    pub fn new(green: Output<'static>, red: Output<'static>, blue: Output<'static>) -> Self {
        Self {
            green,
            red,
            blue,
            state: [false; 3],
        }
    }

    pub fn set(&mut self, id: LedId, on: bool) {
        self.state[id as usize] = on;
        let level = if on { Level::Low } else { Level::High };
        match id {
            LedId::Green => self.green.set_level(level),
            LedId::Red => self.red.set_level(level),
            LedId::Blue => self.blue.set_level(level),
        }
    }

    /// Flip LED `id`; returns the new logical state (`true` = on).
    pub fn toggle(&mut self, id: LedId) -> bool {
        let on = !self.state[id as usize];
        self.set(id, on);
        on
    }

    pub fn set_rgb(&mut self, r: bool, g: bool, b: bool) {
        self.set(LedId::Red, r);
        self.set(LedId::Green, g);
        self.set(LedId::Blue, b);
    }

    pub fn all_off(&mut self) {
        self.set_rgb(false, false, false);
    }

    /// Map joystick direction to a quick RGB hint (live feedback).
    pub fn show_joy(&mut self, dir: JoyDir) {
        match dir {
            JoyDir::None => self.all_off(),
            JoyDir::Select => self.set_rgb(true, true, true),
            JoyDir::Left => self.set_rgb(true, false, false),
            JoyDir::Right => self.set_rgb(false, false, true),
            JoyDir::Up => self.set_rgb(false, true, false),
            JoyDir::Down => self.set_rgb(true, true, false),
        }
    }
}
