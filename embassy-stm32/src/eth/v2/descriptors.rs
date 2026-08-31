use core::sync::atomic::{Ordering, fence};

use vcell::VolatileCell;
use xarxa_driver::PacketBuf;
#[cfg(feature = "ptp")]
use xarxa_driver::Timestamp;
use xarxa_driver::config::PACKET_BUF_SIZE;

#[cfg(eth_v2)]
use crate::pac::ETH;
#[cfg(any(eth_v2a, eth_v2b))]
use crate::pac::ETH1 as ETH;

/// Access a per-channel DMA register at channel 0.
///
/// On eth_v2a the DMA channel registers are arrays (the MAC has multiple DMA
/// channels); on eth_v2 they are plain registers. We only ever use channel 0.
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

/// Transmit and Receive Descriptor fields
#[allow(dead_code)]
mod emac_consts {
    pub const EMAC_DES3_OWN: u32 = 0x8000_0000;
    pub const EMAC_DES3_CTXT: u32 = 0x4000_0000;
    pub const EMAC_DES3_FD: u32 = 0x2000_0000;
    pub const EMAC_DES3_LD: u32 = 0x1000_0000;
    pub const EMAC_DES3_ES: u32 = 0x0000_8000;
    pub const EMAC_DES0_BUF1AP: u32 = 0xFFFF_FFFF;

    pub const EMAC_TDES2_IOC: u32 = 0x8000_0000;
    pub const EMAC_TDES2_TTSE: u32 = 0x4000_0000;
    pub const EMAC_TDES2_B1L: u32 = 0x0000_3FFF;

    // TX checksum insertion control (TDES3, read format), bits [17:16]. 0b11 =
    // insert IP header + payload checksums, with the pseudo-header computed by
    // hardware (full offload).
    pub const EMAC_TDES3_CIC_FULL: u32 = 0x0003_0000;
    pub const EMAC_TDES3_TTSS: u32 = 0x0002_0000;

    pub const EMAC_RDES3_IOC: u32 = 0x4000_0000;
    pub const EMAC_RDES3_PL: u32 = 0x0000_7FFF;
    pub const EMAC_RDES3_BUF1V: u32 = 0x0100_0000;
    pub const EMAC_RDES3_PKTLEN: u32 = 0x0000_7FFF;
    pub const EMAC_RDES3_RS1V: u32 = 0x0400_0000;

    // RX checksum status (RDES1, write-back format). These are NOT folded into
    // the RDES3 error summary, so they must be inspected separately.
    pub const EMAC_RDES1_IPHE: u32 = 0x0000_0008; // IP header checksum error
    pub const EMAC_RDES1_IPCE: u32 = 0x0000_0080; // IP payload (TCP/UDP/ICMP) checksum error
    pub const EMAC_RDES1_PT: u32 = 0x0000_0003; // payload type
    pub const EMAC_RDES1_PT_UDP: u32 = 1;
    pub const EMAC_RDES1_PT_TCP: u32 = 2;
    pub const EMAC_RDES1_TSA: u32 = 0x0000_4000; // timestamp available
}
use emac_consts::*;

/// Transmit Descriptor representation
///
/// * tdes0: transmit buffer address
/// * tdes1:
/// * tdes2: buffer lengths
/// * tdes3: control and payload/frame length
#[repr(C)]
pub(crate) struct TDes {
    tdes0: VolatileCell<u32>,
    tdes1: VolatileCell<u32>,
    tdes2: VolatileCell<u32>,
    tdes3: VolatileCell<u32>,
}

impl TDes {
    pub const fn new() -> Self {
        Self {
            tdes0: VolatileCell::new(0),
            tdes1: VolatileCell::new(0),
            tdes2: VolatileCell::new(0),
            tdes3: VolatileCell::new(0),
        }
    }

    /// Return true if this TDes is not currently owned by the DMA
    fn available(&self) -> bool {
        self.tdes3.get() & EMAC_DES3_OWN == 0
    }

    #[cfg(feature = "ptp")]
    fn timestamp(&self) -> Option<xarxa_driver::Timestamp> {
        (self.tdes3.get() & EMAC_TDES3_TTSS != 0)
            .then(|| xarxa_driver::Timestamp::from_seconds_and_nanos(self.tdes1.get(), self.tdes0.get()))
    }
}

/// What reclaiming a completed transmit descriptor yields: its timestamp with PTP,
/// nothing without.
#[cfg(feature = "ptp")]
type Completion = Option<Timestamp>;
#[cfg(not(feature = "ptp"))]
type Completion = ();

pub(crate) struct TDesRing<'a> {
    descriptors: &'a mut [TDes],
    /// The buffer of each frame in flight, held until the DMA is done with it.
    buffers: &'a mut [Option<PacketBuf>],
    /// Next descriptor to submit.
    index: usize,
    /// Submitted descriptors not yet reclaimed.
    in_flight: usize,
}

impl<'a> TDesRing<'a> {
    /// Initialise this TDesRing. Assume TDesRing is corrupt.
    pub fn new(descriptors: &'a mut [TDes], buffers: &'a mut [Option<PacketBuf>]) -> Self {
        assert!(descriptors.len() > 0);
        assert!(descriptors.len() == buffers.len());

        for td in descriptors.iter_mut() {
            *td = TDes::new();
        }
        for buf in buffers.iter_mut() {
            *buf = None;
        }

        // Initialize the pointers in the DMA engine. (There will be a memory barrier later
        // before the DMA engine is enabled.)
        let dma = ETH.ethernet_dma();
        dma_ch0!(dma, dmac_tx_dlar).write(|w| w.0 = descriptors.as_mut_ptr() as u32);
        dma_ch0!(dma, dmac_tx_rlr).write(|w| w.set_tdrl((descriptors.len() as u16) - 1));
        dma_ch0!(dma, dmac_tx_dtpr).write(|w| w.0 = 0);

        Self {
            descriptors,
            buffers,
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

        #[cfg(feature = "ptp")]
        let timestamp = descriptor.timestamp();
        #[cfg(not(feature = "ptp"))]
        let timestamp = ();

        // Dropping the buffer frees it.
        self.buffers[completion_index] = None;
        self.in_flight -= 1;
        Some(timestamp)
    }

    /// Whether the next `transmit` will be accepted.
    pub(crate) fn can_transmit(&mut self) -> bool {
        // Without PTP nothing else reclaims completed descriptors, so do it here.
        // With PTP, `poll_timestamp` reclaims them so their timestamps are reported.
        #[cfg(not(feature = "ptp"))]
        while self.reclaim_one().is_some() {}

        // If every descriptor is already submitted but not yet reclaimed,
        // the slot at `index` must not be reused.
        if self.in_flight == self.len() {
            return false;
        }

        self.descriptors[self.index].available()
    }

    #[cfg(feature = "ptp")]
    pub(crate) fn poll_timestamp(&mut self) -> Option<xarxa_driver::TxTimestamp> {
        loop {
            let completion_index = self.completion_index();
            let packet_id = self.buffers[completion_index].as_ref().map(|b| b.meta().id);
            let timestamp = self.reclaim_one()?;

            if let Some(timestamp) = timestamp
                && let Some(id) = packet_id
            {
                trace!("eth ptp tx complete idx={} packet_id={}", completion_index, id);
                break Some(xarxa_driver::TxTimestamp { id, timestamp });
            }
        }
    }

    /// Transmit a frame. `can_transmit` must have returned `true`.
    pub(crate) fn transmit(&mut self, buf: PacketBuf) {
        debug_assert!(self.in_flight < self.len());
        let td = &mut self.descriptors[self.index];
        debug_assert!(td.available());
        let len = buf.len();
        assert!(len as u32 <= EMAC_TDES2_B1L);

        // Read format
        td.tdes0.set(buf.as_ptr() as u32);
        let mut tdes2 = len as u32 & EMAC_TDES2_B1L;
        tdes2 |= EMAC_TDES2_IOC;
        #[cfg(feature = "ptp")]
        if buf.meta().request_timestamp {
            tdes2 |= EMAC_TDES2_TTSE;
            trace!(
                "eth ptp tx submit idx={} packet_id={} len={} tdes2={:#010x}",
                self.index,
                buf.meta().id,
                len,
                tdes2
            );
        }
        td.tdes2.set(tdes2);

        // The DMA reads the frame from the buffer, so it must stay alive until
        // the descriptor is reclaimed.
        self.buffers[self.index] = Some(buf);

        // FD: Contains first buffer of packet
        // LD: Contains last buffer of packet
        // Give the DMA engine ownership
        // No checksum insertion: the stack computes checksums in software.
        let tdes3 = EMAC_DES3_FD | EMAC_DES3_LD | EMAC_DES3_OWN;
        td.tdes3.set(tdes3);

        // Ensure changes to the descriptor are committed before DMA engine sees tail pointer store.
        // This will generate an DMB instruction.
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        // signal DMA it can try again.
        // See issue #2129
        dma_ch0!(ETH.ethernet_dma(), dmac_tx_dtpr).write(|w| w.0 = &td as *const _ as u32);

        self.in_flight += 1;

        // Increment index.
        self.index = (self.index + 1) % self.descriptors.len();
    }
}

/// Receive Descriptor representation
///
/// * rdes0: receive buffer address
/// * rdes1:
/// * rdes2:
/// * rdes3: OWN and Status
#[repr(C)]
pub(crate) struct RDes {
    rdes0: VolatileCell<u32>,
    rdes1: VolatileCell<u32>,
    rdes2: VolatileCell<u32>,
    rdes3: VolatileCell<u32>,
}

struct RDesInfo {
    rdes1: u32,
    rdes3: u32,
}

impl RDesInfo {
    /// Return true if this RDes is acceptable to us
    const fn valid(&self) -> bool {
        // Write-back descriptor is valid if it contains the first AND last
        // buffer of the packet AND has no errors AND is not a context descriptor.
        if self.rdes3 & (EMAC_DES3_FD | EMAC_DES3_LD | EMAC_DES3_ES | EMAC_DES3_CTXT) != (EMAC_DES3_FD | EMAC_DES3_LD) {
            return false;
        }

        // Hardware checksum offload: the MAC verified the IPv4 header
        // and the TCP/UDP payload checksums. xarxa is told not to re-verify
        // these (see the driver `capabilities`), so a frame the MAC flagged as
        // bad must be dropped here.

        let pt = self.rdes1 & EMAC_RDES1_PT;
        let tcp_or_udp = pt == EMAC_RDES1_PT_TCP || pt == EMAC_RDES1_PT_UDP;
        if self.rdes1 & EMAC_RDES1_IPHE != 0 || (tcp_or_udp && self.rdes1 & EMAC_RDES1_IPCE != 0) {
            return false;
        }

        true
    }

    #[cfg(feature = "ptp")]
    const fn has_timestamp(&self) -> bool {
        self.rdes3 & EMAC_RDES3_RS1V != 0 && self.rdes1 & EMAC_RDES1_TSA != 0
    }

    /// Return true if this RDes is not currently owned by the DMA
    const fn available(&self) -> bool {
        self.rdes3 & EMAC_DES3_OWN == 0 // Owned by us
    }

    const fn context_available(&self) -> bool {
        self.rdes3 & (EMAC_DES3_OWN | EMAC_DES3_CTXT) == EMAC_DES3_CTXT
    }

    const fn len(&self) -> u32 {
        self.rdes3 & EMAC_RDES3_PKTLEN
    }
}

impl RDes {
    pub const fn new() -> Self {
        Self {
            rdes0: VolatileCell::new(0),
            rdes1: VolatileCell::new(0),
            rdes2: VolatileCell::new(0),
            rdes3: VolatileCell::new(0),
        }
    }

    fn info(&self) -> RDesInfo {
        RDesInfo {
            rdes1: self.rdes1.get(),
            rdes3: self.rdes3.get(),
        }
    }

    fn set_ready(&mut self, buf: *mut u8) {
        self.rdes0.set(buf as u32);
        self.rdes3.set(EMAC_RDES3_BUF1V | EMAC_RDES3_IOC | EMAC_DES3_OWN);
    }

    #[cfg(feature = "ptp")]
    fn context_timestamp(&self) -> Option<Timestamp> {
        let rdes0 = self.rdes0.get();
        let rdes1 = self.rdes1.get();

        if !(rdes0 == u32::MAX && rdes1 == u32::MAX) {
            Some(Timestamp::from_seconds_and_nanos(rdes1, rdes0))
        } else {
            None
        }
    }
}

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

        for (i, desc) in descriptors.iter_mut().enumerate() {
            *desc = RDes::new();
            let buf = buffers[i].get_or_insert_with(|| {
                unwrap!(
                    PacketBuf::try_new(),
                    "packet pool exhausted while filling the ethernet RX ring"
                )
            });
            desc.set_ready(buf.storage_mut().as_mut_ptr());
        }

        let dma = ETH.ethernet_dma();
        dma_ch0!(dma, dmac_rx_dlar).write(|w| w.0 = descriptors.as_mut_ptr() as u32);
        dma_ch0!(dma, dmac_rx_rlr).write(|w| w.set_rdrl((descriptors.len() as u16) - 1));
        dma_ch0!(dma, dmac_rx_dtpr).write(|w| w.0 = 0);

        Self {
            descriptors,
            buffers,
            index: 0,
        }
    }

    fn fast_forward(&mut self) -> Option<RDesInfo> {
        // We might have to process many packets, in case some have been rx'd but are invalid.
        loop {
            let info = self.descriptors[self.index].info();
            if !info.available() {
                break None;
            }

            if info.context_available() {
                self.pop_current();
                continue;
            }

            // If packet is invalid, pop it and try again.
            if !info.valid() {
                debug!("invalid packet: {:08x}", self.descriptors[self.index].rdes0.get());
                self.pop_current();
                continue;
            }

            break Some(info);
        }
    }

    /// Take a received packet, if any.
    ///
    /// The buffer the frame was DMA'd into is handed out, and the descriptor is
    /// re-armed with a fresh one from the pool. If the pool is empty, the frame
    /// is dropped and the descriptor keeps its buffer.
    pub(crate) fn receive(&mut self) -> Option<PacketBuf> {
        // Not sure if the contents of the write buffer on the M7 can affects reads, so we are using
        // a DMB here just in case, it also serves as a hint to the compiler that we're syncing the
        // buffer (I think .-.)
        fence(Ordering::SeqCst);

        let info = self.fast_forward()?;

        #[cfg(feature = "ptp")]
        let timestamp = self.timestamp(&info)?;

        let len = info.len() as usize;
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
            buf.meta_mut().timestamp = match timestamp {
                Timestamp {
                    seconds: 0,
                    quarter_nanos: 0,
                } => None,
                timestamp => Some(timestamp),
            };
        }

        self.buffers[self.index] = Some(fresh);
        self.pop_current();
        Some(buf)
    }

    #[cfg(feature = "ptp")]
    fn timestamp(&self, info: &RDesInfo) -> Option<Timestamp> {
        // RDES1 write-back status is valid only when RS1V is set in RDES3.
        // Descriptors returned to DMA are not required to clear RDES1, so do
        // not interpret TSA unless the hardware says the status word is valid.
        if !info.has_timestamp() {
            return Some(Timestamp::default());
        }

        let next = (self.index + 1) % self.descriptors.len();
        let context = &self.descriptors[next];
        let info = context.info();
        if info.context_available() {
            Some(context.context_timestamp().unwrap_or_default())
        } else if info.available() {
            Some(Timestamp::default())
        } else {
            // Keep the packet queued until the following timestamp context
            // descriptor has been written back. If it becomes a normal packet
            // instead, do not block the RX ring waiting for a timestamp that
            // the hardware did not provide.
            None
        }
    }

    /// Give the current descriptor back to the DMA with the buffer in its slot.
    fn pop_current(&mut self) {
        let rd = &mut self.descriptors[self.index];
        debug_assert!(rd.info().available());

        let ptr = unwrap!(self.buffers[self.index].as_mut()).storage_mut().as_mut_ptr();
        rd.set_ready(ptr);

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        // signal DMA it can try again.
        // See issue #2129
        dma_ch0!(ETH.ethernet_dma(), dmac_rx_dtpr).write(|w| w.0 = &rd as *const _ as u32);

        // Increment index.
        self.index = (self.index + 1) % self.descriptors.len();
    }
}
