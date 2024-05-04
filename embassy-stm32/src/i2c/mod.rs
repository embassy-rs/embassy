//! Inter-Integrated-Circuit (I2C)
#![macro_use]

#[cfg_attr(i2c_v1, path = "v1.rs")]
#[cfg_attr(any(i2c_v2, i2c_v3), path = "v2.rs")]
mod _version;

use core::future::Future;
use core::iter;
use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
#[cfg(feature = "time")]
use embassy_time::{Duration, Instant};

use crate::dma::ChannelAndRequest;
use crate::gpio::{AFType, Pull};
use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::time::Hertz;
use crate::{interrupt, peripherals};

/// I2C error.
#[derive(Debug, PartialEq, Eq)]
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

/// I2C config
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    /// Enable internal pullup on SDA.
    ///
    /// Using external pullup resistors is recommended for I2C. If you do
    /// have external pullups you should not enable this.
    pub sda_pullup: bool,
    /// Enable internal pullup on SCL.
    ///
    /// Using external pullup resistors is recommended for I2C. If you do
    /// have external pullups you should not enable this.
    pub scl_pullup: bool,
    /// Timeout.
    #[cfg(feature = "time")]
    pub timeout: embassy_time::Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sda_pullup: false,
            scl_pullup: false,
            #[cfg(feature = "time")]
            timeout: embassy_time::Duration::from_millis(1000),
        }
    }
}

/// I2C driver.
pub struct I2c<'d, T: Instance, M: Mode> {
    _peri: PeripheralRef<'d, T>,
    tx_dma: Option<ChannelAndRequest<'d>>,
    rx_dma: Option<ChannelAndRequest<'d>>,
    #[cfg(feature = "time")]
    timeout: Duration,
    _phantom: PhantomData<M>,
}

impl<'d, T: Instance> I2c<'d, T, Async> {
    /// Create a new I2C driver.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::EventInterrupt, EventInterruptHandler<T>>
            + interrupt::typelevel::Binding<T::ErrorInterrupt, ErrorInterruptHandler<T>>
            + 'd,
        tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
        rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
        freq: Hertz,
        config: Config,
    ) -> Self {
        Self::new_inner(peri, scl, sda, new_dma!(tx_dma), new_dma!(rx_dma), freq, config)
    }
}

impl<'d, T: Instance> I2c<'d, T, Blocking> {
    /// Create a new blocking I2C driver.
    pub fn new_blocking(
        peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        freq: Hertz,
        config: Config,
    ) -> Self {
        Self::new_inner(peri, scl, sda, None, None, freq, config)
    }
}

impl<'d, T: Instance, M: Mode> I2c<'d, T, M> {
    /// Create a new I2C driver.
    fn new_inner(
        peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        tx_dma: Option<ChannelAndRequest<'d>>,
        rx_dma: Option<ChannelAndRequest<'d>>,
        freq: Hertz,
        config: Config,
    ) -> Self {
        into_ref!(peri, scl, sda);

        T::enable_and_reset();

        scl.set_as_af_pull(
            scl.af_num(),
            AFType::OutputOpenDrain,
            match config.scl_pullup {
                true => Pull::Up,
                false => Pull::None,
            },
        );
        sda.set_as_af_pull(
            sda.af_num(),
            AFType::OutputOpenDrain,
            match config.sda_pullup {
                true => Pull::Up,
                false => Pull::None,
            },
        );

        unsafe { T::EventInterrupt::enable() };
        unsafe { T::ErrorInterrupt::enable() };

        let mut this = Self {
            _peri: peri,
            tx_dma,
            rx_dma,
            #[cfg(feature = "time")]
            timeout: config.timeout,
            _phantom: PhantomData,
        };

        this.init(freq, config);

        this
    }

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

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> crate::pac::i2c::I2c;
    fn state() -> &'static State;
}

/// I2C peripheral instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + 'static {
    /// Event interrupt for this instance
    type EventInterrupt: interrupt::typelevel::Interrupt;
    /// Error interrupt for this instance
    type ErrorInterrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(SclPin, Instance);
pin_trait!(SdaPin, Instance);
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
        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::i2c::I2c {
                crate::pac::$inst
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

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::Read for I2c<'d, T, M> {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, buffer)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::Write for I2c<'d, T, M> {
    type Error = Error;

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, write)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::WriteRead for I2c<'d, T, M> {
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

impl<'d, T: Instance, M: Mode> embedded_hal_1::i2c::ErrorType for I2c<'d, T, M> {
    type Error = Error;
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::i2c::I2c for I2c<'d, T, M> {
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

impl<'d, T: Instance> embedded_hal_async::i2c::I2c for I2c<'d, T, Async> {
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
/// This tells each method what kind of framing to use, to generate a (repeated) start condition (ST
/// or SR), and/or a stop condition (SP). For read operations, this also controls whether to send an
/// ACK or NACK after the last byte received.
///
/// For write operations, the following options are identical because they differ only in the (N)ACK
/// treatment relevant for read operations:
///
/// - `FirstFrame` and `FirstAndNextFrame`
/// - `NextFrame` and `LastFrameNoStop`
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
    /// Sends start or repeated start condition before transfer.
    fn send_start(self) -> bool {
        match self {
            Self::FirstAndLastFrame | Self::FirstFrame | Self::FirstAndNextFrame => true,
            Self::NextFrame | Self::LastFrame | Self::LastFrameNoStop => false,
        }
    }

    /// Sends stop condition after transfer.
    fn send_stop(self) -> bool {
        match self {
            Self::FirstAndLastFrame | Self::LastFrame => true,
            Self::FirstFrame | Self::FirstAndNextFrame | Self::NextFrame | Self::LastFrameNoStop => false,
        }
    }

    /// Sends NACK after last byte received, indicating end of read operation.
    fn send_nack(self) -> bool {
        match self {
            Self::FirstAndLastFrame | Self::FirstFrame | Self::LastFrame | Self::LastFrameNoStop => true,
            Self::FirstAndNextFrame | Self::NextFrame => false,
        }
    }
}

/// Iterates over operations in transaction.
///
/// Returns necessary frame options for each operation to uphold the [transaction contract] and have
/// the right start/stop/(N)ACK conditions on the wire.
///
/// [transaction contract]: embedded_hal_1::i2c::I2c::transaction
#[allow(dead_code)]
fn operation_frames<'a, 'b: 'a>(
    operations: &'a mut [embedded_hal_1::i2c::Operation<'b>],
) -> Result<impl IntoIterator<Item = (&'a mut embedded_hal_1::i2c::Operation<'b>, FrameOptions)>, Error> {
    use embedded_hal_1::i2c::Operation::{Read, Write};

    // Check empty read buffer before starting transaction. Otherwise, we would risk halting with an
    // error in the middle of the transaction.
    //
    // In principle, we could allow empty read frames within consecutive read operations, as long as
    // at least one byte remains in the final (merged) read operation, but that makes the logic more
    // complicated and error-prone.
    if operations.iter().any(|op| match op {
        Read(read) => read.is_empty(),
        Write(_) => false,
    }) {
        return Err(Error::Overrun);
    }

    let mut operations = operations.iter_mut().peekable();

    let mut next_first_frame = true;

    Ok(iter::from_fn(move || {
        let Some(op) = operations.next() else {
            return None;
        };

        // Is `op` first frame of its type?
        let first_frame = next_first_frame;
        let next_op = operations.peek();

        // Get appropriate frame options as combination of the following properties:
        //
        // - For each first operation of its type, generate a (repeated) start condition.
        // - For the last operation overall in the entire transaction, generate a stop condition.
        // - For read operations, check the next operation: if it is also a read operation, we merge
        //   these and send ACK for all bytes in the current operation; send NACK only for the final
        //   read operation's last byte (before write or end of entire transaction) to indicate last
        //   byte read and release the bus for transmission of the bus master's next byte (or stop).
        //
        // We check the third property unconditionally, i.e. even for write opeartions. This is okay
        // because the resulting frame options are identical for write operations.
        let frame = match (first_frame, next_op) {
            (true, None) => FrameOptions::FirstAndLastFrame,
            (true, Some(Read(_))) => FrameOptions::FirstAndNextFrame,
            (true, Some(Write(_))) => FrameOptions::FirstFrame,
            //
            (false, None) => FrameOptions::LastFrame,
            (false, Some(Read(_))) => FrameOptions::NextFrame,
            (false, Some(Write(_))) => FrameOptions::LastFrameNoStop,
        };

        // Pre-calculate if `next_op` is the first operation of its type. We do this here and not at
        // the beginning of the loop because we hand out `op` as iterator value and cannot access it
        // anymore in the next iteration.
        next_first_frame = match (&op, next_op) {
            (_, None) => false,
            (Read(_), Some(Write(_))) | (Write(_), Some(Read(_))) => true,
            (Read(_), Some(Read(_))) | (Write(_), Some(Write(_))) => false,
        };

        Some((op, frame))
    }))
}
