//! EHCI device-mode DMA structures for the MCXA5xx USBHS controller.
//!
//! The MMIO register blocks come from `nxp-pac`; the queue heads and transfer
//! descriptors are RAM data structures consumed directly by the controller.

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
/// dTD buffer pointer page size. Each dTD carries five page pointers.
pub(crate) const DTD_BUFFER_PAGE_SIZE: u32 = 0x1000;
/// dTD buffer pointer page mask.
pub(crate) const DTD_BUFFER_PAGE_MASK: u32 = DTD_BUFFER_PAGE_SIZE - 1;
