//! Adapters between embedded-hal traits.

use embedded_hal_02::{blocking, serial};

/// Wrapper that implements async traits using blocking implementations.
///
/// This allows driver writers to depend on the async traits while still supporting embedded-hal peripheral implementations.
///
/// BlockingAsync will implement any async trait that maps to embedded-hal traits implemented for the wrapped driver.
///
/// Driver users are then free to choose which implementation that is available to them.
pub struct BlockingAsync<T> {
    wrapped: T,
}

impl<T> BlockingAsync<T> {
    /// Create a new instance of a wrapper for a given peripheral.
    pub fn new(wrapped: T) -> Self {
        Self { wrapped }
    }
}

//
// I2C implementations
//
impl<T, E> embedded_hal_1::i2c::ErrorType for BlockingAsync<T>
where
    E: embedded_hal_1::i2c::Error + 'static,
    T: blocking::i2c::WriteRead<Error = E> + blocking::i2c::Read<Error = E> + blocking::i2c::Write<Error = E>,
{
    type Error = E;
}

impl<T, E> embedded_hal_async::i2c::I2c for BlockingAsync<T>
where
    E: embedded_hal_1::i2c::Error + 'static,
    T: blocking::i2c::WriteRead<Error = E> + blocking::i2c::Read<Error = E> + blocking::i2c::Write<Error = E>,
{
    async fn read<'a>(&'a mut self, address: u8, buffer: &'a mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.read(address, buffer)
    }

    async fn write<'a>(&'a mut self, address: u8, bytes: &'a [u8]) -> Result<(), Self::Error> {
        self.wrapped.write(address, bytes)
    }

    async fn write_read<'a>(
        &'a mut self,
        address: u8,
        bytes: &'a [u8],
        buffer: &'a mut [u8],
    ) -> Result<(), Self::Error> {
        self.wrapped.write_read(address, bytes, buffer)
    }

    async fn transaction<'a, 'b>(
        &'a mut self,
        address: u8,
        operations: &'a mut [embedded_hal_async::i2c::Operation<'b>],
    ) -> Result<(), Self::Error> {
        let _ = address;
        let _ = operations;
        todo!()
    }
}

//
// SPI implementatinos
//

impl<T, E> embedded_hal_async::spi::ErrorType for BlockingAsync<T>
where
    E: embedded_hal_1::spi::Error,
    T: blocking::spi::Transfer<u8, Error = E> + blocking::spi::Write<u8, Error = E>,
{
    type Error = E;
}

impl<T, E> embedded_hal_async::spi::SpiBus<u8> for BlockingAsync<T>
where
    E: embedded_hal_1::spi::Error + 'static,
    T: blocking::spi::Transfer<u8, Error = E> + blocking::spi::Write<u8, Error = E>,
{
    async fn transfer<'a>(&'a mut self, read: &'a mut [u8], write: &'a [u8]) -> Result<(), Self::Error> {
        // Ensure we write the expected bytes
        for i in 0..core::cmp::min(read.len(), write.len()) {
            read[i] = write[i].clone();
        }
        self.wrapped.transfer(read)?;
        Ok(())
    }

    async fn transfer_in_place<'a>(&'a mut self, _: &'a mut [u8]) -> Result<(), Self::Error> {
        todo!()
    }
}

impl<T, E> embedded_hal_async::spi::SpiBusFlush for BlockingAsync<T>
where
    E: embedded_hal_1::spi::Error + 'static,
    T: blocking::spi::Transfer<u8, Error = E> + blocking::spi::Write<u8, Error = E>,
{
    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T, E> embedded_hal_async::spi::SpiBusWrite<u8> for BlockingAsync<T>
where
    E: embedded_hal_1::spi::Error + 'static,
    T: blocking::spi::Transfer<u8, Error = E> + blocking::spi::Write<u8, Error = E>,
{
    async fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.wrapped.write(data)?;
        Ok(())
    }
}

impl<T, E> embedded_hal_async::spi::SpiBusRead<u8> for BlockingAsync<T>
where
    E: embedded_hal_1::spi::Error + 'static,
    T: blocking::spi::Transfer<u8, Error = E> + blocking::spi::Write<u8, Error = E>,
{
    async fn read(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.transfer(data)?;
        Ok(())
    }
}

// Uart implementatinos
impl<T, E> embedded_hal_1::serial::ErrorType for BlockingAsync<T>
where
    T: serial::Read<u8, Error = E>,
    E: embedded_hal_1::serial::Error + 'static,
{
    type Error = E;
}

#[cfg(feature = "_todo_embedded_hal_serial")]
impl<T, E> embedded_hal_async::serial::Read for BlockingAsync<T>
where
    T: serial::Read<u8, Error = E>,
    E: embedded_hal_1::serial::Error + 'static,
{
    type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where T: 'a;
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            let mut pos = 0;
            while pos < buf.len() {
                match self.wrapped.read() {
                    Err(nb::Error::WouldBlock) => {}
                    Err(nb::Error::Other(e)) => return Err(e),
                    Ok(b) => {
                        buf[pos] = b;
                        pos += 1;
                    }
                }
            }
            Ok(())
        }
    }
}

#[cfg(feature = "_todo_embedded_hal_serial")]
impl<T, E> embedded_hal_async::serial::Write for BlockingAsync<T>
where
    T: blocking::serial::Write<u8, Error = E> + serial::Read<u8, Error = E>,
    E: embedded_hal_1::serial::Error + 'static,
{
    type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where T: 'a;
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        async move { self.wrapped.bwrite_all(buf) }
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where T: 'a;
    fn flush(&mut self) -> Result<(), Self::Error> {
        async move { self.wrapped.bflush() }
    }
}

/// NOR flash wrapper
use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};
use embedded_storage_async::nor_flash::{NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash};

impl<T> ErrorType for BlockingAsync<T>
where
    T: ErrorType,
{
    type Error = T::Error;
}

impl<T> AsyncNorFlash for BlockingAsync<T>
where
    T: NorFlash,
{
    const WRITE_SIZE: usize = <T as NorFlash>::WRITE_SIZE;
    const ERASE_SIZE: usize = <T as NorFlash>::ERASE_SIZE;

    async fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
        self.wrapped.write(offset, data)
    }

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.wrapped.erase(from, to)
    }
}

impl<T> AsyncReadNorFlash for BlockingAsync<T>
where
    T: ReadNorFlash,
{
    const READ_SIZE: usize = <T as ReadNorFlash>::READ_SIZE;
    async fn read(&mut self, address: u32, data: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.read(address, data)
    }

    fn capacity(&self) -> usize {
        self.wrapped.capacity()
    }
}
