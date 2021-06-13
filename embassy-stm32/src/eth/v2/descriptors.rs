use core::sync::atomic::{fence, Ordering};

use embassy_net::{Packet, PacketBox, PacketBoxExt, PacketBuf};
use vcell::VolatileCell;

use crate::pac::ETH;

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    NoBufferAvailable,
    // TODO: Break down this error into several others
    TransmissionError,
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
    pub const EMAC_TDES2_B1L: u32 = 0x0000_3FFF;

    pub const EMAC_RDES3_IOC: u32 = 0x4000_0000;
    pub const EMAC_RDES3_PL: u32 = 0x0000_7FFF;
    pub const EMAC_RDES3_BUF1V: u32 = 0x0100_0000;
    pub const EMAC_RDES3_PKTLEN: u32 = 0x0000_7FFF;
}
use emac_consts::*;

/// Transmit Descriptor representation
///
/// * tdes0: transmit buffer address
/// * tdes1:
/// * tdes2: buffer lengths
/// * tdes3: control and payload/frame length
#[repr(C)]
struct TDes {
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
    pub fn available(&self) -> bool {
        self.tdes3.get() & EMAC_DES3_OWN == 0
    }
}

pub(crate) struct TDesRing<const N: usize> {
    td: [TDes; N],
    buffers: [Option<PacketBuf>; N],
    tdidx: usize,
}

impl<const N: usize> TDesRing<N> {
    pub const fn new() -> Self {
        const TDES: TDes = TDes::new();
        const BUFFERS: Option<PacketBuf> = None;

        Self {
            td: [TDES; N],
            buffers: [BUFFERS; N],
            tdidx: 0,
        }
    }

    /// Initialise this TDesRing. Assume TDesRing is corrupt
    ///
    /// The current memory address of the buffers inside this TDesRing
    /// will be stored in the descriptors, so ensure the TDesRing is
    /// not moved after initialisation.
    pub(crate) fn init(&mut self) {
        assert!(N > 0);

        for td in self.td.iter_mut() {
            *td = TDes::new();
        }
        self.tdidx = 0;

        // Initialize the pointers in the DMA engine. (There will be a memory barrier later
        // before the DMA engine is enabled.)
        // NOTE (unsafe) Used for atomic writes
        unsafe {
            let dma = ETH.ethernet_dma();

            dma.dmactx_dlar()
                .write(|w| w.0 = &self.td as *const _ as u32);
            dma.dmactx_rlr().write(|w| w.set_tdrl((N as u16) - 1));
            dma.dmactx_dtpr()
                .write(|w| w.0 = &self.td[0] as *const _ as u32);
        }
    }

    /// Return true if a TDes is available for use
    pub(crate) fn available(&self) -> bool {
        self.td[self.tdidx].available()
    }

    pub(crate) fn transmit(&mut self, pkt: PacketBuf) -> Result<(), Error> {
        if !self.available() {
            return Err(Error::NoBufferAvailable);
        }
        let x = self.tdidx;
        let td = &mut self.td[x];

        let pkt_len = pkt.len();
        assert!(pkt_len as u32 <= EMAC_TDES2_B1L);
        let address = pkt.as_ptr() as u32;

        // Read format
        td.tdes0.set(address);
        td.tdes2
            .set(pkt_len as u32 & EMAC_TDES2_B1L | EMAC_TDES2_IOC);

        // FD: Contains first buffer of packet
        // LD: Contains last buffer of packet
        // Give the DMA engine ownership
        td.tdes3.set(EMAC_DES3_FD | EMAC_DES3_LD | EMAC_DES3_OWN);

        self.buffers[x].replace(pkt);

        // Ensure changes to the descriptor are committed before DMA engine sees tail pointer store.
        // This will generate an DMB instruction.
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        // Move the tail pointer (TPR) to the next descriptor
        let x = (x + 1) % N;
        // NOTE(unsafe) Atomic write
        unsafe {
            ETH.ethernet_dma()
                .dmactx_dtpr()
                .write(|w| w.0 = &self.td[x] as *const _ as u32);
        }
        self.tdidx = x;
        Ok(())
    }

    pub(crate) fn on_interrupt(&mut self) -> Result<(), Error> {
        let previous = (self.tdidx + N - 1) % N;
        let td = &self.td[previous];

        // DMB to ensure that we are reading an updated value, probably not needed at the hardware
        // level, but this is also a hint to the compiler that we're syncing on the buffer.
        fence(Ordering::SeqCst);

        let tdes3 = td.tdes3.get();

        if tdes3 & EMAC_DES3_OWN != 0 {
            // Transmission isn't done yet, probably a receive interrupt that fired this
            return Ok(());
        }
        assert!(tdes3 & EMAC_DES3_CTXT == 0);

        // Release the buffer
        self.buffers[previous].take();

        if tdes3 & EMAC_DES3_ES != 0 {
            Err(Error::TransmissionError)
        } else {
            Ok(())
        }
    }
}

/// Receive Descriptor representation
///
/// * rdes0: recieve buffer address
/// * rdes1:
/// * rdes2:
/// * rdes3: OWN and Status
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
        // packet AND no errors AND not a context descriptor
        self.rdes3.get() & (EMAC_DES3_FD | EMAC_DES3_LD | EMAC_DES3_ES | EMAC_DES3_CTXT)
            == (EMAC_DES3_FD | EMAC_DES3_LD)
    }

    /// Return true if this RDes is not currently owned by the DMA
    #[inline(always)]
    pub fn available(&self) -> bool {
        self.rdes3.get() & EMAC_DES3_OWN == 0 // Owned by us
    }

    #[inline(always)]
    pub fn set_ready(&mut self, buf_addr: u32) {
        self.rdes0.set(buf_addr);
        self.rdes3
            .set(EMAC_RDES3_BUF1V | EMAC_RDES3_IOC | EMAC_DES3_OWN);
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
    rd: [RDes; N],
    buffers: [Option<PacketBox>; N],
    read_idx: usize,
    next_tail_idx: usize,
}

impl<const N: usize> RDesRing<N> {
    pub const fn new() -> Self {
        const RDES: RDes = RDes::new();
        const BUFFERS: Option<PacketBox> = None;

        Self {
            rd: [RDES; N],
            buffers: [BUFFERS; N],
            read_idx: 0,
            next_tail_idx: 0,
        }
    }

    pub(crate) fn init(&mut self) {
        assert!(N > 1);

        for desc in self.rd.iter_mut() {
            *desc = RDes::new();
        }

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
            let addr = pkt.as_ptr() as u32;
            *buf = Some(pkt);
            self.rd[index].set_ready(addr);
            last_index = index;
        }
        self.next_tail_idx = (last_index + 1) % N;

        unsafe {
            let dma = ETH.ethernet_dma();

            dma.dmacrx_dlar().write(|w| w.0 = self.rd.as_ptr() as u32);
            dma.dmacrx_rlr().write(|w| w.set_rdrl((N as u16) - 1));

            // We manage to allocate all buffers, set the index to the last one, that means
            // that the DMA won't consider the last one as ready, because it (unfortunately)
            // stops at the tail ptr and wraps at the end of the ring, which means that we
            // can't tell it to stop after the last buffer.
            let tail_ptr = &self.rd[last_index] as *const _ as u32;
            fence(Ordering::Release);

            dma.dmacrx_dtpr().write(|w| w.0 = tail_ptr);
        }
    }

    pub(crate) fn on_interrupt(&mut self) {
        // XXX: Do we need to do anything here ? Maybe we should try to advance the tail ptr, but it
        // would soon hit the read ptr anyway, and we will wake smoltcp's stack on the interrupt
        // which should try to pop a packet...
    }

    pub(crate) fn pop_packet(&mut self) -> Option<PacketBuf> {
        // Not sure if the contents of the write buffer on the M7 can affects reads, so we are using
        // a DMB here just in case, it also serves as a hint to the compiler that we're syncing the
        // buffer (I think .-.)
        fence(Ordering::SeqCst);

        let read_available = self.rd[self.read_idx].available();
        let tail_index = (self.next_tail_idx + N - 1) % N;

        let pkt = if read_available && self.read_idx != tail_index {
            let pkt = self.buffers[self.read_idx].take();
            let len = (self.rd[self.read_idx].rdes3.get() & EMAC_RDES3_PKTLEN) as usize;

            assert!(pkt.is_some());
            let valid = self.rd[self.read_idx].valid();

            self.read_idx = (self.read_idx + 1) % N;
            if valid {
                pkt.map(|p| p.slice(0..len))
            } else {
                None
            }
        } else {
            None
        };

        // Try to advance the tail_idx
        if self.next_tail_idx != self.read_idx {
            match PacketBox::new(Packet::new()) {
                Some(b) => {
                    let addr = b.as_ptr() as u32;
                    self.buffers[self.next_tail_idx].replace(b);
                    self.rd[self.next_tail_idx].set_ready(addr);

                    // "Preceding reads and writes cannot be moved past subsequent writes."
                    fence(Ordering::Release);

                    // NOTE(unsafe) atomic write
                    unsafe {
                        ETH.ethernet_dma()
                            .dmacrx_dtpr()
                            .write(|w| w.0 = &self.rd[self.next_tail_idx] as *const _ as u32);
                    }

                    self.next_tail_idx = (self.next_tail_idx + 1) % N;
                }
                None => {}
            }
        }
        pkt
    }
}

pub struct DescriptorRing<const T: usize, const R: usize> {
    pub(crate) tx: TDesRing<T>,
    pub(crate) rx: RDesRing<R>,
}

impl<const T: usize, const R: usize> DescriptorRing<T, R> {
    pub const fn new() -> Self {
        Self {
            tx: TDesRing::new(),
            rx: RDesRing::new(),
        }
    }

    pub fn init(&mut self) {
        self.tx.init();
        self.rx.init();
    }
}
