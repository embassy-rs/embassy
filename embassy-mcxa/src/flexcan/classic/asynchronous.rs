//! This module holds everything that is specific to the async flavor of the driver.
//!
//! For async example code, see the docs at the top of the `classic` module.

use core::marker::PhantomData;
use core::sync::atomic::Ordering;

use embassy_hal_internal::Peri;
use embassy_sync::channel::{SendDynamicReceiver, SendDynamicSender};

use super::mailbox::tx;
use super::{
    BusErrorMode, Cell, Channel, CriticalSectionRawMutex, FlexCan, FlexCanConfig, FlexCanRx, FlexCanTx, InitError,
    Instance, Mode, ReceiveError, SendError, mailbox, sealed,
};
use crate::flexcan::classic::frame::Frame;
use crate::flexcan::{RxPin, TxPin};
use crate::interrupt::typelevel::{Handler, Interrupt};

/// A software queue for holding received CAN frames.
///
/// `N` is the number of Frames you want the `RxQueue` to hold.
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

/// Async driver mode. Use `FlexCan::new_async()` to construct a driver in
/// this mode.
///
/// This driver mode uses interrupts and provides `async` functions for
/// interacting with FlexCAN.
///
/// For async example code, see the docs at the top of the `classic` module.
#[derive(Clone, Copy)]
pub struct Async {
    /// Receiver for the queue the user provides.
    rx_receiver: SendDynamicReceiver<'static, Frame>,
}

impl sealed::Sealed for Async {}
impl Mode for Async {}

/// Functions for `FlexCan` that are specific to `Async` mode.
impl<'d> FlexCan<'d, Async> {
    /// Constructs a new async FlexCAN driver instance, in Classic mode.
    ///
    /// You must also route this instance's interrupt to an `InterruptHandler` via `bind_interrupts!`,
    /// and provide a `'static` `RxQueue` for received frames to land in.
    ///
    /// For async example code, see the docs at the top of the `classic` module.
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
        info.rx_sender
            .lock(|c: &Cell<Option<SendDynamicSender<'static, Frame>>>| c.set(Some(rx_sender)));

        // Setup the interrupts
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        info.control.unfreeze();

        let mode = Async { rx_receiver };
        let tx = FlexCanTx::new(info, tx_pin, wake_guard.clone(), mode);
        let rx = FlexCanRx::new(info, rx_pin, wake_guard, mode);
        Ok(Self { tx, rx })
    }

    /// Sends a CAN message.
    ///
    /// If there's no space left in the TX buffers, this call asynchronously waits for space to free up, and then tries again.
    ///
    /// Note: During a BusOff event, this function will asynchronously wait until
    /// the bus recovers. This is due to the behavior mentioned above: The TX mailbox
    /// doesn't drain during BusOff (and will eventually fill up), causing this
    /// function to wait until after recovery when buffers start becoming available again.
    ///
    /// Unless explicitly disabled, FlexCAN will recover from BusOff automatically. However
    /// if you need to be notified immediately when a BusOff event occurs, see the `try_send()`
    /// and `error_mode()` functions.
    pub async fn send(&mut self, frame: &Frame) {
        self.tx.send(frame).await
    }

    /// Attempts to send a CAN message.
    ///
    /// This function returns immediately upon being called, either with `Ok(())` or
    /// a `SendError`. For this function's async counterpart, see `send()`.
    pub fn try_send(&mut self, frame: &Frame) -> Result<(), SendError> {
        self.tx.try_send(frame)
    }

    /// Receives a CAN message.
    ///
    /// If there are no new messages, this call asynchronously
    /// waits for new messages to arrive.
    pub async fn receive(&self) -> Frame {
        self.rx.receive().await
    }

    /// Like `receive()`, but returns immediately if there are no new messages (rather than waiting for more to arrive).
    pub fn try_receive(&self) -> Result<Frame, ReceiveError> {
        self.rx.try_receive()
    }

    /// Indicates the number of RX frames dropped so far due to either the software RX `RxQueue` or the FlexCAN hardware FIFO.
    ///
    /// If you're seeing this number increase, you are receiving messages faster than the RX queue can handle.
    /// This may be mitigated by increasing the size of the `RxQueue`.
    ///
    /// Note: As mentioned, this only tracks frames that were dropped from the FlexCAN hardware RX FIFO or from the software `RxQueue`, due to either
    /// being full. Any frame drops that happen at a lower level, or during transmission, are not tracked here.
    pub fn rx_dropped_count(&self) -> RxDroppedCount {
        self.rx.rx_dropped_count()
    }
}

/// Stores information tracking how many RX messages have been dropped thus far, if any.
///
/// Note: This struct only tracks frames that were dropped from the FlexCAN hardware RX FIFO and from this
/// driver's software `RxQueue`, due to either being full. Any frame drops that happened at a lower level, or during
/// transmission, are not tracked here.
pub struct RxDroppedCount {
    /// Tracks the number of frames dropped from the software `RxQueue` used by this driver.
    pub(in crate::flexcan::classic::asynchronous) software_channel: u32,
    /// Tracks the number of frames dropped from the FlexCAN peripheral's hardware RX FIFO.
    pub(in crate::flexcan::classic::asynchronous) hardware_fifo: u32,
}
impl RxDroppedCount {
    /// Indicates the number of RX frames dropped from the software `RxQueue` used by this driver.
    ///
    /// For context, a RX frame will get dropped from the software `RxQueue` when the queue is full. If this increases,
    /// you are likely receiving messages at a higher rate than you are dequeuing/processing them.
    pub fn software_channel(&self) -> u32 {
        self.software_channel
    }

    /// Indicates the number of overflow events experienced by the FlexCAN hardware RX FIFO.
    ///
    /// For context, an overflow event occurs when the FlexCAN recieves a message, but there is no room left in the hardware RX FIFO to store it. If this increases,
    /// you are likely receiving messages at a higher rate than you are dequeuing/processing them.
    ///
    /// Note: The "number of overflow events in the hardware FIFO" is not exactly the same thing as "the number of RX frames dropped by the hardware FIFO". Technically,
    /// a single overflow event (i.e., a single increment to this counter) could indicate more than one frame being dropped if, after a first frame is dropped, a second frame is
    /// dropped before the ISR can increment this counter for the first frame. This is unlikely to happen in practice under normal interrupt latency though, so for most purposes
    /// this counter can probably be intepereted as "the number of RX frames dropped by the hardware FIFO".
    ///
    pub fn hardware_fifo(&self) -> u32 {
        self.hardware_fifo
    }

    /// Indicates the total number of RX frames dropped from the software `RxQueue`, and the number of overflow events that have occured for the FlexCAN hardware RX FIFO.
    ///
    /// Note: This is analagous to `software_channel() + hardware_fifo()`.
    pub fn total(&self) -> u32 {
        self.software_channel + self.hardware_fifo
    }
}

/// Functions for `FlexCanTx` that are specific to `Async` mode.
impl<'d> FlexCanTx<'d, Async> {
    /// Sends a CAN message.
    ///
    /// If there's no space left in the TX buffers, this call asynchronously waits for space to free up, and then tries again.
    ///
    /// Note: During a BusOff event, this function will asynchronously wait until
    /// the bus recovers. This is due to the behavior mentioned above: The TX mailbox
    /// doesn't drain during BusOff (and will eventually fill up), causing this
    /// function to wait until after recovery when buffers start becoming available again.
    ///
    /// Unless explicitly disabled, FlexCAN will recover from BusOff automatically. However
    /// if you need to be notified immediately when a BusOff event occurs, see the `try_send()`
    /// and `error_mode()` functions.
    pub async fn send(&mut self, frame: &Frame) {
        use nb::Error::{Other, WouldBlock};

        let info = self.info;
        let message = tx::TxMessage::from(frame);
        let _ = info
            .tx_waker
            .wait_for(|| match tx::dispatch(self.info, &message) {
                Ok(()) => true,
                Err(WouldBlock) => {
                    self.info.tx_mailbox_full_count.fetch_add(1, Ordering::Relaxed);
                    false
                }
                Err(Other(e)) => match e {},
            })
            .await;
    }

    /// Attempts to send a CAN message.
    ///
    /// This function returns immediately upon being called, either with `Ok(())` or
    /// a `SendError`. For this function's async counterpart, see `send()`.
    pub fn try_send(&mut self, frame: &Frame) -> Result<(), SendError> {
        use nb::Error::{Other, WouldBlock};

        if self.error_mode() == BusErrorMode::BusOff {
            return Err(SendError::BusOff);
        }

        let message = tx::TxMessage::from(frame);
        match tx::dispatch(self.info, &message) {
            Ok(()) => Ok(()),
            Err(WouldBlock) => {
                self.info.tx_mailbox_full_count.fetch_add(1, Ordering::Relaxed);
                Err(SendError::TxMailboxFull)
            }
            Err(Other(e)) => match e {},
        }
    }
}

/// Functions for `FlexCanRx` that are specific to `Async` mode.
impl<'d> FlexCanRx<'d, Async> {
    /// Receives a CAN message.
    ///
    /// If there are no new messages, this call asynchronously
    /// waits for new messages to arrive.
    pub async fn receive(&self) -> Frame {
        self.mode.rx_receiver.receive().await
    }

    /// Like `receive()`, but returns immediately if there are no new messages (rather than waiting for more to arrive).
    pub fn try_receive(&self) -> Result<Frame, ReceiveError> {
        self.mode
            .rx_receiver
            .try_receive()
            .map_err(|_| ReceiveError::NoMessages)
    }

    /// Indicates the number of RX frames dropped so far due to either the software RX `RxQueue` or the FlexCAN hardware FIFO.
    ///
    /// If you're seeing this number increase, you are receiving messages faster than the RX queue can handle.
    /// This may be mitigated by increasing the size of the `RxQueue`.
    ///
    /// Note: As mentioned, this only tracks frames that were dropped from the FlexCAN hardware RX FIFO or from the software `RxQueue`, due to either
    /// being full. Any frame drops that happen at a lower level, or during transmission, are not tracked here.
    pub fn rx_dropped_count(&self) -> RxDroppedCount {
        RxDroppedCount {
            software_channel: self.info.rx_dropped_count_channel.load(Ordering::Relaxed),
            hardware_fifo: self.info.rx_dropped_count_fifo.load(Ordering::Relaxed),
        }
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
        let can = info.control.regs();

        // Check if this interrupt was triggered due to a hardware RX fifo overflow
        // putting this first instead of in RX STUFF since it is good to clear these W1C flags ASAP in case
        // the flag gets triggered again while we're still in the ISR
        if can.erfsr().read().erfovf() {
            can.erfsr().write(|w| w.set_erfovf(true));
            info.rx_dropped_count_fifo.fetch_add(1, Ordering::Relaxed);
        }

        /* TX STUFF: */

        // Reclaim any completed TX buffers. If any were reclaimed, wake tasks waiting in send().
        if mailbox::tx::reclaim_completed(info) {
            info.tx_waker.wake(); // Tell sleepers that there's an available TX buffer now
        }

        /* RX STUFF: */

        // Check if any RX messages can be dequeued, and if so, dequeue them.
        let rx_sender: Option<SendDynamicSender<'static, Frame>> = info
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
                // if the software queue is full, drop the frame, and increment the `rx_dropped_count_channel` counter.
                info.rx_dropped_count_channel.fetch_add(1, Ordering::Relaxed);
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
