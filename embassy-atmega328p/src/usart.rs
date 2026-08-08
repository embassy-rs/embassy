//! Blocking USART0 driver.

use core::fmt;
use core::ptr::{read_volatile, write_volatile};

use crate::gpio::{PD0, PD1};

const UCSR0A: *mut u8 = 0xc0 as *mut u8;
const UCSR0B: *mut u8 = 0xc1 as *mut u8;
const UCSR0C: *mut u8 = 0xc2 as *mut u8;
const UBRR0L: *mut u8 = 0xc4 as *mut u8;
const UBRR0H: *mut u8 = 0xc5 as *mut u8;
const UDR0: *mut u8 = 0xc6 as *mut u8;

const RXC0: u8 = 1 << 7;
const UDRE0: u8 = 1 << 5;
const FE0: u8 = 1 << 4;
const DOR0: u8 = 1 << 3;
const UPE0: u8 = 1 << 2;
const U2X0: u8 = 1 << 1;
const RXEN0: u8 = 1 << 4;
const TXEN0: u8 = 1 << 3;
const UCSZ01: u8 = 1 << 2;
const UCSZ00: u8 = 1 << 1;

/// Ownership token for USART0.
pub struct Usart0Peripheral {
    _private: (),
}

impl Usart0Peripheral {
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

/// USART configuration error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigError {
    /// The requested baud rate cannot be represented by the hardware divider.
    InvalidBaudRate,
}

/// USART receive error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Framing,
    DataOverrun,
    Parity,
}

/// Blocking 8-data-bit, no-parity, one-stop-bit USART0.
pub struct Usart {
    peripheral: Usart0Peripheral,
    rx: PD0,
    tx: PD1,
}

impl Usart {
    /// Configures USART0 in double-speed asynchronous mode.
    pub fn new(peripheral: Usart0Peripheral, rx: PD0, tx: PD1, baud: u32) -> Result<Self, ConfigError> {
        if baud == 0 {
            return Err(ConfigError::InvalidBaudRate);
        }

        let divisor = (u64::from(crate::CPU_HZ) + u64::from(baud) * 4) / (u64::from(baud) * 8);
        if divisor == 0 || divisor > 4096 {
            return Err(ConfigError::InvalidBaudRate);
        }
        let ubrr = (divisor - 1) as u16;
        let [low, high] = ubrr.to_le_bytes();

        unsafe {
            write_volatile(UCSR0A, U2X0);
            write_volatile(UBRR0H, high & 0x0f);
            write_volatile(UBRR0L, low);
            write_volatile(UCSR0C, UCSZ01 | UCSZ00);
            write_volatile(UCSR0B, RXEN0 | TXEN0);
        }

        Ok(Self { peripheral, rx, tx })
    }

    /// Writes one byte, blocking until the transmit register is available.
    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            while read_volatile(UCSR0A) & UDRE0 == 0 {}
            write_volatile(UDR0, byte);
        }
    }

    /// Reads one byte, blocking until data arrives.
    pub fn read_byte(&mut self) -> Result<u8, Error> {
        unsafe {
            while read_volatile(UCSR0A) & RXC0 == 0 {}
            self.read_ready()
        }
    }

    /// Reads a byte without blocking, returning `None` if no byte is ready.
    pub fn try_read(&mut self) -> Option<Result<u8, Error>> {
        if unsafe { read_volatile(UCSR0A) } & RXC0 == 0 {
            None
        } else {
            Some(unsafe { self.read_ready() })
        }
    }

    /// Disables USART0 and returns the peripheral and pin tokens.
    pub fn free(self) -> (Usart0Peripheral, PD0, PD1) {
        unsafe { write_volatile(UCSR0B, 0) };
        (self.peripheral, self.rx, self.tx)
    }

    unsafe fn read_ready(&mut self) -> Result<u8, Error> {
        let status = unsafe { read_volatile(UCSR0A) };
        let data = unsafe { read_volatile(UDR0) };
        if status & FE0 != 0 {
            Err(Error::Framing)
        } else if status & DOR0 != 0 {
            Err(Error::DataOverrun)
        } else if status & UPE0 != 0 {
            Err(Error::Parity)
        } else {
            Ok(data)
        }
    }
}

impl fmt::Write for Usart {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        for byte in value.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}
