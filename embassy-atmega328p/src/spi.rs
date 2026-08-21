//! Blocking SPI master driver.

use core::convert::Infallible;
use core::ptr::{read_volatile, write_volatile};

use embedded_hal::spi::{ErrorType, MODE_0, Mode, Phase, Polarity, SpiBus};

use crate::gpio::{PB2, PB3, PB4, PB5};

const DDRB: *mut u8 = 0x24 as *mut u8;
const PORTB: *mut u8 = 0x25 as *mut u8;
const SPCR: *mut u8 = 0x4c as *mut u8;
const SPSR: *mut u8 = 0x4d as *mut u8;
const SPDR: *mut u8 = 0x4e as *mut u8;

const SPE: u8 = 1 << 6;
const DORD: u8 = 1 << 5;
const MSTR: u8 = 1 << 4;
const CPOL: u8 = 1 << 3;
const CPHA: u8 = 1 << 2;
const SPIF: u8 = 1 << 7;
const SPI2X: u8 = 1 << 0;

/// Ownership token for the SPI peripheral.
pub struct SpiPeripheral {
    _private: (),
}

impl SpiPeripheral {
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

/// SPI bit transmission order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitOrder {
    MostSignificantFirst,
    LeastSignificantFirst,
}

/// SPI configuration error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigError {
    InvalidFrequency,
}

/// SPI master configuration.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub frequency_hz: u32,
    pub mode: Mode,
    pub bit_order: BitOrder,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency_hz: 1_000_000,
            mode: MODE_0,
            bit_order: BitOrder::MostSignificantFirst,
        }
    }
}

/// Blocking SPI master using PB2/SS, PB3/MOSI, PB4/MISO and PB5/SCK.
pub struct Spi {
    peripheral: SpiPeripheral,
    ss: PB2,
    mosi: PB3,
    miso: PB4,
    sck: PB5,
}

impl Spi {
    /// Configures the hardware SPI master at or below `frequency_hz`.
    pub fn new(
        peripheral: SpiPeripheral,
        ss: PB2,
        mosi: PB3,
        miso: PB4,
        sck: PB5,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let (spr, double_speed) = divider_bits(config.frequency_hz)?;
        let mut control = SPE | MSTR | spr;
        if config.bit_order == BitOrder::LeastSignificantFirst {
            control |= DORD;
        }
        if config.mode.polarity == Polarity::IdleHigh {
            control |= CPOL;
        }
        if config.mode.phase == Phase::CaptureOnSecondTransition {
            control |= CPHA;
        }

        avr_device::interrupt::free(|_| unsafe {
            // Keep SS as a driven-high output so the peripheral remains master.
            modify(PORTB, 1 << 2, true);
            modify(DDRB, (1 << 2) | (1 << 3) | (1 << 5), true);
            modify(DDRB, 1 << 4, false);
            modify(PORTB, 1 << 5, config.mode.polarity == Polarity::IdleHigh);
            write_volatile(SPCR, control);
            write_volatile(SPSR, if double_speed { SPI2X } else { 0 });
        });

        Ok(Self {
            peripheral,
            ss,
            mosi,
            miso,
            sck,
        })
    }

    /// Exchanges one byte with the selected slave.
    pub fn transfer_byte(&mut self, byte: u8) -> u8 {
        unsafe {
            write_volatile(SPDR, byte);
            while read_volatile(SPSR) & SPIF == 0 {}
            read_volatile(SPDR)
        }
    }

    /// Disables SPI and returns its ownership tokens.
    pub fn free(self) -> (SpiPeripheral, PB2, PB3, PB4, PB5) {
        unsafe { write_volatile(SPCR, 0) };
        (self.peripheral, self.ss, self.mosi, self.miso, self.sck)
    }
}

impl ErrorType for Spi {
    type Error = Infallible;
}

impl SpiBus<u8> for Spi {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        for word in words {
            *word = self.transfer_byte(0);
        }
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        for &word in words {
            self.transfer_byte(word);
        }
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        let count = read.len().max(write.len());
        for index in 0..count {
            let received = self.transfer_byte(write.get(index).copied().unwrap_or(0));
            if let Some(slot) = read.get_mut(index) {
                *slot = received;
            }
        }
        Ok(())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        for word in words {
            *word = self.transfer_byte(*word);
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn divider_bits(frequency_hz: u32) -> Result<(u8, bool), ConfigError> {
    if frequency_hz == 0 {
        return Err(ConfigError::InvalidFrequency);
    }
    let required = crate::CPU_HZ.div_ceil(frequency_hz);
    let result = if required <= 2 {
        (0b00, true)
    } else if required <= 4 {
        (0b00, false)
    } else if required <= 8 {
        (0b01, true)
    } else if required <= 16 {
        (0b01, false)
    } else if required <= 32 {
        (0b10, true)
    } else if required <= 64 {
        (0b10, false)
    } else if required <= 128 {
        (0b11, false)
    } else {
        return Err(ConfigError::InvalidFrequency);
    };
    Ok(result)
}

unsafe fn modify(register: *mut u8, mask: u8, set: bool) {
    let current = unsafe { read_volatile(register) };
    let value = if set { current | mask } else { current & !mask };
    unsafe { write_volatile(register, value) };
}
