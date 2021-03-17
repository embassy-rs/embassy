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
pub trait FullDuplex<Word> {
    /// An enumeration of SPI errors
    type Error;

    type WriteFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;
    type ReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;
    type WriteReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;

    fn read<'a>(&'a mut self, data: &'a mut [Word]) -> Self::ReadFuture<'_>;
    fn write<'a>(&'a mut self, data: &'a [Word]) -> Self::WriteFuture<'_>;
    fn read_write<'a>(
        &mut self,
        read: &'a mut [Word],
        write: &'a [Word],
    ) -> Self::WriteReadFuture<'_>;
}
