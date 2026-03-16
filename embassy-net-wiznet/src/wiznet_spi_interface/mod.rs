//! Trait for different types of SPI and blanket implementations

mod qspi;
mod spi;

use embedded_hal::spi::ErrorType;

/// Number of lines used by SPI
#[derive(Debug)]
pub enum SpiType {
    /// Regular, full-duplex SPI with 2 data lines
    Single,
    /// Dual SPI - half-duplex using 2 bi-directional data lines
    Dual,
    /// Quad SPI - half-duplex using 4 bi-directional data lines
    Quad,
}

/// Wiznet SPI operations to build transactions with
#[derive(Debug, PartialEq, Eq)]
pub enum WiznetSpiOperation<'a> {
    /// Read data into the provided buffer.
    Read(&'a mut [u8]),
    /// Write data from the provided buffer, discarding possible read data.
    Write(&'a [u8]),
    /// Write data from the provided buffer using a single line.
    ///
    /// Useful for instruction phases
    WriteSingleLine(&'a [u8]),
}

/// Interface for communicating with Wiznet chip with various types of SPI
pub trait WiznetSpiBus<Word: Copy + 'static = u8>: ErrorType {
    /// Type of SPI implemented by the type
    const SPI_TYPE: SpiType;

    /// Perform a transaction against the device.
    ///
    /// - Locks the bus
    /// - Asserts the CS (Chip Select) pin.
    /// - Performs all the operations.
    /// - [Flushes](SpiBus::flush) the bus.
    /// - Deasserts the CS pin.
    /// - Unlocks the bus.
    async fn transaction<'a, const N: usize>(
        &mut self,
        operations: [WiznetSpiOperation<'a>; N],
    ) -> Result<(), Self::Error>;
}
