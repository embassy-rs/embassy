use embassy_time::{Duration, Instant};

use super::{Error, I2c, Instance};

/// An I2C wrapper, which provides `embassy-time` based timeouts for all `embedded-hal` trait methods.
///
/// This is useful for recovering from a shorted bus or a device stuck in a clock stretching state.
/// A regular [I2c] would freeze until condition is removed.
pub struct TimeoutI2c<'d, T: Instance, TXDMA, RXDMA> {
    i2c: &'d mut I2c<'d, T, TXDMA, RXDMA>,
    timeout: Duration,
}

fn timeout_fn(timeout: Duration) -> impl Fn() -> Result<(), Error> {
    let deadline = Instant::now() + timeout;
    move || {
        if Instant::now() > deadline {
            Err(Error::Timeout)
        } else {
            Ok(())
        }
    }
}

impl<'d, T: Instance, TXDMA, RXDMA> TimeoutI2c<'d, T, TXDMA, RXDMA> {
    pub fn new(i2c: &'d mut I2c<'d, T, TXDMA, RXDMA>, timeout: Duration) -> Self {
        Self { i2c, timeout }
    }

    /// Blocking read with a custom timeout
    pub fn blocking_read_timeout(&mut self, addr: u8, read: &mut [u8], timeout: Duration) -> Result<(), Error> {
        self.i2c.blocking_read_timeout(addr, read, timeout_fn(timeout))
    }

    /// Blocking read with default timeout, provided in [`TimeoutI2c::new()`]
    pub fn blocking_read(&mut self, addr: u8, read: &mut [u8]) -> Result<(), Error> {
        self.blocking_read_timeout(addr, read, self.timeout)
    }

    /// Blocking write with a custom timeout
    pub fn blocking_write_timeout(&mut self, addr: u8, write: &[u8], timeout: Duration) -> Result<(), Error> {
        self.i2c.blocking_write_timeout(addr, write, timeout_fn(timeout))
    }

    /// Blocking write with default timeout, provided in [`TimeoutI2c::new()`]
    pub fn blocking_write(&mut self, addr: u8, write: &[u8]) -> Result<(), Error> {
        self.blocking_write_timeout(addr, write, self.timeout)
    }

    /// Blocking write-read with a custom timeout
    pub fn blocking_write_read_timeout(
        &mut self,
        addr: u8,
        write: &[u8],
        read: &mut [u8],
        timeout: Duration,
    ) -> Result<(), Error> {
        self.i2c
            .blocking_write_read_timeout(addr, write, read, timeout_fn(timeout))
    }

    /// Blocking write-read with default timeout, provided in [`TimeoutI2c::new()`]
    pub fn blocking_write_read(&mut self, addr: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        self.blocking_write_read_timeout(addr, write, read, self.timeout)
    }
}

impl<'d, T: Instance, TXDMA, RXDMA> embedded_hal_02::blocking::i2c::Read for TimeoutI2c<'d, T, TXDMA, RXDMA> {
    type Error = Error;

    fn read(&mut self, addr: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(addr, read)
    }
}

impl<'d, T: Instance, TXDMA, RXDMA> embedded_hal_02::blocking::i2c::Write for TimeoutI2c<'d, T, TXDMA, RXDMA> {
    type Error = Error;

    fn write(&mut self, addr: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(addr, write)
    }
}

impl<'d, T: Instance, TXDMA, RXDMA> embedded_hal_02::blocking::i2c::WriteRead for TimeoutI2c<'d, T, TXDMA, RXDMA> {
    type Error = Error;

    fn write_read(&mut self, addr: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(addr, write, read)
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl<'d, T: Instance, TXDMA, RXDMA> embedded_hal_1::i2c::ErrorType for TimeoutI2c<'d, T, TXDMA, RXDMA> {
        type Error = Error;
    }

    impl<'d, T: Instance, TXDMA, RXDMA> embedded_hal_1::i2c::I2c for TimeoutI2c<'d, T, TXDMA, RXDMA> {
        fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_read(address, read)
        }

        fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(address, write)
        }

        fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_write_read(address, write, read)
        }

        fn transaction(
            &mut self,
            _address: u8,
            _operations: &mut [embedded_hal_1::i2c::Operation<'_>],
        ) -> Result<(), Self::Error> {
            todo!();
        }
    }
}
