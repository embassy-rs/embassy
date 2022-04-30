use core::sync::atomic::{compiler_fence, fence, Ordering};

use embassy_net::PacketBuf;
use stm32_metapac::eth::vals::St;
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

/// Transmit Descriptor representation
///
/// * tdes0: control
/// * tdes1: buffer lengths
/// * tdes2: data buffer address
/// * tdes3: next descriptor address
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

    fn set_buffer1(&mut self, buffer: *const u8) {
        self.tdes2.set(buffer as u32);
    }

    fn set_buffer1_len(&mut self, len: usize) {
        self.tdes1
            .set((self.tdes1.get() & !TXDESC_1_TBS_MASK) | ((len as u32) << TXDESC_1_TBS_SHIFT));
    }

    // points to next descriptor (RCH)
    fn set_buffer2(&mut self, buffer: *const u8) {
        self.tdes3.set(buffer as u32);
    }

    fn set_end_of_ring(&mut self) {
        self.tdes0.set(self.tdes0.get() | TXDESC_0_TER);
    }

    // set up as a part fo the ring buffer - configures the tdes
    pub fn setup(&mut self, next: Option<&Self>) {
        // Defer this initialization to this function, so we can have `RingEntry` on bss.
        self.tdes0
            .set(TXDESC_0_TCH | TXDESC_0_IOC | TXDESC_0_FS | TXDESC_0_LS);
        match next {
            Some(next) => self.set_buffer2(next as *const TDes as *const u8),
            None => {
                self.set_buffer2(0 as *const u8);
                self.set_end_of_ring();
            }
        }
    }
}

pub(crate) struct TDesRing<const N: usize> {
    descriptors: [TDes; N],
    buffers: [Option<PacketBuf>; N],
    next_entry: usize,
}

impl<const N: usize> TDesRing<N> {
    pub const fn new() -> Self {
        const TDES: TDes = TDes::new();
        const BUFFERS: Option<PacketBuf> = None;

        Self {
            descriptors: [TDES; N],
            buffers: [BUFFERS; N],
            next_entry: 0,
        }
    }

    /// Initialise this TDesRing. Assume TDesRing is corrupt
    ///
    /// The current memory address of the buffers inside this TDesRing
    /// will be stored in the descriptors, so ensure the TDesRing is
    /// not moved after initialisation.
    pub(crate) fn init(&mut self) {
        assert!(N > 0);

        {
            let mut previous: Option<&mut TDes> = None;
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
        self.next_entry = 0;

        // Register txdescriptor start
        // NOTE (unsafe) Used for atomic writes
        unsafe {
            ETH.ethernet_dma()
                .dmatdlar()
                .write(|w| w.0 = &self.descriptors as *const _ as u32);
        }

        // "Preceding reads and writes cannot be moved past subsequent writes."
        #[cfg(feature = "fence")]
        fence(Ordering::Release);

        // We don't need a compiler fence here because all interactions with `Descriptor` are
        // volatiles

        // Start transmission
        unsafe {
            ETH.ethernet_dma()
                .dmaomr()
                .modify(|w| w.set_st(St::STARTED))
        };
    }

    /// Return true if a TDes is available for use
    pub(crate) fn available(&self) -> bool {
        self.descriptors[self.next_entry].available()
    }

    pub(crate) fn transmit(&mut self, pkt: PacketBuf) -> Result<(), Error> {
        if !self.available() {
            return Err(Error::NoBufferAvailable);
        }

        let descriptor = &mut self.descriptors[self.next_entry];

        let pkt_len = pkt.len();
        let address = pkt.as_ptr() as *const u8;

        descriptor.set_buffer1(address);
        descriptor.set_buffer1_len(pkt_len);

        self.buffers[self.next_entry].replace(pkt);

        descriptor.set_owned();

        // Ensure changes to the descriptor are committed before DMA engine sees tail pointer store.
        // This will generate an DMB instruction.
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        // Move the tail pointer (TPR) to the next descriptor
        self.next_entry = (self.next_entry + 1) % N;

        // Request the DMA engine to poll the latest tx descriptor
        unsafe { ETH.ethernet_dma().dmatpdr().modify(|w| w.0 = 1) }
        Ok(())
    }

    pub(crate) fn on_interrupt(&mut self) -> Result<(), Error> {
        let previous = (self.next_entry + N - 1) % N;
        let td = &self.descriptors[previous];

        // DMB to ensure that we are reading an updated value, probably not needed at the hardware
        // level, but this is also a hint to the compiler that we're syncing on the buffer.
        fence(Ordering::SeqCst);

        let tdes0 = td.tdes0.get();

        if tdes0 & TXDESC_0_OWN != 0 {
            // Transmission isn't done yet, probably a receive interrupt that fired this
            return Ok(());
        }

        // Release the buffer
        self.buffers[previous].take();

        if tdes0 & TXDESC_0_ES != 0 {
            Err(Error::TransmissionError)
        } else {
            Ok(())
        }
    }
}
