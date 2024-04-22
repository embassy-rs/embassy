pub mod filter;
mod registers;

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::waitqueue::AtomicWaker;
pub use embedded_can::{ExtendedId, Id, StandardId};

use self::filter::MasterFilters;
use self::registers::{Registers, RxFifo};
pub use super::common::{BufferedCanReceiver, BufferedCanSender};
use super::frame::{Envelope, Frame};
use super::util;
use crate::can::enums::{BusError, TryReadError};
use crate::gpio::AFType;
use crate::interrupt::typelevel::Interrupt;
use crate::rcc::RccPeripheral;
use crate::{interrupt, peripherals, Peripheral};

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

/// Configuration proxy returned by [`Can::modify_config`].
pub struct CanConfig<'a, T: Instance> {
    can: PhantomData<&'a mut T>,
}

impl<T: Instance> CanConfig<'_, T> {
    /// Configures the bit timings.
    ///
    /// You can use <http://www.bittiming.can-wiki.info/> to calculate the `btr` parameter. Enter
    /// parameters as follows:
    ///
    /// - *Clock Rate*: The input clock speed to the CAN peripheral (*not* the CPU clock speed).
    ///   This is the clock rate of the peripheral bus the CAN peripheral is attached to (eg. APB1).
    /// - *Sample Point*: Should normally be left at the default value of 87.5%.
    /// - *SJW*: Should normally be left at the default value of 1.
    ///
    /// Then copy the `CAN_BUS_TIME` register value from the table and pass it as the `btr`
    /// parameter to this method.
    pub fn set_bit_timing(self, bt: crate::can::util::NominalBitTiming) -> Self {
        Registers(T::regs()).set_bit_timing(bt);
        self
    }

    /// Configure the CAN bit rate.
    ///
    /// This is a helper that internally calls `set_bit_timing()`[Self::set_bit_timing].
    pub fn set_bitrate(self, bitrate: u32) -> Self {
        let bit_timing = util::calc_can_timings(T::frequency(), bitrate).unwrap();
        self.set_bit_timing(bit_timing)
    }

    /// Enables or disables loopback mode: Internally connects the TX and RX
    /// signals together.
    pub fn set_loopback(self, enabled: bool) -> Self {
        Registers(T::regs()).set_loopback(enabled);
        self
    }

    /// Enables or disables silent mode: Disconnects the TX signal from the pin.
    pub fn set_silent(self, enabled: bool) -> Self {
        Registers(T::regs()).set_silent(enabled);
        self
    }

    /// Enables or disables automatic retransmission of messages.
    ///
    /// If this is enabled, the CAN peripheral will automatically try to retransmit each frame
    /// until it can be sent. Otherwise, it will try only once to send each frame.
    ///
    /// Automatic retransmission is enabled by default.
    pub fn set_automatic_retransmit(self, enabled: bool) -> Self {
        Registers(T::regs()).set_automatic_retransmit(enabled);
        self
    }
}

impl<T: Instance> Drop for CanConfig<'_, T> {
    #[inline]
    fn drop(&mut self) {
        Registers(T::regs()).leave_init_mode();
    }
}

/// CAN driver
pub struct Can<'d, T: Instance> {
    peri: PeripheralRef<'d, T>,
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

        Registers(T::regs()).leave_init_mode();

        Self { peri }
    }

    /// Set CAN bit rate.
    pub fn set_bitrate(&mut self, bitrate: u32) {
        let bit_timing = util::calc_can_timings(T::frequency(), bitrate).unwrap();
        self.modify_config().set_bit_timing(bit_timing);
    }

    /// Configure bit timings and silent/loop-back mode.
    ///
    /// Calling this method will enter initialization mode. You must enable the peripheral
    /// again afterwards with [`enable`](Self::enable).
    pub fn modify_config(&mut self) -> CanConfig<'_, T> {
        Registers(T::regs()).enter_init_mode();

        CanConfig { can: PhantomData }
    }

    /// Enables the peripheral and synchronizes with the bus.
    ///
    /// This will wait for 11 consecutive recessive bits (bus idle state).
    /// Contrary to enable method from bxcan library, this will not freeze the executor while waiting.
    pub async fn enable(&mut self) {
        while Registers(T::regs()).enable_non_blocking().is_err() {
            // SCE interrupt is only generated for entering sleep mode, but not leaving.
            // Yield to allow other tasks to execute while can bus is initializing.
            embassy_futures::yield_now().await;
        }
    }

    /// Queues the message to be sent.
    ///
    /// If the TX queue is full, this will wait until there is space, therefore exerting backpressure.
    pub async fn write(&mut self, frame: &Frame) -> TransmitStatus {
        self.split().0.write(frame).await
    }

    /// Attempts to transmit a frame without blocking.
    ///
    /// Returns [Err(TryWriteError::Full)] if all transmit mailboxes are full.
    pub fn try_write(&mut self, frame: &Frame) -> Result<TransmitStatus, TryWriteError> {
        self.split().0.try_write(frame)
    }

    /// Waits for a specific transmit mailbox to become empty
    pub async fn flush(&self, mb: Mailbox) {
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

    /// Attempts to abort the sending of a frame that is pending in a mailbox.
    ///
    /// If there is no frame in the provided mailbox, or its transmission succeeds before it can be
    /// aborted, this function has no effect and returns `false`.
    ///
    /// If there is a frame in the provided mailbox, and it is canceled successfully, this function
    /// returns `true`.
    pub fn abort(&mut self, mailbox: Mailbox) -> bool {
        Registers(T::regs()).abort(mailbox)
    }

    /// Returns `true` if no frame is pending for transmission.
    pub fn is_transmitter_idle(&self) -> bool {
        Registers(T::regs()).is_idle()
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
        (
            CanTx {
                _peri: unsafe { self.peri.clone_unchecked() },
            },
            CanRx {
                peri: unsafe { self.peri.clone_unchecked() },
            },
        )
    }

    /// Return a buffered instance of driver. User must supply Buffers
    pub fn buffered<'c, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize>(
        &'c mut self,
        txb: &'static mut TxBuf<TX_BUF_SIZE>,
        rxb: &'static mut RxBuf<RX_BUF_SIZE>,
    ) -> BufferedCan<'d, T, TX_BUF_SIZE, RX_BUF_SIZE> {
        let (tx, rx) = self.split();
        BufferedCan {
            tx: tx.buffered(txb),
            rx: rx.buffered(rxb),
        }
    }
}

impl<'d, T: FilterOwner> Can<'d, T> {
    /// Accesses the filter banks owned by this CAN peripheral.
    ///
    /// To modify filters of a slave peripheral, `modify_filters` has to be called on the master
    /// peripheral instead.
    pub fn modify_filters(&mut self) -> MasterFilters<'_, T> {
        unsafe { MasterFilters::new(T::regs()) }
    }
}

/// Buffered CAN driver.
pub struct BufferedCan<'d, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> {
    tx: BufferedCanTx<'d, T, TX_BUF_SIZE>,
    rx: BufferedCanRx<'d, T, RX_BUF_SIZE>,
}

impl<'d, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> BufferedCan<'d, T, TX_BUF_SIZE, RX_BUF_SIZE> {
    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: &Frame) {
        self.tx.write(frame).await
    }

    /// Returns a sender that can be used for sending CAN frames.
    pub fn writer(&self) -> BufferedCanSender {
        self.tx.writer()
    }

    /// Async read frame from RX buffer.
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        self.rx.read().await
    }

    /// Attempts to read a CAN frame without blocking.
    ///
    /// Returns [Err(TryReadError::Empty)] if there are no frames in the rx queue.
    pub fn try_read(&mut self) -> Result<Envelope, TryReadError> {
        self.rx.try_read()
    }

    /// Waits while receive queue is empty.
    pub async fn wait_not_empty(&mut self) {
        self.rx.wait_not_empty().await
    }

    /// Returns a receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
    pub fn reader(&self) -> BufferedCanReceiver {
        self.rx.reader()
    }
}

/// CAN driver, transmit half.
pub struct CanTx<'d, T: Instance> {
    _peri: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> CanTx<'d, T> {
    /// Queues the message to be sent.
    ///
    /// If the TX queue is full, this will wait until there is space, therefore exerting backpressure.
    pub async fn write(&mut self, frame: &Frame) -> TransmitStatus {
        poll_fn(|cx| {
            T::state().tx_mode.register(cx.waker());
            if let Ok(status) = Registers(T::regs()).transmit(frame) {
                return Poll::Ready(status);
            }

            Poll::Pending
        })
        .await
    }

    /// Attempts to transmit a frame without blocking.
    ///
    /// Returns [Err(TryWriteError::Full)] if all transmit mailboxes are full.
    pub fn try_write(&mut self, frame: &Frame) -> Result<TransmitStatus, TryWriteError> {
        Registers(T::regs()).transmit(frame).map_err(|_| TryWriteError::Full)
    }

    async fn flush_inner(mb: Mailbox) {
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
    pub async fn flush(&self, mb: Mailbox) {
        Self::flush_inner(mb).await
    }

    async fn flush_any_inner() {
        poll_fn(|cx| {
            T::state().tx_mode.register(cx.waker());

            let tsr = T::regs().tsr().read();
            if tsr.tme(Mailbox::Mailbox0.index())
                || tsr.tme(Mailbox::Mailbox1.index())
                || tsr.tme(Mailbox::Mailbox2.index())
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
            if tsr.tme(Mailbox::Mailbox0.index())
                && tsr.tme(Mailbox::Mailbox1.index())
                && tsr.tme(Mailbox::Mailbox2.index())
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

    /// Attempts to abort the sending of a frame that is pending in a mailbox.
    ///
    /// If there is no frame in the provided mailbox, or its transmission succeeds before it can be
    /// aborted, this function has no effect and returns `false`.
    ///
    /// If there is a frame in the provided mailbox, and it is canceled successfully, this function
    /// returns `true`.
    pub fn abort(&mut self, mailbox: Mailbox) -> bool {
        Registers(T::regs()).abort(mailbox)
    }

    /// Returns `true` if no frame is pending for transmission.
    pub fn is_idle(&self) -> bool {
        Registers(T::regs()).is_idle()
    }

    /// Return a buffered instance of driver. User must supply Buffers
    pub fn buffered<const TX_BUF_SIZE: usize>(
        self,
        txb: &'static mut TxBuf<TX_BUF_SIZE>,
    ) -> BufferedCanTx<'d, T, TX_BUF_SIZE> {
        BufferedCanTx::new(self, txb)
    }
}

/// User supplied buffer for TX buffering
pub type TxBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, Frame, BUF_SIZE>;

/// Buffered CAN driver, transmit half.
pub struct BufferedCanTx<'d, T: Instance, const TX_BUF_SIZE: usize> {
    _tx: CanTx<'d, T>,
    tx_buf: &'static TxBuf<TX_BUF_SIZE>,
}

impl<'d, T: Instance, const TX_BUF_SIZE: usize> BufferedCanTx<'d, T, TX_BUF_SIZE> {
    fn new(_tx: CanTx<'d, T>, tx_buf: &'static TxBuf<TX_BUF_SIZE>) -> Self {
        Self { _tx, tx_buf }.setup()
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        critical_section::with(|_| unsafe {
            let tx_inner = super::common::ClassicBufferedTxInner {
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
    peri: PeripheralRef<'d, T>,
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
        BufferedCanRx::new(self, rxb)
    }
}

/// User supplied buffer for RX Buffering
pub type RxBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, Result<Envelope, BusError>, BUF_SIZE>;

/// CAN driver, receive half in Buffered mode.
pub struct BufferedCanRx<'d, T: Instance, const RX_BUF_SIZE: usize> {
    _rx: CanRx<'d, T>,
    rx_buf: &'static RxBuf<RX_BUF_SIZE>,
}

impl<'d, T: Instance, const RX_BUF_SIZE: usize> BufferedCanRx<'d, T, RX_BUF_SIZE> {
    fn new(_rx: CanRx<'d, T>, rx_buf: &'static RxBuf<RX_BUF_SIZE>) -> Self {
        BufferedCanRx { _rx, rx_buf }.setup()
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        critical_section::with(|_| unsafe {
            let rx_inner = super::common::ClassicBufferedRxInner {
                rx_sender: self.rx_buf.sender().into(),
            };
            T::mut_state().rx_mode = RxMode::Buffered(rx_inner);
        });
        self
    }

    /// Async read frame from RX buffer.
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
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
                        Ok(envelope) => Ok(envelope),
                        Err(e) => Err(TryReadError::BusError(e)),
                    }
                } else {
                    if let Some(err) = Registers(T::regs()).curr_error() {
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

impl<'d, T: Instance> Drop for Can<'d, T> {
    fn drop(&mut self) {
        // Cannot call `free()` because it moves the instance.
        // Manually reset the peripheral.
        T::regs().mcr().write(|w| w.set_reset(true));
        T::disable();
    }
}

/// Identifies one of the two receive FIFOs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Fifo {
    /// First receive FIFO
    Fifo0 = 0,
    /// Second receive FIFO
    Fifo1 = 1,
}

/// Identifies one of the three transmit mailboxes.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Mailbox {
    /// Transmit mailbox 0
    Mailbox0 = 0,
    /// Transmit mailbox 1
    Mailbox1 = 1,
    /// Transmit mailbox 2
    Mailbox2 = 2,
}

/// Contains information about a frame enqueued for transmission via [`Can::transmit`] or
/// [`Tx::transmit`].
pub struct TransmitStatus {
    dequeued_frame: Option<Frame>,
    mailbox: Mailbox,
}

impl TransmitStatus {
    /// Returns the lower-priority frame that was dequeued to make space for the new frame.
    #[inline]
    pub fn dequeued_frame(&self) -> Option<&Frame> {
        self.dequeued_frame.as_ref()
    }

    /// Returns the [`Mailbox`] the frame was enqueued in.
    #[inline]
    pub fn mailbox(&self) -> Mailbox {
        self.mailbox
    }
}

pub(crate) enum RxMode {
    NonBuffered(AtomicWaker),
    Buffered(super::common::ClassicBufferedRxInner),
}

impl RxMode {
    pub fn on_interrupt<T: Instance>(&self, fifo: RxFifo) {
        match self {
            Self::NonBuffered(waker) => {
                // Disable interrupts until read
                let fifo_idx = match fifo {
                    RxFifo::Fifo0 => 0usize,
                    RxFifo::Fifo1 => 1usize,
                };
                T::regs().ier().write(|w| {
                    w.set_fmpie(fifo_idx, false);
                });
                waker.wake();
            }
            Self::Buffered(buf) => {
                loop {
                    match Registers(T::regs()).receive_fifo(fifo) {
                        Some(envelope) => {
                            // NOTE: consensus was reached that if rx_queue is full, packets should be dropped
                            let _ = buf.rx_sender.try_send(Ok(envelope));
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
                let registers = Registers(T::regs());
                if let Some(msg) = registers.receive_fifo(RxFifo::Fifo0) {
                    T::regs().ier().write(|w| {
                        w.set_fmpie(0, true);
                    });
                    Ok(msg)
                } else if let Some(msg) = registers.receive_fifo(RxFifo::Fifo1) {
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
                    if Registers(T::regs()).receive_frame_available() {
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
    Buffered(super::common::ClassicBufferedTxInner),
}

impl TxMode {
    pub fn buffer_free<T: Instance>(&self) -> bool {
        let tsr = T::regs().tsr().read();
        tsr.tme(Mailbox::Mailbox0.index()) || tsr.tme(Mailbox::Mailbox1.index()) || tsr.tme(Mailbox::Mailbox2.index())
    }
    pub fn on_interrupt<T: Instance>(&self) {
        match &T::state().tx_mode {
            TxMode::NonBuffered(waker) => waker.wake(),
            TxMode::Buffered(buf) => {
                while self.buffer_free::<T>() {
                    match buf.tx_receiver.try_receive() {
                        Ok(frame) => {
                            _ = Registers(T::regs()).transmit(&frame);
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
pub trait Instance: Peripheral<P = Self> + SealedInstance + RccPeripheral + 'static {
    /// TX interrupt for this instance.
    type TXInterrupt: crate::interrupt::typelevel::Interrupt;
    /// RX0 interrupt for this instance.
    type RX0Interrupt: crate::interrupt::typelevel::Interrupt;
    /// RX1 interrupt for this instance.
    type RX1Interrupt: crate::interrupt::typelevel::Interrupt;
    /// SCE interrupt for this instance.
    type SCEInterrupt: crate::interrupt::typelevel::Interrupt;
}

/// A bxCAN instance that owns filter banks.
///
/// In master-slave-instance setups, only the master instance owns the filter banks, and needs to
/// split some of them off for use by the slave instance. In that case, the master instance should
/// implement [`FilterOwner`] and [`MasterInstance`], while the slave instance should only implement
/// [`Instance`].
///
/// In single-instance configurations, the instance owns all filter banks and they can not be split
/// off. In that case, the instance should implement [`Instance`] and [`FilterOwner`].
///
/// # Safety
///
/// This trait must only be implemented if the instance does, in fact, own its associated filter
/// banks, and `NUM_FILTER_BANKS` must be correct.
pub unsafe trait FilterOwner: Instance {
    /// The total number of filter banks available to the instance.
    ///
    /// This is usually either 14 or 28, and should be specified in the chip's reference manual or datasheet.
    const NUM_FILTER_BANKS: u8;
}

/// A bxCAN master instance that shares filter banks with a slave instance.
///
/// In master-slave-instance setups, this trait should be implemented for the master instance.
///
/// # Safety
///
/// This trait must only be implemented when there is actually an associated slave instance.
pub unsafe trait MasterInstance: FilterOwner {}

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
        unsafe impl FilterOwner for peripherals::CAN {
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
                unsafe impl FilterOwner for peripherals::CAN1 {
                    const NUM_FILTER_BANKS: u8 = 14;
                }
            } else {
                unsafe impl FilterOwner for peripherals::CAN1 {
                    const NUM_FILTER_BANKS: u8 = 28;
                }
                unsafe impl MasterInstance for peripherals::CAN1 {}
            }
        }
    };
    (can, CAN3) => {
        unsafe impl FilterOwner for peripherals::CAN3 {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);

trait Index {
    fn index(&self) -> usize;
}

impl Index for Mailbox {
    fn index(&self) -> usize {
        match self {
            Mailbox::Mailbox0 => 0,
            Mailbox::Mailbox1 => 1,
            Mailbox::Mailbox2 => 2,
        }
    }
}
