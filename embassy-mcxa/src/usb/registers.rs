//! Minimal register and DMA-structure definitions for the NXP "USBHS" controller
//! (a Chipidea/EHCI USB-OTG core) as found on the MCXA5xx (e.g. MCX-A577).
//!
//! The `nxp-pac` crate does not yet expose a register block for this peripheral on
//! MCXA577, so we define the device-mode subset locally here using volatile MMIO
//! access. The layout matches `PERI_USBHS.h` from the NXP MCUXpresso SDK and is the
//! same Chipidea core already described by `nxp-pac` for the i.MX RT parts.
//!
//! When a USB register block is added to `nxp-pac` for this chip, this module can be
//! replaced by the generated types with no change to the driver logic.

// This is a register/structure-definition module; not every accessor or bit
// constant is used by the current driver, but they document the hardware.
#![allow(dead_code)]

use core::ptr::{read_volatile, write_volatile};

/// Base address of the USBHS1 controller (non-secure alias) on MCXA577.
pub(crate) const USBHS1_BASE: usize = 0x4002_E000;
/// Base address of the USBPHY (USBHS1_PHY, non-secure alias) on MCXA577.
pub(crate) const USBPHY_BASE: usize = 0x4002_F000;

/// Register offsets within the USBHS controller (device-mode relevant subset).
mod offset {
    pub const ID: usize = 0x000;
    pub const USBCMD: usize = 0x140;
    pub const USBSTS: usize = 0x144;
    pub const USBINTR: usize = 0x148;
    pub const FRINDEX: usize = 0x14C;
    pub const DEVICEADDR: usize = 0x154;
    pub const ENDPTLISTADDR: usize = 0x158;
    pub const BURSTSIZE: usize = 0x160;
    pub const PORTSC1: usize = 0x184;
    pub const USBMODE: usize = 0x1A8;
    pub const ENDPTSETUPSTAT: usize = 0x1AC;
    pub const ENDPTPRIME: usize = 0x1B0;
    pub const ENDPTFLUSH: usize = 0x1B4;
    pub const ENDPTSTAT: usize = 0x1B8;
    pub const ENDPTCOMPLETE: usize = 0x1BC;
    /// `ENDPTCTRL0`; subsequent endpoint control registers are at `+4*n`.
    pub const ENDPTCTRL0: usize = 0x1C0;
}

// ---- USBCMD bits ----
/// Run/Stop.
pub(crate) const USBCMD_RS: u32 = 1 << 0;
/// Controller reset.
pub(crate) const USBCMD_RST: u32 = 1 << 1;
/// Setup tripwire.
pub(crate) const USBCMD_SUTW: u32 = 1 << 13;
/// Add dTD tripwire.
pub(crate) const USBCMD_ATDTW: u32 = 1 << 14;

// ---- USBSTS / USBINTR bits (shared layout) ----
/// USB interrupt (transaction complete).
pub(crate) const USBSTS_UI: u32 = 1 << 0;
/// USB error interrupt.
pub(crate) const USBSTS_UEI: u32 = 1 << 1;
/// Port change detect.
pub(crate) const USBSTS_PCI: u32 = 1 << 2;
/// USB reset received.
pub(crate) const USBSTS_URI: u32 = 1 << 6;
/// SOF received.
pub(crate) const USBSTS_SRI: u32 = 1 << 7;
/// DCSuspend (suspend) interrupt.
pub(crate) const USBSTS_SLI: u32 = 1 << 8;

// ---- USBMODE ----
/// Controller mode = device.
pub(crate) const USBMODE_CM_DEVICE: u32 = 0b10;
/// Setup lockout off (use tripwire semantics instead).
pub(crate) const USBMODE_SLOM: u32 = 1 << 3;

// ---- PORTSC1 ----
/// Force full-speed (disable high-speed chirp); makes the device enumerate at FS.
pub(crate) const PORTSC1_PFSC: u32 = 1 << 24;
/// Current connect status.
pub(crate) const PORTSC1_CCS: u32 = 1 << 0;
/// Port reset.
pub(crate) const PORTSC1_PR: u32 = 1 << 8;
/// Port speed mask (bits 27:26): 0=FS, 1=LS, 2=HS.
pub(crate) const PORTSC1_PSPD_SHIFT: u32 = 26;
pub(crate) const PORTSC1_PSPD_MASK: u32 = 0b11 << PORTSC1_PSPD_SHIFT;

// ---- DEVICEADDR ----
/// Device address advance: apply the new address after the status stage completes.
pub(crate) const DEVICEADDR_USBADRA: u32 = 1 << 24;
/// Shift for the 7-bit device address.
pub(crate) const DEVICEADDR_USBADR_SHIFT: u32 = 25;

/// Thin volatile accessor over the USBHS controller register file.
#[derive(Clone, Copy)]
pub(crate) struct UsbHs {
    base: usize,
}

impl UsbHs {
    /// Create an accessor for the USBHS controller at `base`.
    ///
    /// # Safety
    /// `base` must be the address of a USBHS register block and the caller must
    /// ensure exclusive access for the duration of use.
    pub(crate) const unsafe fn new(base: usize) -> Self {
        Self { base }
    }

    #[inline(always)]
    fn r(&self, off: usize) -> u32 {
        // SAFETY: `off` is a valid register offset within the block.
        unsafe { read_volatile((self.base + off) as *const u32) }
    }

    #[inline(always)]
    fn w(&self, off: usize, val: u32) {
        // SAFETY: `off` is a valid register offset within the block.
        unsafe { write_volatile((self.base + off) as *mut u32, val) }
    }

    #[inline(always)]
    fn modify(&self, off: usize, f: impl FnOnce(u32) -> u32) {
        let v = self.r(off);
        self.w(off, f(v));
    }

    pub(crate) fn id(&self) -> u32 {
        self.r(offset::ID)
    }

    pub(crate) fn usbcmd(&self) -> u32 {
        self.r(offset::USBCMD)
    }
    pub(crate) fn set_usbcmd(&self, v: u32) {
        self.w(offset::USBCMD, v)
    }
    pub(crate) fn modify_usbcmd(&self, f: impl FnOnce(u32) -> u32) {
        self.modify(offset::USBCMD, f)
    }

    pub(crate) fn usbsts(&self) -> u32 {
        self.r(offset::USBSTS)
    }
    /// Write-1-to-clear the given status bits.
    pub(crate) fn clear_usbsts(&self, bits: u32) {
        self.w(offset::USBSTS, bits)
    }

    pub(crate) fn usbintr(&self) -> u32 {
        self.r(offset::USBINTR)
    }
    pub(crate) fn set_usbintr(&self, v: u32) {
        self.w(offset::USBINTR, v)
    }

    pub(crate) fn set_deviceaddr(&self, v: u32) {
        self.w(offset::DEVICEADDR, v)
    }

    pub(crate) fn set_endptlistaddr(&self, addr: u32) {
        self.w(offset::ENDPTLISTADDR, addr)
    }

    pub(crate) fn set_burstsize(&self, v: u32) {
        self.w(offset::BURSTSIZE, v)
    }

    pub(crate) fn portsc1(&self) -> u32 {
        self.r(offset::PORTSC1)
    }
    pub(crate) fn modify_portsc1(&self, f: impl FnOnce(u32) -> u32) {
        self.modify(offset::PORTSC1, f)
    }

    pub(crate) fn set_usbmode(&self, v: u32) {
        self.w(offset::USBMODE, v)
    }

    pub(crate) fn endptsetupstat(&self) -> u32 {
        self.r(offset::ENDPTSETUPSTAT)
    }
    pub(crate) fn clear_endptsetupstat(&self, bits: u32) {
        self.w(offset::ENDPTSETUPSTAT, bits)
    }

    pub(crate) fn endptprime(&self) -> u32 {
        self.r(offset::ENDPTPRIME)
    }
    pub(crate) fn set_endptprime(&self, bits: u32) {
        self.w(offset::ENDPTPRIME, bits)
    }

    pub(crate) fn set_endptflush(&self, bits: u32) {
        self.w(offset::ENDPTFLUSH, bits)
    }
    pub(crate) fn endptflush(&self) -> u32 {
        self.r(offset::ENDPTFLUSH)
    }

    pub(crate) fn endptstat(&self) -> u32 {
        self.r(offset::ENDPTSTAT)
    }

    pub(crate) fn endptcomplete(&self) -> u32 {
        self.r(offset::ENDPTCOMPLETE)
    }
    pub(crate) fn clear_endptcomplete(&self, bits: u32) {
        self.w(offset::ENDPTCOMPLETE, bits)
    }

    pub(crate) fn endptctrl(&self, ep: usize) -> u32 {
        self.r(offset::ENDPTCTRL0 + ep * 4)
    }
    pub(crate) fn set_endptctrl(&self, ep: usize, v: u32) {
        self.w(offset::ENDPTCTRL0 + ep * 4, v)
    }
    pub(crate) fn modify_endptctrl(&self, ep: usize, f: impl FnOnce(u32) -> u32) {
        self.modify(offset::ENDPTCTRL0 + ep * 4, f)
    }
}

// ---- ENDPTCTRLn bit fields (per direction) ----
/// RX endpoint enable.
pub(crate) const EPCTRL_RXE: u32 = 1 << 7;
/// RX data toggle reset.
pub(crate) const EPCTRL_RXR: u32 = 1 << 6;
/// RX endpoint stall.
pub(crate) const EPCTRL_RXS: u32 = 1 << 0;
/// RX endpoint type shift (bits 3:2).
pub(crate) const EPCTRL_RXT_SHIFT: u32 = 2;
/// TX endpoint enable.
pub(crate) const EPCTRL_TXE: u32 = 1 << 23;
/// TX data toggle reset.
pub(crate) const EPCTRL_TXR: u32 = 1 << 22;
/// TX endpoint stall.
pub(crate) const EPCTRL_TXS: u32 = 1 << 16;
/// TX endpoint type shift (bits 19:18).
pub(crate) const EPCTRL_TXT_SHIFT: u32 = 18;

/// Device queue head (dQH). One per endpoint and direction (so `2 * N` total),
/// aligned to 64 bytes and laid out as a contiguous array in RAM.
///
/// Layout per the Chipidea/EHCI device specification.
#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub(crate) struct QueueHead {
    /// Capabilities: max packet length, ZLT, IOS, Mult.
    pub(crate) capabilities: u32,
    /// Current dTD pointer (hardware-owned).
    pub(crate) current_dtd: u32,
    /// dTD overlay area (8 words): next pointer, token, 5 buffer pointers.
    pub(crate) next_dtd: u32,
    pub(crate) token: u32,
    pub(crate) buffer: [u32; 5],
    /// Reserved word to pad the overlay to spec.
    pub(crate) _reserved: u32,
    /// Setup packet buffer (8 bytes) for control endpoints.
    pub(crate) setup: [u32; 2],
    /// Padding to fill out to 64 bytes.
    pub(crate) _pad: [u32; 4],
}

impl QueueHead {
    pub(crate) const fn new() -> Self {
        Self {
            capabilities: 0,
            current_dtd: 0,
            next_dtd: 1, // terminate bit set
            token: 0,
            buffer: [0; 5],
            _reserved: 0,
            setup: [0; 2],
            _pad: [0; 4],
        }
    }
}

/// Capability field: ZLT disable.
///
/// The driver sets this because `embassy-usb` drives zero-length packets and
/// control status stages explicitly; the controller should not synthesize them.
pub(crate) const QH_CAP_ZLT: u32 = 1 << 29;
/// Capability field: interrupt-on-setup (control OUT QH).
pub(crate) const QH_CAP_IOS: u32 = 1 << 15;
/// Shift for the max-packet-length field in the QH capabilities word.
pub(crate) const QH_CAP_MAXLEN_SHIFT: u32 = 16;

/// Device transfer descriptor (dTD), 32-byte aligned.
#[repr(C, align(32))]
#[derive(Clone, Copy)]
pub(crate) struct TransferDescriptor {
    /// Next dTD pointer (bit0 = terminate).
    pub(crate) next: u32,
    /// Token: total bytes, IOC, status.
    pub(crate) token: u32,
    /// Buffer page pointers.
    pub(crate) buffer: [u32; 5],
    /// Padding to 32 bytes.
    pub(crate) _pad: u32,
}

impl TransferDescriptor {
    pub(crate) const fn new() -> Self {
        Self {
            next: 1, // terminate
            token: 0,
            buffer: [0; 5],
            _pad: 0,
        }
    }
}

/// dTD next-pointer terminate bit.
pub(crate) const DTD_NEXT_TERMINATE: u32 = 1 << 0;
/// dTD token: total bytes shift (bits 30:16).
pub(crate) const DTD_TOKEN_TOTAL_SHIFT: u32 = 16;
/// dTD token: remaining byte-count mask after shifting by [`DTD_TOKEN_TOTAL_SHIFT`].
pub(crate) const DTD_TOKEN_TOTAL_MASK: u32 = 0x7FFF;
/// dTD token: interrupt on complete.
pub(crate) const DTD_TOKEN_IOC: u32 = 1 << 15;
/// dTD token: active status bit.
pub(crate) const DTD_TOKEN_ACTIVE: u32 = 1 << 7;
/// dTD token: halted status bit.
pub(crate) const DTD_TOKEN_HALTED: u32 = 1 << 6;
/// dTD token: data-buffer error status bit.
pub(crate) const DTD_TOKEN_DATA_BUFFER_ERROR: u32 = 1 << 5;
/// dTD token: transaction error status bit.
pub(crate) const DTD_TOKEN_TRANSACTION_ERROR: u32 = 1 << 3;
/// dTD token: transfer status bits that indicate a completed transfer failed.
pub(crate) const DTD_TOKEN_ERROR_MASK: u32 =
    DTD_TOKEN_HALTED | DTD_TOKEN_DATA_BUFFER_ERROR | DTD_TOKEN_TRANSACTION_ERROR;
/// dTD token mask of error/status bits.
pub(crate) const DTD_TOKEN_STATUS_MASK: u32 = 0xFF;
/// dTD buffer pointer page size. Each dTD carries five page pointers.
pub(crate) const DTD_BUFFER_PAGE_SIZE: u32 = 0x1000;
/// dTD buffer pointer page mask.
pub(crate) const DTD_BUFFER_PAGE_MASK: u32 = DTD_BUFFER_PAGE_SIZE - 1;
