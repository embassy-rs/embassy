use core::sync::atomic::{compiler_fence, fence, Ordering};

use vcell::VolatileCell;

use crate::eth::TX_BUFFER_SIZE;
use crate::pac::ETH;

/// Transmit and Receive Descriptor fields
#[allow(dead_code)]
mod tx_consts {
    pub const TXDESC_0_OWN: u32 = 1 << 31;
    pub const TXDESC_0_IOC: u32 = 1 << 30;
    // First segment of frame
    pub const TXDESC_0_FS: u32 = 1 << 28;
    // Last segment of frame
    pub const TXDESC_0_LS: u32 = 1 << 29;
    // Transmit end of ring
    pub const TXDESC_0_TER: u32 = 1 << 21;
    // Second address chained
    pub const TXDESC_0_TCH: u32 = 1 << 20;
    // Error status
    pub const TXDESC_0_ES: u32 = 1 << 15;

    // Transmit buffer size
    pub const TXDESC_1_TBS_SHIFT: usize = 0;
    pub const TXDESC_1_TBS_MASK: u32 = 0x0fff << TXDESC_1_TBS_SHIFT;
}
use tx_consts::*;

use super::Packet;

/// Transmit Descriptor representation
///
/// * tdes0: control
/// * tdes1: buffer lengths
/// * tdes2: data buffer address
/// * tdes3: next descriptor address
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
        (self.tdes0.get() & TXDESC_0_OWN) == 0
    }

    /// Pass ownership to the DMA engine
    fn set_owned(&mut self) {
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        compiler_fence(Ordering::Release);
        self.tdes0.set(self.tdes0.get() | TXDESC_0_OWN);

        // Used to flush the store buffer as fast as possible to make the buffer available for the
        // DMA.
        fence(Ordering::SeqCst);
    }

    fn set_buffer1(&self, buffer: *const u8) {
        self.tdes2.set(buffer as u32);
    }

    fn set_buffer1_len(&self, len: usize) {
        self.tdes1
            .set((self.tdes1.get() & !TXDESC_1_TBS_MASK) | ((len as u32) << TXDESC_1_TBS_SHIFT));
    }

    // points to next descriptor (RCH)
    fn set_buffer2(&self, buffer: *const u8) {
        self.tdes3.set(buffer as u32);
    }

    fn set_end_of_ring(&self) {
        self.tdes0.set(self.tdes0.get() | TXDESC_0_TER);
    }

    // set up as a part fo the ring buffer - configures the tdes
    fn setup(&self, next: Option<&Self>) {
        // Defer this initialization to this function, so we can have `RingEntry` on bss.
        self.tdes0.set(TXDESC_0_TCH | TXDESC_0_IOC | TXDESC_0_FS | TXDESC_0_LS);
        match next {
            Some(next) => self.set_buffer2(next as *const TDes as *const u8),
            None => {
                self.set_buffer2(0 as *const u8);
                self.set_end_of_ring();
            }
        }
    }
}

pub(crate) struct TDesRing<'a> {
    descriptors: &'a mut [TDes],
    buffers: &'a mut [Packet<TX_BUFFER_SIZE>],
    index: usize,
}

impl<'a> TDesRing<'a> {
    /// Initialise this TDesRing. Assume TDesRing is corrupt
    pub(crate) fn new(descriptors: &'a mut [TDes], buffers: &'a mut [Packet<TX_BUFFER_SIZE>]) -> Self {
        assert!(descriptors.len() > 0);
        assert!(descriptors.len() == buffers.len());

        for (i, entry) in descriptors.iter().enumerate() {
            entry.setup(descriptors.get(i + 1));
        }

        // Register txdescriptor start
        ETH.ethernet_dma()
            .dmatdlar()
            .write(|w| w.0 = descriptors.as_ptr() as u32);

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
        let descriptor = &mut self.descriptors[self.index];
        if descriptor.available() {
            Some(&mut self.buffers[self.index].0)
        } else {
            None
        }
    }

    /// Transmit the packet written in a buffer returned by `available`.
    pub(crate) fn transmit(&mut self, len: usize) {
        let descriptor = &mut self.descriptors[self.index];
        assert!(descriptor.available());

        descriptor.set_buffer1(self.buffers[self.index].0.as_ptr());
        descriptor.set_buffer1_len(len);

        descriptor.set_owned();

        // Ensure changes to the descriptor are committed before DMA engine sees tail pointer store.
        // This will generate an DMB instruction.
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        // Move the index to the next descriptor
        self.index += 1;
        if self.index == self.descriptors.len() {
            self.index = 0
        }
        // Request the DMA engine to poll the latest tx descriptor
        ETH.ethernet_dma().dmatpdr().modify(|w| w.0 = 1)
    }
}
