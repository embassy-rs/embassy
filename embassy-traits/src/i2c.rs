//! Async I2C API
//!
//! This API supports 7-bit and 10-bit addresses. Traits feature an `AddressMode`
//! marker type parameter. Two implementation of the `AddressMode` exist:
//! `SevenBitAddress` and `TenBitAddress`.
//!
//! Through this marker types it is possible to implement each address mode for
//! the traits independently in `embedded-hal` implementations and device drivers
//! can depend only on the mode that they support.
//!
//! Additionally, the I2C 10-bit address mode has been developed to be fully
//! backwards compatible with the 7-bit address mode. This allows for a
//! software-emulated 10-bit addressing implementation if the address mode
//! is not supported by the hardware.
//!
//! Since 7-bit addressing is the mode of the majority of I2C devices,
//! `SevenBitAddress` has been set as default mode and thus can be omitted if desired.
//!
//! ### Device driver compatible only with 7-bit addresses
//!
//! For demonstration purposes the address mode parameter has been omitted in this example.
//!
//! ```
//! # use embedded_hal::blocking::i2c::WriteRead;
//! const ADDR: u8  = 0x15;
//! # const TEMP_REGISTER: u8 = 0x1;
//! pub struct TemperatureSensorDriver<I2C> {
//!     i2c: I2C,
//! }
//!
//! impl<I2C, E> TemperatureSensorDriver<I2C>
//! where
//!     I2C: WriteRead<Error = E>,
//! {
//!     pub fn read_temperature(&mut self) -> Result<u8, E> {
//!         let mut temp = [0];
//!         self.i2c
//!             .write_read(ADDR, &[TEMP_REGISTER], &mut temp)
//!             .await
//!             .and(Ok(temp[0]))
//!     }
//! }
//! ```
//!
//! ### Device driver compatible only with 10-bit addresses
//!
//! ```
//! # use embedded_hal::blocking::i2c::{TenBitAddress, WriteRead};
//! const ADDR: u16  = 0x158;
//! # const TEMP_REGISTER: u8 = 0x1;
//! pub struct TemperatureSensorDriver<I2C> {
//!     i2c: I2C,
//! }
//!
//! impl<I2C, E> TemperatureSensorDriver<I2C>
//! where
//!     I2C: WriteRead<TenBitAddress, Error = E>,
//! {
//!     pub fn read_temperature(&mut self) -> Result<u8, E> {
//!         let mut temp = [0];
//!         self.i2c
//!             .write_read(ADDR, &[TEMP_REGISTER], &mut temp)
//!             .await
//!             .and(Ok(temp[0]))
//!     }
//! }
//! ```

use core::future::Future;

mod private {
    pub trait Sealed {}
}

/// Address mode (7-bit / 10-bit)
///
/// Note: This trait is sealed and should not be implemented outside of this crate.
pub trait AddressMode: private::Sealed {}

/// 7-bit address mode type
pub type SevenBitAddress = u8;

/// 10-bit address mode type
pub type TenBitAddress = u16;

impl private::Sealed for SevenBitAddress {}
impl private::Sealed for TenBitAddress {}

impl AddressMode for SevenBitAddress {}

impl AddressMode for TenBitAddress {}

pub trait I2c<A: AddressMode = SevenBitAddress> {
    /// Error type
    type Error;

    type WriteFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;
    type ReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;
    type WriteReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Reads enough bytes from slave with `address` to fill `buffer`
    ///
    /// # I2C Events (contract)
    ///
    /// ``` text
    /// Master: ST SAD+R        MAK    MAK ...    NMAK SP
    /// Slave:           SAK B0     B1     ... BN
    /// ```
    ///
    /// Where
    ///
    /// - `ST` = start condition
    /// - `SAD+R` = slave address followed by bit 1 to indicate reading
    /// - `SAK` = slave acknowledge
    /// - `Bi` = ith byte of data
    /// - `MAK` = master acknowledge
    /// - `NMAK` = master no acknowledge
    /// - `SP` = stop condition
    fn read<'a>(&'a mut self, address: A, buffer: &'a mut [u8]) -> Self::ReadFuture<'a>;

    /// Sends bytes to slave with address `address`
    ///
    /// # I2C Events (contract)
    ///
    /// ``` text
    /// Master: ST SAD+W     B0     B1     ... BN     SP
    /// Slave:           SAK    SAK    SAK ...    SAK
    /// ```
    ///
    /// Where
    ///
    /// - `ST` = start condition
    /// - `SAD+W` = slave address followed by bit 0 to indicate writing
    /// - `SAK` = slave acknowledge
    /// - `Bi` = ith byte of data
    /// - `SP` = stop condition
    fn write<'a>(&'a mut self, address: A, bytes: &'a [u8]) -> Self::WriteFuture<'a>;

    /// Sends bytes to slave with address `address` and then reads enough bytes to fill `buffer` *in a
    /// single transaction*
    ///
    /// # I2C Events (contract)
    ///
    /// ``` text
    /// Master: ST SAD+W     O0     O1     ... OM     SR SAD+R        MAK    MAK ...    NMAK SP
    /// Slave:           SAK    SAK    SAK ...    SAK          SAK I0     I1     ... IN
    /// ```
    ///
    /// Where
    ///
    /// - `ST` = start condition
    /// - `SAD+W` = slave address followed by bit 0 to indicate writing
    /// - `SAK` = slave acknowledge
    /// - `Oi` = ith outgoing byte of data
    /// - `SR` = repeated start condition
    /// - `SAD+R` = slave address followed by bit 1 to indicate reading
    /// - `Ii` = ith incoming byte of data
    /// - `MAK` = master acknowledge
    /// - `NMAK` = master no acknowledge
    /// - `SP` = stop condition
    fn write_read<'a>(
        &'a mut self,
        address: A,
        bytes: &'a [u8],
        buffer: &'a mut [u8],
    ) -> Self::WriteReadFuture<'a>;
}

pub trait WriteIter<A: AddressMode = SevenBitAddress> {
    /// Error type
    type Error;

    type WriteIterFuture<'a, V>: Future<Output = Result<(), Self::Error>> + 'a
    where
        V: 'a + IntoIterator<Item = u8>,
        Self: 'a;

    /// Sends bytes to slave with address `address`
    ///
    /// # I2C Events (contract)
    ///
    /// Same as `I2c::write`
    fn write_iter<'a, U>(&'a mut self, address: A, bytes: U) -> Self::WriteIterFuture<'a, U>
    where
        U: IntoIterator<Item = u8> + 'a;
}
