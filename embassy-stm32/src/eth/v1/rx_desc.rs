use core::sync::atomic::{compiler_fence, fence, Ordering};

use stm32_metapac::eth::vals::{Rpd, Rps};
use vcell::VolatileCell;

use crate::eth::RX_BUFFER_SIZE;
use crate::pac::ETH;

mod rx_consts {
    /// Owned by DMA engine
    pub const RXDESC_0_OWN: u32 = 1 << 31;
    /// First descriptor
    pub const RXDESC_0_FS: u32 = 1 << 9;
    /// Last descriptor
    pub const RXDESC_0_LS: u32 = 1 << 8;
    /// Error summary
    pub const RXDESC_0_ES: u32 = 1 << 15;
    /// Frame length
    pub const RXDESC_0_FL_MASK: u32 = 0x3FFF;
    pub const RXDESC_0_FL_SHIFT: usize = 16;

    pub const RXDESC_1_RBS_MASK: u32 = 0x0fff;
    /// Second address chained
    pub const RXDESC_1_RCH: u32 = 1 << 14;
    /// End Of Ring
    pub const RXDESC_1_RER: u32 = 1 << 15;
}

use rx_consts::*;

use super::Packet;

/// Receive Descriptor representation
///
/// * rdes0: OWN and Status
/// * rdes1: allocated buffer length
/// * rdes2: data buffer address
/// * rdes3: next descriptor address
#[repr(C)]
pub(crate) struct RDes {
    rdes0: VolatileCell<u32>,
    rdes1: VolatileCell<u32>,
    rdes2: VolatileCell<u32>,
    rdes3: VolatileCell<u32>,
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

    /// Return true if this RDes is acceptable to us
    #[inline(always)]
    fn valid(&self) -> bool {
        // Write-back descriptor is valid if:
        //
        // Contains first buffer of packet AND contains last buf of
        // packet AND no errors
        (self.rdes0.get() & (RXDESC_0_ES | RXDESC_0_FS | RXDESC_0_LS)) == (RXDESC_0_FS | RXDESC_0_LS)
    }

    /// Return true if this RDes is not currently owned by the DMA
    #[inline(always)]
    fn available(&self) -> bool {
        self.rdes0.get() & RXDESC_0_OWN == 0 // Owned by us
    }

    /// Configures the reception buffer address and length and passed descriptor ownership to the DMA
    #[inline(always)]
    fn set_ready(&self, buf: *mut u8) {
        self.rdes1
            .set(self.rdes1.get() | (RX_BUFFER_SIZE as u32) & RXDESC_1_RBS_MASK);
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

    #[inline(always)]
    fn packet_len(&self) -> usize {
        ((self.rdes0.get() >> RXDESC_0_FL_SHIFT) & RXDESC_0_FL_MASK) as usize
    }

    fn setup(&self, next: Option<&Self>, buf: *mut u8) {
        // Defer this initialization to this function, so we can have `RingEntry` on bss.
        self.rdes1.set(self.rdes1.get() | RXDESC_1_RCH);

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
    index: usize,
}

impl<'a> RDesRing<'a> {
    pub(crate) fn new(descriptors: &'a mut [RDes], buffers: &'a mut [Packet<RX_BUFFER_SIZE>]) -> Self {
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
            index: 0,
        }
    }

    pub(crate) fn demand_poll(&self) {
        ETH.ethernet_dma().dmarpdr().write(|w| w.set_rpd(Rpd::POLL));
    }

    /// Get current `RunningState`
    fn running_state(&self) -> RunningState {
        match ETH.ethernet_dma().dmasr().read().rps() {
            //  Reset or Stop Receive Command issued
            Rps::STOPPED => RunningState::Stopped,
            //  Fetching receive transfer descriptor
            Rps::RUNNINGFETCHING => RunningState::Running,
            //  Waiting for receive packet
            Rps::RUNNINGWAITING => RunningState::Running,
            //  Receive descriptor unavailable
            Rps::SUSPENDED => RunningState::Stopped,
            //  Closing receive descriptor
            Rps::_RESERVED_5 => RunningState::Running,
            //  Transferring the receive packet data from receive buffer to host memory
            Rps::RUNNINGWRITING => RunningState::Running,
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
        loop {
            let descriptor = &mut self.descriptors[self.index];
            if !descriptor.available() {
                return None;
            }

            // If packet is invalid, pop it and try again.
            if !descriptor.valid() {
                warn!("invalid packet: {:08x}", descriptor.rdes0.get());
                self.pop_packet();
                continue;
            }

            break;
        }

        let descriptor = &mut self.descriptors[self.index];
        let len = descriptor.packet_len();
        return Some(&mut self.buffers[self.index].0[..len]);
    }

    /// Pop the packet previously returned by `available`.
    pub(crate) fn pop_packet(&mut self) {
        let descriptor = &mut self.descriptors[self.index];
        assert!(descriptor.available());

        self.descriptors[self.index].set_ready(self.buffers[self.index].0.as_mut_ptr());

        self.demand_poll();

        // Increment index.
        self.index += 1;
        if self.index == self.descriptors.len() {
            self.index = 0
        }
    }
}
