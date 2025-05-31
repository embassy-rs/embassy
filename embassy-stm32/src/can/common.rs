use embassy_sync::channel::{SendDynamicReceiver, SendDynamicSender};

use super::enums::*;
use super::frame::*;

pub(crate) struct ClassicBufferedRxInner {
    pub rx_sender: SendDynamicSender<'static, Result<Envelope, BusError>>,
}
pub(crate) struct ClassicBufferedTxInner {
    pub tx_receiver: SendDynamicReceiver<'static, Frame>,
}

#[cfg(any(can_fdcan_v1, can_fdcan_h7))]

pub(crate) struct FdBufferedRxInner {
    pub rx_sender: SendDynamicSender<'static, Result<FdEnvelope, BusError>>,
}

#[cfg(any(can_fdcan_v1, can_fdcan_h7))]
pub(crate) struct FdBufferedTxInner {
    pub tx_receiver: SendDynamicReceiver<'static, FdFrame>,
}

/// Sender that can be used for sending CAN frames.
pub struct BufferedSender<'ch, FRAME> {
    pub(crate) tx_buf: embassy_sync::channel::SendDynamicSender<'ch, FRAME>,
    pub(crate) waker: fn(),
    pub(crate) lifetime: TransmitterLifetime,
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
        Self {
            tx_buf: self.tx_buf,
            waker: self.waker,
            lifetime: self.lifetime.clone(),
        }
    }
}

/// Sender that can be used for sending Classic CAN frames.
pub type BufferedCanSender = BufferedSender<'static, Frame>;

/// Receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
pub struct BufferedReceiver<'ch, ENVELOPE> {
    pub(crate) rx_buf: embassy_sync::channel::SendDynamicReceiver<'ch, Result<ENVELOPE, BusError>>,
    pub(crate) lifetime: ReceiverLifetime,
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
        Self {
            rx_buf: self.rx_buf,
            lifetime: self.lifetime.clone(),
        }
    }
}

/// A BufferedCanReceiver for Classic CAN frames.
pub type BufferedCanReceiver = BufferedReceiver<'static, Envelope>;

pub(crate) trait UserInstanceType {
    fn register(internal_operation: fn(InternalOperation));
    fn deregister(internal_operation: fn(InternalOperation));
}

pub(crate) struct UserInstanceReceiver {}
impl UserInstanceType for UserInstanceReceiver {
    fn register(internal_operation: fn(InternalOperation)) {
        internal_operation(InternalOperation::NotifyReceiverCreated);
    }

    fn deregister(internal_operation: fn(InternalOperation)) {
        internal_operation(InternalOperation::NotifyReceiverDestroyed);
    }
}

pub(crate) struct UserInstanceTransmitter {}
impl UserInstanceType for UserInstanceTransmitter {
    fn register(internal_operation: fn(InternalOperation)) {
        internal_operation(InternalOperation::NotifySenderCreated);
    }

    fn deregister(internal_operation: fn(InternalOperation)) {
        internal_operation(InternalOperation::NotifySenderDestroyed);
    }
}
pub(crate) struct UserInstanceTransceiver {}
impl UserInstanceType for UserInstanceTransceiver {
    fn register(internal_operation: fn(InternalOperation)) {
        internal_operation(InternalOperation::NotifySenderCreated);
        internal_operation(InternalOperation::NotifyReceiverCreated);
    }

    fn deregister(internal_operation: fn(InternalOperation)) {
        internal_operation(InternalOperation::NotifySenderDestroyed);
        internal_operation(InternalOperation::NotifyReceiverDestroyed);
    }
}

pub(crate) struct Lifetime<T: UserInstanceType> {
    pub(crate) internal_operation: fn(InternalOperation),
    _marker: core::marker::PhantomData<T>,
}

impl<T: UserInstanceType> Lifetime<T> {
    pub(crate) fn new(internal_operation: fn(InternalOperation)) -> Self {
        T::register(internal_operation);
        Self {
            internal_operation,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<T: UserInstanceType> Clone for Lifetime<T> {
    fn clone(&self) -> Self {
        T::register(self.internal_operation);
        Self {
            internal_operation: self.internal_operation,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<T> Drop for Lifetime<T>
where
    T: UserInstanceType,
{
    fn drop(&mut self) {
        T::deregister(self.internal_operation);
    }
}

pub(crate) type ReceiverLifetime = Lifetime<UserInstanceReceiver>;
pub(crate) type TransmitterLifetime = Lifetime<UserInstanceTransmitter>;
