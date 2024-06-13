pub mod filter;
mod registers;

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::interrupt::InterruptExt;
use embassy_hal_internal::into_ref;
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
use crate::rcc::{self, RccPeripheral};
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
        info!("sce irq");
        let msr = T::regs().msr();
        let msr_val = msr.read();

        if msr_val.slaki() {
            msr.modify(|m| m.set_slaki(true));
            T::state().err_waker.wake();
        } else if msr_val.erri() {
            info!("Error interrupt");
            // Disable the interrupt, but don't acknowledge the error, so that it can be
            // forwarded off the the bus message consumer. If we don't provide some way for
            // downstream code to determine that it has already provided this bus error instance
            // to the bus message consumer, we are doomed to re-provide a single error instance for
            // an indefinite amount of time.
            let ier = T::regs().ier();
            ier.modify(|i| i.set_errie(false));

            T::state().err_waker.wake();
        }
    }
}

/// Configuration proxy returned by [`Can::modify_config`].
pub struct CanConfig<'a> {
    phantom: PhantomData<&'a ()>,
    info: &'static Info,
    periph_clock: crate::time::Hertz,
}

impl CanConfig<'_> {
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
        self.info.regs.set_bit_timing(bt);
        self
    }

    /// Configure the CAN bit rate.
    ///
    /// This is a helper that internally calls `set_bit_timing()`[Self::set_bit_timing].
    pub fn set_bitrate(self, bitrate: u32) -> Self {
        let bit_timing = util::calc_can_timings(self.periph_clock, bitrate).unwrap();
        self.set_bit_timing(bit_timing)
    }

    /// Enables or disables loopback mode: Internally connects the TX and RX
    /// signals together.
    pub fn set_loopback(self, enabled: bool) -> Self {
        self.info.regs.set_loopback(enabled);
        self
    }

    /// Enables or disables silent mode: Disconnects the TX signal from the pin.
    pub fn set_silent(self, enabled: bool) -> Self {
        self.info.regs.set_silent(enabled);
        self
    }

    /// Enables or disables automatic retransmission of frames.
    ///
    /// If this is enabled, the CAN peripheral will automatically try to retransmit each frame
    /// until it can be sent. Otherwise, it will try only once to send each frame.
    ///
    /// Automatic retransmission is enabled by default.
    pub fn set_automatic_retransmit(self, enabled: bool) -> Self {
        self.info.regs.set_automatic_retransmit(enabled);
        self
    }
}

impl Drop for CanConfig<'_> {
    #[inline]
    fn drop(&mut self) {
        self.info.regs.leave_init_mode();
    }
}

/// CAN driver
pub struct Can<'d> {
    phantom: PhantomData<&'d ()>,
    info: &'static Info,
    state: &'static State,
    periph_clock: crate::time::Hertz,
}

/// Error returned by `try_write`
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryWriteError {
    /// All transmit mailboxes are full
    Full,
}

impl<'d> Can<'d> {
    /// Creates a new Bxcan instance, keeping the peripheral in sleep mode.
    /// You must call [Can::enable_non_blocking] to use the peripheral.
    pub fn new<T: Instance>(
        _peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irqs: impl interrupt::typelevel::Binding<T::TXInterrupt, TxInterruptHandler<T>>
            + interrupt::typelevel::Binding<T::RX0Interrupt, Rx0InterruptHandler<T>>
            + interrupt::typelevel::Binding<T::RX1Interrupt, Rx1InterruptHandler<T>>
            + interrupt::typelevel::Binding<T::SCEInterrupt, SceInterruptHandler<T>>
            + 'd,
    ) -> Self {
        into_ref!(_peri, rx, tx);
        let info = T::info();
        let regs = &T::info().regs;

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        rcc::enable_and_reset::<T>();

        {
            regs.0.ier().write(|w| {
                w.set_errie(true);
                w.set_fmpie(0, true);
                w.set_fmpie(1, true);
                w.set_tmeie(true);
                w.set_bofie(true);
                w.set_epvie(true);
                w.set_ewgie(true);
                w.set_lecie(true);
            });

            regs.0.mcr().write(|w| {
                // Enable timestamps on rx messages

                w.set_ttcm(true);
            });
        }

        unsafe {
            info.tx_interrupt.unpend();
            info.tx_interrupt.enable();
            info.rx0_interrupt.unpend();
            info.rx0_interrupt.enable();
            info.rx1_interrupt.unpend();
            info.rx1_interrupt.enable();
            info.sce_interrupt.unpend();
            info.sce_interrupt.enable();
        }

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        Registers(T::regs()).leave_init_mode();

        Self {
            phantom: PhantomData,
            info: T::info(),
            state: T::state(),
            periph_clock: T::frequency(),
        }
    }

    /// Set CAN bit rate.
    pub fn set_bitrate(&mut self, bitrate: u32) {
        let bit_timing = util::calc_can_timings(self.periph_clock, bitrate).unwrap();
        self.modify_config().set_bit_timing(bit_timing);
    }

    /// Configure bit timings and silent/loop-back mode.
    ///
    /// Calling this method will enter initialization mode. You must enable the peripheral
    /// again afterwards with [`enable`](Self::enable).
    pub fn modify_config(&mut self) -> CanConfig<'_> {
        self.info.regs.enter_init_mode();

        CanConfig {
            phantom: self.phantom,
            info: self.info,
            periph_clock: self.periph_clock,
        }
    }

    /// Enables the peripheral and synchronizes with the bus.
    ///
    /// This will wait for 11 consecutive recessive bits (bus idle state).
    /// Contrary to enable method from bxcan library, this will not freeze the executor while waiting.
    pub async fn enable(&mut self) {
        while self.info.regs.enable_non_blocking().is_err() {
            // SCE interrupt is only generated for entering sleep mode, but not leaving.
            // Yield to allow other tasks to execute while can bus is initializing.
            embassy_futures::yield_now().await;
        }
    }

    /// Enables or disables the peripheral from automatically wakeup when a SOF is detected on the bus
    /// while the peripheral is in sleep mode
    pub fn set_automatic_wakeup(&mut self, enabled: bool) {
        self.info.regs.set_automatic_wakeup(enabled);
    }

    /// Manually wake the peripheral from sleep mode.
    ///
    /// Waking the peripheral manually does not trigger a wake-up interrupt.
    /// This will wait until the peripheral has acknowledged it has awoken from sleep mode
    pub fn wakeup(&mut self) {
        self.info.regs.wakeup()
    }

    /// Check if the peripheral is currently in sleep mode
    pub fn is_sleeping(&self) -> bool {
        self.info.regs.0.msr().read().slak()
    }

    /// Put the peripheral in sleep mode
    ///
    /// When the peripherial is in sleep mode, messages can still be queued for transmission
    /// and any previously received messages can be read from the receive FIFOs, however
    /// no messages will be transmitted and no additional messages will be received.
    ///
    /// If the peripheral has automatic wakeup enabled, when a Start-of-Frame is detected
    /// the peripheral will automatically wake and receive the incoming message.
    pub async fn sleep(&mut self) {
        self.info.regs.0.ier().modify(|i| i.set_slkie(true));
        self.info.regs.0.mcr().modify(|m| m.set_sleep(true));

        poll_fn(|cx| {
            self.state.err_waker.register(cx.waker());
            if self.is_sleeping() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        self.info.regs.0.ier().modify(|i| i.set_slkie(false));
    }

    /// Enable FIFO scheduling of outgoing frames.
    ///
    /// If this is enabled, frames will be transmitted in the order that they are passed to
    /// [`write()`][Self::write] or [`try_write()`][Self::try_write()].
    ///
    /// If this is disabled, frames are transmitted in order of priority.
    ///
    /// FIFO scheduling is disabled by default.
    pub fn set_tx_fifo_scheduling(&mut self, enabled: bool) {
        self.info.regs.set_tx_fifo_scheduling(enabled)
    }

    /// Checks if FIFO scheduling of outgoing frames is enabled.
    pub fn tx_fifo_scheduling_enabled(&self) -> bool {
        self.info.regs.tx_fifo_scheduling_enabled()
    }

    /// Queues the message to be sent.
    ///
    /// If the TX queue is full, this will wait until there is space, therefore exerting backpressure.
    pub async fn write(&mut self, frame: &Frame) -> TransmitStatus {
        self.split().0.write(frame).await
    }

    /// Attempts to transmit a frame without blocking.
    ///
    /// Returns [Err(TryWriteError::Full)] if the frame can not be queued for transmission now.
    ///
    /// If FIFO scheduling is enabled, any empty mailbox will be used.
    ///
    /// Otherwise, the frame will only be accepted if there is no frame with the same priority already queued.
    /// This is done to work around a hardware limitation that could lead to out-of-order delivery
    /// of frames with the same priority.
    pub fn try_write(&mut self, frame: &Frame) -> Result<TransmitStatus, TryWriteError> {
        self.split().0.try_write(frame)
    }

    /// Waits for a specific transmit mailbox to become empty
    pub async fn flush(&self, mb: Mailbox) {
        CanTx {
            _phantom: PhantomData,
            info: self.info,
            state: self.state,
        }
        .flush_inner(mb)
        .await;
    }

    /// Waits until any of the transmit mailboxes become empty
    ///
    /// Note that [`Self::try_write()`] may fail with [`TryWriteError::Full`],
    /// even after the future returned by this function completes.
    /// This will happen if FIFO scheduling of outgoing frames is not enabled,
    /// and a frame with equal priority is already queued for transmission.
    pub async fn flush_any(&self) {
        CanTx {
            _phantom: PhantomData,
            info: self.info,
            state: self.state,
        }
        .flush_any_inner()
        .await
    }

    /// Waits until all of the transmit mailboxes become empty
    pub async fn flush_all(&self) {
        CanTx {
            _phantom: PhantomData,
            info: self.info,
            state: self.state,
        }
        .flush_all_inner()
        .await
    }

    /// Attempts to abort the sending of a frame that is pending in a mailbox.
    ///
    /// If there is no frame in the provided mailbox, or its transmission succeeds before it can be
    /// aborted, this function has no effect and returns `false`.
    ///
    /// If there is a frame in the provided mailbox, and it is canceled successfully, this function
    /// returns `true`.
    pub fn abort(&mut self, mailbox: Mailbox) -> bool {
        self.info.regs.abort(mailbox)
    }

    /// Returns `true` if no frame is pending for transmission.
    pub fn is_transmitter_idle(&self) -> bool {
        self.info.regs.is_idle()
    }

    /// Read a CAN frame.
    ///
    /// If no CAN frame is in the RX buffer, this will wait until there is one.
    ///
    /// Returns a tuple of the time the message was received and the message frame
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        self.state.rx_mode.read(self.info, self.state).await
    }

    /// Attempts to read a CAN frame without blocking.
    ///
    /// Returns [Err(TryReadError::Empty)] if there are no frames in the rx queue.
    pub fn try_read(&mut self) -> Result<Envelope, TryReadError> {
        self.state.rx_mode.try_read(self.info)
    }

    /// Waits while receive queue is empty.
    pub async fn wait_not_empty(&mut self) {
        self.state.rx_mode.wait_not_empty(self.info, self.state).await
    }

    /// Split the CAN driver into transmit and receive halves.
    ///
    /// Useful for doing separate transmit/receive tasks.
    pub fn split<'c>(&'c mut self) -> (CanTx<'d>, CanRx<'d>) {
        (
            CanTx {
                _phantom: PhantomData,
                info: self.info,
                state: self.state,
            },
            CanRx {
                _phantom: PhantomData,
                info: self.info,
                state: self.state,
            },
        )
    }

    /// Return a buffered instance of driver. User must supply Buffers
    pub fn buffered<'c, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize>(
        &'c mut self,
        txb: &'static mut TxBuf<TX_BUF_SIZE>,
        rxb: &'static mut RxBuf<RX_BUF_SIZE>,
    ) -> BufferedCan<'d, TX_BUF_SIZE, RX_BUF_SIZE> {
        let (tx, rx) = self.split();
        BufferedCan {
            tx: tx.buffered(txb),
            rx: rx.buffered(rxb),
        }
    }
}

impl<'d> Can<'d> {
    /// Accesses the filter banks owned by this CAN peripheral.
    ///
    /// To modify filters of a slave peripheral, `modify_filters` has to be called on the master
    /// peripheral instead.
    pub fn modify_filters(&mut self) -> MasterFilters<'_> {
        unsafe { MasterFilters::new(self.info) }
    }
}

/// Buffered CAN driver.
pub struct BufferedCan<'d, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> {
    tx: BufferedCanTx<'d, TX_BUF_SIZE>,
    rx: BufferedCanRx<'d, RX_BUF_SIZE>,
}

impl<'d, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> BufferedCan<'d, TX_BUF_SIZE, RX_BUF_SIZE> {
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
pub struct CanTx<'d> {
    _phantom: PhantomData<&'d ()>,
    info: &'static Info,
    state: &'static State,
}

impl<'d> CanTx<'d> {
    /// Queues the message to be sent.
    ///
    /// If the TX queue is full, this will wait until there is space, therefore exerting backpressure.
    pub async fn write(&mut self, frame: &Frame) -> TransmitStatus {
        poll_fn(|cx| {
            self.state.tx_mode.register(cx.waker());
            if let Ok(status) = self.info.regs.transmit(frame) {
                return Poll::Ready(status);
            }

            Poll::Pending
        })
        .await
    }

    /// Attempts to transmit a frame without blocking.
    ///
    /// Returns [Err(TryWriteError::Full)] if the frame can not be queued for transmission now.
    ///
    /// If FIFO scheduling is enabled, any empty mailbox will be used.
    ///
    /// Otherwise, the frame will only be accepted if there is no frame with the same priority already queued.
    /// This is done to work around a hardware limitation that could lead to out-of-order delivery
    /// of frames with the same priority.
    pub fn try_write(&mut self, frame: &Frame) -> Result<TransmitStatus, TryWriteError> {
        self.info.regs.transmit(frame).map_err(|_| TryWriteError::Full)
    }

    async fn flush_inner(&self, mb: Mailbox) {
        poll_fn(|cx| {
            self.state.tx_mode.register(cx.waker());
            if self.info.regs.0.tsr().read().tme(mb.index()) {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Waits for a specific transmit mailbox to become empty
    pub async fn flush(&self, mb: Mailbox) {
        self.flush_inner(mb).await
    }

    async fn flush_any_inner(&self) {
        poll_fn(|cx| {
            self.state.tx_mode.register(cx.waker());

            let tsr = self.info.regs.0.tsr().read();
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
    ///
    /// Note that [`Self::try_write()`] may fail with [`TryWriteError::Full`],
    /// even after the future returned by this function completes.
    /// This will happen if FIFO scheduling of outgoing frames is not enabled,
    /// and a frame with equal priority is already queued for transmission.
    pub async fn flush_any(&self) {
        self.flush_any_inner().await
    }

    async fn flush_all_inner(&self) {
        poll_fn(|cx| {
            self.state.tx_mode.register(cx.waker());

            let tsr = self.info.regs.0.tsr().read();
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
        self.flush_all_inner().await
    }

    /// Attempts to abort the sending of a frame that is pending in a mailbox.
    ///
    /// If there is no frame in the provided mailbox, or its transmission succeeds before it can be
    /// aborted, this function has no effect and returns `false`.
    ///
    /// If there is a frame in the provided mailbox, and it is canceled successfully, this function
    /// returns `true`.
    pub fn abort(&mut self, mailbox: Mailbox) -> bool {
        self.info.regs.abort(mailbox)
    }

    /// Returns `true` if no frame is pending for transmission.
    pub fn is_idle(&self) -> bool {
        self.info.regs.is_idle()
    }

    /// Return a buffered instance of driver. User must supply Buffers
    pub fn buffered<const TX_BUF_SIZE: usize>(
        self,
        txb: &'static mut TxBuf<TX_BUF_SIZE>,
    ) -> BufferedCanTx<'d, TX_BUF_SIZE> {
        BufferedCanTx::new(self.info, self.state, self, txb)
    }
}

/// User supplied buffer for TX buffering
pub type TxBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, Frame, BUF_SIZE>;

/// Buffered CAN driver, transmit half.
pub struct BufferedCanTx<'d, const TX_BUF_SIZE: usize> {
    info: &'static Info,
    state: &'static State,
    _tx: CanTx<'d>,
    tx_buf: &'static TxBuf<TX_BUF_SIZE>,
}

impl<'d, const TX_BUF_SIZE: usize> BufferedCanTx<'d, TX_BUF_SIZE> {
    fn new(info: &'static Info, state: &'static State, _tx: CanTx<'d>, tx_buf: &'static TxBuf<TX_BUF_SIZE>) -> Self {
        Self {
            info,
            state,
            _tx,
            tx_buf,
        }
        .setup()
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        critical_section::with(|_| {
            let tx_inner = super::common::ClassicBufferedTxInner {
                tx_receiver: self.tx_buf.receiver().into(),
            };
            let state = self.state as *const State;
            unsafe {
                let mut_state = state as *mut State;
                (*mut_state).tx_mode = TxMode::Buffered(tx_inner);
            }
        });
        self
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: &Frame) {
        self.tx_buf.send(*frame).await;
        let waker = self.info.tx_waker;
        waker(); // Wake for Tx
    }

    /// Returns a sender that can be used for sending CAN frames.
    pub fn writer(&self) -> BufferedCanSender {
        BufferedCanSender {
            tx_buf: self.tx_buf.sender().into(),
            waker: self.info.tx_waker,
        }
    }
}

impl<'d, const TX_BUF_SIZE: usize> Drop for BufferedCanTx<'d, TX_BUF_SIZE> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            let state = self.state as *const State;
            unsafe {
                let mut_state = state as *mut State;
                (*mut_state).tx_mode = TxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
            }
        });
    }
}

/// CAN driver, receive half.
#[allow(dead_code)]
pub struct CanRx<'d> {
    _phantom: PhantomData<&'d ()>,
    info: &'static Info,
    state: &'static State,
}

impl<'d> CanRx<'d> {
    /// Read a CAN frame.
    ///
    /// If no CAN frame is in the RX buffer, this will wait until there is one.
    ///
    /// Returns a tuple of the time the message was received and the message frame
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        self.state.rx_mode.read(self.info, self.state).await
    }

    /// Attempts to read a CAN frame without blocking.
    ///
    /// Returns [Err(TryReadError::Empty)] if there are no frames in the rx queue.
    pub fn try_read(&mut self) -> Result<Envelope, TryReadError> {
        self.state.rx_mode.try_read(self.info)
    }

    /// Waits while receive queue is empty.
    pub async fn wait_not_empty(&mut self) {
        self.state.rx_mode.wait_not_empty(self.info, self.state).await
    }

    /// Return a buffered instance of driver. User must supply Buffers
    pub fn buffered<const RX_BUF_SIZE: usize>(
        self,
        rxb: &'static mut RxBuf<RX_BUF_SIZE>,
    ) -> BufferedCanRx<'d, RX_BUF_SIZE> {
        BufferedCanRx::new(self.info, self.state, self, rxb)
    }
}

/// User supplied buffer for RX Buffering
pub type RxBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, Result<Envelope, BusError>, BUF_SIZE>;

/// CAN driver, receive half in Buffered mode.
pub struct BufferedCanRx<'d, const RX_BUF_SIZE: usize> {
    info: &'static Info,
    state: &'static State,
    _rx: CanRx<'d>,
    rx_buf: &'static RxBuf<RX_BUF_SIZE>,
}

impl<'d, const RX_BUF_SIZE: usize> BufferedCanRx<'d, RX_BUF_SIZE> {
    fn new(info: &'static Info, state: &'static State, _rx: CanRx<'d>, rx_buf: &'static RxBuf<RX_BUF_SIZE>) -> Self {
        BufferedCanRx {
            info,
            state,
            _rx,
            rx_buf,
        }
        .setup()
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        critical_section::with(|_| {
            let rx_inner = super::common::ClassicBufferedRxInner {
                rx_sender: self.rx_buf.sender().into(),
            };
            let state = self.state as *const State;
            unsafe {
                let mut_state = state as *mut State;
                (*mut_state).rx_mode = RxMode::Buffered(rx_inner);
            }
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
        match &self.state.rx_mode {
            RxMode::Buffered(_) => {
                if let Ok(result) = self.rx_buf.try_receive() {
                    match result {
                        Ok(envelope) => Ok(envelope),
                        Err(e) => Err(TryReadError::BusError(e)),
                    }
                } else {
                    if let Some(err) = self.info.regs.curr_error() {
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

impl<'d, const RX_BUF_SIZE: usize> Drop for BufferedCanRx<'d, RX_BUF_SIZE> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            let state = self.state as *const State;
            unsafe {
                let mut_state = state as *mut State;
                (*mut_state).rx_mode = RxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
            }
        });
    }
}

impl Drop for Can<'_> {
    fn drop(&mut self) {
        // Cannot call `free()` because it moves the instance.
        // Manually reset the peripheral.
        self.info.regs.0.mcr().write(|w| w.set_reset(true));
        self.info.regs.enter_init_mode();
        self.info.regs.leave_init_mode();
        //rcc::disable::<T>();
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

    pub(crate) async fn read(&self, info: &Info, state: &State) -> Result<Envelope, BusError> {
        match self {
            Self::NonBuffered(waker) => {
                poll_fn(|cx| {
                    state.err_waker.register(cx.waker());
                    waker.register(cx.waker());
                    match self.try_read(info) {
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
    pub(crate) fn try_read(&self, info: &Info) -> Result<Envelope, TryReadError> {
        match self {
            Self::NonBuffered(_) => {
                let registers = &info.regs;
                if let Some(msg) = registers.receive_fifo(RxFifo::Fifo0) {
                    registers.0.ier().write(|w| {
                        w.set_fmpie(0, true);
                    });
                    Ok(msg)
                } else if let Some(msg) = registers.receive_fifo(RxFifo::Fifo1) {
                    registers.0.ier().write(|w| {
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
    pub(crate) async fn wait_not_empty(&self, info: &Info, state: &State) {
        match &state.rx_mode {
            Self::NonBuffered(waker) => {
                poll_fn(|cx| {
                    waker.register(cx.waker());
                    if info.regs.receive_frame_available() {
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

pub(crate) enum TxMode {
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

pub(crate) struct State {
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

pub(crate) struct Info {
    regs: Registers,
    tx_interrupt: crate::interrupt::Interrupt,
    rx0_interrupt: crate::interrupt::Interrupt,
    rx1_interrupt: crate::interrupt::Interrupt,
    sce_interrupt: crate::interrupt::Interrupt,
    tx_waker: fn(),

    /// The total number of filter banks available to the instance.
    ///
    /// This is usually either 14 or 28, and should be specified in the chip's reference manual or datasheet.
    num_filter_banks: u8,
}

trait SealedInstance {
    fn info() -> &'static Info;
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

            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: Registers(crate::pac::$inst),
                    tx_interrupt: crate::_generated::peripheral_interrupts::$inst::TX::IRQ,
                    rx0_interrupt: crate::_generated::peripheral_interrupts::$inst::RX0::IRQ,
                    rx1_interrupt: crate::_generated::peripheral_interrupts::$inst::RX1::IRQ,
                    sce_interrupt: crate::_generated::peripheral_interrupts::$inst::SCE::IRQ,
                    tx_waker: crate::_generated::peripheral_interrupts::$inst::TX::pend,
                    num_filter_banks: peripherals::$inst::NUM_FILTER_BANKS,
                };
                &INFO
            }
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
                any(stm32l4, stm32f72x, stm32f73x),
                not(any(stm32l49x, stm32l4ax))
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
    (can, CAN2) => {
        unsafe impl FilterOwner for peripherals::CAN2 {
            const NUM_FILTER_BANKS: u8 = 0;
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
