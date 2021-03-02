//! Blocking I2C API
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
//! ## Examples
//!
//! ### `embedded-hal` implementation for an MCU
//! Here is an example of an embedded-hal implementation of the `Write` trait
//! for both modes:
//! ```
//! # use embedded_hal::blocking::i2c::{SevenBitAddress, TenBitAddress, Write};
//! /// I2C0 hardware peripheral which supports both 7-bit and 10-bit addressing.
//! pub struct I2c0;
//!
//! impl Write<SevenBitAddress> for I2c0
//! {
//! #   type Error = ();
//! #
//!     fn try_write(&mut self, addr: u8, output: &[u8]) -> Result<(), Self::Error> {
//!         // ...
//! #       Ok(())
//!     }
//! }
//!
//! impl Write<TenBitAddress> for I2c0
//! {
//! #   type Error = ();
//! #
//!     fn try_write(&mut self, addr: u16, output: &[u8]) -> Result<(), Self::Error> {
//!         // ...
//! #       Ok(())
//!     }
//! }
//! ```
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
//!             .try_write_read(ADDR, &[TEMP_REGISTER], &mut temp)
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
//!             .try_write_read(ADDR, &[TEMP_REGISTER], &mut temp)
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

/// Blocking read
pub trait Read<A: AddressMode = SevenBitAddress> {
    /// Error type
    type Error;

    type ReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;

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
    fn read<'a>(&mut self, address: A, buffer: &mut [u8]) -> Self::ReadFuture<'a>;
}

/// Blocking write
pub trait Write<A: AddressMode = SevenBitAddress> {
    /// Error type
    type Error;

    type WriteFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;

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
    fn write<'a>(&mut self, address: A, bytes: &[u8]) -> Self::WriteFuture<'a>;
}

/// Blocking write (iterator version)
pub trait WriteIter<A: AddressMode = SevenBitAddress> {
    /// Error type
    type Error;

    type WriteIterFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;

    /// Sends bytes to slave with address `address`
    ///
    /// # I2C Events (contract)
    ///
    /// Same as `Write`
    fn write_iter<'a, B>(&mut self, address: A, bytes: B) -> Self::WriteIterFuture<'a>
    where
        B: IntoIterator<Item = u8>;
}

/// Blocking write + read
pub trait WriteRead<A: AddressMode = SevenBitAddress> {
    /// Error type
    type Error;

    type WriteReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;

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
        &mut self,
        address: A,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Self::WriteReadFuture<'a>;
}

/// Blocking write (iterator version) + read
pub trait WriteIterRead<A: AddressMode = SevenBitAddress> {
    /// Error type
    type Error;

    type WriteIterReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;

    /// Sends bytes to slave with address `address` and then reads enough bytes to fill `buffer` *in a
    /// single transaction*
    ///
    /// # I2C Events (contract)
    ///
    /// Same as the `WriteRead` trait
    fn write_iter_read<'a, B>(
        &mut self,
        address: A,
        bytes: B,
        buffer: &mut [u8],
    ) -> Self::WriteIterReadFuture<'a>
    where
        B: IntoIterator<Item = u8>;
}

/// Transactional I2C operation.
///
/// Several operations can be combined as part of a transaction.
#[derive(Debug, PartialEq)]
pub enum Operation<'a> {
    /// Read data into the provided buffer
    Read(&'a mut [u8]),
    /// Write data from the provided buffer
    Write(&'a [u8]),
}

/// Transactional I2C interface.
///
/// This allows combining operations within an I2C transaction.
pub trait Transactional<A: AddressMode = SevenBitAddress> {
    /// Error type
    type Error;

    type TransactionalFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;

    /// Execute the provided operations on the I2C bus.
    ///
    /// Transaction contract:
    /// - Before executing the first operation an ST is sent automatically. This is followed by SAD+R/W as appropriate.
    /// - Data from adjacent operations of the same type are sent after each other without an SP or SR.
    /// - Between adjacent operations of a different type an SR and SAD+R/W is sent.
    /// - After executing the last operation an SP is sent automatically.
    /// - If the last operation is a `Read` the master does not send an acknowledge for the last byte.
    ///
    /// - `ST` = start condition
    /// - `SAD+R/W` = slave address followed by bit 1 to indicate reading or 0 to indicate writing
    /// - `SR` = repeated start condition
    /// - `SP` = stop condition
    fn exec<'a>(
        &mut self,
        address: A,
        operations: &mut [Operation<'a>],
    ) -> Self::TransactionalFuture<'a>;
}

/// Transactional I2C interface (iterator version).
///
/// This allows combining operation within an I2C transaction.
pub trait TransactionalIter<A: AddressMode = SevenBitAddress> {
    /// Error type
    type Error;

    type TransactionalIterFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;

    /// Execute the provided operations on the I2C bus (iterator version).
    ///
    /// Transaction contract:
    /// - Before executing the first operation an ST is sent automatically. This is followed by SAD+R/W as appropriate.
    /// - Data from adjacent operations of the same type are sent after each other without an SP or SR.
    /// - Between adjacent operations of a different type an SR and SAD+R/W is sent.
    /// - After executing the last operation an SP is sent automatically.
    /// - If the last operation is a `Read` the master does not send an acknowledge for the last byte.
    ///
    /// - `ST` = start condition
    /// - `SAD+R/W` = slave address followed by bit 1 to indicate reading or 0 to indicate writing
    /// - `SR` = repeated start condition
    /// - `SP` = stop condition
    fn exec_iter<'a, O>(&mut self, address: A, operations: O) -> Self::TransactionalIterFuture<'a>
    where
        O: IntoIterator<Item = Operation<'a>>;
}

/// Default implementation of `blocking::i2c::Write`, `blocking::i2c::Read` and
/// `blocking::i2c::WriteRead` traits for `blocking::i2c::Transactional` implementers.
///
/// If you implement `blocking::i2c::Transactional` for your I2C peripheral,
/// you can use this default implementation so that you do not need to implement
/// the `blocking::i2c::Write`, `blocking::i2c::Read` and `blocking::i2c::WriteRead`
/// traits as well.
/// ```
/// use embedded_hal::blocking::i2c;
///
/// struct I2c1;
///
/// impl i2c::Transactional<i2c::SevenBitAddress> for I2c1 {
/// #    type Error = ();
///     fn try_exec<'a>(
///         &mut self,
///         address: i2c::SevenBitAddress,
///         operations: &mut [i2c::Operation<'a>],
///     ) -> Result<(), Self::Error> {
///         // ...
///         # Ok(())
///     }
/// }
///
/// // This is all you need to do:
/// impl i2c::transactional::Default<i2c::SevenBitAddress> for I2c1 {};
///
/// // Then you can use `Write` and so on:
/// use i2c::Write;
///
/// let mut i2c1 = I2c1{};
/// i2c1.try_write(0x01, &[0xAB, 0xCD]).unwrap();
/// ```
pub mod transactional {
    use core::future::Future;

    use super::{AddressMode, Operation, Read, Transactional, Write, WriteRead};

    /// Default implementation of `blocking::i2c::Write`, `blocking::i2c::Read` and
    /// `blocking::i2c::WriteRead` traits for `blocking::i2c::Transactional` implementers.
    pub trait Default<A: AddressMode>: Transactional<A> {}

    //    impl<A, E, S> Write<A> for S
    //    where
    //        A: AddressMode + 'static,
    //        S: self::Default<A> + Transactional<A, Error = E> + 'static,
    //        E: 'static,
    //    {
    //        type Error = E;
    //
    //        type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
    //
    //        fn write<'a>(&mut self, address: A, bytes: &[u8]) -> Self::WriteFuture<'a> {
    //            self.exec(address, &mut [Operation::Write(bytes)])
    //        }
    //    }
    /*
        impl<A, E, S> Read<A> for S
        where
            A: AddressMode,
            S: self::Default<A> + Transactional<A, Error = E>,
        {
            type Error = E;

            fn read(&mut self, address: A, buffer: &mut [u8]) -> Result<(), Self::Error> {
                self.exec(address, &mut [Operation::Read(buffer)])
            }
        }

        impl<A, E, S> WriteRead<A> for S
        where
            A: AddressMode,
            S: self::Default<A> + Transactional<A, Error = E>,
        {
            type Error = E;

            fn write_read(
                &mut self,
                address: A,
                bytes: &[u8],
                buffer: &mut [u8],
            ) -> Result<(), Self::Error> {
                self.exec(
                    address,
                    &mut [Operation::Write(bytes), Operation::Read(buffer)],
                )
            }
        }
    */
}
