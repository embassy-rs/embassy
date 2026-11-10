//! USB MIDI 2.0 Group Terminal Block (GTB) definitions and serialization.

use core::fmt;

/// Class-specific Group Terminal Block descriptor type (USB-MIDI 2.0 Table 5-1).
pub const CS_GR_TRM_BLOCK: u8 = 0x26;
/// Group Terminal Block Header descriptor subtype (USB-MIDI 2.0 Table 5-5).
pub const GR_TRM_BLOCK_HEADER: u8 = 0x01;
/// Group Terminal Block descriptor subtype (USB-MIDI 2.0 Table 5-6).
pub const GR_TRM_BLOCK: u8 = 0x02;

/// Length of the Group Terminal Block Header descriptor in bytes.
pub const GR_TRM_BLOCK_HEADER_LEN: u8 = 5;
/// Length of a single Group Terminal Block descriptor in bytes.
pub const GR_TRM_BLOCK_LEN: u8 = 13;

/// Class-specific endpoint descriptor subtype for MIDI 2.0 (USB-MIDI 2.0 Table 5-4).
pub const MS_GENERAL_2_0: u8 = 0x02;

/// USB-MIDI specification release 2.0, in binary-coded decimal.
pub const MIDI2_VERSION: u16 = 0x0200;

/// Maximum number of Group Terminal Blocks supported by a device function.
pub const MAX_GTBS: usize = 16;
/// Maximum number of member group terminals per block.
pub const MAX_GROUPS_PER_BLOCK: usize = 16;

/// Group Terminal Block error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GtbError {
    /// Provided buffer is too small to serialize or parse descriptors.
    BufferTooSmall,
    /// Descriptor format or header is invalid.
    InvalidDescriptor,
    /// Capacity exceeded.
    Capacity,
}

impl fmt::Display for GtbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferTooSmall => write!(f, "buffer too small for GTB descriptor"),
            Self::InvalidDescriptor => write!(f, "invalid GTB descriptor"),
            Self::Capacity => write!(f, "GTB capacity exceeded"),
        }
    }
}

/// Group Terminal Block Type (USB-MIDI 2.0 Table 5-6).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GroupTerminalBlockType {
    /// Member Group Terminals are both input and output.
    Bidirectional = 0x00,
    /// Member Group Terminals are input only.
    InputOnly = 0x01,
    /// Member Group Terminals are output only.
    OutputOnly = 0x02,
}

impl TryFrom<u8> for GroupTerminalBlockType {
    type Error = GtbError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Bidirectional),
            0x01 => Ok(Self::InputOnly),
            0x02 => Ok(Self::OutputOnly),
            _ => Err(GtbError::InvalidDescriptor),
        }
    }
}

/// Default MIDI Protocol for a Group Terminal Block (USB-MIDI 2.0 Table 5-6).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MidiProtocol {
    /// Protocol unspecified.
    Unspecified = 0x00,
    /// MIDI 1.0 protocol (UMP 32-bit and 64-bit messages without high resolution).
    Midi1_0 = 0x01,
    /// MIDI 2.0 protocol.
    Midi2_0 = 0x02,
}

impl TryFrom<u8> for MidiProtocol {
    type Error = GtbError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Unspecified),
            0x01 => Ok(Self::Midi1_0),
            0x02 => Ok(Self::Midi2_0),
            _ => Err(GtbError::InvalidDescriptor),
        }
    }
}

/// A parsed or constructed Group Terminal Block descriptor entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GroupTerminalBlock {
    /// Unique identifier for this Group Terminal Block (1-based, 1..=32).
    pub block_id: u8,
    /// Block type (Bidirectional, Input-only, or Output-only).
    pub block_type: GroupTerminalBlockType,
    /// The first member Group Terminal associated with this block (0..=15).
    pub first_group: u8,
    /// Number of Group Terminals spanned by this block (1..=16).
    pub num_groups: u8,
    /// Index of string descriptor for this block, or 0 when absent.
    pub string_index: u8,
    /// Default MIDI protocol.
    pub protocol: MidiProtocol,
    /// Maximum input bandwidth in 10.667 kbps units (0 = unspecified).
    pub max_input_bandwidth: u16,
    /// Maximum output bandwidth in 10.667 kbps units (0 = unspecified).
    pub max_output_bandwidth: u16,
}

impl GroupTerminalBlock {
    /// Validate that the group terminal indices are within specification bounds.
    pub const fn is_valid(&self) -> bool {
        self.block_id > 0
            && self.first_group < 16
            && self.num_groups > 0
            && (self.first_group as usize + self.num_groups as usize) <= 16
    }
}

/// Configuration for declaring a Group Terminal Block on a device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GroupTerminalBlockConfig<'d> {
    /// Unique identifier for this Group Terminal Block (1-based, 1..=32).
    pub block_id: u8,
    /// Block type (Bidirectional, Input-only, or Output-only).
    pub block_type: GroupTerminalBlockType,
    /// The first member Group Terminal (0..=15).
    pub first_group: u8,
    /// Number of Group Terminals spanned by this block (1..=16).
    pub num_groups: u8,
    /// Default MIDI protocol.
    pub protocol: MidiProtocol,
    /// Optional friendly name string descriptor.
    pub name: Option<&'d str>,
    /// Maximum input bandwidth (0 = unspecified).
    pub max_input_bandwidth: u16,
    /// Maximum output bandwidth (0 = unspecified).
    pub max_output_bandwidth: u16,
}

impl Default for GroupTerminalBlockConfig<'_> {
    fn default() -> Self {
        Self {
            block_id: 1,
            block_type: GroupTerminalBlockType::Bidirectional,
            first_group: 0,
            num_groups: 1,
            protocol: MidiProtocol::Midi2_0,
            name: None,
            max_input_bandwidth: 0,
            max_output_bandwidth: 0,
        }
    }
}

/// Serialize a list of Group Terminal Blocks into the complete descriptor block
/// (Header descriptor followed by all Block descriptors).
///
/// Returns the total number of bytes written.
pub fn serialize_gtb_descriptors(blocks: &[GroupTerminalBlock], out: &mut [u8]) -> Result<usize, GtbError> {
    let total_len = GR_TRM_BLOCK_HEADER_LEN as usize + blocks.len() * GR_TRM_BLOCK_LEN as usize;
    if out.len() < total_len {
        return Err(GtbError::BufferTooSmall);
    }

    // Write Header descriptor (5 bytes)
    out[0] = GR_TRM_BLOCK_HEADER_LEN;
    out[1] = CS_GR_TRM_BLOCK;
    out[2] = GR_TRM_BLOCK_HEADER;
    let total_len_le = (total_len as u16).to_le_bytes();
    out[3] = total_len_le[0];
    out[4] = total_len_le[1];

    // Write Block descriptors (13 bytes each)
    let mut offset = GR_TRM_BLOCK_HEADER_LEN as usize;
    for block in blocks {
        if !block.is_valid() {
            return Err(GtbError::InvalidDescriptor);
        }
        out[offset] = GR_TRM_BLOCK_LEN;
        out[offset + 1] = CS_GR_TRM_BLOCK;
        out[offset + 2] = GR_TRM_BLOCK;
        out[offset + 3] = block.block_id;
        out[offset + 4] = block.block_type as u8;
        out[offset + 5] = block.first_group;
        out[offset + 6] = block.num_groups;
        out[offset + 7] = block.string_index;
        out[offset + 8] = block.protocol as u8;

        let in_bw_le = block.max_input_bandwidth.to_le_bytes();
        out[offset + 9] = in_bw_le[0];
        out[offset + 10] = in_bw_le[1];

        let out_bw_le = block.max_output_bandwidth.to_le_bytes();
        out[offset + 11] = out_bw_le[0];
        out[offset + 12] = out_bw_le[1];

        offset += GR_TRM_BLOCK_LEN as usize;
    }

    Ok(total_len)
}

/// Parse a raw buffer containing the Group Terminal Block Header and Block descriptors.
pub fn parse_gtb_descriptors<const N: usize>(raw: &[u8]) -> Result<heapless::Vec<GroupTerminalBlock, N>, GtbError> {
    if raw.len() < GR_TRM_BLOCK_HEADER_LEN as usize {
        return Err(GtbError::InvalidDescriptor);
    }

    // Validate Header
    if raw[0] != GR_TRM_BLOCK_HEADER_LEN || raw[1] != CS_GR_TRM_BLOCK || raw[2] != GR_TRM_BLOCK_HEADER {
        return Err(GtbError::InvalidDescriptor);
    }

    let total_len = u16::from_le_bytes([raw[3], raw[4]]) as usize;
    if raw.len() < total_len {
        return Err(GtbError::InvalidDescriptor);
    }

    let mut blocks = heapless::Vec::new();
    let mut offset = GR_TRM_BLOCK_HEADER_LEN as usize;

    while offset + GR_TRM_BLOCK_LEN as usize <= total_len {
        let b = &raw[offset..offset + GR_TRM_BLOCK_LEN as usize];
        if b[0] != GR_TRM_BLOCK_LEN || b[1] != CS_GR_TRM_BLOCK || b[2] != GR_TRM_BLOCK {
            return Err(GtbError::InvalidDescriptor);
        }

        let block = GroupTerminalBlock {
            block_id: b[3],
            block_type: GroupTerminalBlockType::try_from(b[4])?,
            first_group: b[5],
            num_groups: b[6],
            string_index: b[7],
            protocol: MidiProtocol::try_from(b[8])?,
            max_input_bandwidth: u16::from_le_bytes([b[9], b[10]]),
            max_output_bandwidth: u16::from_le_bytes([b[11], b[12]]),
        };

        if !block.is_valid() {
            return Err(GtbError::InvalidDescriptor);
        }

        blocks.push(block).map_err(|_| GtbError::Capacity)?;
        offset += GR_TRM_BLOCK_LEN as usize;
    }

    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gtb_serialization_roundtrip() {
        let blocks = [
            GroupTerminalBlock {
                block_id: 1,
                block_type: GroupTerminalBlockType::Bidirectional,
                first_group: 0,
                num_groups: 2,
                string_index: 3,
                protocol: MidiProtocol::Midi2_0,
                max_input_bandwidth: 0,
                max_output_bandwidth: 0,
            },
            GroupTerminalBlock {
                block_id: 2,
                block_type: GroupTerminalBlockType::InputOnly,
                first_group: 2,
                num_groups: 1,
                string_index: 0,
                protocol: MidiProtocol::Midi1_0,
                max_input_bandwidth: 3125,
                max_output_bandwidth: 0,
            },
        ];

        let mut buf = [0u8; 64];
        let written = serialize_gtb_descriptors(&blocks, &mut buf).unwrap();
        assert_eq!(written, 5 + 13 * 2);

        let parsed: heapless::Vec<GroupTerminalBlock, 4> = parse_gtb_descriptors(&buf[..written]).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], blocks[0]);
        assert_eq!(parsed[1], blocks[1]);
    }
}
