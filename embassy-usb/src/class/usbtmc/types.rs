//! USB Test & Measurement Class (USBTMC) and USB488 types, headers, and constants.

use crate::driver::EndpointError;

/// USB class code for Application Specific Class (used by USBTMC).
pub const USB_CLASS_APPL_SPEC: u8 = 0xFE;
/// USB subclass code for USB Test and Measurement Class.
pub const USB_SUBCLASS_TEST_MEASUREMENT: u8 = 0x03;
/// USB protocol code for standard USBTMC.
pub const USB_PROTOCOL_USBTMC: u8 = 0x00;
/// USB protocol code for USB488 (USBTMC with IEEE 488.2 / SCPI extensions).
pub const USB_PROTOCOL_USB488: u8 = 0x01;

// Message IDs (USBTMC 1.0 Table 2 & USB488 Table 1)
/// Host sends a device-dependent command message to the device (Bulk-OUT).
pub const MSG_ID_DEV_DEP_MSG_OUT: u8 = 1;
/// Host requests a device-dependent message from the device (Bulk-OUT).
pub const MSG_ID_REQUEST_DEV_DEP_MSG_IN: u8 = 2;
/// Device sends a device-dependent message to the host (Bulk-IN).
pub const MSG_ID_DEV_DEP_MSG_IN: u8 = 2;
/// Vendor-specific message from host to device (Bulk-OUT).
pub const MSG_ID_VENDOR_SPECIFIC_OUT: u8 = 126;
/// Host requests a vendor-specific message from device (Bulk-OUT).
pub const MSG_ID_REQUEST_VENDOR_SPECIFIC_IN: u8 = 127;
/// Vendor-specific message from device to host (Bulk-IN).
pub const MSG_ID_VENDOR_SPECIFIC_IN: u8 = 127;
/// USB488 Trigger message (Bulk-OUT).
pub const MSG_ID_TRIGGER: u8 = 128;

// Class-specific Control Requests (USBTMC 1.0 Table 17 & USB488 Table 9)
/// Abort a Bulk-OUT transfer.
pub const REQ_INITIATE_ABORT_BULK_OUT: u8 = 1;
/// Check status of aborting Bulk-OUT transfer.
pub const REQ_CHECK_ABORT_BULK_OUT_STATUS: u8 = 2;
/// Abort a Bulk-IN transfer.
pub const REQ_INITIATE_ABORT_BULK_IN: u8 = 3;
/// Check status of aborting Bulk-IN transfer.
pub const REQ_CHECK_ABORT_BULK_IN_STATUS: u8 = 4;
/// Clear device input/output buffers and halt Bulk-OUT endpoint.
pub const REQ_INITIATE_CLEAR: u8 = 5;
/// Check status of clear operation.
pub const REQ_CHECK_CLEAR_STATUS: u8 = 6;
/// Get USBTMC and USB488 capabilities (24 bytes).
pub const REQ_GET_CAPABILITIES: u8 = 7;
/// Optional indicator pulse (e.g. to blink an LED or identify instrument).
pub const REQ_INDICATOR_PULSE: u8 = 64;

// USB488 Subclass Requests
/// Read IEEE 488.2 status byte (*STB).
pub const REQ_USB488_READ_STATUS_BYTE: u8 = 128;
/// Remote Enable control.
pub const REQ_USB488_REN_CONTROL: u8 = 160;
/// Go to Local control.
pub const REQ_USB488_GO_TO_LOCAL: u8 = 161;
/// Local Lockout control.
pub const REQ_USB488_LOCAL_LOCKOUT: u8 = 162;

// USB488 Interrupt Notification Identifiers (USB488 Table 13)
/// USB488 Service Request (SRQ) notification sent on Interrupt-IN.
pub const USB488_BNOTIFY1_SRQ: u8 = 0x81;

/// USBTMC status codes returned in control responses (USBTMC 1.0 Table 18 & USB488 Table 12).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum UsbtmcStatus {
    /// Operation succeeded.
    Success = 0x01,
    /// Operation is pending.
    Pending = 0x02,
    /// Operation failed.
    Failed = 0x80,
    /// Transfer not in progress.
    TransferNotInProgress = 0x81,
    /// Split transfer not in progress.
    SplitNotInProgress = 0x82,
    /// Split transfer in progress.
    SplitInProgress = 0x83,
    /// Interrupt-IN endpoint is busy (USB488).
    InterruptInBusy = 0x20,
}

/// Errors that can occur during USBTMC communication.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UsbtmcError {
    /// USB endpoint communication error.
    Endpoint(EndpointError),
    /// Invalid header tag (`bTag == 0` or `bTagInverse != !bTag`).
    InvalidTag,
    /// Packet was shorter than the expected header size.
    PacketTooShort,
    /// Unexpected message ID received.
    UnexpectedMsgId(u8),
    /// User buffer was too small to hold the message payload.
    BufferTooSmall,
    /// The message transfer was truncated or ended early.
    MessageTruncated,
    /// Device is disconnected or endpoint is disabled.
    NotConnected,
}

impl From<EndpointError> for UsbtmcError {
    fn from(e: EndpointError) -> Self {
        Self::Endpoint(e)
    }
}

impl core::fmt::Display for UsbtmcError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Endpoint(e) => write!(f, "Endpoint error: {:?}", e),
            Self::InvalidTag => write!(f, "Invalid USBTMC tag"),
            Self::PacketTooShort => write!(f, "Packet too short"),
            Self::UnexpectedMsgId(id) => write!(f, "Unexpected MsgID: {}", id),
            Self::BufferTooSmall => write!(f, "User buffer too small"),
            Self::MessageTruncated => write!(f, "Message truncated"),
            Self::NotConnected => write!(f, "Not connected"),
        }
    }
}

impl core::error::Error for UsbtmcError {}

/// Calculates the number of padding bytes (0..3) required to align `len` to a 4-byte boundary.
#[inline]
pub const fn padding_bytes(len: usize) -> usize {
    (4 - (len % 4)) % 4
}

/// Base 4-byte header common to all USBTMC bulk messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MsgHeader {
    /// Message type identifier.
    pub msg_id: u8,
    /// Transfer tag (1..=255).
    pub b_tag: u8,
    /// Bitwise NOT of `b_tag`.
    pub b_tag_inverse: u8,
}

impl MsgHeader {
    /// Header size in bytes.
    pub const SIZE: usize = 4;

    /// Validates and parses the 4-byte base header.
    pub fn parse(buf: &[u8; 4]) -> Result<Self, UsbtmcError> {
        let msg_id = buf[0];
        let b_tag = buf[1];
        let b_tag_inverse = buf[2];
        if b_tag == 0 || b_tag_inverse != !b_tag {
            return Err(UsbtmcError::InvalidTag);
        }
        Ok(Self {
            msg_id,
            b_tag,
            b_tag_inverse,
        })
    }

    /// Serializes the 4-byte base header into `buf`.
    pub fn serialize(&self, buf: &mut [u8]) {
        buf[0] = self.msg_id;
        buf[1] = self.b_tag;
        buf[2] = self.b_tag_inverse;
        buf[3] = 0x00;
    }
}

/// 12-byte header for a DEV_DEP_MSG_OUT message (sent by host on Bulk-OUT).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DevDepMsgOutHeader {
    /// Message transfer tag.
    pub b_tag: u8,
    /// Payload size in bytes.
    pub transfer_size: u32,
    /// Whether this transfer contains the end of the message (EOM).
    pub eom: bool,
}

impl DevDepMsgOutHeader {
    /// Total header size in bytes.
    pub const SIZE: usize = 12;

    /// Parses the 12-byte header from a slice.
    pub fn parse(buf: &[u8; 12]) -> Result<Self, UsbtmcError> {
        let base = MsgHeader::parse(buf[0..4].try_into().unwrap())?;
        if base.msg_id != MSG_ID_DEV_DEP_MSG_OUT && base.msg_id != MSG_ID_VENDOR_SPECIFIC_OUT {
            return Err(UsbtmcError::UnexpectedMsgId(base.msg_id));
        }
        let transfer_size = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        let eom = (buf[8] & 0x01) != 0;
        Ok(Self {
            b_tag: base.b_tag,
            transfer_size,
            eom,
        })
    }
}

/// 12-byte header for a REQUEST_DEV_DEP_MSG_IN request (sent by host on Bulk-OUT).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RequestDevDepMsgInHeader {
    /// Request transfer tag.
    pub b_tag: u8,
    /// Maximum response bytes the host expects to receive.
    pub transfer_size: u32,
    /// Whether termination character checking is enabled.
    pub term_char_enabled: bool,
    /// Termination character byte.
    pub term_char: u8,
}

impl RequestDevDepMsgInHeader {
    /// Total header size in bytes.
    pub const SIZE: usize = 12;

    /// Parses the 12-byte header from a slice.
    pub fn parse(buf: &[u8; 12]) -> Result<Self, UsbtmcError> {
        let base = MsgHeader::parse(buf[0..4].try_into().unwrap())?;
        if base.msg_id != MSG_ID_REQUEST_DEV_DEP_MSG_IN && base.msg_id != MSG_ID_REQUEST_VENDOR_SPECIFIC_IN {
            return Err(UsbtmcError::UnexpectedMsgId(base.msg_id));
        }
        let transfer_size = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        let term_char_enabled = (buf[8] & 0x01) != 0;
        let term_char = buf[9];
        Ok(Self {
            b_tag: base.b_tag,
            transfer_size,
            term_char_enabled,
            term_char,
        })
    }
}

/// 12-byte header for a DEV_DEP_MSG_IN response (sent by device on Bulk-IN).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DevDepMsgInHeader {
    /// Echoed host transfer tag.
    pub b_tag: u8,
    /// Payload size in bytes.
    pub transfer_size: u32,
    /// Whether this transfer contains the end of the message (EOM).
    pub eom: bool,
    /// Whether the transfer was terminated due to encountering `term_char`.
    pub using_term_char: bool,
}

impl DevDepMsgInHeader {
    /// Total header size in bytes.
    pub const SIZE: usize = 12;

    /// Serializes the 12-byte header into `buf`.
    pub fn serialize(&self, buf: &mut [u8]) {
        let base = MsgHeader {
            msg_id: MSG_ID_DEV_DEP_MSG_IN,
            b_tag: self.b_tag,
            b_tag_inverse: !self.b_tag,
        };
        base.serialize(&mut buf[0..4]);
        buf[4..8].copy_from_slice(&self.transfer_size.to_le_bytes());
        let mut attr = 0u8;
        if self.eom {
            attr |= 0x01;
        }
        if self.using_term_char {
            attr |= 0x02;
        }
        buf[8] = attr;
        buf[9] = 0;
        buf[10] = 0;
        buf[11] = 0;
    }
}

/// Device and interface capabilities (USBTMC 1.0 Table 37 & USB488 Table 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Capabilities {
    /// Interface is listen-only.
    pub listen_only: bool,
    /// Interface is talk-only.
    pub talk_only: bool,
    /// Interface supports indicator pulse request.
    pub supports_indicator_pulse: bool,
    /// Device can end bulk IN transfer on term char.
    pub can_end_bulk_in_on_term_char: bool,

    // USB488 capabilities
    /// Supports USB488 trigger command.
    pub supports_trigger: bool,
    /// Supports REN, GTL, LLO.
    pub supports_ren_gtl_llo: bool,
    /// Is IEEE 488.2 compliant.
    pub is_488_2: bool,
    /// Supports Device Trigger (DT1).
    pub dt1: bool,
    /// Supports Remote/Local (RL1).
    pub rl1: bool,
    /// Supports Service Requests (SR1) via interrupt endpoint.
    pub sr1: bool,
    /// Supports SCPI commands.
    pub scpi: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            listen_only: false,
            talk_only: false,
            supports_indicator_pulse: true,
            can_end_bulk_in_on_term_char: false,
            supports_trigger: true,
            supports_ren_gtl_llo: false,
            is_488_2: true,
            dt1: false,
            rl1: false,
            sr1: false,
            scpi: true,
        }
    }
}

impl Capabilities {
    /// Encodes the capabilities into the 24-byte USBTMC GET_CAPABILITIES response.
    pub fn to_bytes(&self, usb488: bool) -> [u8; 24] {
        let mut buf = [0u8; 24];
        buf[0] = UsbtmcStatus::Success as u8;
        buf[1] = 0x00; // reserved
        buf[2..4].copy_from_slice(&0x0100u16.to_le_bytes()); // bcdUSBTMC = 1.00

        let mut intfc_caps = 0u8;
        if self.listen_only {
            intfc_caps |= 0x01;
        }
        if self.talk_only {
            intfc_caps |= 0x02;
        }
        if self.supports_indicator_pulse {
            intfc_caps |= 0x04;
        }
        buf[4] = intfc_caps;

        let mut dev_caps = 0u8;
        if self.can_end_bulk_in_on_term_char {
            dev_caps |= 0x01;
        }
        buf[5] = dev_caps;

        // bytes 6..12 are reserved (0)

        if usb488 {
            buf[12..14].copy_from_slice(&0x0100u16.to_le_bytes()); // bcdUSB488 = 1.00

            let mut intfc_caps_488 = 0u8;
            if self.supports_trigger {
                intfc_caps_488 |= 0x01;
            }
            if self.supports_ren_gtl_llo {
                intfc_caps_488 |= 0x02;
            }
            if self.is_488_2 {
                intfc_caps_488 |= 0x04;
            }
            buf[14] = intfc_caps_488;

            let mut dev_caps_488 = 0u8;
            if self.dt1 {
                dev_caps_488 |= 0x01;
            }
            if self.rl1 {
                dev_caps_488 |= 0x02;
            }
            if self.sr1 {
                dev_caps_488 |= 0x04;
            }
            if self.scpi {
                dev_caps_488 |= 0x08;
            }
            buf[15] = dev_caps_488;
        }

        // bytes 16..24 are reserved (0)
        buf
    }
}

/// A received message from the host.
#[derive(Debug, PartialEq, Eq)]
pub struct ReceivedMessage<'a> {
    /// Payload slice written into user's buffer.
    pub payload: &'a [u8],
    /// Whether End of Message (EOM) was asserted.
    pub eom: bool,
    /// Message transfer tag.
    pub tag: u8,
}

/// An event received on the Bulk-OUT endpoint.
#[derive(Debug, PartialEq, Eq)]
pub enum UsbtmcEvent<'a> {
    /// Device-dependent message received from the host (command payload).
    Message(ReceivedMessage<'a>),
    /// Host is requesting a response on Bulk-IN.
    RequestIn(RequestDevDepMsgInHeader),
    /// USB488 Trigger message received.
    Trigger(u8),
    /// Vendor-specific OUT message received.
    VendorSpecific(ReceivedMessage<'a>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_bytes() {
        assert_eq!(padding_bytes(0), 0);
        assert_eq!(padding_bytes(1), 3);
        assert_eq!(padding_bytes(2), 2);
        assert_eq!(padding_bytes(3), 1);
        assert_eq!(padding_bytes(4), 0);
        assert_eq!(padding_bytes(12), 0);
        assert_eq!(padding_bytes(13), 3);
        assert_eq!(padding_bytes(14), 2);
        assert_eq!(padding_bytes(15), 1);
        assert_eq!(padding_bytes(16), 0);
    }

    #[test]
    fn test_msg_header_roundtrip() {
        let hdr = MsgHeader {
            msg_id: MSG_ID_DEV_DEP_MSG_OUT,
            b_tag: 42,
            b_tag_inverse: !42,
        };
        let mut buf = [0u8; 4];
        hdr.serialize(&mut buf);
        assert_eq!(buf, [1, 42, 213, 0]);

        let parsed = MsgHeader::parse(&buf).unwrap();
        assert_eq!(parsed, hdr);
    }

    #[test]
    fn test_msg_header_invalid_tag() {
        // bTag = 0 is invalid
        let buf = [1, 0, 0xFF, 0];
        assert_eq!(MsgHeader::parse(&buf), Err(UsbtmcError::InvalidTag));

        // bTagInverse != !bTag is invalid
        let buf = [1, 42, 42, 0];
        assert_eq!(MsgHeader::parse(&buf), Err(UsbtmcError::InvalidTag));
    }

    #[test]
    fn test_dev_dep_msg_out_header() {
        let mut buf = [0u8; 12];
        buf[0] = MSG_ID_DEV_DEP_MSG_OUT;
        buf[1] = 5;
        buf[2] = !5;
        buf[3] = 0;
        buf[4..8].copy_from_slice(&100u32.to_le_bytes());
        buf[8] = 0x01; // EOM

        let hdr = DevDepMsgOutHeader::parse(&buf).unwrap();
        assert_eq!(hdr.b_tag, 5);
        assert_eq!(hdr.transfer_size, 100);
        assert!(hdr.eom);
    }

    #[test]
    fn test_request_dev_dep_msg_in_header() {
        let mut buf = [0u8; 12];
        buf[0] = MSG_ID_REQUEST_DEV_DEP_MSG_IN;
        buf[1] = 9;
        buf[2] = !9;
        buf[3] = 0;
        buf[4..8].copy_from_slice(&256u32.to_le_bytes());
        buf[8] = 0x01; // TermCharEnabled
        buf[9] = b'\n'; // TermChar

        let hdr = RequestDevDepMsgInHeader::parse(&buf).unwrap();
        assert_eq!(hdr.b_tag, 9);
        assert_eq!(hdr.transfer_size, 256);
        assert!(hdr.term_char_enabled);
        assert_eq!(hdr.term_char, b'\n');
    }

    #[test]
    fn test_dev_dep_msg_in_header_serialize() {
        let hdr = DevDepMsgInHeader {
            b_tag: 7,
            transfer_size: 15,
            eom: true,
            using_term_char: false,
        };
        let mut buf = [0u8; 12];
        hdr.serialize(&mut buf);
        assert_eq!(buf[0], MSG_ID_DEV_DEP_MSG_IN);
        assert_eq!(buf[1], 7);
        assert_eq!(buf[2], !7);
        assert_eq!(buf[3], 0);
        assert_eq!(u32::from_le_bytes(buf[4..8].try_into().unwrap()), 15);
        assert_eq!(buf[8], 0x01); // EOM
    }

    #[test]
    fn test_capabilities_to_bytes() {
        let caps = Capabilities::default();
        let bytes = caps.to_bytes(true);
        assert_eq!(bytes[0], UsbtmcStatus::Success as u8);
        assert_eq!(bytes[2..4], [0x00, 0x01]); // bcdUSBTMC 1.00
        assert_eq!(bytes[12..14], [0x00, 0x01]); // bcdUSB488 1.00
        assert_eq!(bytes[14] & 0x04, 0x04); // is_488_2
        assert_eq!(bytes[15] & 0x08, 0x08); // scpi
    }
}
