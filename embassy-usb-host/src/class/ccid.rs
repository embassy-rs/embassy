//! USB Chip/Smart Card Interface Device (CCID) host support.
//!
//! This minimal driver discovers a CCID interface and implements the standard
//! `PC_to_RDR_GetSlotStatus` exchange over its bulk endpoints.

use embassy_usb_driver::host::{PipeError, UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType};

use crate::descriptor::ConfigurationDescriptorChain;
use crate::handler::EnumerationInfo;

const USB_CLASS_CCID: u8 = 0x0b;
const TRANSFER_BULK: u8 = 0x02;
/// `bDescriptorType` of the CCID class-specific (functional) descriptor. Shares its
/// value with the HID descriptor, so its length is what tells the two apart.
const CCID_FUNCTIONAL_DESCRIPTOR: u8 = 0x21;
/// `bLength` of the CCID functional descriptor.
const CCID_FUNCTIONAL_LEN: usize = 54;
/// Offset of `bMaxSlotIndex` within the CCID functional descriptor.
const CCID_MAX_SLOT_INDEX: usize = 4;
const PC_TO_RDR_ICC_POWER_ON: u8 = 0x62;
const PC_TO_RDR_ICC_POWER_OFF: u8 = 0x63;
const PC_TO_RDR_GET_SLOT_STATUS: u8 = 0x65;
const PC_TO_RDR_XFR_BLOCK: u8 = 0x6f;
const RDR_TO_PC_DATA_BLOCK: u8 = 0x80;
const RDR_TO_PC_SLOT_STATUS: u8 = 0x81;
const HEADER_LEN: usize = 10;
const MAX_APDU_LEN: usize = 261;
const MAX_CCID_MESSAGE_LEN: usize = HEADER_LEN + MAX_APDU_LEN;

/// CCID interface and endpoint information.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CcidInfo {
    /// CCID interface number.
    pub interface_number: u8,
    /// Device-to-host bulk endpoint address and maximum packet size.
    pub bulk_in: (u8, u16),
    /// Host-to-device bulk endpoint address and maximum packet size.
    pub bulk_out: (u8, u16),
    /// Number of card slots the reader exposes, from `bMaxSlotIndex + 1`. One when
    /// the interface has no CCID functional descriptor.
    pub slots: u8,
}

/// Find the first CCID interface with bulk IN and OUT endpoints.
///
/// An interface qualifies either by declaring class [`USB_CLASS_CCID`], or by carrying
/// a CCID functional descriptor. The latter matters because readers are commonly
/// shipped under the vendor-specific class while speaking standard CCID.
pub fn find_ccid(config_desc: &[u8]) -> Option<CcidInfo> {
    let cfg = ConfigurationDescriptorChain::try_from_slice(config_desc).ok()?;
    for iface in cfg.iter_interface() {
        // `bMaxSlotIndex + 1`, and proof this is a CCID interface whatever its class.
        let slots = iface.iter_descriptors().find_map(|(_, desc)| {
            (desc.len() >= CCID_FUNCTIONAL_LEN && desc[1] == CCID_FUNCTIONAL_DESCRIPTOR)
                .then(|| desc[CCID_MAX_SLOT_INDEX].saturating_add(1))
        });

        if iface.interface_class != USB_CLASS_CCID && slots.is_none() {
            continue;
        }

        let mut bulk_in = None;
        let mut bulk_out = None;
        for ep in iface.iter_endpoints() {
            if ep.transfer_type() != TRANSFER_BULK {
                continue;
            }
            if ep.is_in() {
                bulk_in = Some((ep.endpoint_address, ep.max_packet_size));
            } else {
                bulk_out = Some((ep.endpoint_address, ep.max_packet_size));
            }
        }

        if let (Some(bulk_in), Some(bulk_out)) = (bulk_in, bulk_out) {
            return Some(CcidInfo {
                interface_number: iface.interface_number,
                bulk_in,
                bulk_out,
                slots: slots.unwrap_or(1),
            });
        }
    }
    None
}

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

/// CCID host error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CcidError {
    /// A USB transfer failed.
    Transfer(PipeError),
    /// No usable CCID interface was found.
    NoInterface,
    /// The controller could not allocate a bulk pipe.
    NoPipe,
    /// The reader rejected the command, carrying its `bStatus` and `bError` bytes.
    CommandFailedWith {
        /// CCID `bStatus`: bits 6-7 hold the command result.
        status: u8,
        /// CCID `bError`: the reason the command was rejected.
        error: u8,
    },
    /// The reader returned a malformed or mismatched response.
    InvalidResponse,
    /// An APDU or response exceeds the driver's fixed transfer buffer.
    MessageTooLong,
}

impl From<PipeError> for CcidError {
    fn from(error: PipeError) -> Self {
        Self::Transfer(error)
    }
}

impl core::fmt::Display for CcidError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_) => write!(f, "CCID USB transfer failed"),
            Self::NoInterface => write!(f, "no usable CCID interface found"),
            Self::NoPipe => write!(f, "no free USB host pipe"),
            Self::InvalidResponse => write!(f, "invalid CCID response"),
            Self::MessageTooLong => write!(f, "CCID message too long"),
            Self::CommandFailedWith { status, error } => {
                write!(f, "CCID command failed (bStatus {status:#04x}, bError {error:#04x})")
            }
        }
    }
}

impl core::error::Error for CcidError {}

/// Host driver for one USB CCID interface.
pub struct CcidHost<'d, A: UsbHostAllocator<'d>> {
    in_pipe: A::Pipe<pipe::Bulk, pipe::In>,
    out_pipe: A::Pipe<pipe::Bulk, pipe::Out>,
    info: CcidInfo,
    sequence: u8,
}

impl<'d, A: UsbHostAllocator<'d>> CcidHost<'d, A> {
    /// Discover the CCID interface and allocate its bulk pipes.
    pub fn new(alloc: &A, config_desc: &[u8], enum_info: &EnumerationInfo) -> Result<Self, CcidError> {
        let info = find_ccid(config_desc).ok_or(CcidError::NoInterface)?;
        let split = enum_info.split();
        let address = enum_info.device_address;
        let endpoint = |(ep, max_packet_size), direction| EndpointInfo {
            addr: EndpointAddress::from_parts((ep & 0x0f) as usize, direction),
            ep_type: EndpointType::Bulk,
            max_packet_size,
            interval_ms: 0,
        };

        let in_pipe = alloc
            .alloc_pipe::<pipe::Bulk, pipe::In>(address, &endpoint(info.bulk_in, UsbDirection::In), split)
            .map_err(|_| CcidError::NoPipe)?;
        let out_pipe = alloc
            .alloc_pipe::<pipe::Bulk, pipe::Out>(address, &endpoint(info.bulk_out, UsbDirection::Out), split)
            .map_err(|_| CcidError::NoPipe)?;

        Ok(Self {
            in_pipe,
            out_pipe,
            info,
            sequence: 0,
        })
    }

    /// Discovered interface and endpoint information.
    pub fn info(&self) -> &CcidInfo {
        &self.info
    }

    async fn slot_command(&mut self, message_type: u8, slot: u8, param: u8) -> Result<SlotStatus, CcidError> {
        let sequence = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        let command = [message_type, 0, 0, 0, 0, slot, sequence, param, 0, 0];
        self.out_pipe.request_out(&command, false).await?;

        let mut response = [0u8; 64];
        let len = self.in_pipe.request_in(&mut response).await?;
        if len < HEADER_LEN
            || response[0] != RDR_TO_PC_SLOT_STATUS
            || response[1..5] != [0, 0, 0, 0]
            || response[5] != slot
            || response[6] != sequence
        {
            return Err(CcidError::InvalidResponse);
        }

        let status = response[7];
        let card_state = match status & 0x03 {
            0 => CardState::PresentActive,
            1 => CardState::PresentInactive,
            2 => CardState::NotPresent,
            _ => CardState::Reserved,
        };
        Ok(SlotStatus {
            slot,
            sequence,
            card_state,
            command_status: (status >> 6) & 0x03,
            error: response[8],
            clock_status: response[9],
        })
    }

    /// Query one reader slot without changing card power or protocol state.
    pub async fn get_slot_status(&mut self, slot: u8) -> Result<SlotStatus, CcidError> {
        self.slot_command(PC_TO_RDR_GET_SLOT_STATUS, slot, 0).await
    }

    /// Power off one ICC slot.
    pub async fn power_off(&mut self, slot: u8) -> Result<SlotStatus, CcidError> {
        self.slot_command(PC_TO_RDR_ICC_POWER_OFF, slot, 0).await
    }

    /// Activate one ICC slot and return its Answer to Reset (ATR).
    ///
    /// `voltage` is the CCID `bPowerSelect` value; zero lets the reader choose.
    pub async fn power_on(&mut self, slot: u8, voltage: u8, atr: &mut [u8]) -> Result<usize, CcidError> {
        let sequence = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        let command = [PC_TO_RDR_ICC_POWER_ON, 0, 0, 0, 0, slot, sequence, voltage, 0, 0];
        self.out_pipe.request_out(&command, false).await?;

        let mut message = [0u8; MAX_CCID_MESSAGE_LEN];
        let len = self.in_pipe.request_in(&mut message).await?;
        if len < HEADER_LEN || message[0] != RDR_TO_PC_DATA_BLOCK || message[5] != slot || message[6] != sequence {
            return Err(CcidError::InvalidResponse);
        }
        if (message[7] >> 6) & 0x03 != 0 {
            return Err(CcidError::CommandFailedWith {
                status: message[7],
                error: message[8],
            });
        }

        let atr_len = u32::from_le_bytes(message[1..5].try_into().unwrap()) as usize;
        if atr_len > atr.len() || HEADER_LEN + atr_len > len {
            return Err(CcidError::MessageTooLong);
        }
        atr[..atr_len].copy_from_slice(&message[HEADER_LEN..HEADER_LEN + atr_len]);
        Ok(atr_len)
    }

    /// Exchange one ISO 7816 APDU using `PC_to_RDR_XfrBlock`.
    ///
    /// Returns the APDU response length, including its trailing SW1/SW2 bytes.
    pub async fn transfer_block(&mut self, slot: u8, apdu: &[u8], response: &mut [u8]) -> Result<usize, CcidError> {
        if apdu.len() > MAX_APDU_LEN {
            return Err(CcidError::MessageTooLong);
        }

        let sequence = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        let mut command = [0u8; MAX_CCID_MESSAGE_LEN];
        command[0] = PC_TO_RDR_XFR_BLOCK;
        command[1..5].copy_from_slice(&(apdu.len() as u32).to_le_bytes());
        command[5] = slot;
        command[6] = sequence;
        command[HEADER_LEN..HEADER_LEN + apdu.len()].copy_from_slice(apdu);
        self.out_pipe
            .request_out(&command[..HEADER_LEN + apdu.len()], false)
            .await?;

        let mut message = [0u8; MAX_CCID_MESSAGE_LEN];
        let len = self.in_pipe.request_in(&mut message).await?;
        if len < HEADER_LEN || message[0] != RDR_TO_PC_DATA_BLOCK || message[5] != slot || message[6] != sequence {
            return Err(CcidError::InvalidResponse);
        }
        if (message[7] >> 6) & 0x03 != 0 {
            return Err(CcidError::CommandFailedWith {
                status: message[7],
                error: message[8],
            });
        }

        let payload_len = u32::from_le_bytes(message[1..5].try_into().unwrap()) as usize;
        if payload_len > response.len() || HEADER_LEN + payload_len > len {
            return Err(CcidError::MessageTooLong);
        }
        response[..payload_len].copy_from_slice(&message[HEADER_LEN..HEADER_LEN + payload_len]);
        Ok(payload_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CCID_CONFIG: &[u8] = &[
        9, 2, 32, 0, 1, 1, 0, 0x80, 50, 9, 4, 0, 0, 2, 0x0b, 0, 0, 0, 7, 5, 0x81, 2, 64, 0, 0, 7, 5, 0x02, 2, 64, 0, 0,
    ];

    #[test]
    fn finds_ccid_bulk_endpoints() {
        let info = find_ccid(CCID_CONFIG).unwrap();
        assert_eq!(info.interface_number, 0);
        assert_eq!(info.bulk_in, (0x81, 64));
        assert_eq!(info.bulk_out, (0x02, 64));
    }
}
