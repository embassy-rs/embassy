use core::sync::atomic::{Ordering, fence};

use vcell::VolatileCell;
#[cfg(feature = "ptp")]
use xarxa_driver::Timestamp;

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
    pub(crate) fn available(&self) -> bool {
        self.tdes3.get() & EMAC_DES3_OWN == 0
    }

    /// Program the descriptor in read format: buffer address, length, and
    /// interrupt-on-completion.
    pub(crate) fn set_frame(&mut self, buf: *const u8, len: usize) {
        assert!(len as u32 <= EMAC_TDES2_B1L);
        self.tdes0.set(buf as u32);
        self.tdes2.set((len as u32) & EMAC_TDES2_B1L | EMAC_TDES2_IOC);
    }

    /// Request (or cancel) a transmit timestamp for the frame in this descriptor.
    ///
    /// `set_frame` rewrites `tdes2` on every transmit, so a `false` request
    /// needs no explicit clear.
    #[cfg(feature = "ptp")]
    pub(crate) fn set_timestamp_request(&mut self, request: bool) {
        if request {
            self.tdes2.set(self.tdes2.get() | EMAC_TDES2_TTSE);
        }
    }

    /// Pass ownership to the DMA engine.
    ///
    /// FD: Contains first buffer of packet
    /// LD: Contains last buffer of packet
    /// CIC_FULL: let the MAC compute and insert the IP/TCP/UDP checksums.
    pub(crate) fn set_owned(&mut self) {
        self.tdes3
            .set(EMAC_DES3_FD | EMAC_DES3_LD | EMAC_DES3_OWN | EMAC_TDES3_CIC_FULL);
    }

    #[cfg(feature = "ptp")]
    pub(crate) fn timestamp(&self) -> Option<Timestamp> {
        (self.tdes3.get() & EMAC_TDES3_TTSS != 0)
            .then(|| Timestamp::from_seconds_and_nanos(self.tdes1.get(), self.tdes0.get()))
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

pub(crate) struct RDesInfo {
    pub(crate) rdes0: u32,
    rdes1: u32,
    rdes3: u32,
}

impl RDesInfo {
    /// Return true if this RDes is acceptable to us
    pub(crate) const fn valid(&self) -> bool {
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
    pub(crate) const fn has_timestamp(&self) -> bool {
        self.rdes3 & EMAC_RDES3_RS1V != 0 && self.rdes1 & EMAC_RDES1_TSA != 0
    }

    /// Return true if this RDes is not currently owned by the DMA
    pub(crate) const fn available(&self) -> bool {
        self.rdes3 & EMAC_DES3_OWN == 0 // Owned by us
    }

    pub(crate) const fn context_available(&self) -> bool {
        self.rdes3 & (EMAC_DES3_OWN | EMAC_DES3_CTXT) == EMAC_DES3_CTXT
    }

    pub(crate) const fn len(&self) -> u32 {
        self.rdes3 & EMAC_RDES3_PKTLEN
    }

    /// Length of the received frame.
    pub(crate) const fn frame_len(&self) -> usize {
        self.len() as usize
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

    pub(crate) fn info(&self) -> RDesInfo {
        RDesInfo {
            rdes0: self.rdes0.get(),
            rdes1: self.rdes1.get(),
            rdes3: self.rdes3.get(),
        }
    }

    pub(crate) fn set_ready(&mut self, buf: *mut u8) {
        self.rdes0.set(buf as u32);
        // The buffer address must be visible to the DMA before it is handed
        // ownership, or it writes the next frame into the previous buffer, which
        // by now belongs to the stack.
        fence(Ordering::Release);
        self.rdes3.set(EMAC_RDES3_BUF1V | EMAC_RDES3_IOC | EMAC_DES3_OWN);
    }

    #[cfg(feature = "ptp")]
    pub(crate) fn context_timestamp(&self) -> Option<Timestamp> {
        let rdes0 = self.rdes0.get();
        let rdes1 = self.rdes1.get();

        if !(rdes0 == u32::MAX && rdes1 == u32::MAX) {
            Some(Timestamp::from_seconds_and_nanos(rdes1, rdes0))
        } else {
            None
        }
    }
}
