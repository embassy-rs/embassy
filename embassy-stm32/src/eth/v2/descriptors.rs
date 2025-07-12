use core::sync::atomic::{fence, Ordering};

use vcell::VolatileCell;

use crate::eth::{Packet, RX_BUFFER_SIZE, TX_BUFFER_SIZE};
use crate::pac::ETH;

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
}

pub(crate) struct TDesRing<'a> {
    descriptors: &'a mut [TDes],
    buffers: &'a mut [Packet<TX_BUFFER_SIZE>],
    index: usize,
}

impl<'a> TDesRing<'a> {
    /// Initialise this TDesRing. Assume TDesRing is corrupt.
    pub fn new(descriptors: &'a mut [TDes], buffers: &'a mut [Packet<TX_BUFFER_SIZE>]) -> Self {
        assert!(descriptors.len() > 0);
        assert!(descriptors.len() == buffers.len());

        for td in descriptors.iter_mut() {
            *td = TDes::new();
        }

        // Initialize the pointers in the DMA engine. (There will be a memory barrier later
        // before the DMA engine is enabled.)
        let dma = ETH.ethernet_dma();
        dma.dmactx_dlar().write(|w| w.0 = descriptors.as_mut_ptr() as u32);
        dma.dmactx_rlr().write(|w| w.set_tdrl((descriptors.len() as u16) - 1));
        dma.dmactx_dtpr().write(|w| w.0 = 0);

        Self {
            descriptors,
            buffers,
            index: 0,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.descriptors.len()
    }

    /// Return the next available packet buffer for transmitting, or None
    pub(crate) fn available(&mut self) -> Option<&mut [u8]> {
        let d = &mut self.descriptors[self.index];
        if d.available() {
            Some(&mut self.buffers[self.index].0)
        } else {
            None
        }
    }

    /// Transmit the packet written in a buffer returned by `available`.
    pub(crate) fn transmit(&mut self, len: usize) {
        let td = &mut self.descriptors[self.index];
        assert!(td.available());
        assert!(len as u32 <= EMAC_TDES2_B1L);

        // Read format
        td.tdes0.set(self.buffers[self.index].0.as_ptr() as u32);
        td.tdes2.set(len as u32 & EMAC_TDES2_B1L | EMAC_TDES2_IOC);

        // FD: Contains first buffer of packet
        // LD: Contains last buffer of packet
        // Give the DMA engine ownership
        td.tdes3.set(EMAC_DES3_FD | EMAC_DES3_LD | EMAC_DES3_OWN);

        // Ensure changes to the descriptor are committed before DMA engine sees tail pointer store.
        // This will generate an DMB instruction.
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        // signal DMA it can try again.
        // See issue #2129
        ETH.ethernet_dma().dmactx_dtpr().write(|w| w.0 = &td as *const _ as u32);

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
        // packet AND no errors AND not a context descriptor
        self.rdes3.get() & (EMAC_DES3_FD | EMAC_DES3_LD | EMAC_DES3_ES | EMAC_DES3_CTXT)
            == (EMAC_DES3_FD | EMAC_DES3_LD)
    }

    /// Return true if this RDes is not currently owned by the DMA
    #[inline(always)]
    fn available(&self) -> bool {
        self.rdes3.get() & EMAC_DES3_OWN == 0 // Owned by us
    }

    #[inline(always)]
    fn set_ready(&mut self, buf: *mut u8) {
        self.rdes0.set(buf as u32);
        self.rdes3.set(EMAC_RDES3_BUF1V | EMAC_RDES3_IOC | EMAC_DES3_OWN);
    }
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

        for (i, desc) in descriptors.iter_mut().enumerate() {
            *desc = RDes::new();
            desc.set_ready(buffers[i].0.as_mut_ptr());
        }

        let dma = ETH.ethernet_dma();
        dma.dmacrx_dlar().write(|w| w.0 = descriptors.as_mut_ptr() as u32);
        dma.dmacrx_rlr().write(|w| w.set_rdrl((descriptors.len() as u16) - 1));
        dma.dmacrx_dtpr().write(|w| w.0 = 0);

        Self {
            descriptors,
            buffers,
            index: 0,
        }
    }

    /// Get a received packet if any, or None.
    pub(crate) fn available(&mut self) -> Option<&mut [u8]> {
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
        let len = (descriptor.rdes3.get() & EMAC_RDES3_PKTLEN) as usize;
        return Some(&mut self.buffers[self.index].0[..len]);
    }

    /// Pop the packet previously returned by `available`.
    pub(crate) fn pop_packet(&mut self) {
        let rd = &mut self.descriptors[self.index];
        assert!(rd.available());

        rd.set_ready(self.buffers[self.index].0.as_mut_ptr());

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        // signal DMA it can try again.
        // See issue #2129
        ETH.ethernet_dma().dmacrx_dtpr().write(|w| w.0 = &rd as *const _ as u32);

        // Increment index.
        self.index = (self.index + 1) % self.descriptors.len();
    }
}
