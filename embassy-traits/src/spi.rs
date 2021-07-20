//! Async SPI API

use core::future::Future;

/// Full duplex (master mode)
///
/// # Notes
///
/// - It's the task of the user of this interface to manage the slave select lines
///
/// - Due to how full duplex SPI works each `read` call must be preceded by a `write` call.
///
/// - `read` calls only return the data received with the last `write` call.
/// Previously received data is discarded
///
/// - Data is only guaranteed to be clocked out when the `read` call succeeds.
/// The slave select line shouldn't be released before that.
///
/// - Some SPIs can work with 8-bit *and* 16-bit words. You can overload this trait with different
/// `Word` types to allow operation in both modes.

pub trait Spi<Word> {
    /// An enumeration of SPI errors
    type Error;
}

pub trait FullDuplex<Word>: Spi<Word> + Write<Word> + Read<Word> {
    type WriteReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    fn read_write<'a>(
        &'a mut self,
        read: &'a mut [Word],
        write: &'a [Word],
    ) -> Self::WriteReadFuture<'a>;
}

pub trait Write<Word>: Spi<Word> {
    type WriteFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    fn write<'a>(&'a mut self, data: &'a [Word]) -> Self::WriteFuture<'a>;
}

pub trait Read<Word>: Write<Word> {
    type ReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    fn read<'a>(&'a mut self, data: &'a mut [Word]) -> Self::ReadFuture<'a>;
}
