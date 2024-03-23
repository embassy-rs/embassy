use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::task::Poll;

pub mod bx;

pub use bx::{filter, Data, ExtendedId, Fifo, Frame, Header, Id, StandardId};
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::AFType;
use crate::interrupt::typelevel::Interrupt;
use crate::rcc::RccPeripheral;
use crate::{interrupt, peripherals, Peripheral};

pub mod enums;
pub mod frame;
pub mod util;

mod common;
pub use self::common::{BufferedCanReceiver, BufferedCanSender, Timestamp};

/// Contains CAN frame and additional metadata.
///
/// Timestamp is available if `time` feature is enabled.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Envelope {
    /// Reception time.
    #[cfg(feature = "time")]
    pub ts: embassy_time::Instant,
    /// The actual CAN frame.
    pub frame: Frame,
}

/// Interrupt handler.
pub struct TxInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::TXInterrupt> for TxInterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::regs().tsr().write(|v| {
            v.set_rqcp(0, true);
            v.set_rqcp(1, true);
            v.set_rqcp(2, true);
        });
        T::state().tx_mode.on_interrupt::<T>();
    }
}

/// RX0 interrupt handler.
pub struct Rx0InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::RX0Interrupt> for Rx0InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::state().rx_mode.on_interrupt::<T>(RxFifo::Fifo0);
    }
}

/// RX1 interrupt handler.
pub struct Rx1InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::RX1Interrupt> for Rx1InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::state().rx_mode.on_interrupt::<T>(RxFifo::Fifo1);
    }
}

/// SCE interrupt handler.
pub struct SceInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::SCEInterrupt> for SceInterruptHandler<T> {
    unsafe fn on_interrupt() {
        // info!("sce irq");
        let msr = T::regs().msr();
        let msr_val = msr.read();

        if msr_val.erri() {
            msr.modify(|v| v.set_erri(true));
            T::state().err_waker.wake();
        }
    }
}

/// CAN driver
pub struct Can<'d, T: Instance> {
    can: crate::can::bx::Can<BxcanInstance<'d, T>>,
}

/// Error returned by `try_write`
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryWriteError {
    /// All transmit mailboxes are full
    Full,
}

impl<'d, T: Instance> Can<'d, T> {
    /// Creates a new Bxcan instance, keeping the peripheral in sleep mode.
    /// You must call [Can::enable_non_blocking] to use the peripheral.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irqs: impl interrupt::typelevel::Binding<T::TXInterrupt, TxInterruptHandler<T>>
            + interrupt::typelevel::Binding<T::RX0Interrupt, Rx0InterruptHandler<T>>
            + interrupt::typelevel::Binding<T::RX1Interrupt, Rx1InterruptHandler<T>>
            + interrupt::typelevel::Binding<T::SCEInterrupt, SceInterruptHandler<T>>
            + 'd,
    ) -> Self {
        into_ref!(peri, rx, tx);

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        T::enable_and_reset();

        {
            T::regs().ier().write(|w| {
                w.set_errie(true);
                w.set_fmpie(0, true);
                w.set_fmpie(1, true);
                w.set_tmeie(true);
            });

            T::regs().mcr().write(|w| {
                // Enable timestamps on rx messages

                w.set_ttcm(true);
            });
        }

        unsafe {
            T::TXInterrupt::unpend();
            T::TXInterrupt::enable();

            T::RX0Interrupt::unpend();
            T::RX0Interrupt::enable();

            T::RX1Interrupt::unpend();
            T::RX1Interrupt::enable();

            T::SCEInterrupt::unpend();
            T::SCEInterrupt::enable();
        }

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        let can = crate::can::bx::Can::builder(BxcanInstance(peri), T::regs()).leave_disabled();
        Self { can }
    }

    /// Set CAN bit rate.
    pub fn set_bitrate(&mut self, bitrate: u32) {
        let bit_timing = util::calc_can_timings(T::frequency(), bitrate).unwrap();
        self.can.modify_config().set_bit_timing(bit_timing).leave_disabled();
    }

    /// Enables the peripheral and synchronizes with the bus.
    ///
    /// This will wait for 11 consecutive recessive bits (bus idle state).
    /// Contrary to enable method from bxcan library, this will not freeze the executor while waiting.
    pub async fn enable(&mut self) {
        while self.registers.enable_non_blocking().is_err() {
            // SCE interrupt is only generated for entering sleep mode, but not leaving.
            // Yield to allow other tasks to execute while can bus is initializing.
            embassy_futures::yield_now().await;
        }
    }

    /// Queues the message to be sent.
    ///
    /// If the TX queue is full, this will wait until there is space, therefore exerting backpressure.
    pub async fn write(&mut self, frame: &Frame) -> crate::can::bx::TransmitStatus {
        self.split().0.write(frame).await
    }

    /// Attempts to transmit a frame without blocking.
    ///
    /// Returns [Err(TryWriteError::Full)] if all transmit mailboxes are full.
    pub fn try_write(&mut self, frame: &Frame) -> Result<crate::can::bx::TransmitStatus, TryWriteError> {
        self.split().0.try_write(frame)
    }

    /// Waits for a specific transmit mailbox to become empty
    pub async fn flush(&self, mb: crate::can::bx::Mailbox) {
        CanTx::<T>::flush_inner(mb).await
    }

    /// Waits until any of the transmit mailboxes become empty
    pub async fn flush_any(&self) {
        CanTx::<T>::flush_any_inner().await
    }

    /// Waits until all of the transmit mailboxes become empty
    pub async fn flush_all(&self) {
        CanTx::<T>::flush_all_inner().await
    }

    /// Read a CAN frame.
    ///
    /// If no CAN frame is in the RX buffer, this will wait until there is one.
    ///
    /// Returns a tuple of the time the message was received and the message frame
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        T::state().rx_mode.read::<T>().await
    }

    /// Attempts to read a CAN frame without blocking.
    ///
    /// Returns [Err(TryReadError::Empty)] if there are no frames in the rx queue.
    pub fn try_read(&mut self) -> Result<Envelope, TryReadError> {
        T::state().rx_mode.try_read::<T>()
    }

    /// Waits while receive queue is empty.
    pub async fn wait_not_empty(&mut self) {
        T::state().rx_mode.wait_not_empty::<T>().await
    }

    /// Split the CAN driver into transmit and receive halves.
    ///
    /// Useful for doing separate transmit/receive tasks.
    pub fn split<'c>(&'c mut self) -> (CanTx<'d, T>, CanRx<'d, T>) {
        let (tx, rx) = self.can.split_by_ref();
        (CanTx { tx }, CanRx { rx })
    }
}

impl<'d, T: Instance> AsMut<crate::can::bx::Can<BxcanInstance<'d, T>>> for Can<'d, T> {
    /// Get mutable access to the lower-level driver from the `bxcan` crate.
    fn as_mut(&mut self) -> &mut crate::can::bx::Can<BxcanInstance<'d, T>> {
        &mut self.can
    }
}

/// CAN driver, transmit half.
pub struct CanTx<'d, T: Instance> {
    tx: crate::can::bx::Tx<BxcanInstance<'d, T>>,
}

impl<'d, T: Instance> CanTx<'d, T> {
    /// Queues the message to be sent.
    ///
    /// If the TX queue is full, this will wait until there is space, therefore exerting backpressure.
    pub async fn write(&mut self, frame: &Frame) -> crate::can::bx::TransmitStatus {
        poll_fn(|cx| {
            T::state().tx_mode.register(cx.waker());
            if let Ok(status) = self.tx.transmit(frame) {
                return Poll::Ready(status);
            }

            Poll::Pending
        })
        .await
    }

    /// Attempts to transmit a frame without blocking.
    ///
    /// Returns [Err(TryWriteError::Full)] if all transmit mailboxes are full.
    pub fn try_write(&mut self, frame: &Frame) -> Result<crate::can::bx::TransmitStatus, TryWriteError> {
        self.tx.transmit(frame).map_err(|_| TryWriteError::Full)
    }

    async fn flush_inner(mb: crate::can::bx::Mailbox) {
        poll_fn(|cx| {
            T::state().tx_mode.register(cx.waker());
            if T::regs().tsr().read().tme(mb.index()) {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Waits for a specific transmit mailbox to become empty
    pub async fn flush(&self, mb: crate::can::bx::Mailbox) {
        Self::flush_inner(mb).await
    }

    async fn flush_any_inner() {
        poll_fn(|cx| {
            T::state().tx_mode.register(cx.waker());

            let tsr = T::regs().tsr().read();
            if tsr.tme(crate::can::bx::Mailbox::Mailbox0.index())
                || tsr.tme(crate::can::bx::Mailbox::Mailbox1.index())
                || tsr.tme(crate::can::bx::Mailbox::Mailbox2.index())
            {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Waits until any of the transmit mailboxes become empty
    pub async fn flush_any(&self) {
        Self::flush_any_inner().await
    }

    async fn flush_all_inner() {
        poll_fn(|cx| {
            T::state().tx_mode.register(cx.waker());

            let tsr = T::regs().tsr().read();
            if tsr.tme(crate::can::bx::Mailbox::Mailbox0.index())
                && tsr.tme(crate::can::bx::Mailbox::Mailbox1.index())
                && tsr.tme(crate::can::bx::Mailbox::Mailbox2.index())
            {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Waits until all of the transmit mailboxes become empty
    pub async fn flush_all(&self) {
        Self::flush_all_inner().await
    }

    /// Return a buffered instance of driver. User must supply Buffers
    pub fn buffered<const TX_BUF_SIZE: usize>(
        self,
        txb: &'static mut TxBuf<TX_BUF_SIZE>,
    ) -> BufferedCanTx<'d, T, TX_BUF_SIZE> {
        BufferedCanTx::new(self.tx, txb)
    }
}

/// User supplied buffer for TX buffering
pub type TxBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, Frame, BUF_SIZE>;

/// CAN driver, transmit half.
pub struct BufferedCanTx<'d, T: Instance, const TX_BUF_SIZE: usize> {
    _tx: crate::can::bx::Tx<BxcanInstance<'d, T>>,
    tx_buf: &'static TxBuf<TX_BUF_SIZE>,
}

impl<'d, T: Instance, const TX_BUF_SIZE: usize> BufferedCanTx<'d, T, TX_BUF_SIZE> {
    fn new(_tx: crate::can::bx::Tx<BxcanInstance<'d, T>>, tx_buf: &'static TxBuf<TX_BUF_SIZE>) -> Self {
        Self { _tx, tx_buf }.setup()
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        critical_section::with(|_| unsafe {
            let tx_inner = self::common::ClassicBufferedTxInner {
                tx_receiver: self.tx_buf.receiver().into(),
            };
            T::mut_state().tx_mode = TxMode::Buffered(tx_inner);
        });
        self
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: &Frame) {
        self.tx_buf.send(*frame).await;
        T::TXInterrupt::pend(); // Wake for Tx
    }

    /// Returns a sender that can be used for sending CAN frames.
    pub fn writer(&self) -> BufferedCanSender {
        BufferedCanSender {
            tx_buf: self.tx_buf.sender().into(),
            waker: T::TXInterrupt::pend,
        }
    }
}

impl<'d, T: Instance, const TX_BUF_SIZE: usize> Drop for BufferedCanTx<'d, T, TX_BUF_SIZE> {
    fn drop(&mut self) {
        critical_section::with(|_| unsafe {
            T::mut_state().tx_mode = TxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
        });
    }
}

/// CAN driver, receive half.
#[allow(dead_code)]
pub struct CanRx<'d, T: Instance> {
    rx: crate::can::bx::Rx<BxcanInstance<'d, T>>,
}

impl<'d, T: Instance> CanRx<'d, T> {
    /// Read a CAN frame.
    ///
    /// If no CAN frame is in the RX buffer, this will wait until there is one.
    ///
    /// Returns a tuple of the time the message was received and the message frame
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        T::state().rx_mode.read::<T>().await
    }

    /// Attempts to read a CAN frame without blocking.
    ///
    /// Returns [Err(TryReadError::Empty)] if there are no frames in the rx queue.
    pub fn try_read(&mut self) -> Result<Envelope, TryReadError> {
        T::state().rx_mode.try_read::<T>()
    }

    /// Waits while receive queue is empty.
    pub async fn wait_not_empty(&mut self) {
        T::state().rx_mode.wait_not_empty::<T>().await
    }

    /// Return a buffered instance of driver. User must supply Buffers
    pub fn buffered<const RX_BUF_SIZE: usize>(
        self,
        rxb: &'static mut RxBuf<RX_BUF_SIZE>,
    ) -> BufferedCanRx<'d, T, RX_BUF_SIZE> {
        BufferedCanRx::new(self.rx, rxb)
    }
}

/// User supplied buffer for RX Buffering
pub type RxBuf<const BUF_SIZE: usize> =
    Channel<CriticalSectionRawMutex, Result<(Frame, Timestamp), BusError>, BUF_SIZE>;

/// CAN driver, receive half in Buffered mode.
pub struct BufferedCanRx<'d, T: Instance, const RX_BUF_SIZE: usize> {
    _rx: crate::can::bx::Rx<BxcanInstance<'d, T>>,
    rx_buf: &'static RxBuf<RX_BUF_SIZE>,
}

impl<'d, T: Instance, const RX_BUF_SIZE: usize> BufferedCanRx<'d, T, RX_BUF_SIZE> {
    fn new(_rx: crate::can::bx::Rx<BxcanInstance<'d, T>>, rx_buf: &'static RxBuf<RX_BUF_SIZE>) -> Self {
        BufferedCanRx { _rx, rx_buf }.setup()
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        critical_section::with(|_| unsafe {
            let rx_inner = self::common::ClassicBufferedRxInner {
                rx_sender: self.rx_buf.sender().into(),
            };
            T::mut_state().rx_mode = RxMode::Buffered(rx_inner);
        });
        self
    }

    /// Async read frame from RX buffer.
    pub async fn read(&mut self) -> Result<(Frame, Timestamp), BusError> {
        self.rx_buf.receive().await
    }

    /// Attempts to read a CAN frame without blocking.
    ///
    /// Returns [Err(TryReadError::Empty)] if there are no frames in the rx queue.
    pub fn try_read(&mut self) -> Result<Envelope, TryReadError> {
        match &T::state().rx_mode {
            RxMode::Buffered(_) => {
                if let Ok(result) = self.rx_buf.try_receive() {
                    match result {
                        Ok((frame, ts)) => Ok(Envelope { ts, frame }),
                        Err(e) => Err(TryReadError::BusError(e)),
                    }
                } else {
                    let registers = crate::can::bx::Registers { canregs: T::regs() };
                    if let Some(err) = registers.curr_error() {
                        return Err(TryReadError::BusError(err));
                    } else {
                        Err(TryReadError::Empty)
                    }
                }
            }
            _ => {
                panic!("Bad Mode")
            }
        }
    }

    /// Waits while receive queue is empty.
    pub async fn wait_not_empty(&mut self) {
        poll_fn(|cx| self.rx_buf.poll_ready_to_receive(cx)).await
    }

    /// Returns a receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
    pub fn reader(&self) -> BufferedCanReceiver {
        self.rx_buf.receiver().into()
    }
}

impl<'d, T: Instance, const RX_BUF_SIZE: usize> Drop for BufferedCanRx<'d, T, RX_BUF_SIZE> {
    fn drop(&mut self) {
        critical_section::with(|_| unsafe {
            T::mut_state().rx_mode = RxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
        });
    }
}

use crate::can::bx::RxFifo;

impl<'d, T: Instance> Drop for Can<'d, T> {
    fn drop(&mut self) {
        // Cannot call `free()` because it moves the instance.
        // Manually reset the peripheral.
        T::regs().mcr().write(|w| w.set_reset(true));
        T::disable();
    }
}

impl<'d, T: Instance> Deref for Can<'d, T> {
    type Target = crate::can::bx::Can<BxcanInstance<'d, T>>;

    fn deref(&self) -> &Self::Target {
        &self.can
    }
}

impl<'d, T: Instance> DerefMut for Can<'d, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.can
    }
}

use crate::can::enums::{BusError, TryReadError};

pub(crate) enum RxMode {
    NonBuffered(AtomicWaker),
    Buffered(crate::can::_version::common::ClassicBufferedRxInner),
}

impl RxMode {
    pub fn on_interrupt<T: Instance>(&self, fifo: crate::can::_version::bx::RxFifo) {
        match self {
            Self::NonBuffered(waker) => {
                // Disable interrupts until read
                let fifo_idx = match fifo {
                    crate::can::_version::bx::RxFifo::Fifo0 => 0usize,
                    crate::can::_version::bx::RxFifo::Fifo1 => 1usize,
                };
                T::regs().ier().write(|w| {
                    w.set_fmpie(fifo_idx, false);
                });
                waker.wake();
            }
            Self::Buffered(buf) => {
                let regsisters = crate::can::bx::Registers { canregs: T::regs() };

                loop {
                    match regsisters.receive_fifo(fifo) {
                        Some(envelope) => {
                            // NOTE: consensus was reached that if rx_queue is full, packets should be dropped
                            let _ = buf.rx_sender.try_send(Ok((envelope.frame, envelope.ts)));
                        }
                        None => return,
                    };
                }
            }
        }
    }

    pub async fn read<T: Instance>(&self) -> Result<Envelope, BusError> {
        match self {
            Self::NonBuffered(waker) => {
                poll_fn(|cx| {
                    T::state().err_waker.register(cx.waker());
                    waker.register(cx.waker());
                    match self.try_read::<T>() {
                        Ok(result) => Poll::Ready(Ok(result)),
                        Err(TryReadError::Empty) => Poll::Pending,
                        Err(TryReadError::BusError(be)) => Poll::Ready(Err(be)),
                    }
                })
                .await
            }
            _ => {
                panic!("Bad Mode")
            }
        }
    }
    pub fn try_read<T: Instance>(&self) -> Result<Envelope, TryReadError> {
        match self {
            Self::NonBuffered(_) => {
                let registers = crate::can::bx::Registers { canregs: T::regs() };
                if let Some(msg) = registers.receive_fifo(super::bx::RxFifo::Fifo0) {
                    T::regs().ier().write(|w| {
                        w.set_fmpie(0, true);
                    });
                    Ok(msg)
                } else if let Some(msg) = registers.receive_fifo(super::bx::RxFifo::Fifo1) {
                    T::regs().ier().write(|w| {
                        w.set_fmpie(1, true);
                    });
                    Ok(msg)
                } else if let Some(err) = registers.curr_error() {
                    Err(TryReadError::BusError(err))
                } else {
                    Err(TryReadError::Empty)
                }
            }
            _ => {
                panic!("Bad Mode")
            }
        }
    }
    pub async fn wait_not_empty<T: Instance>(&self) {
        match &T::state().rx_mode {
            Self::NonBuffered(waker) => {
                poll_fn(|cx| {
                    waker.register(cx.waker());
                    let registers = crate::can::bx::Registers { canregs: T::regs() };
                    if registers.receive_frame_available() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await
            }
            _ => {
                panic!("Bad Mode")
            }
        }
    }
}

enum TxMode {
    NonBuffered(AtomicWaker),
    Buffered(self::common::ClassicBufferedTxInner),
}

impl TxMode {
    pub fn buffer_free<T: Instance>(&self) -> bool {
        let tsr = T::regs().tsr().read();
        tsr.tme(crate::can::bx::Mailbox::Mailbox0.index())
            || tsr.tme(crate::can::bx::Mailbox::Mailbox1.index())
            || tsr.tme(crate::can::bx::Mailbox::Mailbox2.index())
    }
    pub fn on_interrupt<T: Instance>(&self) {
        match &T::state().tx_mode {
            TxMode::NonBuffered(waker) => waker.wake(),
            TxMode::Buffered(buf) => {
                while self.buffer_free::<T>() {
                    match buf.tx_receiver.try_receive() {
                        Ok(frame) => {
                            let mut registers = crate::can::bx::Registers { canregs: T::regs() };
                            _ = registers.transmit(&frame);
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn register(&self, arg: &core::task::Waker) {
        match self {
            TxMode::NonBuffered(waker) => {
                waker.register(arg);
            }
            _ => {
                panic!("Bad mode");
            }
        }
    }
}

struct State {
    pub(crate) rx_mode: RxMode,
    pub(crate) tx_mode: TxMode,
    pub err_waker: AtomicWaker,
}

impl State {
    pub const fn new() -> Self {
        Self {
            rx_mode: RxMode::NonBuffered(AtomicWaker::new()),
            tx_mode: TxMode::NonBuffered(AtomicWaker::new()),
            err_waker: AtomicWaker::new(),
        }
    }
}

trait SealedInstance {
    fn regs() -> crate::pac::can::Can;
    fn state() -> &'static State;
    unsafe fn mut_state() -> &'static mut State;
}

/// CAN instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + RccPeripheral + 'static {
    /// TX interrupt for this instance.
    type TXInterrupt: crate::interrupt::typelevel::Interrupt;
    /// RX0 interrupt for this instance.
    type RX0Interrupt: crate::interrupt::typelevel::Interrupt;
    /// RX1 interrupt for this instance.
    type RX1Interrupt: crate::interrupt::typelevel::Interrupt;
    /// SCE interrupt for this instance.
    type SCEInterrupt: crate::interrupt::typelevel::Interrupt;
}

/// BXCAN instance newtype.
pub struct BxcanInstance<'a, T>(PeripheralRef<'a, T>);

unsafe impl<'d, T: Instance> crate::can::bx::Instance for BxcanInstance<'d, T> {}

foreach_peripheral!(
    (can, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {

            fn regs() -> crate::pac::can::Can {
                crate::pac::$inst
            }

            unsafe fn mut_state() -> & 'static mut State {
                static mut STATE: State = State::new();
                &mut *core::ptr::addr_of_mut!(STATE)
            }
            fn state() -> &'static State {
                unsafe { peripherals::$inst::mut_state() }
            }
        }

        impl Instance for peripherals::$inst {
            type TXInterrupt = crate::_generated::peripheral_interrupts::$inst::TX;
            type RX0Interrupt = crate::_generated::peripheral_interrupts::$inst::RX0;
            type RX1Interrupt = crate::_generated::peripheral_interrupts::$inst::RX1;
            type SCEInterrupt = crate::_generated::peripheral_interrupts::$inst::SCE;
        }
    };
);

foreach_peripheral!(
    (can, CAN) => {
        unsafe impl<'d> crate::can::bx::FilterOwner for BxcanInstance<'d, peripherals::CAN> {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
    // CAN1 and CAN2 is a combination of master and slave instance.
    // CAN1 owns the filter bank and needs to be enabled in order
    // for CAN2 to receive messages.
    (can, CAN1) => {
        cfg_if::cfg_if! {
            if #[cfg(all(
                any(stm32l4, stm32f72, stm32f73),
                not(any(stm32l49, stm32l4a))
            ))] {
                // Most L4 devices and some F7 devices use the name "CAN1"
                // even if there is no "CAN2" peripheral.
                unsafe impl<'d> crate::can::bx::FilterOwner for BxcanInstance<'d, peripherals::CAN1> {
                    const NUM_FILTER_BANKS: u8 = 14;
                }
            } else {
                unsafe impl<'d> crate::can::bx::FilterOwner for BxcanInstance<'d, peripherals::CAN1> {
                    const NUM_FILTER_BANKS: u8 = 28;
                }
                unsafe impl<'d> crate::can::bx::MasterInstance for BxcanInstance<'d, peripherals::CAN1> {}
            }
        }
    };
    (can, CAN3) => {
        unsafe impl<'d> crate::can::bx::FilterOwner for BxcanInstance<'d, peripherals::CAN3> {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);

trait Index {
    fn index(&self) -> usize;
}

impl Index for crate::can::bx::Mailbox {
    fn index(&self) -> usize {
        match self {
            crate::can::bx::Mailbox::Mailbox0 => 0,
            crate::can::bx::Mailbox::Mailbox1 => 1,
            crate::can::bx::Mailbox::Mailbox2 => 2,
        }
    }
}
