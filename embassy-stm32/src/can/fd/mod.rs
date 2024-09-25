//! Module containing that which is specific to fdcan hardware variant

use core::{
    future::poll_fn,
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
    task::Poll,
};

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

#[derive(Debug, Clone, Copy)]
pub enum RxQueue {
    Q0 = 0,
    Q1 = 1,
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
        if let Some(element) = self.info.low.rx_fifo_read(0) {
            let ts = self
                .info
                .calc_timestamp(self.state.ns_per_timer_tick, element.timestamp);
            let data = &element.data[0..element.header.len() as usize];
            let frame = match BaseFrame::from_header(element.header, data) {
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
}

impl<'d, M: CanMode> Can<'d, M> {
    /// Queues the message to be sent but exerts backpressure.
    /// If the given queue is full, this call will wait asynchronously
    /// until it can be added to the queue.
    /// Returns a `TxRef` which can be used to monitor the completion
    /// of the transmission.
    pub async fn tx_queue(&self, frame: &BaseFrame<M>) -> TxRef {
        poll_fn(|cx| {
            self.state.tx_waker.register(cx.waker());

            match self.info.low.tx_fifo_add(frame.header(), frame.data()) {
                // Frame was successfully added to slot.
                Some(slot_idx) => {
                    let generation = self.state.tx_generations[slot_idx as usize].fetch_add(1, Ordering::Relaxed);

                    Poll::Ready(TxRef {
                        slot_idx,
                        generation,
                        state: self.state,
                        completed: false,
                    })
                }
                // No free slot, retry later.
                None => Poll::Pending,
            }
        })
        .await
    }
}

// == TX
// Buffer add req -> TXBAR
// Buffer pending, TXBRP AS, TXBTO DE, TXBCF DE
//
// Buffer transmitted, TXBTO AS, TXBRP DE
//
// Buffer cancel req -> TXBCR
// Buffer failed, TXBCF AS, TXBRP DE

/// Reference to a frame which is in the process of being transmitted.
pub struct TxRef {
    slot_idx: u8,
    generation: u32,
    state: &'static State,
    completed: bool,
}

impl TxRef {
    /// Waits for transmission to finish.
    pub async fn wait(&mut self) {
        poll_fn(|cx| {
            if self.poll_done() {
                Poll::Ready(())
            } else {
                self.state.tx_done_waker.register(cx.waker());
                Poll::Pending
            }
        })
        .await
    }

    /// Polls the transmission, returns true if it is finished.
    pub fn poll_done(&mut self) -> bool {
        // If there is a result stored in self, we are finished.
        if self.completed {
            return true;
        }

        // Check the generation for the current slot, if it is different
        // the slot finished.
        let current_generation = self.state.tx_generations[self.slot_idx as usize].load(Ordering::Relaxed);
        if current_generation != self.generation {
            // Store completion state to handle subsequent calls.
            self.completed = true;
            return true;
        }

        return false;
    }

    /// Begins cancellation of the transmission.
    /// Cancellation may take some time, and may race with
    /// successful transmission.
    /// You can call `wait` to wait for transmission completion
    /// or call `poll` to check the state at any point.
    pub fn cancel(&mut self) -> &mut Self {
        // Set the tx busy flag.
        // This prevents any other transmission from taking our slot
        self.state.info.low.tx_cancel(1u32 << self.slot_idx);
        self
    }
}

pub(self) struct State {
    pub info: &'static Info,

    pub rx_waker: AtomicWaker,
    pub tx_waker: AtomicWaker,

    pub tx_done_waker: AtomicWaker,
    pub tx_busy: AtomicU32,
    pub tx_generations: [AtomicU32; 32],

    pub err_waker: AtomicWaker,

    pub ns_per_timer_tick: u64,
}

impl State {
    const fn new(info: &'static Info) -> Self {
        Self {
            info,

            rx_waker: AtomicWaker::new(),
            tx_waker: AtomicWaker::new(),

            tx_done_waker: AtomicWaker::new(),
            tx_busy: AtomicU32::new(0),
            tx_generations: [
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
                AtomicU32::new(0),
            ],

            err_waker: AtomicWaker::new(),
            ns_per_timer_tick: 0,
        }
    }
}
