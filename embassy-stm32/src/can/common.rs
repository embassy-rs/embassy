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
    pub(crate) info: TxInfoRef,
}

impl<'ch, FRAME> BufferedSender<'ch, FRAME> {
    /// Async write frame to TX buffer.
    pub fn try_write(&mut self, frame: FRAME) -> Result<(), embassy_sync::channel::TrySendError<FRAME>> {
        self.tx_buf.try_send(frame)?;
        (self.info.tx_waker)();
        Ok(())
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: FRAME) {
        self.tx_buf.send(frame).await;
        (self.info.tx_waker)();
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
            info: TxInfoRef::new(&self.info),
        }
    }
}

/// Sender that can be used for sending Classic CAN frames.
pub type BufferedCanSender = BufferedSender<'static, Frame>;

/// Receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
pub struct BufferedReceiver<'ch, ENVELOPE> {
    pub(crate) rx_buf: embassy_sync::channel::SendDynamicReceiver<'ch, Result<ENVELOPE, BusError>>,
    pub(crate) info: RxInfoRef,
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
            info: RxInfoRef::new(&self.info),
        }
    }
}

/// A BufferedCanReceiver for Classic CAN frames.
pub type BufferedCanReceiver = BufferedReceiver<'static, Envelope>;

/// Provides a reference to the driver internals and implements RAII for the internal reference
/// counting. Each type that can operate on the driver should contain either InfoRef
/// or the similar TxInfoRef or RxInfoRef. The new method and the Drop impl will automatically
/// call the reference counting function. Like this, the reference counting function does not
/// need to be called manually for each type.
pub(crate) struct InfoRef {
    info: &'static super::Info,
}
impl InfoRef {
    pub(crate) fn new(info: &'static super::Info) -> Self {
        info.adjust_reference_counter(RefCountOp::NotifyReceiverCreated);
        info.adjust_reference_counter(RefCountOp::NotifySenderCreated);
        Self { info }
    }
}

impl Drop for InfoRef {
    fn drop(&mut self) {
        self.info.adjust_reference_counter(RefCountOp::NotifyReceiverDestroyed);
        self.info.adjust_reference_counter(RefCountOp::NotifySenderDestroyed);
    }
}

impl core::ops::Deref for InfoRef {
    type Target = &'static super::Info;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

/// Provides a reference to the driver internals and implements RAII for the internal reference
/// counting for Tx only types.
/// See InfoRef for further doc.
pub(crate) struct TxInfoRef {
    info: &'static super::Info,
}

impl TxInfoRef {
    pub(crate) fn new(info: &'static super::Info) -> Self {
        info.adjust_reference_counter(RefCountOp::NotifySenderCreated);
        Self { info }
    }
}

impl Drop for TxInfoRef {
    fn drop(&mut self) {
        self.info.adjust_reference_counter(RefCountOp::NotifySenderDestroyed);
    }
}

impl core::ops::Deref for TxInfoRef {
    type Target = &'static super::Info;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

/// Provides a reference to the driver internals and implements RAII for the internal reference
/// counting for Rx only types.
/// See InfoRef for further doc.
pub(crate) struct RxInfoRef {
    info: &'static super::Info,
}

impl RxInfoRef {
    pub(crate) fn new(info: &'static super::Info) -> Self {
        info.adjust_reference_counter(RefCountOp::NotifyReceiverCreated);
        Self { info }
    }
}

impl Drop for RxInfoRef {
    fn drop(&mut self) {
        self.info.adjust_reference_counter(RefCountOp::NotifyReceiverDestroyed);
    }
}

impl core::ops::Deref for RxInfoRef {
    type Target = &'static super::Info;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}
