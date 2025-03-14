use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use super::enums::*;
use super::frame::*;

pub(crate) struct ClassicBufferedRxInner {
    pub rx_sender: DynamicSender<'static, Result<Envelope, BusError>>,
}
pub(crate) struct ClassicBufferedTxInner {
    pub tx_receiver: DynamicReceiver<'static, Frame>,
}

#[cfg(any(can_fdcan_v1, can_fdcan_h7))]

pub(crate) struct FdBufferedRxInner {
    pub rx_sender: DynamicSender<'static, Result<FdEnvelope, BusError>>,
}

#[cfg(any(can_fdcan_v1, can_fdcan_h7))]
pub(crate) struct FdBufferedTxInner {
    pub tx_receiver: DynamicReceiver<'static, FdFrame>,
}

/// Sender that can be used for sending CAN frames.
pub struct BufferedSender<'ch, FRAME> {
    pub(crate) tx_buf: embassy_sync::channel::DynamicSender<'ch, FRAME>,
    pub(crate) waker: fn(),
    pub(crate) internal_operation: fn(InternalOperation),
}

impl<'ch, FRAME> BufferedSender<'ch, FRAME> {
    /// Async write frame to TX buffer.
    pub fn try_write(&mut self, frame: FRAME) -> Result<(), embassy_sync::channel::TrySendError<FRAME>> {
        self.tx_buf.try_send(frame)?;
        (self.waker)();
        Ok(())
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: FRAME) {
        self.tx_buf.send(frame).await;
        (self.waker)();
    }

    /// Allows a poll_fn to poll until the channel is ready to write
    pub fn poll_ready_to_send(&self, cx: &mut core::task::Context<'_>) -> core::task::Poll<()> {
        self.tx_buf.poll_ready_to_send(cx)
    }
}

impl<'ch, FRAME> Clone for BufferedSender<'ch, FRAME> {
    fn clone(&self) -> Self {
        (self.internal_operation)(InternalOperation::NotifySenderCreated);
        Self {
            tx_buf: self.tx_buf,
            waker: self.waker,
            internal_operation: self.internal_operation,
        }
    }
}

impl<'ch, FRAME> Drop for BufferedSender<'ch, FRAME> {
    fn drop(&mut self) {
        (self.internal_operation)(InternalOperation::NotifySenderDestroyed);
    }
}

/// Sender that can be used for sending Classic CAN frames.
pub type BufferedCanSender = BufferedSender<'static, Frame>;

/// Receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
pub struct BufferedReceiver<'ch, ENVELOPE> {
    pub(crate) rx_buf: embassy_sync::channel::DynamicReceiver<'ch, Result<ENVELOPE, BusError>>,
    pub(crate) internal_operation: fn(InternalOperation),
}

impl<'ch, ENVELOPE> BufferedReceiver<'ch, ENVELOPE> {
    /// Receive the next frame.
    ///
    /// See [`Channel::receive()`].
    pub fn receive(&self) -> embassy_sync::channel::DynamicReceiveFuture<'_, Result<ENVELOPE, BusError>> {
        self.rx_buf.receive()
    }

    /// Attempt to immediately receive the next frame.
    ///
    /// See [`Channel::try_receive()`]
    pub fn try_receive(&self) -> Result<Result<ENVELOPE, BusError>, embassy_sync::channel::TryReceiveError> {
        self.rx_buf.try_receive()
    }

    /// Allows a poll_fn to poll until the channel is ready to receive
    ///
    /// See [`Channel::poll_ready_to_receive()`]
    pub fn poll_ready_to_receive(&self, cx: &mut core::task::Context<'_>) -> core::task::Poll<()> {
        self.rx_buf.poll_ready_to_receive(cx)
    }

    /// Poll the channel for the next frame
    ///
    /// See [`Channel::poll_receive()`]
    pub fn poll_receive(&self, cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<ENVELOPE, BusError>> {
        self.rx_buf.poll_receive(cx)
    }
}

impl<'ch, ENVELOPE> Clone for BufferedReceiver<'ch, ENVELOPE> {
    fn clone(&self) -> Self {
        (self.internal_operation)(InternalOperation::NotifyReceiverCreated);
        Self {
            rx_buf: self.rx_buf,
            internal_operation: self.internal_operation,
        }
    }
}

impl<'ch, ENVELOPE> Drop for BufferedReceiver<'ch, ENVELOPE> {
    fn drop(&mut self) {
        (self.internal_operation)(InternalOperation::NotifyReceiverDestroyed);
    }
}

/// A BufferedCanReceiver for Classic CAN frames.
pub type BufferedCanReceiver = BufferedReceiver<'static, Envelope>;
