//! CCID protocol types, constants, and structures shared between device and host.

/// USB Class code for CCID (Integrated Circuit(s) Cards Interface Device).
pub const USB_CLASS_CCID: u8 = 0x0B;

/// USB Subclass code for CCID.
pub const USB_SUBCLASS_CCID: u8 = 0x00;

/// USB Protocol code for CCID.
pub const USB_PROTOCOL_CCID: u8 = 0x00;

/// `bDescriptorType` of the CCID class-specific functional descriptor.
pub const CCID_FUNCTIONAL_DESCRIPTOR: u8 = 0x21;

/// Length in bytes of the standard CCID functional descriptor (54 bytes).
pub const CCID_FUNCTIONAL_LEN: usize = 54;

/// Offset of `bMaxSlotIndex` within the CCID functional descriptor.
pub const CCID_MAX_SLOT_INDEX: usize = 4;

/// Header length of all CCID messages (10 bytes).
pub const HEADER_LEN: usize = 10;

/// Maximum standard short APDU payload length.
pub const MAX_APDU_LEN: usize = 261;

/// Maximum standard CCID message length (10-byte header + 261-byte APDU).
pub const MAX_CCID_MESSAGE_LEN: usize = HEADER_LEN + MAX_APDU_LEN;

// ── Bulk-OUT: PC-to-RDR Commands (Host to Device) ─────────────────────────────
/// Set communication parameters for a slot.
pub const PC_TO_RDR_SET_PARAMETERS: u8 = 0x61;
/// Power on and reset the smart card (requests ATR).
pub const PC_TO_RDR_ICC_POWER_ON: u8 = 0x62;
/// Power off the smart card.
pub const PC_TO_RDR_ICC_POWER_OFF: u8 = 0x63;
/// Query slot and card status.
pub const PC_TO_RDR_GET_SLOT_STATUS: u8 = 0x65;
/// Secure PIN verification/modification.
pub const PC_TO_RDR_SECURE: u8 = 0x69;
/// Get communication parameters for a slot.
pub const PC_TO_RDR_GET_PARAMETERS: u8 = 0x6C;
/// Reset communication parameters for a slot to defaults.
pub const PC_TO_RDR_RESET_PARAMETERS: u8 = 0x6D;
/// Vendor-specific escape command.
pub const PC_TO_RDR_ESCAPE: u8 = 0x6B;
/// Transfer an APDU block to the smart card.
pub const PC_TO_RDR_XFR_BLOCK: u8 = 0x6F;
/// Abort an ongoing transfer.
pub const PC_TO_RDR_ABORT: u8 = 0x72;

// ── Bulk-IN: RDR-to-PC Responses (Device to Host) ─────────────────────────────
/// Response carrying data (e.g. ATR or APDU response).
pub const RDR_TO_PC_DATA_BLOCK: u8 = 0x80;
/// Response carrying slot and ICC status.
pub const RDR_TO_PC_SLOT_STATUS: u8 = 0x81;
/// Response carrying communication parameters.
pub const RDR_TO_PC_PARAMETERS: u8 = 0x82;
/// Response carrying escape data.
pub const RDR_TO_PC_ESCAPE: u8 = 0x83;
/// Response carrying data rate and clock frequency.
pub const RDR_TO_PC_DATA_RATE_AND_CLOCK_FREQUENCY: u8 = 0x84;

// ── Interrupt-IN Messages (Device to Host) ────────────────────────────────────
/// Slot status change event.
pub const RDR_TO_PC_NOTIFY_SLOT_CHANGE: u8 = 0x50;
/// Hardware error event.
pub const RDR_TO_PC_HARDWARE_ERROR: u8 = 0x51;

// ── Control-EP Requests ───────────────────────────────────────────────────────
/// Abort command on control endpoint.
pub const CCID_REQ_ABORT: u8 = 0x01;
/// Get clock frequencies supported by the reader.
pub const CCID_REQ_GET_CLOCK_FREQUENCIES: u8 = 0x02;
/// Get data rates supported by the reader.
pub const CCID_REQ_GET_DATA_RATES: u8 = 0x03;

// ── Status bit masks (bStatus) ────────────────────────────────────────────────
/// Command executed without error.
pub const CCID_STATUS_CMD_SUCCESS: u8 = 0x00;
/// Command failed (see bError for reason).
pub const CCID_STATUS_CMD_FAILED: u8 = 0x40;
/// Time extension requested.
pub const CCID_STATUS_TIME_EXTENSION: u8 = 0x80;

/// ICC present and active.
pub const CCID_ICC_PRESENT_ACTIVE: u8 = 0x00;
/// ICC present and inactive.
pub const CCID_ICC_PRESENT_INACTIVE: u8 = 0x01;
/// ICC not present.
pub const CCID_ICC_NOT_PRESENT: u8 = 0x02;

// ── Error Codes (bError) ──────────────────────────────────────────────────────
/// No error.
pub const CCID_ERR_NONE: u8 = 0x00;
/// Command not supported.
pub const CCID_ERR_CMD_NOT_SUPPORTED: u8 = 0x00;
/// Slot does not exist.
pub const CCID_ERR_SLOT_NOT_FOUND: u8 = 0x05;
/// Bad ATR response from card.
pub const CCID_ERR_BAD_ATR: u8 = 0x05;
/// ICC mute / unresponsive.
pub const CCID_ERR_ICC_MUTE: u8 = 0xFE;
/// Transfer overrun.
pub const CCID_ERR_XFR_OVERRUN: u8 = 0xFC;
/// Hardware error.
pub const CCID_ERR_HW_ERROR: u8 = 0xFB;

/// Smart-card presence reported by a CCID slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CardState {
    /// A card is present and active.
    PresentActive,
    /// A card is present but inactive.
    PresentInactive,
    /// No card is present.
    NotPresent,
    /// The reader returned the reserved ICC status value.
    Reserved,
}

impl CardState {
    /// Return the 2-bit ICC status code corresponding to this state.
    pub const fn to_icc_status(self) -> u8 {
        match self {
            Self::PresentActive => CCID_ICC_PRESENT_ACTIVE,
            Self::PresentInactive => CCID_ICC_PRESENT_INACTIVE,
            Self::NotPresent => CCID_ICC_NOT_PRESENT,
            Self::Reserved => 0x03,
        }
    }

    /// Decode the 2-bit ICC status code into a [`CardState`].
    pub const fn from_icc_status(val: u8) -> Self {
        match val & 0x03 {
            0 => Self::PresentActive,
            1 => Self::PresentInactive,
            2 => Self::NotPresent,
            _ => Self::Reserved,
        }
    }
}

/// Decoded `RDR_to_PC_SlotStatus` response.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SlotStatus {
    /// Slot number returned by the reader.
    pub slot: u8,
    /// Command sequence returned by the reader.
    pub sequence: u8,
    /// Card presence and activation state.
    pub card_state: CardState,
    /// CCID command-status field (zero means processed without error).
    pub command_status: u8,
    /// CCID error byte.
    pub error: u8,
    /// Reader clock-status byte.
    pub clock_status: u8,
}
