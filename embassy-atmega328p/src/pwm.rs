//! Fast-PWM outputs backed by Timer0 and Timer2.

use core::ptr::{read_volatile, write_volatile};

use crate::gpio::{PB3, PD3, PD5, PD6};

const DDRB: *mut u8 = 0x24 as *mut u8;
const DDRD: *mut u8 = 0x2a as *mut u8;
const TCCR0A: *mut u8 = 0x44 as *mut u8;
const TCCR0B: *mut u8 = 0x45 as *mut u8;
const OCR0A: *mut u8 = 0x47 as *mut u8;
const OCR0B: *mut u8 = 0x48 as *mut u8;
const TCCR2A: *mut u8 = 0xb0 as *mut u8;
const TCCR2B: *mut u8 = 0xb1 as *mut u8;
const OCR2A: *mut u8 = 0xb3 as *mut u8;
const OCR2B: *mut u8 = 0xb4 as *mut u8;

const COM_A1: u8 = 1 << 7;
const COM_B1: u8 = 1 << 5;
const WGM_8BIT_FAST: u8 = (1 << 1) | (1 << 0);

/// Ownership token for Timer0.
pub struct Timer0 {
    _private: (),
}

impl Timer0 {
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

/// Ownership token for Timer2.
pub struct Timer2 {
    _private: (),
}

impl Timer2 {
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

/// Timer0 clock prescaler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Timer0Prescaler {
    Div1 = 1,
    Div8 = 2,
    Div64 = 3,
    Div256 = 4,
    Div1024 = 5,
}

impl Timer0Prescaler {
    const fn divisor(self) -> u32 {
        match self {
            Self::Div1 => 1,
            Self::Div8 => 8,
            Self::Div64 => 64,
            Self::Div256 => 256,
            Self::Div1024 => 1024,
        }
    }
}

/// Timer2 clock prescaler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Timer2Prescaler {
    Div1 = 1,
    Div8 = 2,
    Div32 = 3,
    Div64 = 4,
    Div128 = 5,
    Div256 = 6,
    Div1024 = 7,
}

impl Timer2Prescaler {
    const fn divisor(self) -> u32 {
        match self {
            Self::Div1 => 1,
            Self::Div8 => 8,
            Self::Div32 => 32,
            Self::Div64 => 64,
            Self::Div128 => 128,
            Self::Div256 => 256,
            Self::Div1024 => 1024,
        }
    }
}

/// Two-channel 8-bit PWM using Timer0: channel A on PD6, channel B on PD5.
pub struct Pwm0 {
    timer: Timer0,
    channel_a: PD6,
    channel_b: PD5,
    prescaler: Timer0Prescaler,
}

impl Pwm0 {
    /// Starts Timer0 in non-inverting fast-PWM mode.
    pub fn new(timer: Timer0, channel_a: PD6, channel_b: PD5, prescaler: Timer0Prescaler) -> Self {
        avr_device::interrupt::free(|_| unsafe {
            modify(DDRD, (1 << 6) | (1 << 5), true);
            write_volatile(OCR0A, 0);
            write_volatile(OCR0B, 0);
            write_volatile(TCCR0A, COM_A1 | COM_B1 | WGM_8BIT_FAST);
            write_volatile(TCCR0B, prescaler as u8);
        });
        Self {
            timer,
            channel_a,
            channel_b,
            prescaler,
        }
    }

    /// Sets channel A (PD6) duty, from 0 to 255.
    pub fn set_duty_a(&mut self, duty: u8) {
        unsafe { write_volatile(OCR0A, duty) };
    }

    /// Sets channel B (PD5) duty, from 0 to 255.
    pub fn set_duty_b(&mut self, duty: u8) {
        unsafe { write_volatile(OCR0B, duty) };
    }

    /// Returns the fast-PWM repetition frequency.
    pub const fn frequency_hz(&self) -> u32 {
        crate::CPU_HZ / self.prescaler.divisor() / 256
    }

    /// Stops Timer0 and returns its ownership tokens.
    pub fn free(self) -> (Timer0, PD6, PD5) {
        unsafe {
            write_volatile(TCCR0A, 0);
            write_volatile(TCCR0B, 0);
        }
        (self.timer, self.channel_a, self.channel_b)
    }
}

/// Two-channel 8-bit PWM using Timer2: channel A on PB3, channel B on PD3.
pub struct Pwm2 {
    timer: Timer2,
    channel_a: PB3,
    channel_b: PD3,
    prescaler: Timer2Prescaler,
}

impl Pwm2 {
    /// Starts Timer2 in non-inverting fast-PWM mode.
    pub fn new(timer: Timer2, channel_a: PB3, channel_b: PD3, prescaler: Timer2Prescaler) -> Self {
        avr_device::interrupt::free(|_| unsafe {
            modify(DDRB, 1 << 3, true);
            modify(DDRD, 1 << 3, true);
            write_volatile(OCR2A, 0);
            write_volatile(OCR2B, 0);
            write_volatile(TCCR2A, COM_A1 | COM_B1 | WGM_8BIT_FAST);
            write_volatile(TCCR2B, prescaler as u8);
        });
        Self {
            timer,
            channel_a,
            channel_b,
            prescaler,
        }
    }

    /// Sets channel A (PB3) duty, from 0 to 255.
    pub fn set_duty_a(&mut self, duty: u8) {
        unsafe { write_volatile(OCR2A, duty) };
    }

    /// Sets channel B (PD3) duty, from 0 to 255.
    pub fn set_duty_b(&mut self, duty: u8) {
        unsafe { write_volatile(OCR2B, duty) };
    }

    /// Returns the fast-PWM repetition frequency.
    pub const fn frequency_hz(&self) -> u32 {
        crate::CPU_HZ / self.prescaler.divisor() / 256
    }

    /// Stops Timer2 and returns its ownership tokens.
    pub fn free(self) -> (Timer2, PB3, PD3) {
        unsafe {
            write_volatile(TCCR2A, 0);
            write_volatile(TCCR2B, 0);
        }
        (self.timer, self.channel_a, self.channel_b)
    }
}

unsafe fn modify(register: *mut u8, mask: u8, set: bool) {
    let current = unsafe { read_volatile(register) };
    let value = if set { current | mask } else { current & !mask };
    unsafe { write_volatile(register, value) };
}
