use core::sync::atomic::{compiler_fence, fence, Ordering};

use embassy_net::{Packet, PacketBox, PacketBoxExt, PacketBuf};
use stm32_metapac::eth::vals::{DmaomrSr, Rpd, Rps};
use vcell::VolatileCell;

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

/// Receive Descriptor representation
///
/// * rdes0: OWN and Status
/// * rdes1: allocated buffer length
/// * rdes2: data buffer address
/// * rdes3: next descriptor address
#[repr(C)]
struct RDes {
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
    pub fn valid(&self) -> bool {
        // Write-back descriptor is valid if:
        //
        // Contains first buffer of packet AND contains last buf of
        // packet AND no errors
        (self.rdes0.get() & (RXDESC_0_ES | RXDESC_0_FS | RXDESC_0_LS)) == (RXDESC_0_FS | RXDESC_0_LS)
    }

    /// Return true if this RDes is not currently owned by the DMA
    #[inline(always)]
    pub fn available(&self) -> bool {
        self.rdes0.get() & RXDESC_0_OWN == 0 // Owned by us
    }

    /// Configures the reception buffer address and length and passed descriptor ownership to the DMA
    #[inline(always)]
    pub fn set_ready(&mut self, buf_addr: u32, buf_len: usize) {
        self.rdes1.set(self.rdes1.get() | (buf_len as u32) & RXDESC_1_RBS_MASK);
        self.rdes2.set(buf_addr);

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
    fn set_buffer2(&mut self, buffer: *const u8) {
        self.rdes3.set(buffer as u32);
    }

    #[inline(always)]
    fn set_end_of_ring(&mut self) {
        self.rdes1.set(self.rdes1.get() | RXDESC_1_RER);
    }

    #[inline(always)]
    fn packet_len(&self) -> usize {
        ((self.rdes0.get() >> RXDESC_0_FL_SHIFT) & RXDESC_0_FL_MASK) as usize
    }

    pub fn setup(&mut self, next: Option<&Self>) {
        // Defer this initialization to this function, so we can have `RingEntry` on bss.
        self.rdes1.set(self.rdes1.get() | RXDESC_1_RCH);

        match next {
            Some(next) => self.set_buffer2(next as *const _ as *const u8),
            None => {
                self.set_buffer2(0 as *const u8);
                self.set_end_of_ring();
            }
        }
    }
}
/// Running state of the `RxRing`
#[derive(PartialEq, Eq, Debug)]
pub enum RunningState {
    Unknown,
    Stopped,
    Running,
}

impl RunningState {
    /// whether self equals to `RunningState::Running`
    pub fn is_running(&self) -> bool {
        *self == RunningState::Running
    }
}

/// Rx ring of descriptors and packets
///
/// This ring has three major locations that work in lock-step. The DMA will never write to the tail
/// index, so the `read_index` must never pass the tail index. The `next_tail_index` is always 1
/// slot ahead of the real tail index, and it must never pass the `read_index` or it could overwrite
/// a packet still to be passed to the application.
///
///                                                                   nt can't pass r (no alloc)
/// +---+---+---+---+  Read ok       +---+---+---+---+ No Read       +---+---+---+---+
/// |   |   |   |   |  ------------> |   |   |   |   | ------------> |   |   |   |   |
/// +---+---+---+---+  Allocation ok +---+---+---+---+               +---+---+---+---+
///   ^           ^t                   ^t  ^                           ^t  ^
///   |r                                   |r                              |r
///   |nt                                  |nt                             |nt
///
///
/// +---+---+---+---+  Read ok         +---+---+---+---+ Can't read    +---+---+---+---+
/// |   |   |   |   |  ------------>   |   |   |   |   | ------------> |   |   |   |   |
/// +---+---+---+---+  Allocation fail +---+---+---+---+ Allocation ok +---+---+---+---+
///       ^   ^t  ^                              ^t  ^                   ^       ^   ^t
///       |r      |                              |r  |                   |       |r
///               |nt                                |nt                 |nt
///
pub(crate) struct RDesRing<const N: usize> {
    descriptors: [RDes; N],
    buffers: [Option<PacketBox>; N],
    read_index: usize,
    next_tail_index: usize,
}

impl<const N: usize> RDesRing<N> {
    pub const fn new() -> Self {
        const RDES: RDes = RDes::new();
        const BUFFERS: Option<PacketBox> = None;

        Self {
            descriptors: [RDES; N],
            buffers: [BUFFERS; N],
            read_index: 0,
            next_tail_index: 0,
        }
    }

    pub(crate) fn init(&mut self) {
        assert!(N > 1);
        let mut last_index = 0;
        for (index, buf) in self.buffers.iter_mut().enumerate() {
            let pkt = match PacketBox::new(Packet::new()) {
                Some(p) => p,
                None => {
                    if index == 0 {
                        panic!("Could not allocate at least one buffer for Ethernet receiving");
                    } else {
                        break;
                    }
                }
            };
            self.descriptors[index].set_ready(pkt.as_ptr() as u32, pkt.len());
            *buf = Some(pkt);
            last_index = index;
        }
        self.next_tail_index = (last_index + 1) % N;

        // not sure if this is supposed to span all of the descriptor or just those that contain buffers
        {
            let mut previous: Option<&mut RDes> = None;
            for entry in self.descriptors.iter_mut() {
                if let Some(prev) = &mut previous {
                    prev.setup(Some(entry));
                }
                previous = Some(entry);
            }

            if let Some(entry) = &mut previous {
                entry.setup(None);
            }
        }

        // Register txdescriptor start
        // NOTE (unsafe) Used for atomic writes
        unsafe {
            ETH.ethernet_dma()
                .dmardlar()
                .write(|w| w.0 = &self.descriptors as *const _ as u32);
        };
        // We already have fences in `set_owned`, which is called in `setup`

        // Start receive
        unsafe { ETH.ethernet_dma().dmaomr().modify(|w| w.set_sr(DmaomrSr::STARTED)) };

        self.demand_poll();
    }

    fn demand_poll(&self) {
        unsafe { ETH.ethernet_dma().dmarpdr().write(|w| w.set_rpd(Rpd::POLL)) };
    }

    pub(crate) fn on_interrupt(&mut self) {
        // XXX: Do we need to do anything here ? Maybe we should try to advance the tail ptr, but it
        // would soon hit the read ptr anyway, and we will wake smoltcp's stack on the interrupt
        // which should try to pop a packet...
    }

    /// Get current `RunningState`
    fn running_state(&self) -> RunningState {
        match unsafe { ETH.ethernet_dma().dmasr().read().rps() } {
            //  Reset or Stop Receive Command issued
            Rps::STOPPED => RunningState::Stopped,
            //  Fetching receive transfer descriptor
            Rps::RUNNINGFETCHING => RunningState::Running,
            //  Waiting for receive packet
            Rps::RUNNINGWAITING => RunningState::Running,
            //  Receive descriptor unavailable
            Rps::SUSPENDED => RunningState::Stopped,
            //  Closing receive descriptor
            Rps(0b101) => RunningState::Running,
            //  Transferring the receive packet data from receive buffer to host memory
            Rps::RUNNINGWRITING => RunningState::Running,
            _ => RunningState::Unknown,
        }
    }

    pub(crate) fn pop_packet(&mut self) -> Option<PacketBuf> {
        if !self.running_state().is_running() {
            self.demand_poll();
        }
        // Not sure if the contents of the write buffer on the M7 can affects reads, so we are using
        // a DMB here just in case, it also serves as a hint to the compiler that we're syncing the
        // buffer (I think .-.)
        fence(Ordering::SeqCst);

        let read_available = self.descriptors[self.read_index].available();
        let tail_index = (self.next_tail_index + N - 1) % N;

        let pkt = if read_available && self.read_index != tail_index {
            let pkt = self.buffers[self.read_index].take();
            let len = self.descriptors[self.read_index].packet_len();

            assert!(pkt.is_some());
            let valid = self.descriptors[self.read_index].valid();

            self.read_index = (self.read_index + 1) % N;
            if valid {
                pkt.map(|p| p.slice(0..len))
            } else {
                None
            }
        } else {
            None
        };

        // Try to advance the tail_index
        if self.next_tail_index != self.read_index {
            match PacketBox::new(Packet::new()) {
                Some(b) => {
                    let addr = b.as_ptr() as u32;
                    let buffer_len = b.len();
                    self.buffers[self.next_tail_index].replace(b);
                    self.descriptors[self.next_tail_index].set_ready(addr, buffer_len);

                    // "Preceding reads and writes cannot be moved past subsequent writes."
                    fence(Ordering::Release);

                    self.next_tail_index = (self.next_tail_index + 1) % N;
                }
                None => {}
            }
        }
        pkt
    }
}
