use core::future::Future;
use embedded_hal::blocking;
use embedded_hal::serial;

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
// I2C implementatinos
//

impl<T, E> crate::i2c::I2c for BlockingAsync<T>
where
    E: 'static,
    T: blocking::i2c::WriteRead<Error = E>
        + blocking::i2c::Read<Error = E>
        + blocking::i2c::Write<Error = E>,
{
    type Error = E;

    #[rustfmt::skip]
    type WriteFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;
    #[rustfmt::skip]
    type ReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;
    #[rustfmt::skip]
    type WriteReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

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
}

//
// SPI implementatinos
//

impl<T, E, Word> crate::spi::Spi<Word> for BlockingAsync<T>
where
    T: blocking::spi::Write<Word, Error = E>,
{
    type Error = E;
}

impl<T, E, Word> crate::spi::FullDuplex<Word> for BlockingAsync<T>
where
    E: 'static,
    Word: Clone,
    T: blocking::spi::Transfer<Word, Error = E> + blocking::spi::Write<Word, Error = E>,
{
    #[rustfmt::skip]
    type WriteReadFuture<'a> where Word: 'a, Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn read_write<'a>(
        &'a mut self,
        read: &'a mut [Word],
        write: &'a [Word],
    ) -> Self::WriteReadFuture<'a> {
        async move {
            // Ensure we write the expected bytes
            for i in 0..core::cmp::min(read.len(), write.len()) {
                read[i] = write[i].clone();
            }
            self.wrapped.transfer(read)?;
            Ok(())
        }
    }
}

impl<T, E, Word> crate::spi::Write<Word> for BlockingAsync<T>
where
    E: 'static,
    Word: Clone,
    T: blocking::spi::Write<Word, Error = E>,
{
    #[rustfmt::skip]
    type WriteFuture<'a> where Word: 'a, Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn write<'a>(&'a mut self, data: &'a [Word]) -> Self::WriteFuture<'a> {
        async move { self.wrapped.write(data) }
    }
}

impl<T, E, Word> crate::spi::Read<Word> for BlockingAsync<T>
where
    E: 'static,
    Word: Clone,
    T: blocking::spi::Transfer<Word, Error = E> + blocking::spi::Write<Word, Error = E>,
{
    #[rustfmt::skip]
    type ReadFuture<'a> where Word: 'a, Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn read<'a>(&'a mut self, data: &'a mut [Word]) -> Self::ReadFuture<'a> {
        async move {
            self.wrapped.transfer(data)?;
            Ok(())
        }
    }
}

// Uart implementatinos
impl<T> crate::uart::Read for BlockingAsync<T>
where
    T: serial::Read<u8>,
{
    #[rustfmt::skip]
    type ReadFuture<'a> where T: 'a = impl Future<Output = Result<(), crate::uart::Error>> + 'a;
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            let mut pos = 0;
            while pos < buf.len() {
                match self.wrapped.read() {
                    Err(nb::Error::WouldBlock) => {}
                    Err(_) => return Err(crate::uart::Error::Other),
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

impl<T> crate::uart::Write for BlockingAsync<T>
where
    T: blocking::serial::Write<u8>,
{
    #[rustfmt::skip]
    type WriteFuture<'a> where T: 'a = impl Future<Output = Result<(), crate::uart::Error>> + 'a;
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            self.wrapped
                .bwrite_all(buf)
                .map_err(|_| crate::uart::Error::Other)?;
            self.wrapped.bflush().map_err(|_| crate::uart::Error::Other)
        }
    }
}
