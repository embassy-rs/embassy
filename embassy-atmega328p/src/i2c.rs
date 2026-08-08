//! Blocking TWI/I2C master driver.

use core::ptr::{read_volatile, write_volatile};

use embedded_hal::i2c::{ErrorKind, ErrorType, I2c as I2cTrait, NoAcknowledgeSource, Operation};

use crate::gpio::{PC4, PC5};

const TWBR: *mut u8 = 0xb8 as *mut u8;
const TWSR: *mut u8 = 0xb9 as *mut u8;
const TWDR: *mut u8 = 0xbb as *mut u8;
const TWCR: *mut u8 = 0xbc as *mut u8;
const DDRC: *mut u8 = 0x27 as *mut u8;
const PORTC: *mut u8 = 0x28 as *mut u8;

const TWINT: u8 = 1 << 7;
const TWEA: u8 = 1 << 6;
const TWSTA: u8 = 1 << 5;
const TWSTO: u8 = 1 << 4;
const TWEN: u8 = 1 << 2;

/// Ownership token for the TWI peripheral.
pub struct TwiPeripheral {
    _private: (),
}

impl TwiPeripheral {
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

/// I2C configuration error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigError {
    InvalidFrequency,
}

/// I2C transfer error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Bus,
    ArbitrationLoss,
    AddressNack,
    DataNack,
    UnexpectedStatus(u8),
}

impl embedded_hal::i2c::Error for Error {
    fn kind(&self) -> ErrorKind {
        match self {
            Error::Bus => ErrorKind::Bus,
            Error::ArbitrationLoss => ErrorKind::ArbitrationLoss,
            Error::AddressNack => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address),
            Error::DataNack => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data),
            Error::UnexpectedStatus(_) => ErrorKind::Other,
        }
    }
}

/// Blocking seven-bit-address TWI/I2C master on PC4/SDA and PC5/SCL.
pub struct I2c {
    peripheral: TwiPeripheral,
    sda: PC4,
    scl: PC5,
}

impl I2c {
    /// Configures TWI at the requested bus frequency.
    pub fn new(peripheral: TwiPeripheral, sda: PC4, scl: PC5, frequency_hz: u32) -> Result<Self, ConfigError> {
        if frequency_hz == 0 || frequency_hz > crate::CPU_HZ / 16 {
            return Err(ConfigError::InvalidFrequency);
        }
        let divider = crate::CPU_HZ.div_ceil(frequency_hz).saturating_sub(16).div_ceil(2);
        if divider > u32::from(u8::MAX) {
            return Err(ConfigError::InvalidFrequency);
        }

        unsafe {
            // Release SDA/SCL. External pull-up resistors are required.
            modify(DDRC, (1 << 4) | (1 << 5), false);
            modify(PORTC, (1 << 4) | (1 << 5), false);
            write_volatile(TWSR, 0); // prescaler = 1
            write_volatile(TWBR, divider as u8);
            write_volatile(TWCR, TWEN);
        }
        Ok(Self { peripheral, sda, scl })
    }

    /// Disables TWI and returns its ownership tokens.
    pub fn free(self) -> (TwiPeripheral, PC4, PC5) {
        unsafe { write_volatile(TWCR, 0) };
        (self.peripheral, self.sda, self.scl)
    }

    fn start(&mut self, address: u8, read: bool) -> Result<(), Error> {
        unsafe {
            write_volatile(TWCR, TWINT | TWSTA | TWEN);
            wait_for_interrupt();
        }
        match status() {
            0x08 | 0x10 => {}
            value => return Err(status_error(value, true)),
        }

        unsafe {
            write_volatile(TWDR, (address << 1) | u8::from(read));
            write_volatile(TWCR, TWINT | TWEN);
            wait_for_interrupt();
        }
        match status() {
            0x18 | 0x40 => Ok(()),
            0x20 | 0x48 => Err(Error::AddressNack),
            value => Err(status_error(value, true)),
        }
    }

    fn write_data(&mut self, bytes: &[u8]) -> Result<(), Error> {
        for &byte in bytes {
            unsafe {
                write_volatile(TWDR, byte);
                write_volatile(TWCR, TWINT | TWEN);
                wait_for_interrupt();
            }
            match status() {
                0x28 => {}
                0x30 => return Err(Error::DataNack),
                value => return Err(status_error(value, false)),
            }
        }
        Ok(())
    }

    fn read_data(&mut self, bytes: &mut [u8], remaining: &mut usize) -> Result<(), Error> {
        for byte in bytes {
            *remaining -= 1;
            let acknowledge = *remaining != 0;
            unsafe {
                write_volatile(TWCR, TWINT | TWEN | if acknowledge { TWEA } else { 0 });
                wait_for_interrupt();
            }
            match (status(), acknowledge) {
                (0x50, true) | (0x58, false) => unsafe { *byte = read_volatile(TWDR) },
                (value, _) => return Err(status_error(value, false)),
            }
        }
        Ok(())
    }

    fn stop(&mut self) {
        unsafe {
            write_volatile(TWCR, TWINT | TWEN | TWSTO);
            while read_volatile(TWCR) & TWSTO != 0 {}
        }
    }
}

impl ErrorType for I2c {
    type Error = Error;
}

impl I2cTrait<u8> for I2c {
    fn transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Self::Error> {
        if address > 0x7f {
            return Err(Error::AddressNack);
        }
        if operations.is_empty() {
            return Ok(());
        }

        let result = self.transaction_inner(address, operations);
        if !matches!(result, Err(Error::ArbitrationLoss)) {
            self.stop();
        }
        result
    }
}

impl I2c {
    fn transaction_inner(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Error> {
        let mut index = 0;
        while index < operations.len() {
            let read_group = matches!(operations[index], Operation::Read(_));
            let mut end = index + 1;
            while end < operations.len() && matches!(operations[end], Operation::Read(_)) == read_group {
                end += 1;
            }

            self.start(address, read_group)?;
            if read_group {
                let mut remaining = operations[index..end]
                    .iter()
                    .map(|operation| match operation {
                        Operation::Read(bytes) => bytes.len(),
                        Operation::Write(_) => 0,
                    })
                    .sum();
                for operation in operations.iter_mut().take(end).skip(index) {
                    if let Operation::Read(bytes) = operation {
                        self.read_data(bytes, &mut remaining)?;
                    }
                }
            } else {
                for operation in &operations[index..end] {
                    if let Operation::Write(bytes) = operation {
                        self.write_data(bytes)?;
                    }
                }
            }
            index = end;
        }
        Ok(())
    }
}

fn status() -> u8 {
    unsafe { read_volatile(TWSR) & 0xf8 }
}

fn status_error(status: u8, address_phase: bool) -> Error {
    match status {
        0x00 => Error::Bus,
        0x38 => Error::ArbitrationLoss,
        0x20 | 0x48 if address_phase => Error::AddressNack,
        0x30 => Error::DataNack,
        value => Error::UnexpectedStatus(value),
    }
}

unsafe fn wait_for_interrupt() {
    while unsafe { read_volatile(TWCR) } & TWINT == 0 {}
}

unsafe fn modify(register: *mut u8, mask: u8, set: bool) {
    let current = unsafe { read_volatile(register) };
    let value = if set { current | mask } else { current & !mask };
    unsafe { write_volatile(register, value) };
}
