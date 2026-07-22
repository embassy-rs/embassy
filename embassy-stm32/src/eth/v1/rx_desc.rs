use core::sync::atomic::{Ordering, compiler_fence, fence};

#[cfg(feature = "ptp")]
use embassy_net_driver::PacketMeta;
use stm32_metapac::eth::vals::{Rpd, Rps};
use vcell::VolatileCell;

use crate::eth::RX_BUFFER_SIZE;
#[cfg(feature = "ptp")]
use crate::eth::packet_state::RxPacketStateRing;
use crate::pac::ETH;

mod rx_consts {
    /// Owned by DMA engine
    pub const RXDESC_0_OWN: u32 = 1 << 31;
    /// Error summary
    pub const RXDESC_0_ES: u32 = 1 << 15;
    /// Frame length mask
    pub const RXDESC_0_FL_MASK: u32 = 0x3FFF;
    pub const RXDESC_0_FL_SHIFT: usize = 16;

    /// First descriptor
    pub const RXDESC_0_FS: u32 = 1 << 9;
    /// Last descriptor
    pub const RXDESC_0_LS: u32 = 1 << 8;
    #[cfg(any(eth_v1b, eth_v1c))]
    /// Payload checksum error / Extended status available
    pub const RXDESC_0_PCE_ESA: u32 = 1 << 0;

    pub const RXDESC_1_RBS1_MASK: u32 = 0x1FFF;
    /// Second address chained
    pub const RXDESC_1_RCH: u32 = 1 << 14;
    /// End Of Ring
    pub const RXDESC_1_RER: u32 = 1 << 15;

    #[cfg(any(eth_v1b, eth_v1c))]
    /// IP checksum bypassed (hardware didn't compute checksum for this frame)
    pub const RXDESC_4_IPCB: u32 = 1 << 5;
    #[cfg(any(eth_v1b, eth_v1c))]
    /// IP payload error
    pub const RXDESC_4_IPPE: u32 = 1 << 4;
    #[cfg(any(eth_v1b, eth_v1c))]
    /// IP header error
    pub const RXDESC_4_IPHE: u32 = 1 << 3;
}

use rx_consts::*;

use super::Packet;

/// Enhanced Receive Descriptor representation (8 words, 32 bytes)
///
/// * rdes0: OWN and Status
/// * rdes1: allocated buffer length / control
/// * rdes2: data buffer address
/// * rdes3: next descriptor address
/// * rdes4: extended status (IP checksum info, PTP info)
/// * rdes5: reserved
/// * rdes6: timestamp low
/// * rdes7: timestamp high
#[repr(C)]
pub(crate) struct RDes {
    rdes0: VolatileCell<u32>,
    rdes1: VolatileCell<u32>,
    rdes2: VolatileCell<u32>,
    rdes3: VolatileCell<u32>,
    #[cfg(any(eth_v1b, eth_v1c))]
    rdes4: VolatileCell<u32>,
    #[cfg(any(eth_v1b, eth_v1c))]
    rdes5: VolatileCell<u32>,
    #[cfg(any(eth_v1b, eth_v1c))]
    rdes6: VolatileCell<u32>,
    #[cfg(any(eth_v1b, eth_v1c))]
    rdes7: VolatileCell<u32>,
}

struct RDesInfo {
    rdes0: u32,
    #[cfg(any(eth_v1b, eth_v1c))]
    rdes4: u32,
}

impl RDesInfo {
    /// Return true if this RDes is acceptable to us.
    /// Checks: frame is complete (FS+LS), no DMA errors, and IP checksums are valid (if applicable).
    const fn valid(&self) -> bool {
        let rdes0 = self.rdes0;

        // Write-back descriptor is valid if:
        //
        // Contains first buffer of packet AND contains last buf of
        // packet AND no errors
        if !(rdes0 & (RXDESC_0_ES | RXDESC_0_FS | RXDESC_0_LS)) == (RXDESC_0_FS | RXDESC_0_LS) {
            return false;
        }

        #[cfg(any(eth_v1b, eth_v1c))]
        // If extended status is not available, we can't verify checksums in hardware.
        // This happens for non-IP frames (ARP, etc.). In that case, basic_valid is sufficient.
        if (rdes0 & RXDESC_0_PCE_ESA) == 0 {
            return true;
        }

        #[cfg(any(eth_v1b, eth_v1c))]
        // Extended status (RDES4) is valid. Check IP checksum results.
        let rdes4 = self.rdes4;

        #[cfg(any(eth_v1b, eth_v1c))]
        // If hardware bypassed checksum computation, we can't validate here.
        // This can happen for jumbo frames, frames with options, etc.
        // The caller should do software checksum in this case.
        if (rdes4 & RXDESC_4_IPCB) != 0 {
            return true; // Let caller handle software checksum
        }

        #[cfg(any(eth_v1b, eth_v1c))]
        // Check for IP header error
        if (rdes4 & RXDESC_4_IPHE) != 0 {
            return false;
        }

        #[cfg(any(eth_v1b, eth_v1c))]
        // Check for IP payload (TCP/UDP/ICMP) error
        if (rdes4 & RXDESC_4_IPPE) != 0 {
            return false;
        }

        // All hardware checksums passed (or frame type had no checksum to verify)
        true
    }

    /// Return true if this RDes is not currently owned by the DMA
    const fn available(&self) -> bool {
        self.rdes0 & RXDESC_0_OWN == 0 // Owned by us
    }

    const fn packet_len(&self) -> usize {
        ((self.rdes0 >> RXDESC_0_FL_SHIFT) & RXDESC_0_FL_MASK) as usize
    }
}

impl RDes {
    pub const fn new() -> Self {
        Self {
            rdes0: VolatileCell::new(0),
            rdes1: VolatileCell::new(0),
            rdes2: VolatileCell::new(0),
            rdes3: VolatileCell::new(0),
            #[cfg(any(eth_v1b, eth_v1c))]
            rdes4: VolatileCell::new(0),
            #[cfg(any(eth_v1b, eth_v1c))]
            rdes5: VolatileCell::new(0),
            #[cfg(any(eth_v1b, eth_v1c))]
            rdes6: VolatileCell::new(0),
            #[cfg(any(eth_v1b, eth_v1c))]
            rdes7: VolatileCell::new(0),
        }
    }

    fn info(&self) -> RDesInfo {
        RDesInfo {
            rdes0: self.rdes0.get(),
            #[cfg(any(eth_v1b, eth_v1c))]
            rdes4: self.rdes4.get(),
        }
    }

    /// Configures the reception buffer address and length and passed descriptor ownership to the DMA
    #[inline(always)]
    fn set_ready(&self, buf: *mut u8) {
        self.rdes1
            .set(self.rdes1.get() | (RX_BUFFER_SIZE as u32) & RXDESC_1_RBS1_MASK);
        self.rdes2.set(buf as u32);

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        compiler_fence(Ordering::Release);

        self.rdes0.set(self.rdes0.get() | RXDESC_0_OWN);

        // Used to flush the store buffer as fast as possible to make the buffer available for the
        // DMA.
        fence(Ordering::SeqCst);
    }

    // points to next descriptor (RCH)
    #[inline(always)]
    fn set_buffer2(&self, buffer: *const u8) {
        self.rdes3.set(buffer as u32);
    }

    #[inline(always)]
    fn set_end_of_ring(&self) {
        self.rdes1.set(self.rdes1.get() | RXDESC_1_RER);
    }

    fn setup(&self, next: Option<&Self>, buf: *mut u8) {
        // Defer this initialization to this function, so we can have `RingEntry` on bss.
        self.rdes0.set(0);
        self.rdes1.set(self.rdes1.get() | RXDESC_1_RCH);
        #[cfg(any(eth_v1b, eth_v1c))]
        self.rdes4.set(0);
        #[cfg(any(eth_v1b, eth_v1c))]
        self.rdes5.set(0);
        #[cfg(any(eth_v1b, eth_v1c))]
        self.rdes6.set(0);
        #[cfg(any(eth_v1b, eth_v1c))]
        self.rdes7.set(0);

        match next {
            Some(next) => self.set_buffer2(next as *const _ as *const u8),
            None => {
                self.set_buffer2(0 as *const u8);
                self.set_end_of_ring();
            }
        }

        self.set_ready(buf);
    }
}

/// Running state of the `RxRing`
#[derive(PartialEq, Eq, Debug)]
pub enum RunningState {
    Unknown,
    Stopped,
    Running,
}

/// Rx ring of descriptors and packets
pub(crate) struct RDesRing<'a> {
    descriptors: &'a mut [RDes],
    buffers: &'a mut [Packet<RX_BUFFER_SIZE>],
    #[cfg(feature = "ptp")]
    state: RxPacketStateRing<'a>,
    index: usize,
}

impl<'a> RDesRing<'a> {
    pub(crate) fn new(
        descriptors: &'a mut [RDes],
        buffers: &'a mut [Packet<RX_BUFFER_SIZE>],
        #[cfg(feature = "ptp")] state: RxPacketStateRing<'a>,
    ) -> Self {
        assert!(descriptors.len() > 1);
        assert!(descriptors.len() == buffers.len());

        for (i, entry) in descriptors.iter().enumerate() {
            entry.setup(descriptors.get(i + 1), buffers[i].0.as_mut_ptr());
        }

        // Register rx descriptor start
        ETH.ethernet_dma()
            .dmardlar()
            .write(|w| w.0 = descriptors.as_ptr() as u32);
        // We already have fences in `set_owned`, which is called in `setup`

        Self {
            descriptors,
            buffers,
            #[cfg(feature = "ptp")]
            state,
            index: 0,
        }
    }

    pub(crate) fn demand_poll(&self) {
        ETH.ethernet_dma().dmarpdr().write(|w| w.set_rpd(Rpd::Poll));
    }

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

    /// Get a received packet if any, or None.
    pub(crate) fn available(&mut self) -> Option<&mut [u8]> {
        if self.running_state() != RunningState::Running {
            self.demand_poll();
        }

        // Not sure if the contents of the write buffer on the M7 can affects reads, so we are using
        // a DMB here just in case, it also serves as a hint to the compiler that we're syncing the
        // buffer (I think .-.)
        fence(Ordering::SeqCst);

        // We might have to process many packets, in case some have been rx'd but are invalid.
        let info = loop {
            let info = self.descriptors[self.index].info();
            if !info.available() {
                return None;
            }

            // If packet is invalid, pop it and try again.
            if !info.valid() {
                warn!("invalid packet: {:08x}", info.rdes0);
                self.pop_packet();
                continue;
            }

            break info;
        };

        let len = info.packet_len();
        #[cfg(feature = "ptp")]
        self.state.capture(self.index, None);
        return Some(&mut self.buffers[self.index].0[..len]);
    }

    #[cfg(feature = "ptp")]
    pub(crate) fn meta(&self) -> PacketMeta {
        self.state.meta(self.index)
    }

    /// Pop the packet previously returned by `available`.
    pub(crate) fn pop_packet(&mut self) {
        let descriptor = &mut self.descriptors[self.index];
        debug_assert!(descriptor.info().available());

        #[cfg(feature = "ptp")]
        self.state.clear(self.index);
        self.descriptors[self.index].set_ready(self.buffers[self.index].0.as_mut_ptr());

        self.demand_poll();

        // Increment index.
        self.index += 1;
        if self.index == self.descriptors.len() {
            self.index = 0
        }
    }
}
