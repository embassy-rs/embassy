use embassy_time::{Duration, Instant};

use super::{Error, I2c, Instance};

/// An I2C wrapper, which provides `embassy-time` based timeouts for all `embedded-hal` trait methods.
pub struct TimeoutI2c<'d, T: Instance> {
    i2c: &'d mut I2c<'d, T>,
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

impl<'d, T: Instance> TimeoutI2c<'d, T> {
    pub fn new(i2c: &'d mut I2c<'d, T>, timeout: Duration) -> Self {
        Self { i2c, timeout }
    }

    /// Blocking read with a custom timeout
    pub fn blocking_read_timeout(&mut self, addr: u8, buffer: &mut [u8], timeout: Duration) -> Result<(), Error> {
        self.i2c.blocking_read_timeout(addr, buffer, timeout_fn(timeout))
    }

    /// Blocking read with default timeout, provided in [`TimeoutI2c::new()`]
    pub fn blocking_read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.blocking_read_timeout(addr, buffer, self.timeout)
    }

    /// Blocking write with a custom timeout
    pub fn blocking_write_timeout(&mut self, addr: u8, bytes: &[u8], timeout: Duration) -> Result<(), Error> {
        self.i2c.blocking_write_timeout(addr, bytes, timeout_fn(timeout))
    }

    /// Blocking write with default timeout, provided in [`TimeoutI2c::new()`]
    pub fn blocking_write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        self.blocking_write_timeout(addr, bytes, self.timeout)
    }

    /// Blocking write-read with a custom timeout
    pub fn blocking_write_read_timeout(
        &mut self,
        addr: u8,
        bytes: &[u8],
        buffer: &mut [u8],
        timeout: Duration,
    ) -> Result<(), Error> {
        self.i2c
            .blocking_write_read_timeout(addr, bytes, buffer, timeout_fn(timeout))
    }

    /// Blocking write-read with default timeout, provided in [`TimeoutI2c::new()`]
    pub fn blocking_write_read(&mut self, addr: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        self.blocking_write_read_timeout(addr, bytes, buffer, self.timeout)
    }
}

impl<'d, T: Instance> embedded_hal_02::blocking::i2c::Read for TimeoutI2c<'d, T> {
    type Error = Error;

    fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(addr, buffer)
    }
}

impl<'d, T: Instance> embedded_hal_02::blocking::i2c::Write for TimeoutI2c<'d, T> {
    type Error = Error;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(addr, bytes)
    }
}

impl<'d, T: Instance> embedded_hal_02::blocking::i2c::WriteRead for TimeoutI2c<'d, T> {
    type Error = Error;

    fn write_read(&mut self, addr: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(addr, bytes, buffer)
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl<'d, T: Instance> embedded_hal_1::i2c::ErrorType for TimeoutI2c<'d, T> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_1::i2c::I2c for TimeoutI2c<'d, T> {
        fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_read(address, buffer)
        }

        fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(address, buffer)
        }

        fn write_iter<B>(&mut self, _address: u8, _bytes: B) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>,
        {
            todo!();
        }

        fn write_iter_read<B>(&mut self, _address: u8, _bytes: B, _buffer: &mut [u8]) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>,
        {
            todo!();
        }

        fn write_read(&mut self, address: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_write_read(address, wr_buffer, rd_buffer)
        }

        fn transaction<'a>(
            &mut self,
            _address: u8,
            _operations: &mut [embedded_hal_1::i2c::Operation<'a>],
        ) -> Result<(), Self::Error> {
            todo!();
        }

        fn transaction_iter<'a, O>(&mut self, _address: u8, _operations: O) -> Result<(), Self::Error>
        where
            O: IntoIterator<Item = embedded_hal_1::i2c::Operation<'a>>,
        {
            todo!();
        }
    }
}
