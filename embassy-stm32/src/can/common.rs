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
#[derive(Copy, Clone)]
pub struct BufferedCanSender {
    pub(crate) tx_buf: embassy_sync::channel::DynamicSender<'static, Frame>,
    pub(crate) waker: fn(),
}

impl BufferedCanSender {
    /// Async write frame to TX buffer.
    pub fn try_write(&mut self, frame: Frame) -> Result<(), embassy_sync::channel::TrySendError<Frame>> {
        self.tx_buf.try_send(frame)?;
        (self.waker)();
        Ok(())
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: Frame) {
        self.tx_buf.send(frame).await;
        (self.waker)();
    }

    /// Allows a poll_fn to poll until the channel is ready to write
    pub fn poll_ready_to_send(&self, cx: &mut core::task::Context<'_>) -> core::task::Poll<()> {
        self.tx_buf.poll_ready_to_send(cx)
    }
}

/// Receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
pub type BufferedCanReceiver = embassy_sync::channel::DynamicReceiver<'static, Result<Envelope, BusError>>;
