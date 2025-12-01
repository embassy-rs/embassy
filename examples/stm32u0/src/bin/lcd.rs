#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::lcd::{Bias, BlinkFreq, BlinkSelector, Config, Duty, Lcd, LcdPin};
use embassy_stm32::peripherals::LCD;
use embassy_stm32::time::Hertz;
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // The RTC clock = the LCD clock and must be running
    {
        use embassy_stm32::rcc::*;
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.hsi = true;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI, // 16 MHz
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL7, // 16 * 7 = 112 MHz
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2), // 112 / 2 = 56 MHz
        });
        config.rcc.ls = LsConfig::default_lsi();
    }

    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut config = Config::default();
    config.bias = Bias::Third;
    config.duty = Duty::Quarter;
    config.target_fps = Hertz(100);

    let mut lcd = Lcd::new(
        p.LCD,
        config,
        p.PC3,
        [
            LcdPin::new_com(p.PA8),
            LcdPin::new_com(p.PA9),
            LcdPin::new_com(p.PA10),
            LcdPin::new_seg(p.PB1),
            LcdPin::new_com(p.PB9),
            LcdPin::new_seg(p.PB11),
            LcdPin::new_seg(p.PB14),
            LcdPin::new_seg(p.PB15),
            LcdPin::new_seg(p.PC4),
            LcdPin::new_seg(p.PC5),
            LcdPin::new_seg(p.PC6),
            LcdPin::new_seg(p.PC8),
            LcdPin::new_seg(p.PC9),
            LcdPin::new_seg(p.PC10),
            LcdPin::new_seg(p.PC11),
            LcdPin::new_seg(p.PD8),
            LcdPin::new_seg(p.PD9),
            LcdPin::new_seg(p.PD12),
            LcdPin::new_seg(p.PD13),
            LcdPin::new_seg(p.PD0),
            LcdPin::new_seg(p.PD1),
            LcdPin::new_seg(p.PD3),
            LcdPin::new_seg(p.PD4),
            LcdPin::new_seg(p.PD5),
            LcdPin::new_seg(p.PD6),
            LcdPin::new_seg(p.PE7),
            LcdPin::new_seg(p.PE8),
            LcdPin::new_seg(p.PE9),
        ],
    );

    lcd.set_blink(BlinkSelector::All, BlinkFreq::Hz4);
    {
        let mut buffer = DisplayBuffer::new();
        for i in 0..4 {
            buffer.write_colon(i);
            buffer.write(&mut lcd);
            embassy_time::Timer::after_millis(200).await;
            buffer.write_dot(i);
            buffer.write(&mut lcd);
            embassy_time::Timer::after_millis(200).await;
        }
        for i in 0..4 {
            buffer.write_bar(i);
            buffer.write(&mut lcd);
            embassy_time::Timer::after_millis(200).await;
        }
    }

    embassy_time::Timer::after_millis(1000).await;

    lcd.set_blink(BlinkSelector::None, BlinkFreq::Hz4);

    const MESSAGE: &str = "Hello embassy people. Hope you like this LCD demo :}      ";
    loop {
        print_message(MESSAGE, &mut lcd, Duration::from_millis(250)).await;
        print_message(characters::ALL_CHARS, &mut lcd, Duration::from_millis(500)).await;
    }
}

async fn print_message(message: &str, lcd: &mut Lcd<'_, LCD>, delay: Duration) {
    let mut display_buffer = DisplayBuffer::new();

    let mut char_buffer = [' '; 6];
    for char in message.chars() {
        char_buffer.copy_within(1.., 0);
        char_buffer[5] = char;

        display_buffer.clear();
        for (i, char) in char_buffer.iter().enumerate() {
            display_buffer.write_char(i, *char);
        }
        display_buffer.write(lcd);

        embassy_time::Timer::after(delay).await;
    }
}

/// Display layout for the U0-DK
mod display_layout {
    // Character layout. There are 6 characters, left-to-right
    //         T
    //     ─────────
    //    │    N    │
    //    │ │  │  │ │
    // TL │ └┐ │ ┌┘ │ TR
    //    │NW│ │ │NE│
    //    │    │    │
    //     W─── ───E
    //    │    │    │
    //    │SW│ │ │SE│
    // BL │ ┌┘ │ └┐ │ BR
    //    │ │  │  │ │
    //    │    S    │
    //     ─────────
    //         B

    pub const CHAR_N_COM: u8 = 3;
    pub const CHAR_N_SEG: [u8; 6] = [39, 37, 35, 48, 26, 33];
    pub const CHAR_NW_COM: u8 = 3;
    pub const CHAR_NW_SEG: [u8; 6] = [49, 38, 36, 34, 27, 24];
    pub const CHAR_W_COM: u8 = 0;
    pub const CHAR_W_SEG: [u8; 6] = CHAR_NW_SEG;
    pub const CHAR_SW_COM: u8 = 2;
    pub const CHAR_SW_SEG: [u8; 6] = CHAR_NW_SEG;
    pub const CHAR_S_COM: u8 = 2;
    pub const CHAR_S_SEG: [u8; 6] = [22, 6, 46, 11, 15, 29];
    pub const CHAR_SE_COM: u8 = 3;
    pub const CHAR_SE_SEG: [u8; 6] = CHAR_S_SEG;
    pub const CHAR_E_COM: u8 = 0;
    pub const CHAR_E_SEG: [u8; 6] = [23, 45, 47, 14, 28, 32];
    pub const CHAR_NE_COM: u8 = 2;
    pub const CHAR_NE_SEG: [u8; 6] = CHAR_N_SEG;
    pub const CHAR_T_COM: u8 = 1;
    pub const CHAR_T_SEG: [u8; 6] = CHAR_N_SEG;
    pub const CHAR_TL_COM: u8 = 1;
    pub const CHAR_TL_SEG: [u8; 6] = CHAR_NW_SEG;
    pub const CHAR_BL_COM: u8 = 0;
    pub const CHAR_BL_SEG: [u8; 6] = CHAR_S_SEG;
    pub const CHAR_B_COM: u8 = 1;
    pub const CHAR_B_SEG: [u8; 6] = CHAR_S_SEG;
    pub const CHAR_BR_COM: u8 = 1;
    pub const CHAR_BR_SEG: [u8; 6] = CHAR_E_SEG;
    pub const CHAR_TR_COM: u8 = 0;
    pub const CHAR_TR_SEG: [u8; 6] = CHAR_N_SEG;

    pub const COLON_COM: u8 = 2;
    pub const COLON_SEG: [u8; 4] = [23, 45, 47, 14];
    pub const DOT_COM: u8 = 3;
    pub const DOT_SEG: [u8; 4] = COLON_SEG;
    /// COM + SEG, bar from top to bottom
    pub const BAR: [(u8, u8); 4] = [(2, 28), (3, 28), (2, 32), (3, 32)];
}

mod characters {
    use super::CharSegment::{self, *};

    pub const CHAR_0: &[CharSegment] = &[T, TL, BL, B, BR, TR, NW, SE];
    pub const CHAR_1: &[CharSegment] = &[NE, TR, BR];
    pub const CHAR_2: &[CharSegment] = &[T, BL, B, TR, E, W];
    pub const CHAR_3: &[CharSegment] = &[T, B, BR, TR, E];
    pub const CHAR_4: &[CharSegment] = &[TL, BR, TR, E, W];
    pub const CHAR_5: &[CharSegment] = &[T, TL, B, BR, E, W];
    pub const CHAR_6: &[CharSegment] = &[T, TL, BL, B, BR, E, W];
    pub const CHAR_7: &[CharSegment] = &[T, NE, S];
    pub const CHAR_8: &[CharSegment] = &[T, TL, BL, B, BR, TR, E, W];
    pub const CHAR_9: &[CharSegment] = &[T, TL, BR, TR, E, W];

    pub const CHAR_COLON: &[CharSegment] = &[N, S];
    pub const CHAR_SEMICOLON: &[CharSegment] = &[N, SW];
    pub const CHAR_EQUALS: &[CharSegment] = &[E, W, B];
    pub const CHAR_SLASH: &[CharSegment] = &[SW, NE];
    pub const CHAR_BACKSLASH: &[CharSegment] = &[SE, NW];
    pub const CHAR_PLUS: &[CharSegment] = &[N, E, S, W];
    pub const CHAR_STAR: &[CharSegment] = &[NE, N, NW, SE, S, SW];
    pub const CHAR_QUOTE: &[CharSegment] = &[N];
    pub const CHAR_BACKTICK: &[CharSegment] = &[NW];
    pub const CHAR_DASH: &[CharSegment] = &[W, E];
    pub const CHAR_COMMA: &[CharSegment] = &[SW];
    pub const CHAR_DOT: &[CharSegment] = &[S];
    pub const CHAR_CURLYOPEN: &[CharSegment] = &[T, NW, W, SW, B];
    pub const CHAR_CURLYCLOSE: &[CharSegment] = &[T, NE, E, SE, B];
    pub const CHAR_AMPERSAND: &[CharSegment] = &[T, NE, NW, W, BL, B, SE];

    pub const CHAR_A: &[CharSegment] = &[T, TL, TR, E, W, BL, BR];
    pub const CHAR_B: &[CharSegment] = &[T, TR, BR, B, N, S, E];
    pub const CHAR_C: &[CharSegment] = &[T, TL, BL, B];
    pub const CHAR_D: &[CharSegment] = &[T, TR, BR, B, N, S];
    pub const CHAR_E: &[CharSegment] = &[T, TL, BL, B, W];
    pub const CHAR_F: &[CharSegment] = &[T, TL, BL, W];
    pub const CHAR_G: &[CharSegment] = &[T, TL, BL, B, BR, E];
    pub const CHAR_H: &[CharSegment] = &[TL, BL, E, W, TR, BR];
    pub const CHAR_I: &[CharSegment] = &[T, N, S, B];
    pub const CHAR_J: &[CharSegment] = &[TR, BR, B, BL];
    pub const CHAR_K: &[CharSegment] = &[TL, BL, W, NE, SE];
    pub const CHAR_L: &[CharSegment] = &[TL, BL, B];
    pub const CHAR_M: &[CharSegment] = &[BL, TL, NW, NE, TR, BR];
    pub const CHAR_N: &[CharSegment] = &[BL, TL, NW, SE, BR, TR];
    pub const CHAR_O: &[CharSegment] = &[T, TL, BL, B, BR, TR];
    pub const CHAR_P: &[CharSegment] = &[BL, TL, T, TR, E, W];
    pub const CHAR_Q: &[CharSegment] = &[T, TL, BL, B, BR, TR, SE];
    pub const CHAR_R: &[CharSegment] = &[BL, TL, T, TR, E, W, SE];
    pub const CHAR_S: &[CharSegment] = &[T, NW, E, BR, B];
    pub const CHAR_T: &[CharSegment] = &[T, N, S];
    pub const CHAR_U: &[CharSegment] = &[TL, BL, B, BR, TR];
    pub const CHAR_V: &[CharSegment] = &[TL, BL, SW, NE];
    pub const CHAR_W: &[CharSegment] = &[TL, BL, SW, SE, BR, TR];
    pub const CHAR_X: &[CharSegment] = &[NE, NW, SE, SW];
    pub const CHAR_Y: &[CharSegment] = &[NE, NW, S];
    pub const CHAR_Z: &[CharSegment] = &[T, NE, SW, B];

    pub const CHAR_UNKNOWN: &[CharSegment] = &[N, NW, W, SW, S, SE, E, NE, T, TL, BL, B, BR, TR];

    pub const ALL_CHARS: &str =
        "0 1 2 3 4 5 6 7 8 9 : ; = / \\ + * ' ` - , . { } & A B C D E F G H I J K L M N O P Q R S T U V W X Y Z � ";

    pub fn get_char_segments(val: char) -> &'static [CharSegment] {
        match val {
            val if val.is_whitespace() => &[],

            '0' => CHAR_0,
            '1' => CHAR_1,
            '2' => CHAR_2,
            '3' => CHAR_3,
            '4' => CHAR_4,
            '5' => CHAR_5,
            '6' => CHAR_6,
            '7' => CHAR_7,
            '8' => CHAR_8,
            '9' => CHAR_9,

            ':' => CHAR_COLON,
            ';' => CHAR_SEMICOLON,
            '=' => CHAR_EQUALS,
            '/' => CHAR_SLASH,
            '\\' => CHAR_BACKSLASH,
            '+' => CHAR_PLUS,
            '*' => CHAR_STAR,
            '\'' => CHAR_QUOTE,
            '`' => CHAR_BACKTICK,
            '-' => CHAR_DASH,
            ',' => CHAR_COMMA,
            '.' => CHAR_DOT,
            '{' => CHAR_CURLYOPEN,
            '}' => CHAR_CURLYCLOSE,
            '&' => CHAR_AMPERSAND,

            'A' | 'a' => CHAR_A,
            'B' | 'b' => CHAR_B,
            'C' | 'c' => CHAR_C,
            'D' | 'd' => CHAR_D,
            'E' | 'e' => CHAR_E,
            'F' | 'f' => CHAR_F,
            'G' | 'g' => CHAR_G,
            'H' | 'h' => CHAR_H,
            'I' | 'i' => CHAR_I,
            'J' | 'j' => CHAR_J,
            'K' | 'k' => CHAR_K,
            'L' | 'l' => CHAR_L,
            'M' | 'm' => CHAR_M,
            'N' | 'n' => CHAR_N,
            'O' | 'o' => CHAR_O,
            'P' | 'p' => CHAR_P,
            'Q' | 'q' => CHAR_Q,
            'R' | 'r' => CHAR_R,
            'S' | 's' => CHAR_S,
            'T' | 't' => CHAR_T,
            'U' | 'u' => CHAR_U,
            'V' | 'v' => CHAR_V,
            'W' | 'w' => CHAR_W,
            'X' | 'x' => CHAR_X,
            'Y' | 'y' => CHAR_Y,
            'Z' | 'z' => CHAR_Z,

            _ => CHAR_UNKNOWN,
        }
    }
}

pub struct DisplayBuffer {
    pixels: [u64; 4],
}

impl DisplayBuffer {
    pub const fn new() -> Self {
        Self { pixels: [0; 4] }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    fn write_char_segment(&mut self, index: usize, value: CharSegment) {
        defmt::assert!(index < 6);
        let (com, segments) = value.get_com_seg();
        self.pixels[com as usize] |= 1 << segments[index];
    }

    pub fn write_char(&mut self, index: usize, val: char) {
        let segments = characters::get_char_segments(val);

        for segment in segments {
            self.write_char_segment(index, *segment);
        }
    }

    pub fn write(&self, lcd: &mut Lcd<'_, LCD>) {
        lcd.write_com_segments(0, self.pixels[0]);
        lcd.write_com_segments(1, self.pixels[1]);
        lcd.write_com_segments(2, self.pixels[2]);
        lcd.write_com_segments(3, self.pixels[3]);
        lcd.submit_frame();
    }

    pub fn write_colon(&mut self, index: usize) {
        defmt::assert!(index < 4);
        self.pixels[display_layout::COLON_COM as usize] |= 1 << display_layout::COLON_SEG[index];
    }

    pub fn write_dot(&mut self, index: usize) {
        defmt::assert!(index < 4);
        self.pixels[display_layout::DOT_COM as usize] |= 1 << display_layout::DOT_SEG[index];
    }

    pub fn write_bar(&mut self, index: usize) {
        defmt::assert!(index < 4);
        let (bar_com, bar_seg) = display_layout::BAR[index];
        self.pixels[bar_com as usize] |= 1 << bar_seg;
    }
}

impl Default for DisplayBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum CharSegment {
    /// North
    N,
    /// North west
    NW,
    /// West
    W,
    /// South west
    SW,
    /// South
    S,
    /// South East
    SE,
    /// East
    E,
    /// North East
    NE,
    /// Top
    T,
    /// Top left
    TL,
    /// Bottom left
    BL,
    /// Bottom
    B,
    /// Bottom right
    BR,
    /// Top right
    TR,
}

impl CharSegment {
    fn get_com_seg(&self) -> (u8, [u8; 6]) {
        match self {
            CharSegment::N => (display_layout::CHAR_N_COM, display_layout::CHAR_N_SEG),
            CharSegment::NW => (display_layout::CHAR_NW_COM, display_layout::CHAR_NW_SEG),
            CharSegment::W => (display_layout::CHAR_W_COM, display_layout::CHAR_W_SEG),
            CharSegment::SW => (display_layout::CHAR_SW_COM, display_layout::CHAR_SW_SEG),
            CharSegment::S => (display_layout::CHAR_S_COM, display_layout::CHAR_S_SEG),
            CharSegment::SE => (display_layout::CHAR_SE_COM, display_layout::CHAR_SE_SEG),
            CharSegment::E => (display_layout::CHAR_E_COM, display_layout::CHAR_E_SEG),
            CharSegment::NE => (display_layout::CHAR_NE_COM, display_layout::CHAR_NE_SEG),
            CharSegment::T => (display_layout::CHAR_T_COM, display_layout::CHAR_T_SEG),
            CharSegment::TL => (display_layout::CHAR_TL_COM, display_layout::CHAR_TL_SEG),
            CharSegment::BL => (display_layout::CHAR_BL_COM, display_layout::CHAR_BL_SEG),
            CharSegment::B => (display_layout::CHAR_B_COM, display_layout::CHAR_B_SEG),
            CharSegment::BR => (display_layout::CHAR_BR_COM, display_layout::CHAR_BR_SEG),
            CharSegment::TR => (display_layout::CHAR_TR_COM, display_layout::CHAR_TR_SEG),
        }
    }
}
