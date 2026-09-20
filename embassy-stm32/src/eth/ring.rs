//! DMA descriptor rings, shared between the v1 and v2 ethernet MACs.
//!
//! The two MAC generations program the DMA descriptor rings differently, so
//! the divergent parts are `#[cfg]`-gated inline: descriptor initialization
//! and tail-pointer signaling in [`RDesRing::new`]/[`TDesRing::new`], the
//! descriptor read format in [`TDesRing::transmit`], and the v1 receive poll
//! (`demand_poll`/`running_state`). The ring bookkeeping (buffer ownership,
//! reclaim order, timestamps) is common.

use core::sync::atomic::{Ordering, fence};

#[cfg(feature = "ptp")]
use heapless::deque::DequeView;
#[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
use stm32_metapac::eth::vals::{Rpd, Rps};
use xarxa_driver::PacketBuf;
#[cfg(feature = "ptp")]
use xarxa_driver::Timestamp;
#[cfg(feature = "ptp")]
use xarxa_driver::TxTimestamp;
use xarxa_driver::config::PACKET_BUF_SIZE;

use crate::eth::{RDes, RDesInfo, TDes};
#[cfg(any(eth_v1a, eth_v1b, eth_v1c, eth_v2))]
use crate::pac::ETH;
#[cfg(any(eth_v2a, eth_v2b))]
use crate::pac::ETH1 as ETH;

/// Access a per-channel DMA register at channel 0.
///
/// On eth_v2a the DMA channel registers are arrays (the MAC has multiple DMA
/// channels); on eth_v2/eth_v2b they are plain registers. We only ever use
/// channel 0.
#[cfg(any(eth_v2, eth_v2a, eth_v2b))]
macro_rules! dma_ch0 {
    ($dma:expr, $reg:ident) => {{
        #[cfg(any(eth_v2, eth_v2b))]
        {
            $dma.$reg()
        }
        #[cfg(eth_v2a)]
        {
            $dma.$reg(0)
        }
    }};
}

#[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
/// Running state of the `RxRing`
#[derive(PartialEq, Eq, Debug)]
enum RunningState {
    Unknown,
    Stopped,
    Running,
}

/// What reclaiming a completed transmit descriptor yields: its timestamp with PTP,
/// nothing without.
#[cfg(feature = "ptp")]
type Completion = Option<Timestamp>;
#[cfg(not(feature = "ptp"))]
type Completion = ();

/// Rx ring of descriptors and packets
pub(crate) struct RDesRing<'a> {
    descriptors: &'a mut [RDes],
    /// One buffer per descriptor, DMA'd into in place. Always `Some` outside of
    /// `receive`.
    buffers: &'a mut [Option<PacketBuf>],
    index: usize,
}

impl<'a> RDesRing<'a> {
    pub(crate) fn new(descriptors: &'a mut [RDes], buffers: &'a mut [Option<PacketBuf>]) -> Self {
        assert!(descriptors.len() > 1);
        assert!(descriptors.len() == buffers.len());

        for i in 0..descriptors.len() {
            let buf = buffers[i].get_or_insert_with(|| {
                unwrap!(
                    PacketBuf::try_new(),
                    "packet pool exhausted while filling the ethernet RX ring"
                )
            });

            #[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
            // Chained descriptors: link each entry to the next and hand it to
            // the DMA. We already have fences in `set_owned`, which is called
            // in `setup`.
            descriptors[i].setup(descriptors.get(i + 1), buf.storage_mut().as_mut_ptr());

            #[cfg(any(eth_v2, eth_v2a, eth_v2b))]
            {
                descriptors[i] = RDes::new();
                descriptors[i].set_ready(buf.storage_mut().as_mut_ptr());
            }
        }

        // Register rx descriptor start.
        #[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
        ETH.ethernet_dma()
            .dmardlar()
            .write(|w| w.0 = descriptors.as_ptr() as u32);

        #[cfg(any(eth_v2, eth_v2a, eth_v2b))]
        {
            let dma = ETH.ethernet_dma();
            dma_ch0!(dma, dmac_rx_dlar).write(|w| w.0 = descriptors.as_mut_ptr() as u32);
            dma_ch0!(dma, dmac_rx_rlr).write(|w| w.set_rdrl((descriptors.len() as u16) - 1));
            dma_ch0!(dma, dmac_rx_dtpr).write(|w| w.0 = 0);
        }

        Self {
            descriptors,
            buffers,
            index: 0,
        }
    }

    #[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
    /// Get current `RunningState`
    fn running_state(&self) -> RunningState {
        match ETH.ethernet_dma().dmasr().read().rps() {
            //  Reset or Stop Receive Command issued
            Rps::Stopped => RunningState::Stopped,
            //  Fetching receive transfer descriptor
            Rps::RunningFetching => RunningState::Running,
            //  Waiting for receive packet
            Rps::RunningWaiting => RunningState::Running,
            //  Receive descriptor unavailable
            Rps::Suspended => RunningState::Stopped,
            //  Closing receive descriptor
            Rps::_RESERVED_5 => RunningState::Running,
            //  Transferring the receive packet data from receive buffer to host memory
            Rps::RunningWriting => RunningState::Running,
            _ => RunningState::Unknown,
        }
    }

    /// Take a received packet, if any.
    ///
    /// The buffer the frame was DMA'd into is handed out, and the descriptor is
    /// re-armed with a fresh one from the pool. If the pool is empty, the frame
    /// is dropped and the descriptor keeps its buffer.
    pub(crate) fn receive(&mut self) -> Option<PacketBuf> {
        #[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
        if self.running_state() != RunningState::Running {
            ETH.ethernet_dma().dmarpdr().write(|w| w.set_rpd(Rpd::Poll));
        }

        // Not sure if the contents of the write buffer on the M7 can affects reads, so we are using
        // a DMB here just in case, it also serves as a hint to the compiler that we're syncing the
        // buffer (I think .-.)
        fence(Ordering::SeqCst);

        let info = self.fast_forward()?;

        #[cfg(feature = "ptp")]
        // On the v2 MAC this waits for the timestamp context descriptor
        // following the frame; `None` keeps the frame queued.
        let timestamp = self.timestamp(&info)?;

        let len = info.frame_len();
        if len > PACKET_BUF_SIZE {
            debug!("oversized packet: {}", len);
            self.pop_current();
            return None;
        }

        let Some(fresh) = PacketBuf::try_new() else {
            warn!("packet pool exhausted, dropping received frame");
            self.pop_current();
            return None;
        };

        let mut buf = unwrap!(self.buffers[self.index].take());
        buf.set_len(len);
        #[cfg(feature = "ptp")]
        {
            buf.meta_mut().timestamp = timestamp;
        }

        self.buffers[self.index] = Some(fresh);
        self.pop_current();
        Some(buf)
    }

    /// Advance past descriptors that don't hold a receivable frame — empty,
    /// invalid, or (on the v2 MAC) a timestamp context descriptor — and return
    /// the status of the next received frame, or `None` if there is none.
    fn fast_forward(&mut self) -> Option<RDesInfo> {
        // We might have to process many packets, in case some have been rx'd but are invalid.
        loop {
            let info = self.descriptors[self.index].info();
            if !info.available() {
                return None;
            }

            #[cfg(any(eth_v2, eth_v2a, eth_v2b))]
            // Timestamp context descriptor for the previous packet: skip it.
            if info.context_available() {
                self.pop_current();
                continue;
            }

            // If packet is invalid, pop it and try again.
            if !info.valid() {
                debug!("invalid packet: {:08x}", info.rdes0);
                self.pop_current();
                continue;
            }

            return Some(info);
        }
    }

    /// Timestamp to attach to the current frame: the inner `Option` is the
    /// value stored in the frame's metadata (`None` = no timestamp captured);
    /// the outer `None` keeps the frame queued until its timestamp arrives.
    #[cfg(all(feature = "ptp", any(eth_v1a, eth_v1b, eth_v1c)))]
    fn timestamp(&self, _info: &RDesInfo) -> Option<Completion> {
        Some(self.descriptors[self.index].timestamp())
    }

    #[cfg(all(feature = "ptp", any(eth_v2, eth_v2a, eth_v2b)))]
    fn timestamp(&self, info: &RDesInfo) -> Option<Completion> {
        // RDES1 write-back status is valid only when RS1V is set in RDES3.
        // Descriptors returned to DMA are not required to clear RDES1, so do
        // not interpret TSA unless the hardware says the status word is valid.
        if !info.has_timestamp() {
            return Some(None);
        }

        let next = (self.index + 1) % self.descriptors.len();
        let context = &self.descriptors[next];
        let info = context.info();
        let timestamp = if info.context_available() {
            context.context_timestamp()
        } else if info.available() {
            // The following descriptor is a normal packet: the MAC did not
            // append a timestamp context descriptor, so don't block the RX
            // ring waiting for one.
            None
        } else {
            // Keep the packet queued until the following timestamp context
            // descriptor has been written back.
            return None;
        };

        // The MAC signals a missing timestamp with an all-zero value.
        Some(match timestamp {
            Some(Timestamp {
                seconds: 0,
                quarter_nanos: 0,
            }) => None,
            timestamp => timestamp,
        })
    }

    /// Give the current descriptor back to the DMA, keeping its buffer.
    fn pop_current(&mut self) {
        let descriptor = &mut self.descriptors[self.index];
        debug_assert!(descriptor.info().available());

        let ptr = unwrap!(self.buffers[self.index].as_mut()).storage_mut().as_mut_ptr();
        descriptor.set_ready(ptr);

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        #[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
        ETH.ethernet_dma().dmarpdr().write(|w| w.set_rpd(Rpd::Poll));

        #[cfg(any(eth_v2, eth_v2a, eth_v2b))]
        // The DMA stops fetching at the tail pointer, so it must point one
        // PAST the descriptor just re-armed: the next slot in the ring.
        // See issue #2129
        {
            let next = (self.index + 1) % self.descriptors.len();
            let tail = &raw const self.descriptors[next] as *const RDes as u32;
            dma_ch0!(ETH.ethernet_dma(), dmac_rx_dtpr).write(|w| w.0 = tail);
        }

        // Increment index.
        self.index = (self.index + 1) % self.descriptors.len();
    }
}

pub(crate) struct TDesRing<'a> {
    descriptors: &'a mut [TDes],
    /// The buffer of each frame in flight, held until the DMA is done with it.
    buffers: &'a mut [Option<PacketBuf>],
    #[cfg(feature = "ptp")]
    /// Retained timestamps.
    timestamps: &'a mut DequeView<TxTimestamp>,
    /// Next descriptor to submit.
    index: usize,
    /// Submitted descriptors not yet reclaimed.
    in_flight: usize,
}

impl<'a> TDesRing<'a> {
    /// Initialise this TDesRing. Assume TDesRing is corrupt.
    pub(crate) fn new(
        descriptors: &'a mut [TDes],
        buffers: &'a mut [Option<PacketBuf>],
        #[cfg(feature = "ptp")] timestamps: &'a mut DequeView<TxTimestamp>,
    ) -> Self {
        #[cfg(feature = "ptp")]
        timestamps.clear();
        assert!(!descriptors.is_empty());
        assert!(descriptors.len() == buffers.len());

        #[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
        // Chained descriptors: link each entry to the next.
        for (i, entry) in descriptors.iter().enumerate() {
            entry.setup(descriptors.get(i + 1));
        }

        #[cfg(any(eth_v2, eth_v2a, eth_v2b))]
        for td in descriptors.iter_mut() {
            *td = TDes::new();
        }
        for buf in buffers.iter_mut() {
            *buf = None;
        }

        // Register tx descriptor start.
        #[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
        ETH.ethernet_dma()
            .dmatdlar()
            .write(|w| w.0 = descriptors.as_ptr() as u32);

        // Initialize the pointers in the DMA engine. (There will be a memory barrier later
        // before the DMA engine is enabled.)
        #[cfg(any(eth_v2, eth_v2a, eth_v2b))]
        {
            let dma = ETH.ethernet_dma();
            dma_ch0!(dma, dmac_tx_dlar).write(|w| w.0 = descriptors.as_mut_ptr() as u32);
            dma_ch0!(dma, dmac_tx_rlr).write(|w| w.set_tdrl((descriptors.len() as u16) - 1));
            dma_ch0!(dma, dmac_tx_dtpr).write(|w| w.0 = 0);
        }

        Self {
            descriptors,
            buffers,
            #[cfg(feature = "ptp")]
            timestamps,
            index: 0,
            in_flight: 0,
        }
    }

    pub(crate) const fn len(&self) -> usize {
        self.descriptors.len()
    }

    /// The oldest submitted descriptor not yet reclaimed.
    const fn completion_index(&self) -> usize {
        (self.index + self.len() - self.in_flight) % self.len()
    }

    /// Reclaim the oldest completed descriptor: free its buffer and return its
    /// transmit timestamp, if any. `None` if nothing completed.
    fn reclaim_one(&mut self) -> Option<Completion> {
        if self.in_flight == 0 {
            return None;
        }
        let completion_index = self.completion_index();
        let descriptor = &self.descriptors[completion_index];
        if !descriptor.available() {
            return None;
        }

        // Observe DMA write-back before reading the timestamp or releasing the buffer.
        fence(Ordering::Acquire);
        #[cfg(feature = "ptp")]
        let timestamp = descriptor.timestamp();
        #[cfg(not(feature = "ptp"))]
        let timestamp = ();

        // Dropping the buffer frees it.
        self.buffers[completion_index] = None;
        self.in_flight -= 1;
        Some(timestamp)
    }

    pub(crate) fn fast_forward(&mut self) {
        loop {
            let completion_index = self.completion_index();
            #[cfg(feature = "ptp")]
            let packet_id = self.buffers[completion_index].as_ref().map(|b| b.meta().id);
            let Some(completion) = self.reclaim_one() else {
                break;
            };

            #[cfg(feature = "ptp")]
            if let Some(timestamp) = completion
                && let Some(id) = packet_id
            {
                trace!("eth ptp tx complete idx={} packet_id={}", completion_index, id);

                if self.timestamps.push_back(TxTimestamp { id, timestamp }).is_err() {
                    warn!("dropping tx timestamp with id: {}", id)
                }
            }

            #[cfg(not(feature = "ptp"))]
            let _ = completion;
            #[cfg(not(feature = "ptp"))]
            let _ = completion_index;
        }
    }

    #[cfg(feature = "ptp")]
    pub(crate) fn poll_timestamp(&mut self) -> Option<TxTimestamp> {
        self.timestamps.pop_front()
    }

    /// Whether the next `transmit` will be accepted.
    pub(crate) fn can_transmit(&mut self) -> bool {
        // If every descriptor is already submitted but not yet reclaimed,
        // the slot at `index` must not be reused.
        if self.in_flight == self.len() {
            return false;
        }

        self.descriptors[self.index].available()
    }

    /// Transmit a frame. `can_transmit` must have returned `true`.
    pub(crate) fn transmit(&mut self, buf: PacketBuf) {
        debug_assert!(self.in_flight < self.len());
        let descriptor = &mut self.descriptors[self.index];
        debug_assert!(descriptor.available());

        // Read format: buffer address, length, interrupt on completion, and
        // optionally a transmit timestamp request.
        descriptor.set_frame(buf.as_ptr(), buf.len());

        #[cfg(feature = "ptp")]
        if buf.meta().request_timestamp {
            descriptor.set_timestamp_request(true);

            trace!(
                "eth ptp tx submit idx={} packet_id={} len={}",
                self.index,
                buf.meta().id,
                buf.len()
            );
        } else {
            descriptor.set_timestamp_request(false);
        }

        // The DMA reads the frame from the buffer, so it must stay alive until
        // the descriptor is reclaimed.
        self.buffers[self.index] = Some(buf);

        // The frame contents and the rest of the descriptor must be visible to the
        // DMA before it is handed ownership, or it transmits whatever the buffer
        // held before.
        fence(Ordering::Release);

        // FD: Contains first buffer of packet
        // LD: Contains last buffer of packet
        // Give the DMA engine ownership
        // CIC_FULL: let the MAC compute and insert the IP/TCP/UDP checksums.
        descriptor.set_owned();

        // Ensure changes to the descriptor are committed before DMA engine sees tail pointer store.
        // This will generate an DMB instruction.
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        #[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
        // Request the DMA engine to poll the latest tx descriptor.
        ETH.ethernet_dma().dmatpdr().modify(|w| w.0 = 1);

        #[cfg(any(eth_v2, eth_v2a, eth_v2b))]
        // The DMA stops fetching at the tail pointer, so it must point one
        // PAST the descriptor just submitted: the next slot to be filled.
        // See issue #2129
        {
            let next = (self.index + 1) % self.descriptors.len();
            let tail = &raw const self.descriptors[next] as *const TDes as u32;
            dma_ch0!(ETH.ethernet_dma(), dmac_tx_dtpr).write(|w| w.0 = tail);
        }

        self.in_flight += 1;

        // Increment index.
        self.index = (self.index + 1) % self.descriptors.len();
    }
}
