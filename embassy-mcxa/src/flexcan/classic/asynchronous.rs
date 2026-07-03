//! This module holds everything that is specific to the async flavor of the driver.

use core::marker::PhantomData;
use core::sync::atomic::Ordering;

use embassy_hal_internal::Peri;
use embassy_sync::channel::{SendDynamicReceiver, SendDynamicSender};

use super::mailbox::tx;
use super::{
    AtomicU32, BusErrorMode, Cell, Channel, CriticalSectionRawMutex, FlexCan, FlexCanConfig, FlexCanRx, FlexCanTx,
    InitError, Instance, Mode, Mutex, ReceiveError, SendError, WaitCell, mailbox, sealed,
};
use crate::flexcan::classic::frame::Frame;
use crate::flexcan::{RxPin, TxPin};
use crate::interrupt::typelevel::{Handler, Interrupt};

/// Async-only state for a single `classic::FlexCan` instance.
pub(crate) struct AsyncState {
    /// Waker used to wake tasks awaiting on a CAN send() call.
    pub tx_waker: WaitCell,

    /// Handle to the RX queue's sender.
    pub rx_sender: Mutex<CriticalSectionRawMutex, Cell<Option<SendDynamicSender<'static, Frame>>>>,

    /// Stores a count of the number of RX frames dropped so far due to the RX Channel being full.
    pub rx_dropped_count: AtomicU32,
}

/// A software queue for holding received CAN frames.
pub struct RxQueue<const N: usize>(Channel<CriticalSectionRawMutex, Frame, N>);

impl<const N: usize> RxQueue<N> {
    /// Creates a new `RxQueue`.
    pub const fn new() -> Self {
        Self(Channel::new())
    }
}

impl<const N: usize> Default for RxQueue<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Async driver mode.
///
/// This driver mode uses interrupts and provides `async` functions for
/// interacting with FlexCAN.
#[derive(Clone, Copy)]
pub struct Async {
    /// Async-specific state stuff.
    state: &'static AsyncState,

    /// Receiver for the queue the user provides.
    rx_receiver: SendDynamicReceiver<'static, Frame>,
}

impl sealed::Sealed for Async {}
impl Mode for Async {}

impl<'d> FlexCan<'d, Async> {
    /// Constructs a new async FlexCAN driver instance, in Classic mode.
    ///
    /// You must also route this instance's interrupt to an `InterruptHandler` via `bind_interrupts!`,
    /// and provide a `'static` `RxQueue` for received frames to land in.
    pub fn new_async<T: Instance, const N: usize>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        rx_queue: &'static mut RxQueue<N>,
        config: FlexCanConfig<'_>,
    ) -> Result<Self, InitError> {
        let (info, rx_pin, tx_pin, wake_guard) = super::init::<T>(peri, rx, tx, &config)?;

        // Take ownership of the user's RX queue.
        let rx_queue: &'static RxQueue<N> = rx_queue;
        let rx_sender: SendDynamicSender<'static, Frame> = rx_queue.0.sender().into();
        let rx_receiver: SendDynamicReceiver<'static, Frame> = rx_queue.0.receiver().into();
        let state = T::async_state();
        state
            .rx_sender
            .lock(|c: &Cell<Option<SendDynamicSender<'static, Frame>>>| c.set(Some(rx_sender)));

        // Setup the interrupts
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        info.control.unfreeze();

        let mode = Async { state, rx_receiver };
        let tx = FlexCanTx::new(info, tx_pin, wake_guard.clone(), mode);
        let rx = FlexCanRx::new(info, rx_pin, wake_guard, mode);
        Ok(Self { tx, rx })
    }

    #[doc = docs::doc_send!()]
    pub async fn send(&mut self, frame: &Frame) {
        self.tx.send(frame).await
    }
    #[doc = docs::doc_try_send!()]
    pub fn try_send(&mut self, frame: &Frame) -> Result<(), SendError> {
        self.tx.try_send(frame)
    }
    #[doc = docs::doc_receive!()]
    pub async fn receive(&self) -> Frame {
        self.rx.receive().await
    }
    #[doc = docs::doc_try_receive!()]
    pub fn try_receive(&self) -> Result<Frame, ReceiveError> {
        self.rx.try_receive()
    }
    #[doc = docs::doc_rx_dropped_count!()]
    pub fn rx_dropped_count(&self) -> u32 {
        self.rx.rx_dropped_count()
    }
}

impl<'d> FlexCanTx<'d, Async> {
    #[doc = docs::doc_send!()]
    pub async fn send(&mut self, frame: &Frame) {
        use nb::Error::{Other, WouldBlock};

        let state = self.mode.state;
        let message = tx::TxMessage::from(frame);
        let _ = state
            .tx_waker
            .wait_for(|| match tx::dispatch(self.info, &message) {
                Ok(()) => true,
                Err(WouldBlock) => {
                    self.info.tx_mailbox_full_count.fetch_add(1, Ordering::Acquire);
                    false
                }
                Err(Other(e)) => match e {},
            })
            .await;
    }
    #[doc = docs::doc_try_send!()]
    pub fn try_send(&mut self, frame: &Frame) -> Result<(), SendError> {
        use nb::Error::{Other, WouldBlock};

        if self.error_mode() == BusErrorMode::BusOff {
            return Err(SendError::BusOff);
        }

        let message = tx::TxMessage::from(frame);
        match tx::dispatch(self.info, &message) {
            Ok(()) => Ok(()),
            Err(WouldBlock) => {
                self.info.tx_mailbox_full_count.fetch_add(1, Ordering::Acquire);
                Err(SendError::TxMailboxFull)
            }
            Err(Other(e)) => match e {},
        }
    }
}

impl<'d> FlexCanRx<'d, Async> {
    #[doc = docs::doc_receive!()]
    pub async fn receive(&self) -> Frame {
        self.mode.rx_receiver.receive().await
    }
    #[doc = docs::doc_try_receive!()]
    pub fn try_receive(&self) -> Result<Frame, ReceiveError> {
        self.mode
            .rx_receiver
            .try_receive()
            .map_err(|_| ReceiveError::NoMessages)
    }
    #[doc = docs::doc_rx_dropped_count!()]
    pub fn rx_dropped_count(&self) -> u32 {
        self.mode.state.rx_dropped_count.load(Ordering::Acquire)
    }
}

/// FlexCAN interrupt handler.
/// Construct this in a `bind_interrupts!` block to route an IRQ (e.g., CAN0, CAN1) here.
///
/// Note: This is only required (and only relevant) for the async driver.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        let async_state = T::async_state();
        let can = info.control.regs();

        /* TX STUFF: */

        // Reclaim any completed TX buffers. If any were reclaimed, wake tasks waiting in send().
        if mailbox::tx::reclaim_completed(info) {
            async_state.tx_waker.wake(); // Tell sleepers that there's an available TX buffer now
        }

        /* RX STUFF: */

        // Check if any RX messages can be dequeued, and if so, dequeue them.
        let rx_sender: Option<SendDynamicSender<'static, Frame>> = async_state
            .rx_sender
            .lock(|c: &Cell<Option<SendDynamicSender<'static, Frame>>>| c.get());
        while let Some(message) = mailbox::rx::fifo::get(info) {
            // Dequeue a frame from the hardware RX FIFO
            let frame: Frame = match message.try_into() {
                Ok(message) => message,

                // The try_into() shouldn't actually be able to fail since the PAC already ensures std()/ext() can't
                // exceed 11 bits/29 bits, but if it does somehow, just drop the frame.
                Err(_) => {
                    continue;
                }
            };

            // Push the frame into the software RX queue.
            let dropped = match rx_sender {
                Some::<SendDynamicSender<'static, Frame>>(sender) => sender.try_send(frame).is_err(),
                None => true,
            };
            if dropped {
                // if the software queue is full, drop the frame, and increment the `rx_dropped_count` counter.
                async_state.rx_dropped_count.fetch_add(1, Ordering::Acquire);
            }
        }

        /* BUSOFF STUFF: */
        let esr1 = can.esr1().read();

        // Handle when BusOff has triggered
        if esr1.boffint() {
            // Acknowledge the flag (write 1 to clear)
            can.esr1().write(|w| w.set_boffint(true));
            let _ = can.esr1().read(); // make sure the clear lands before returning
        }

        // Handle when BusOff autorecovery has finished
        if esr1.boffdoneint() == crate::pac::can::Boffdoneint::BusOffDone {
            // Acknowledge the flag (write 1 to clear)
            can.esr1()
                .write(|w| w.set_boffdoneint(crate::pac::can::Boffdoneint::BusOffDone));
            let _ = can.esr1().read(); // Make surethe clear lands before returning
        }
    }
}

/// Shared rustdoc for the async public TX/RX methods. These methods are exposed (with
/// identical documentation) on `FlexCan`, `FlexCanTx`, and `FlexCanRx`, so their doc comments
/// are defined once here and applied via `#[doc = ...]`.
pub(in crate::flexcan::classic) mod docs {
    macro_rules! doc_send {
        () => {
            concat!(
                "Sends a CAN message.\n",
                "\n",
                "If there's no space left in the TX buffers, this\n",
                "call asynchronously waits for space to free up, and then tries again.\n",
                "\n",
                "Note: During a BusOff event, this function will asynchronously wait until\n",
                "the bus recovers. This is due to the behavior mentioned above: The TX mailbox\n",
                "doesn't drain during BusOff (and will eventually fill up), causing this\n",
                "function to wait until after recovery when buffers start becoming available again.\n",
                "\n",
                "Unless explicitly disabled, FlexCAN will recover from BusOff automatically. However,\n",
                "if you need to be notified immediately when a BusOff event occurs, see the `try_send()`\n",
                "and `error_mode()` functions.",
            )
        };
    }
    pub(in crate::flexcan::classic) use doc_send;

    macro_rules! doc_try_send {
        () => {
            concat!(
                "Attempts to send a CAN message.\n",
                "\n",
                "This function returns immediately upon being called, either with `Ok(())` or\n",
                "a `SendError`. For this function's async counterpart, see `send()`.",
            )
        };
    }
    pub(in crate::flexcan::classic) use doc_try_send;

    macro_rules! doc_receive {
        () => {
            concat!(
                "Receives a CAN message.\n",
                "\n",
                "If there are no new messages, this call asynchronously\n",
                "waits for new messages to arrive.\n",
            )
        };
    }
    pub(in crate::flexcan::classic) use doc_receive;

    macro_rules! doc_try_receive {
        () => { concat!(
            "Like `receive()`, but returns immediately if there are no new messages (rather than waiting for more to arrive).",
        ) };
    }
    pub(in crate::flexcan::classic) use doc_try_receive;

    macro_rules! doc_rx_dropped_count {
        () => { concat!(
            "Indicates the number of RX frames dropped so far due to the RX queue being full.",
            "If you're seeing this number increase, you are receiving messages faster than the RX queue can handle.",
            "This can be mitigated by increasing the size of the RX queue.\n",
            "\nNote: This function tracks frames dropped specifically due to the RX queue being full. It doesn't track other
            sources of dropped frames that may have occured at a lower level.\n",
        ) };
    }
    pub(in crate::flexcan::classic) use doc_rx_dropped_count;
}
