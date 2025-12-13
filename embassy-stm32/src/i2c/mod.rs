//! Inter-Integrated-Circuit (I2C)
#![macro_use]

#[cfg_attr(i2c_v1, path = "v1.rs")]
#[cfg_attr(any(i2c_v2, i2c_v3), path = "v2.rs")]
mod _version;

mod config;

use core::future::Future;
use core::iter;
use core::marker::PhantomData;

pub use config::*;
use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;
#[cfg(feature = "time")]
use embassy_time::{Duration, Instant};
use mode::MasterMode;
pub use mode::{Master, MultiMaster};

use crate::dma::ChannelAndRequest;
use crate::gpio::{AnyPin, SealedPin as _};
use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::rcc::{RccInfo, SealedRccPeripheral};
use crate::time::Hertz;
use crate::{interrupt, peripherals};

/// I2C error.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Bus error
    Bus,
    /// Arbitration lost
    Arbitration,
    /// ACK not received (either to the address or to a data byte)
    Nack,
    /// Timeout
    Timeout,
    /// CRC error
    Crc,
    /// Overrun error
    Overrun,
    /// Zero-length transfers are not allowed.
    ZeroLengthTransfer,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::Bus => "Bus Error",
            Self::Arbitration => "Arbitration Lost",
            Self::Nack => "ACK Not Received",
            Self::Timeout => "Request Timed Out",
            Self::Crc => "CRC Mismatch",
            Self::Overrun => "Buffer Overrun",
            Self::ZeroLengthTransfer => "Zero-Length Transfers are not allowed",
        };

        write!(f, "{}", message)
    }
}

impl core::error::Error for Error {}

/// I2C modes
pub mod mode {
    trait SealedMode {}

    /// Trait for I2C master operations.
    #[allow(private_bounds)]
    pub trait MasterMode: SealedMode {}

    /// Mode allowing for I2C master operations.
    pub struct Master;
    /// Mode allowing for I2C master and slave operations.
    pub struct MultiMaster;

    impl SealedMode for Master {}
    impl MasterMode for Master {}

    impl SealedMode for MultiMaster {}
    impl MasterMode for MultiMaster {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// The command kind to the slave from the master
pub enum SlaveCommandKind {
    /// Write to the slave
    Write,
    /// Read from the slave
    Read,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// The command kind to the slave from the master and the address that the slave matched
pub struct SlaveCommand {
    /// The kind of command
    pub kind: SlaveCommandKind,
    /// The address that the slave matched
    pub address: Address,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// The status of the slave send operation
pub enum SendStatus {
    /// The slave send operation is done, all bytes have been sent and the master is not requesting more
    Done,
    /// The slave send operation is done, but there are leftover bytes that the master did not read
    LeftoverBytes(usize),
}

struct I2CDropGuard<'d> {
    info: &'static Info,
    scl: Option<Peri<'d, AnyPin>>,
    sda: Option<Peri<'d, AnyPin>>,
}
impl<'d> Drop for I2CDropGuard<'d> {
    fn drop(&mut self) {
        if let Some(x) = self.scl.as_ref() {
            x.set_as_disconnected()
        }
        if let Some(x) = self.sda.as_ref() {
            x.set_as_disconnected()
        }

        self.info.rcc.disable();
    }
}

/// I2C driver.
pub struct I2c<'d, M: Mode, IM: MasterMode> {
    info: &'static Info,
    state: &'static State,
    kernel_clock: Hertz,
    tx_dma: Option<ChannelAndRequest<'d>>,
    rx_dma: Option<ChannelAndRequest<'d>>,
    #[cfg(feature = "time")]
    timeout: Duration,
    _phantom: PhantomData<M>,
    _phantom2: PhantomData<IM>,
    _drop_guard: I2CDropGuard<'d>,
}

impl<'d> I2c<'d, Async, Master> {
    /// Create a new I2C driver.
    pub fn new<T: Instance, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        scl: Peri<'d, if_afio!(impl SclPin<T, A>)>,
        sda: Peri<'d, if_afio!(impl SdaPin<T, A>)>,
        _irq: impl interrupt::typelevel::Binding<T::EventInterrupt, EventInterruptHandler<T>>
        + interrupt::typelevel::Binding<T::ErrorInterrupt, ErrorInterruptHandler<T>>
        + 'd,
        tx_dma: Peri<'d, impl TxDma<T>>,
        rx_dma: Peri<'d, impl RxDma<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(scl, config.scl_af()),
            new_pin!(sda, config.sda_af()),
            new_dma!(tx_dma),
            new_dma!(rx_dma),
            config,
        )
    }
}

impl<'d> I2c<'d, Blocking, Master> {
    /// Create a new blocking I2C driver.
    pub fn new_blocking<T: Instance, #[cfg(afio)] A>(
        peri: Peri<'d, T>,
        scl: Peri<'d, if_afio!(impl SclPin<T, A>)>,
        sda: Peri<'d, if_afio!(impl SdaPin<T, A>)>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(scl, config.scl_af()),
            new_pin!(sda, config.sda_af()),
            None,
            None,
            config,
        )
    }
}

impl<'d, M: Mode> I2c<'d, M, Master> {
    /// Create a new I2C driver.
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Option<Peri<'d, AnyPin>>,
        sda: Option<Peri<'d, AnyPin>>,
        tx_dma: Option<ChannelAndRequest<'d>>,
        rx_dma: Option<ChannelAndRequest<'d>>,
        config: Config,
    ) -> Self {
        unsafe { T::EventInterrupt::enable() };
        unsafe { T::ErrorInterrupt::enable() };

        let mut this = Self {
            info: T::info(),
            state: T::state(),
            kernel_clock: T::frequency(),
            tx_dma,
            rx_dma,
            #[cfg(feature = "time")]
            timeout: config.timeout,
            _phantom: PhantomData,
            _phantom2: PhantomData,
            _drop_guard: I2CDropGuard {
                info: T::info(),
                scl,
                sda,
            },
        };

        this.enable_and_init(config);

        this
    }

    fn enable_and_init(&mut self, config: Config) {
        self.info.rcc.enable_and_reset_without_stop();
        self.init(config);
    }
}

impl<'d, M: Mode, IM: MasterMode> I2c<'d, M, IM> {
    fn timeout(&self) -> Timeout {
        Timeout {
            #[cfg(feature = "time")]
            deadline: Instant::now() + self.timeout,
        }
    }
}

#[derive(Copy, Clone)]
struct Timeout {
    #[cfg(feature = "time")]
    deadline: Instant,
}

#[allow(dead_code)]
impl Timeout {
    #[inline]
    fn check(self) -> Result<(), Error> {
        #[cfg(feature = "time")]
        if Instant::now() > self.deadline {
            return Err(Error::Timeout);
        }

        Ok(())
    }

    #[inline]
    fn with<R>(self, fut: impl Future<Output = Result<R, Error>>) -> impl Future<Output = Result<R, Error>> {
        #[cfg(feature = "time")]
        {
            use futures_util::FutureExt;

            embassy_futures::select::select(embassy_time::Timer::at(self.deadline), fut).map(|r| match r {
                embassy_futures::select::Either::First(_) => Err(Error::Timeout),
                embassy_futures::select::Either::Second(r) => r,
            })
        }

        #[cfg(not(feature = "time"))]
        fut
    }
}

struct State {
    #[allow(unused)]
    waker: AtomicWaker,
}

impl State {
    const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

struct Info {
    regs: crate::pac::i2c::I2c,
    rcc: RccInfo,
}

peri_trait!(
    irqs: [EventInterrupt, ErrorInterrupt],
);

pin_trait!(SclPin, Instance, @A);
pin_trait!(SdaPin, Instance, @A);
dma_trait!(RxDma, Instance);
dma_trait!(TxDma, Instance);

/// Event interrupt handler.
pub struct EventInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::EventInterrupt> for EventInterruptHandler<T> {
    unsafe fn on_interrupt() {
        _version::on_interrupt::<T>()
    }
}

/// Error interrupt handler.
pub struct ErrorInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::ErrorInterrupt> for ErrorInterruptHandler<T> {
    unsafe fn on_interrupt() {
        _version::on_interrupt::<T>()
    }
}

foreach_peripheral!(
    (i2c, $inst:ident) => {
        #[allow(private_interfaces)]
        impl SealedInstance for peripherals::$inst {
            fn info() -> &'static Info {
                static INFO: Info = Info{
                    regs: crate::pac::$inst,
                    rcc: crate::peripherals::$inst::RCC_INFO,
                };
                &INFO
            }
            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl Instance for peripherals::$inst {
            type EventInterrupt = crate::_generated::peripheral_interrupts::$inst::EV;
            type ErrorInterrupt = crate::_generated::peripheral_interrupts::$inst::ER;
        }
    };
);

impl<'d, M: Mode, IM: MasterMode> embedded_hal_02::blocking::i2c::Read for I2c<'d, M, IM> {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, buffer)
    }
}

impl<'d, M: Mode, IM: MasterMode> embedded_hal_02::blocking::i2c::Write for I2c<'d, M, IM> {
    type Error = Error;

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, write)
    }
}

impl<'d, M: Mode, IM: MasterMode> embedded_hal_02::blocking::i2c::WriteRead for I2c<'d, M, IM> {
    type Error = Error;

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(address, write, read)
    }
}

impl embedded_hal_1::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match *self {
            Self::Bus => embedded_hal_1::i2c::ErrorKind::Bus,
            Self::Arbitration => embedded_hal_1::i2c::ErrorKind::ArbitrationLoss,
            Self::Nack => {
                embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Unknown)
            }
            Self::Timeout => embedded_hal_1::i2c::ErrorKind::Other,
            Self::Crc => embedded_hal_1::i2c::ErrorKind::Other,
            Self::Overrun => embedded_hal_1::i2c::ErrorKind::Overrun,
            Self::ZeroLengthTransfer => embedded_hal_1::i2c::ErrorKind::Other,
        }
    }
}

impl<'d, M: Mode, IM: MasterMode> embedded_hal_1::i2c::ErrorType for I2c<'d, M, IM> {
    type Error = Error;
}

impl<'d, M: Mode, IM: MasterMode> embedded_hal_1::i2c::I2c for I2c<'d, M, IM> {
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
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.blocking_transaction(address, operations)
    }
}

impl<'d, IM: MasterMode> embedded_hal_async::i2c::I2c for I2c<'d, Async, IM> {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.read(address, read).await
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.write(address, write).await
    }

    async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.write_read(address, write, read).await
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.transaction(address, operations).await
    }
}

/// Frame type in I2C transaction.
///
/// This tells each method what kind of frame to use, to generate a (repeated) start condition (ST
/// or SR), and/or a stop condition (SP). For read operations, this also controls whether to send an
/// ACK or NACK after the last byte received.
///
/// For write operations, the following options are identical because they differ only in the (N)ACK
/// treatment relevant for read operations:
///
/// - `FirstFrame` and `FirstAndNextFrame` behave identically for writes
/// - `NextFrame` and `LastFrameNoStop` behave identically for writes
///
/// Abbreviations used below:
///
/// - `ST` = start condition
/// - `SR` = repeated start condition
/// - `SP` = stop condition
/// - `ACK`/`NACK` = last byte in read operation
#[derive(Copy, Clone)]
#[allow(dead_code)]
enum FrameOptions {
    /// `[ST/SR]+[NACK]+[SP]` First frame (of this type) in transaction and also last frame overall.
    FirstAndLastFrame,
    /// `[ST/SR]+[NACK]` First frame of this type in transaction, last frame in a read operation but
    /// not the last frame overall.
    FirstFrame,
    /// `[ST/SR]+[ACK]` First frame of this type in transaction, neither last frame overall nor last
    /// frame in a read operation.
    FirstAndNextFrame,
    /// `[ACK]` Middle frame in a read operation (neither first nor last).
    NextFrame,
    /// `[NACK]+[SP]` Last frame overall in this transaction but not the first frame.
    LastFrame,
    /// `[NACK]` Last frame in a read operation but not last frame overall in this transaction.
    LastFrameNoStop,
}

#[allow(dead_code)]
impl FrameOptions {
    /// Returns true if a start or repeated start condition should be generated before this operation.
    fn send_start(self) -> bool {
        match self {
            Self::FirstAndLastFrame | Self::FirstFrame | Self::FirstAndNextFrame => true,
            Self::NextFrame | Self::LastFrame | Self::LastFrameNoStop => false,
        }
    }

    /// Returns true if a stop condition should be generated after this operation.
    fn send_stop(self) -> bool {
        match self {
            Self::FirstAndLastFrame | Self::LastFrame => true,
            Self::FirstFrame | Self::FirstAndNextFrame | Self::NextFrame | Self::LastFrameNoStop => false,
        }
    }

    /// Returns true if NACK should be sent after the last byte received in a read operation.
    ///
    /// This signals the end of a read sequence and releases the bus for the master's
    /// next transmission (or stop condition).
    fn send_nack(self) -> bool {
        match self {
            Self::FirstAndLastFrame | Self::FirstFrame | Self::LastFrame | Self::LastFrameNoStop => true,
            Self::FirstAndNextFrame | Self::NextFrame => false,
        }
    }
}

/// Analyzes I2C transaction operations and assigns appropriate frame to each.
///
/// This function processes a sequence of I2C operations and determines the correct
/// frame configuration for each operation to ensure proper I2C protocol compliance.
/// It handles the complex logic of:
///
/// - Generating start conditions for the first operation of each type (read/write)
/// - Generating stop conditions for the final operation in the entire transaction
/// - Managing ACK/NACK behavior for read operations, including merging consecutive reads
/// - Ensuring proper bus handoff between different operation types
///
/// **Transaction Contract Compliance:**
/// The frame assignments ensure compliance with the embedded-hal I2C transaction contract,
/// where consecutive operations of the same type are logically merged while maintaining
/// proper protocol boundaries.
///
/// **Error Handling:**
/// Returns an error if any read operation has an empty buffer, as this would create
/// an invalid I2C transaction that could halt mid-execution.
///
/// # Arguments
/// * `operations` - Mutable slice of I2C operations from embedded-hal
///
/// # Returns
/// An iterator over (operation, frame) pairs, or an error if the transaction is invalid
///
#[allow(dead_code)]
fn operation_frames<'a, 'b: 'a>(
    operations: &'a mut [embedded_hal_1::i2c::Operation<'b>],
) -> Result<impl IntoIterator<Item = (&'a mut embedded_hal_1::i2c::Operation<'b>, FrameOptions)>, Error> {
    use embedded_hal_1::i2c::Operation::{Read, Write};

    // Validate that no read operations have empty buffers before starting the transaction.
    // Empty read operations would risk halting with an error mid-transaction.
    //
    // Note: We could theoretically allow empty read operations within consecutive read
    // sequences as long as the final merged read has at least one byte, but this would
    // complicate the logic significantly and create error-prone edge cases.
    if operations.iter().any(|op| match op {
        Read(read) => read.is_empty(),
        Write(_) => false,
    }) {
        return Err(Error::Overrun);
    }

    let mut operations = operations.iter_mut().peekable();
    let mut next_first_operation = true;

    Ok(iter::from_fn(move || {
        let current_op = operations.next()?;

        // Determine if this is the first operation of its type (read or write)
        let is_first_of_type = next_first_operation;
        let next_op = operations.peek();

        // Compute the appropriate frame based on three key properties:
        //
        // 1. **Start Condition**: Generate (repeated) start for first operation of each type
        // 2. **Stop Condition**: Generate stop for the final operation in the entire transaction
        // 3. **ACK/NACK for Reads**: For read operations, send ACK if more reads follow in the
        //    sequence, or NACK for the final read in a sequence (before write or transaction end)
        //
        // The third property is checked for all operations since the resulting frame
        // configurations are identical for write operations regardless of ACK/NACK treatment.
        let frame = match (is_first_of_type, next_op) {
            // First operation of type, and it's also the final operation overall
            (true, None) => FrameOptions::FirstAndLastFrame,
            // First operation of type, next operation is also a read (continue read sequence)
            (true, Some(Read(_))) => FrameOptions::FirstAndNextFrame,
            // First operation of type, next operation is write (end current sequence)
            (true, Some(Write(_))) => FrameOptions::FirstFrame,

            // Continuation operation, and it's the final operation overall
            (false, None) => FrameOptions::LastFrame,
            // Continuation operation, next operation is also a read (continue read sequence)
            (false, Some(Read(_))) => FrameOptions::NextFrame,
            // Continuation operation, next operation is write (end current sequence, no stop)
            (false, Some(Write(_))) => FrameOptions::LastFrameNoStop,
        };

        // Pre-calculate whether the next operation will be the first of its type.
        // This is done here because we consume `current_op` as the iterator value
        // and cannot access it in the next iteration.
        next_first_operation = match (&current_op, next_op) {
            // No next operation
            (_, None) => false,
            // Operation type changes: next will be first of its type
            (Read(_), Some(Write(_))) | (Write(_), Some(Read(_))) => true,
            // Operation type continues: next will not be first of its type
            (Read(_), Some(Read(_))) | (Write(_), Some(Write(_))) => false,
        };

        Some((current_op, frame))
    }))
}
