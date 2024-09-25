#![allow(dead_code)]
use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use super::enums::*;
use super::frame::*;

pub(crate) struct BufferedRxInner<M: CanMode> {
    pub rx_sender: DynamicSender<'static, Result<BaseEnvelope<M>, BusError>>,
}
pub(crate) struct BufferedTxInner<M: CanMode> {
    pub tx_receiver: DynamicReceiver<'static, BaseFrame<M>>,
}

/// Sender that can be used for sending CAN frames.
#[derive(Copy, Clone)]
pub struct BufferedCanSender<M: CanMode> {
    pub(crate) tx_buf: embassy_sync::channel::DynamicSender<'static, BaseFrame<M>>,
    pub(crate) waker: fn(),
}

impl<M: CanMode> BufferedCanSender<M> {
    /// Async write frame to TX buffer.
    pub fn try_write(&mut self, frame: BaseFrame<M>) -> Result<(), embassy_sync::channel::TrySendError<BaseFrame<M>>> {
        self.tx_buf.try_send(frame)?;
        (self.waker)();
        Ok(())
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: BaseFrame<M>) {
        self.tx_buf.send(frame).await;
        (self.waker)();
    }

    /// Allows a poll_fn to poll until the channel is ready to write
    pub fn poll_ready_to_send(&self, cx: &mut core::task::Context<'_>) -> core::task::Poll<()> {
        self.tx_buf.poll_ready_to_send(cx)
    }
}

/// Receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
pub type BufferedCanReceiver<M> = embassy_sync::channel::DynamicReceiver<'static, Result<BaseEnvelope<M>, BusError>>;

trait Sealed {}
#[allow(private_bounds)]
pub trait CanMode: Sealed + defmt::Format + core::fmt::Debug + Copy + Clone + Sized + 'static {
    const MAX_DATA_LEN: usize;
    type Data: CanData + core::fmt::Debug + Copy + Clone + defmt::Format;

    fn dyn_can_mode() -> DynCanMode;
}

pub enum DynCanMode {
    Classic,
    Fd,
}

/// Marker type used to indicate a CAN peripheral being used in Classic CAN mode.
/// In classic CAN mode, frame data is limited to 8 bytes.
#[derive(defmt::Format, Debug, Copy, Clone)]
pub enum Classic {}
impl Sealed for Classic {}
impl CanMode for Classic {
    const MAX_DATA_LEN: usize = 8;
    type Data = Data;

    fn dyn_can_mode() -> DynCanMode {
        DynCanMode::Classic
    }
}

/// Marker type used to indicate a CAN peripheral being used in FDCAN mode.
/// In FDCAN mode, frame data is limited to 64 bytes.
#[derive(defmt::Format, Debug, Copy, Clone)]
pub enum Fd {}
impl Sealed for Fd {}
impl CanMode for Fd {
    const MAX_DATA_LEN: usize = 64;
    type Data = FdData;

    fn dyn_can_mode() -> DynCanMode {
        DynCanMode::Fd
    }
}
