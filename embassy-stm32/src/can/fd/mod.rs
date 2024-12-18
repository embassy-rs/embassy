//! Module containing that which is specific to fdcan hardware variant

use core::{
    future::poll_fn,
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
    task::{Poll, Waker},
};

use bit_field::BitField;
use configurator::Properties;
use embassy_sync::waitqueue::AtomicWaker;
use low_level::{util::RxElementData, RxLevels};
use peripheral::Info;
use util::AtomicResourceAllocator;

use crate::can::common::CanMode;

use super::{BaseEnvelope, BaseFrame, BusOff, OperatingMode};

pub mod config;
pub mod filter;

pub(crate) mod configurator;
pub(crate) mod interrupt;
pub(crate) mod low_level;
pub(crate) mod peripheral;
mod util;

pub(self) use interrupt::{IT0InterruptHandler, IT1InterruptHandler};
pub(self) use util::calc_ns_per_timer_tick;

/// FDCAN Instance
pub struct Can<'d, M> {
    _phantom: PhantomData<&'d M>,
    info: &'static Info,
    state: &'static State,
    _mode: OperatingMode,
    properties: Properties,
}

/// The CANFD/M_CAN peripheral has two configurable Rx FIFOs.
#[derive(Debug, Clone, Copy)]
pub enum RxFifo {
    /// FIFO 1
    F0 = 0,
    /// FIFO 2
    F1 = 1,
}

impl<'d, M: CanMode> Can<'d, M> {
    /// Get driver properties
    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    /// This waits until a bus off error occurs.
    ///
    /// Useful in conjunction with ErrorHandlingMode::Central if you
    /// have a dedicated bus error handling task.
    pub async fn wait_bus_off(&self) -> BusOff {
        poll_fn(|cx| {
            self.state.err_waker.register(cx.waker());

            if self
                .state
                .state_flags
                .load(Ordering::Relaxed)
                .get_bit(State::STATE_FLAG_BUS_OFF)
            {
                Poll::Ready(BusOff)
            } else {
                Poll::Pending
            }
        })
        .await
    }

    /// Initiates the Bus Off error recovery procedure
    pub fn recover_bus_off(&self) {
        self.info.low.regs.cccr().modify(|w| w.set_init(false));

        // We only need to wait for the command to make it across to the
        // CAN clock domain, busywaiting should be fine here.
        while self.info.low.regs.cccr().read().init() {}

        self.state
            .state_flags
            .fetch_and(!(1 << State::STATE_FLAG_BUS_OFF), Ordering::Relaxed);
    }
}

/// Rx
impl<'d, M: CanMode> Can<'d, M> {
    async fn rx_inner<T>(
        &self,
        inner: impl Fn(&Waker) -> Option<(T, RxElementData)>,
    ) -> Result<(T, BaseEnvelope<M>), BusOff> {
        poll_fn(|cx| {
            let bus_off = self
                .state
                .state_flags
                .load(Ordering::Relaxed)
                .get_bit(State::STATE_FLAG_BUS_OFF);
            let propagate_rx = self
                .state
                .settings_flags
                .load(Ordering::Relaxed)
                .get_bit(State::FLAG_PROPAGATE_ERRORS_TO_RX);

            if bus_off && propagate_rx {
                return Poll::Ready(Err(BusOff));
            }

            match inner(cx.waker()) {
                Some((passthrough, element)) => {
                    let ts = self
                        .info
                        .calc_timestamp(self.state.ns_per_timer_tick, element.timestamp);
                    let len = element.header.len() as usize;
                    let data = &element.data[0..len];
                    // This should only fail when the data doesn't fit into the `Data` type,
                    // but we validate this at driver creation time. This should never fail.
                    let frame = BaseFrame::new(element.header, data).unwrap();
                    Poll::Ready(Ok((passthrough, BaseEnvelope { ts, frame })))
                }
                None => Poll::Pending,
            }
        })
        .await
    }

    /// Dedicated Rx buffers can be used as destinations of simple single ID
    /// filters.
    pub fn alloc_dedicated_rx(&self) -> Option<u8> {
        self.state.dedicated_rx_alloc.allocate().map(|v| v as u8)
    }

    /// Receive a frame from any buffer or FIFO.
    pub async fn rx_any(&self) -> Result<BaseEnvelope<M>, BusOff> {
        defmt::info!("rx_any");
        self.rx_inner(|waker| {
            self.state.rx_fifo_waker[0].register(waker);
            self.state.rx_fifo_waker[1].register(waker);
            self.state.rx_dedicated_waker.register(waker);

            defmt::info!("levels: {}", self.info.low.get_rx_levels());

            match self.info.low.get_rx_levels() {
                RxLevels { rx_0_level: v, .. } if v > 0 => self.info.low.rx_fifo_read(0).map(|v| ((), v)),
                RxLevels { rx_1_level: v, .. } if v > 0 => self.info.low.rx_fifo_read(1).map(|v| ((), v)),
                RxLevels { buf_mask: v, .. } if v != 0 => self.info.low.rx_buffer_read(u64::MAX).map(|v| ((), v.1)),
                _ => None,
            }
        })
        .await
        .map(|v| v.1)
        .inspect(|v| defmt::info!("envelope: {}", v))
    }

    /// Receive a frame from the given FIFO
    ///
    /// This blocks until either:
    /// * A frame is available
    /// * Depending on the driver error config, a BusOff error occurs.
    pub async fn rx_fifo(&self, fifo: RxFifo) -> Result<BaseEnvelope<M>, BusOff> {
        self.rx_inner(|waker| {
            self.state.rx_fifo_waker[fifo as usize].register(waker);
            self.info.low.rx_fifo_read(fifo as u8).map(|e| ((), e))
        })
        .await
        .map(|v| v.1)
    }

    /// Receive a frame from the first of the available dedicated Rx buffers,
    /// selected by `buffer_mask`.
    /// Returns the index of the dedicated buffer the frame was pulled from,
    /// along with the envelope.
    ///
    /// This blocks until either:
    /// * A frame is available
    /// * Depending on the driver error config, a BusOff error occurs.
    #[cfg(can_fdcan_h7)]
    pub async fn rx_buffers(&self, buffer_mask: u64) -> Result<(u8, BaseEnvelope<M>), BusOff> {
        self.rx_inner(|waker| {
            self.state.rx_dedicated_waker.register(waker);
            self.info.low.rx_buffer_read(buffer_mask)
        })
        .await
    }

    /// Receive a frame from the indicated dedicated Rx buffer index.
    ///
    /// This blocks until either:
    /// * A frame is available
    /// * Depending on the driver error config, a BusOff error occurs.
    #[cfg(can_fdcan_h7)]
    pub async fn rx_buffer(&self, idx: u8) -> Result<BaseEnvelope<M>, BusOff> {
        self.rx_inner(|waker| {
            self.state.rx_dedicated_waker.register(waker);
            self.info.low.rx_buffer_read(1u64 << (idx as usize))
        })
        .await
        .map(|v| v.1)
    }
}

/// Tx
impl<'d, M: CanMode> Can<'d, M> {
    /// Queues the message to be sent but exerts backpressure.
    /// If the given queue is full, this call will wait asynchronously
    /// until it can be added to the queue.
    /// Returns a `TxRef` which can be used to monitor the completion
    /// of the transmission.
    pub async fn tx_queue(&self, frame: &BaseFrame<M>) -> TxRef {
        poll_fn(|cx| {
            // This functions returns pending if there are no free slots,
            // we register to the Tx Done waker to get notified when slots
            // free up.
            self.state.tx_done_waker.register(cx.waker());

            let before_add = |idx: u8| {
                self.state.sync_tx_status();
                let prev = self.state.tx_slot_busy_mask.fetch_or(1 << idx, Ordering::Relaxed);
                // Debug assertion:
                // `idx` from the TX FIFO add logic should always not have a pending TX.
                // Therefore, after calling `sync_tx_status`, the bit should never be set.
                assert!(prev & (1 << idx) == 0);
            };

            // TODO call queue function instead if in queue mode, it is more slot efficient.
            match self.info.low.tx_fifo_add(frame.header(), frame.data(), before_add) {
                // Frame was successfully added to slot.
                Some(slot_idx) => {
                    let generation = self.state.tx_slot_generations[slot_idx as usize].load(Ordering::Relaxed);

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

/// Reference to a frame which is in the process of being transmitted.
pub struct TxRef {
    slot_idx: u8,
    generation: u32,
    state: &'static State,
    completed: bool,
}

impl TxRef {
    /// Waits for transmission to finish.
    pub async fn wait(&mut self) -> Result<(), BusOff> {
        poll_fn(|cx| {
            if self.poll_done() {
                return Poll::Ready(Ok(()));
            }

            let bus_off = self
                .state
                .state_flags
                .load(Ordering::Relaxed)
                .get_bit(State::STATE_FLAG_BUS_OFF);
            let propagate_tx = self
                .state
                .settings_flags
                .load(Ordering::Relaxed)
                .get_bit(State::FLAG_PROPAGATE_ERRORS_TO_TX);

            if bus_off && propagate_tx {
                // If we are in BUS_OFF and the Rx propagation bit is set, we return a bus off error.
                Poll::Ready(Err(BusOff))
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
        let current_generation = self.state.tx_slot_generations[self.slot_idx as usize].load(Ordering::Relaxed);
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

    // Wakers
    pub rx_dedicated_waker: AtomicWaker,
    pub rx_fifo_waker: [AtomicWaker; 2],
    pub tx_done_waker: AtomicWaker,
    pub err_waker: AtomicWaker,

    /// State flags
    pub state_flags: AtomicU32,

    /// Settings flags
    pub settings_flags: AtomicU32,

    /// Bitmask where each bit represents one TX slot.
    ///
    /// After transmission is done or cancelled, a bit will be set in
    /// either the "Tx Occurred" or "Tx Cancelled" register, and the
    /// "Tx Pending" bit will be cleared.
    ///
    /// We need to bump the generation for the Tx slot exactly once,
    /// and there is no way to clear the "Tx Occurred" or "Tx Cancelled"
    /// registers without transmitting a new message in the slot.
    ///
    /// The `tx_slot_busy` bitmask is set when a Tx slot is populated,
    /// and cleared when the status bit for the slot is handled
    /// appropriately. This gets us "exactly once" acknowledgement of
    /// each slot.
    pub tx_slot_busy_mask: AtomicU32,

    /// Each Tx slot has a generation number. This generation number is
    /// incremented each time a transmission is finished.
    ///
    /// If a TxRef referred to only the slot index, we would have no way
    /// of knowing if it is referring to the current message in the Tx
    /// slot of if it is referring to a previour usage of it.
    ///
    /// To solve this issue, a TxRef always refers to BOTH the index and
    /// the generation.
    ///
    /// ## Access
    /// * Incremented on Tx done/cancel
    /// * Loaded by `TxRef` to check if generation matches current
    #[cfg(can_fdcan_h7)]
    pub tx_slot_generations: [AtomicU32; 32],
    #[cfg(not(can_fdcan_h7))]
    pub tx_slot_generations: [AtomicU32; 3],

    pub standard_filter_alloc: AtomicResourceAllocator<128, 4>,
    pub extended_filter_alloc: AtomicResourceAllocator<64, 2>,
    pub dedicated_rx_alloc: AtomicResourceAllocator<64, 2>,

    pub ns_per_timer_tick: u64,
}

impl State {
    const FLAG_AUTO_RECOVER_BUS_OFF: usize = 0;
    const FLAG_PROPAGATE_ERRORS_TO_TX: usize = 1;
    const FLAG_PROPAGATE_ERRORS_TO_RX: usize = 2;

    /// Stored in the status flag because reading the error flag clears counters.
    /// Maintained by the interrupts.
    const STATE_FLAG_BUS_OFF: usize = 0;

    const fn new(info: &'static Info) -> Self {
        Self {
            info,

            // TODO set settings
            settings_flags: AtomicU32::new(0),
            state_flags: AtomicU32::new(0),

            rx_dedicated_waker: AtomicWaker::new(),
            rx_fifo_waker: [AtomicWaker::new(), AtomicWaker::new()],
            tx_done_waker: AtomicWaker::new(),
            err_waker: AtomicWaker::new(),

            tx_slot_busy_mask: AtomicU32::new(0),
            #[cfg(can_fdcan_h7)]
            tx_slot_generations: util::new_atomic_u32_array(),
            #[cfg(not(can_fdcan_h7))]
            tx_slot_generations: util::new_atomic_u32_array(),

            standard_filter_alloc: AtomicResourceAllocator::new(),
            extended_filter_alloc: AtomicResourceAllocator::new(),
            dedicated_rx_alloc: AtomicResourceAllocator::new(),

            ns_per_timer_tick: 0,
        }
    }

    /// Can only be called while the state mutex is locked!
    fn sync_tx_status(&self) {
        critical_section::with(|_| {
            // When a slot is transmitted, a bit is set in either of these two registers.
            let tx_cancel = self.info.low.regs.txbcf().read().0;
            let tx_finish = self.info.low.regs.txbto().read().0;

            // Only slots which are currently marked as busy are taken into account.
            let tx_busy = self.tx_slot_busy_mask.load(Ordering::Relaxed);

            // Slots which have a cancel/finish bit and are marked as busy have not
            // been previously acked.
            let to_bump = (tx_cancel | tx_finish) & tx_busy;

            for n in 0..32 {
                if to_bump.get_bit(n) {
                    self.tx_slot_generations[n].fetch_add(1, Ordering::Relaxed);
                }
            }

            // Generation should only be bumbed once per slot finish, clear the bits we handled.
            self.tx_slot_busy_mask.fetch_and(!to_bump, Ordering::Relaxed);
        });
    }
}
