use core::future::Future;
use embedded_hal_02::blocking;
use embedded_hal_02::serial;

/// BlockingAsync is a wrapper that implements async traits using blocking peripherals. This allows
/// driver writers to depend on the async traits while still supporting embedded-hal peripheral implementations.
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
    T: blocking::i2c::WriteRead<Error = E>
        + blocking::i2c::Read<Error = E>
        + blocking::i2c::Write<Error = E>,
{
    type Error = E;
}

impl<T, E> embedded_hal_async::i2c::I2c for BlockingAsync<T>
where
    E: embedded_hal_1::i2c::Error + 'static,
    T: blocking::i2c::WriteRead<Error = E>
        + blocking::i2c::Read<Error = E>
        + blocking::i2c::Write<Error = E>,
{
    type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
    type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
    type WriteReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

    fn read<'a>(&'a mut self, address: u8, buffer: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move { self.wrapped.read(address, buffer) }
    }

    fn write<'a>(&'a mut self, address: u8, bytes: &'a [u8]) -> Self::WriteFuture<'a> {
        async move { self.wrapped.write(address, bytes) }
    }

    fn write_read<'a>(
        &'a mut self,
        address: u8,
        bytes: &'a [u8],
        buffer: &'a mut [u8],
    ) -> Self::WriteReadFuture<'a> {
        async move { self.wrapped.write_read(address, bytes, buffer) }
    }

    type TransactionFuture<'a, 'b> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a, 'b: 'a;

    fn transaction<'a, 'b>(
        &'a mut self,
        address: u8,
        operations: &'a mut [embedded_hal_async::i2c::Operation<'b>],
    ) -> Self::TransactionFuture<'a, 'b> {
        let _ = address;
        let _ = operations;
        async move { todo!() }
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
    type TransferFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

    fn transfer<'a>(&'a mut self, read: &'a mut [u8], write: &'a [u8]) -> Self::TransferFuture<'a> {
        async move {
            // Ensure we write the expected bytes
            for i in 0..core::cmp::min(read.len(), write.len()) {
                read[i] = write[i].clone();
            }
            self.wrapped.transfer(read)?;
            Ok(())
        }
    }

    type TransferInPlaceFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

    fn transfer_in_place<'a>(&'a mut self, _: &'a mut [u8]) -> Self::TransferInPlaceFuture<'a> {
        async move { todo!() }
    }
}

impl<T, E> embedded_hal_async::spi::SpiBusFlush for BlockingAsync<T>
where
    E: embedded_hal_1::spi::Error + 'static,
    T: blocking::spi::Transfer<u8, Error = E> + blocking::spi::Write<u8, Error = E>,
{
    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        async move { Ok(()) }
    }
}

impl<T, E> embedded_hal_async::spi::SpiBusWrite<u8> for BlockingAsync<T>
where
    E: embedded_hal_1::spi::Error + 'static,
    T: blocking::spi::Transfer<u8, Error = E> + blocking::spi::Write<u8, Error = E>,
{
    type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

    fn write<'a>(&'a mut self, data: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            self.wrapped.write(data)?;
            Ok(())
        }
    }
}

impl<T, E> embedded_hal_async::spi::SpiBusRead<u8> for BlockingAsync<T>
where
    E: embedded_hal_1::spi::Error + 'static,
    T: blocking::spi::Transfer<u8, Error = E> + blocking::spi::Write<u8, Error = E>,
{
    type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

    fn read<'a>(&'a mut self, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            self.wrapped.transfer(data)?;
            Ok(())
        }
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
    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        async move { self.wrapped.bflush() }
    }
}

/// NOR flash wrapper
use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};
use embedded_storage_async::nor_flash::{AsyncNorFlash, AsyncReadNorFlash};

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

    type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
    fn write<'a>(&'a mut self, offset: u32, data: &'a [u8]) -> Self::WriteFuture<'a> {
        async move { self.wrapped.write(offset, data) }
    }

    type EraseFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
    fn erase<'a>(&'a mut self, from: u32, to: u32) -> Self::EraseFuture<'a> {
        async move { self.wrapped.erase(from, to) }
    }
}

impl<T> AsyncReadNorFlash for BlockingAsync<T>
where
    T: ReadNorFlash,
{
    const READ_SIZE: usize = <T as ReadNorFlash>::READ_SIZE;
    type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
    fn read<'a>(&'a mut self, address: u32, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move { self.wrapped.read(address, data) }
    }

    fn capacity(&self) -> usize {
        self.wrapped.capacity()
    }
}
