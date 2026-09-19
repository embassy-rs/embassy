use core::sync::atomic::{Ordering, compiler_fence, fence};

use xarxa_driver::config::PACKET_BUF_SIZE;

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
    #[cfg(feature = "ptp")]
    /// Timestamp available
    pub const RXDESC_4_TSA: u32 = 1 << 14;
}

use rx_consts::*;

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

pub(crate) struct RDesInfo {
    pub(crate) rdes0: u32,
    #[cfg(any(eth_v1b, eth_v1c))]
    rdes4: u32,
}

impl RDesInfo {
    /// Return true if this RDes is acceptable to us.
    /// Checks: frame is complete (FS+LS), no DMA errors, and IP checksums are valid (if applicable).
    pub(crate) const fn valid(&self) -> bool {
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
    pub(crate) const fn available(&self) -> bool {
        self.rdes0 & RXDESC_0_OWN == 0 // Owned by us
    }

    pub(crate) const fn packet_len(&self) -> usize {
        ((self.rdes0 >> RXDESC_0_FL_SHIFT) & RXDESC_0_FL_MASK) as usize
    }

    /// Length of the received frame, FCS excluded.
    pub(crate) const fn frame_len(&self) -> usize {
        let len = self.packet_len();
        // See `Ethernet::new`: the v1a MAC can't strip the FCS itself, so the
        // reported length includes it.
        #[cfg(eth_v1a)]
        let len = len.saturating_sub(4);
        len
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

    pub(crate) fn info(&self) -> RDesInfo {
        RDesInfo {
            rdes0: self.rdes0.get(),
            #[cfg(any(eth_v1b, eth_v1c))]
            rdes4: self.rdes4.get(),
        }
    }

    #[cfg(feature = "ptp")]
    pub(crate) fn timestamp(&self) -> Option<Timestamp> {
        #[cfg(any(eth_v1b, eth_v1c))]
        {
            if self.rdes4.get() & RXDESC_4_TSA == 0 {
                return None;
            }
            let rdes6 = self.rdes6.get();
            let rdes7 = self.rdes7.get();
            if rdes6 == u32::MAX && rdes7 == u32::MAX {
                None
            } else {
                Some(Timestamp::from_seconds_and_nanos(rdes7, rdes6))
            }
        }
        #[cfg(not(any(eth_v1b, eth_v1c)))]
        {
            None
        }
    }

    /// Configures the reception buffer address and length and passed descriptor ownership to the DMA
    #[inline(always)]
    pub(crate) fn set_ready(&self, buf: *mut u8) {
        self.rdes1
            .set(self.rdes1.get() | (PACKET_BUF_SIZE as u32) & RXDESC_1_RBS1_MASK);
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

    pub(crate) fn setup(&self, next: Option<&Self>, buf: *mut u8) {
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

    #[cfg(any(eth_v1b, eth_v1c))]
    // CIC: Checksum Insertion Control (bits 23:22)
    pub const TXDESC_0_CIC_SHIFT: usize = 22;
    #[cfg(any(eth_v1b, eth_v1c))]
    pub const TXDESC_0_CIC_MASK: u32 = 0b11 << TXDESC_0_CIC_SHIFT;
    #[cfg(any(eth_v1b, eth_v1c))]
    // No checksum insertion
    pub const TXDESC_0_CIC_NONE: u32 = 0b00 << TXDESC_0_CIC_SHIFT;
    #[cfg(any(eth_v1b, eth_v1c))]
    // IP header only
    pub const TXDESC_0_CIC_IP: u32 = 0b01 << TXDESC_0_CIC_SHIFT;
    #[cfg(any(eth_v1b, eth_v1c))]
    // IP header + payload (no pseudo)
    pub const TXDESC_0_CIC_IP_PL: u32 = 0b10 << TXDESC_0_CIC_SHIFT;
    #[cfg(any(eth_v1b, eth_v1c))]
    // Full: IP + payload + pseudo-header
    pub const TXDESC_0_CIC_FULL: u32 = 0b11 << TXDESC_0_CIC_SHIFT;
    #[cfg(any(eth_v1a))]
    // Full: IP + payload + pseudo-header
    pub const TXDESC_0_CIC_FULL: u32 = 0;

    // Transmit buffer size
    pub const TXDESC_1_TBS_SHIFT: usize = 0;
    pub const TXDESC_1_TBS_MASK: u32 = 0x0fff << TXDESC_1_TBS_SHIFT;

    #[cfg(any(eth_v1b, eth_v1c))]
    // Transmit Time Stamp Enable
    pub const TXDESC_0_TTSE: u32 = 1 << 25;
    #[cfg(any(eth_v1b, eth_v1c))]
    // Transmit Time Stamp Status (write-back)
    pub const TXDESC_0_TTSS: u32 = 1 << 17;
}
use tx_consts::*;
use vcell::VolatileCell;
#[cfg(feature = "ptp")]
use xarxa_driver::Timestamp;

/// Enhanced Transmit Descriptor representation (8 words, 32 bytes)
///
/// * tdes0: control (OWN, IOC, FS, LS, TER, TCH, CIC, etc.)
/// * tdes1: buffer lengths
/// * tdes2: data buffer address
/// * tdes3: next descriptor address
/// * tdes4: extended status / timestamp control
/// * tdes5: reserved
/// * tdes6: timestamp low
/// * tdes7: timestamp high
#[repr(C)]
pub(crate) struct TDes {
    tdes0: VolatileCell<u32>,
    tdes1: VolatileCell<u32>,
    tdes2: VolatileCell<u32>,
    tdes3: VolatileCell<u32>,
    #[cfg(any(eth_v1b, eth_v1c))]
    tdes4: VolatileCell<u32>,
    #[cfg(any(eth_v1b, eth_v1c))]
    tdes5: VolatileCell<u32>,
    #[cfg(any(eth_v1b, eth_v1c))]
    tdes6: VolatileCell<u32>,
    #[cfg(any(eth_v1b, eth_v1c))]
    tdes7: VolatileCell<u32>,
}

impl TDes {
    pub const fn new() -> Self {
        Self {
            tdes0: VolatileCell::new(0),
            tdes1: VolatileCell::new(0),
            tdes2: VolatileCell::new(0),
            tdes3: VolatileCell::new(0),
            #[cfg(any(eth_v1b, eth_v1c))]
            tdes4: VolatileCell::new(0),
            #[cfg(any(eth_v1b, eth_v1c))]
            tdes5: VolatileCell::new(0),
            #[cfg(any(eth_v1b, eth_v1c))]
            tdes6: VolatileCell::new(0),
            #[cfg(any(eth_v1b, eth_v1c))]
            tdes7: VolatileCell::new(0),
        }
    }

    /// Return true if this TDes is not currently owned by the DMA
    pub(crate) fn available(&self) -> bool {
        (self.tdes0.get() & TXDESC_0_OWN) == 0
    }

    /// Pass ownership to the DMA engine
    pub(crate) fn set_owned(&mut self) {
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::Release);

        compiler_fence(Ordering::Release);
        self.tdes0.set(self.tdes0.get() | TXDESC_0_OWN);

        // Used to flush the store buffer as fast as possible to make the buffer available for the
        // DMA.
        fence(Ordering::SeqCst);
    }

    /// Program the descriptor in read format: buffer address and length.
    pub(crate) fn set_frame(&mut self, buf: *const u8, len: usize) {
        self.tdes2.set(buf as u32);
        self.tdes1
            .set((self.tdes1.get() & !TXDESC_1_TBS_MASK) | ((len as u32) << TXDESC_1_TBS_SHIFT));
    }

    /// Request (or cancel) a transmit timestamp for the frame in this descriptor.
    #[cfg(all(feature = "ptp", any(eth_v1b, eth_v1c)))]
    pub(crate) fn set_timestamp_request(&self, request: bool) {
        if request {
            self.tdes0.set(self.tdes0.get() | TXDESC_0_TTSE);
        } else {
            self.tdes0.set(self.tdes0.get() & !TXDESC_0_TTSE);
        }
    }

    // points to next descriptor (RCH)
    fn set_buffer2(&self, buffer: *const u8) {
        self.tdes3.set(buffer as u32);
    }

    fn set_end_of_ring(&self) {
        self.tdes0.set(self.tdes0.get() | TXDESC_0_TER);
    }

    // set up as a part of the ring buffer - configures the tdes
    pub(crate) fn setup(&self, next: Option<&Self>) {
        // Defer this initialization to this function, so we can have `RingEntry` on bss.
        // Enable full checksum insertion (IP header + TCP/UDP payload + pseudo-header)
        self.tdes0
            .set(TXDESC_0_TCH | TXDESC_0_IOC | TXDESC_0_FS | TXDESC_0_LS | TXDESC_0_CIC_FULL);
        // Clear extended status and timestamp fields
        #[cfg(any(eth_v1b, eth_v1c))]
        self.tdes4.set(0);
        #[cfg(any(eth_v1b, eth_v1c))]
        self.tdes5.set(0);
        #[cfg(any(eth_v1b, eth_v1c))]
        self.tdes6.set(0);
        #[cfg(any(eth_v1b, eth_v1c))]
        self.tdes7.set(0);
        match next {
            Some(next) => self.set_buffer2(next as *const TDes as *const u8),
            None => {
                self.set_buffer2(0 as *const u8);
                self.set_end_of_ring();
            }
        }
    }

    #[cfg(feature = "ptp")]
    pub(crate) fn timestamp(&self) -> Option<Timestamp> {
        #[cfg(any(eth_v1b, eth_v1c))]
        {
            (self.tdes0.get() & TXDESC_0_TTSS != 0)
                .then(|| Timestamp::from_seconds_and_nanos(self.tdes7.get(), self.tdes6.get()))
        }
        #[cfg(not(any(eth_v1b, eth_v1c)))]
        {
            None
        }
    }
}
