//! Module containing that which is specific to fdcan hardware variant

use core::{future::poll_fn, marker::PhantomData, sync::atomic::AtomicU32, task::Poll};

use configurator::Properties;
use embassy_sync::waitqueue::AtomicWaker;
use peripheral::Info;

use crate::can::common::CanMode;

use super::{BaseEnvelope, BaseFrame, BusError, CanHeader as _, OperatingMode};

pub mod config;
pub mod filter;

pub(crate) mod configurator;
pub(crate) mod interrupt;
#[cfg_attr(can_fdcan_h7, path = "limits/m_can.rs")]
#[cfg_attr(not(can_fdcan_h7), path = "limits/simplified.rs")]
mod limits;
pub(crate) mod low_level;
pub(crate) mod peripheral;
mod util;

pub(self) use interrupt::{IT0InterruptHandler, IT1InterruptHandler};
pub(self) use peripheral::*;
pub(self) use util::calc_ns_per_timer_tick;

/// FDCAN Instance
pub struct Can<'d, M> {
    _phantom: PhantomData<&'d M>,
    config: crate::can::fd::config::FdCanConfig,
    info: &'static Info,
    state: &'static State,
    _mode: OperatingMode,
    properties: Properties,
}

impl<'d, M: CanMode> Can<'d, M> {
    /// Get driver properties
    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    /// Flush one of the TX mailboxes.
    pub async fn flush(&self, idx: usize) {
        poll_fn(|cx| {
            //self.state.tx_mode.register(cx.waker());
            self.state.tx_waker.register(cx.waker());

            if idx > 3 {
                panic!("Bad mailbox");
            }
            let idx = 1 << idx;
            if !self.info.low.regs.txbrp().read().trp(idx) {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    fn internal_try_read(&self) -> Option<Result<BaseEnvelope<M>, BusError>> {
        if let Some((header, ts, data)) = self.info.low.rx_fifo_read(0) {
            let ts = self.info.calc_timestamp(self.state.ns_per_timer_tick, ts);
            let data = &data[0..header.len() as usize];
            let frame = match BaseFrame::from_header(header, data) {
                Ok(frame) => frame,
                Err(_) => panic!(),
            };
            Some(Ok(BaseEnvelope { ts, frame }))
        } else if let Some(err) = self.info.low.curr_error() {
            // TODO: this is probably wrong
            Some(Err(err))
        } else {
            None
        }
    }

    async fn internal_read(&self) -> Result<BaseEnvelope<M>, BusError> {
        poll_fn(move |cx| {
            self.state.err_waker.register(cx.waker());
            self.state.rx_waker.register(cx.waker());
            match self.internal_try_read() {
                Some(result) => Poll::Ready(result),
                None => Poll::Pending,
            }
        })
        .await
    }

    /// Returns the next received message frame
    pub async fn read(&mut self) -> Result<BaseEnvelope<M>, BusError> {
        //self.state.rx_mode.read(self.info, self.state).await
        self.internal_read().await
    }

    /// Returns the next received message frame, if available.
    /// If there is no frame currently available to be read, this will return immediately.
    pub fn try_read(&mut self) -> Option<Result<BaseEnvelope<M>, BusError>> {
        //self.state.rx_mode.try_read(self.info, self.state)
        self.internal_try_read()
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    async fn internal_write(&self, frame: &BaseFrame<M>) -> Option<BaseFrame<M>> {
        poll_fn(|cx| {
            self.state.tx_waker.register(cx.waker());

            match self.info.low.tx_fifo_add(frame.header(), frame.data()) {
                Some(_idx) => Poll::Ready(None),
                None => Poll::Pending,
            }

            //if !info.low.bad_write(frame) {
            //    return Poll::Ready(*frame);
            //}
            //if let Ok(dropped) = info.low.bad_write(frame) {
            //    return Poll::Ready(dropped);
            //}

            // Couldn't replace any lower priority frames.  Need to wait for some mailboxes
            // to clear.
            //Poll::Pending
        })
        .await
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    pub async fn write(&mut self, frame: &BaseFrame<M>) -> Option<BaseFrame<M>> {
        self.internal_write(frame).await
    }

    /// Queues the message to be sent. Handles rescheduling lower priority message
    /// if one was removed from the mailbox.
    pub async fn write_blocking(&mut self, frame: &BaseFrame<M>) {
        let mut frame_opt = self.internal_write(frame).await;
        while let Some(frame) = frame_opt {
            frame_opt = self.internal_write(&frame).await;
        }
    }
}

pub(self) struct State {
    pub rx_waker: AtomicWaker,
    pub tx_waker: AtomicWaker,

    pub refcount: AtomicU32,

    pub err_waker: AtomicWaker,

    pub ns_per_timer_tick: u64,
}

impl State {
    const fn new() -> Self {
        Self {
            rx_waker: AtomicWaker::new(),
            tx_waker: AtomicWaker::new(),
            refcount: AtomicU32::new(0),
            err_waker: AtomicWaker::new(),
            ns_per_timer_tick: 0,
        }
    }
}
