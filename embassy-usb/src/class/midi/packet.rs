//! Packet processing for MIDI 1.0 events.

use super::MAX_MIDI_JACKS;

/// Packet containing the encoded event.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiPacket {
    /// Packet data.
    raw: [u8; 4],
}

impl TryFrom<&[u8]> for MidiPacket {
    type Error = MidiPacketError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let Ok(raw) = value.try_into() else {
            return Err(MidiPacketError::InvalidPacket);
        };

        Ok(Self { raw })
    }
}

impl MidiPacket {
    /// Returns the cable number.
    pub fn cable_number(&self) -> u8 {
        self.raw[0] >> 4
    }

    /// Returns a slice to the event bytes with a length suitable for the event.
    pub fn event(&self) -> &[u8] {
        let cin = self.raw[0] & 0x0F;

        match CodeIndexNumber::try_from(cin) {
            Ok(cin) => {
                let size = cin.event_len();
                &self.raw[1..1 + size]
            }

            // Can't really happen because of limited `cin` value range.
            Err(_) => &[],
        }
    }

    /// Returns a tuple of (cable_no, event).
    pub fn decode(&self) -> (u8, &[u8]) {
        (self.cable_number(), self.event())
    }

    /// Returns a reference to the packet bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.raw
    }

    /// Returns the packet bytes as owned array.
    pub fn to_bytes(&self) -> [u8; 4] {
        self.raw
    }

    /// Tries to create a packet from an event.
    pub fn try_encode(cable_number: u8, event: &[u8]) -> Result<Self, MidiPacketError> {
        if cable_number >= MAX_MIDI_JACKS {
            return Err(MidiPacketError::InvalidCableNumber);
        }

        let cin = CodeIndexNumber::try_from_event(event)?;
        let event_len = cin.event_len();

        if event.len() < event_len {
            return Err(MidiPacketError::InvalidEventLength);
        }

        let mut raw = [0; 4];
        raw[0] = cable_number << 4 | cin as u8;
        raw[1..1 + event_len].copy_from_slice(&event[..event_len]);

        Ok(Self { raw })
    }

    /// Checks if the event is part of a SysEx message.
    pub fn is_sysex(&self) -> bool {
        let Ok(cin) = CodeIndexNumber::try_from(self.raw[0] & 0x0F) else {
            return false;
        };

        match cin {
            CodeIndexNumber::SysexStartsOrContinues
            | CodeIndexNumber::SysexEnds2Bytes
            | CodeIndexNumber::SysexEnds3Bytes => true,
            CodeIndexNumber::SystemCommon1Byte => self.raw[1] == 0xF7,
            CodeIndexNumber::SingleByte => self.raw[1] < 0x80,
            _ => false,
        }
    }

    /// Checks if the event is the start of a SysEx message.
    pub fn is_sysex_start(&self) -> bool {
        self.is_sysex() && self.raw[1] == 0xF0
    }

    /// Checks if the event is the end of a SysEx message.
    pub fn is_sysex_end(&self) -> bool {
        self.is_sysex() && (self.raw[1] == 0xF7 || self.raw[2] == 0xF7 || self.raw[3] == 0xF7)
    }
}

/// Packet errors.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MidiPacketError {
    /// Invalid packet.
    InvalidPacket,

    /// Cable number is out of the allowed range.
    InvalidCableNumber,

    /// Code index number cannot be determined from the event.
    InvalidCodeIndexNumber,

    /// Event does not contain any data.
    EmptyEvent,

    /// Status byte of the event is invalid.
    InvalidEventStatus,

    /// Event length does not match its status byte.
    InvalidEventLength,
}

/// Code Index Number (CIN) classifications.
///
/// See specification *USB Device Class Definition for MIDI Devices*
/// Table 4-1 for reference.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
enum CodeIndexNumber {
    /// Miscellaneous function codes. Reserved for future extensions.
    MiscFunction = 0x00,

    /// Cable events. Reserved for future expansion.
    CableEvents = 0x1,

    /// Two-byte System Common messages like MTC, SongSelect, etc.
    SystemCommon2Bytes = 0x2,

    /// Three-byte System Common messages like SPP, etc.
    SystemCommon3Bytes = 0x3,

    /// SysEx starts or continues.
    SysexStartsOrContinues = 0x4,

    /// Single-byte System Common Message or SysEx ends with following single byte.
    SystemCommon1Byte = 0x5,

    /// SysEx ends with following two bytes.
    SysexEnds2Bytes = 0x6,

    /// SysEx ends with following three bytes.
    SysexEnds3Bytes = 0x7,

    /// Note-off.
    NoteOff = 0x8,

    /// Note-on.
    NoteOn = 0x9,

    /// Poly-KeyPress.
    PolyKeyPress = 0xA,

    /// Control Change.
    ControlChange = 0xB,

    /// Program Change.
    ProgramChange = 0xC,

    /// Channel Pressure.
    ChannelPressure = 0xD,

    /// PitchBend Change.
    PitchBendChange = 0xE,

    /// Single Byte.
    SingleByte = 0xF,
}

impl TryFrom<u8> for CodeIndexNumber {
    type Error = MidiPacketError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == CodeIndexNumber::MiscFunction as u8 => Ok(CodeIndexNumber::MiscFunction),
            x if x == CodeIndexNumber::CableEvents as u8 => Ok(CodeIndexNumber::CableEvents),
            x if x == CodeIndexNumber::SystemCommon2Bytes as u8 => Ok(CodeIndexNumber::SystemCommon2Bytes),
            x if x == CodeIndexNumber::SystemCommon3Bytes as u8 => Ok(CodeIndexNumber::SystemCommon3Bytes),
            x if x == CodeIndexNumber::SysexStartsOrContinues as u8 => Ok(CodeIndexNumber::SysexStartsOrContinues),
            x if x == CodeIndexNumber::SystemCommon1Byte as u8 => Ok(CodeIndexNumber::SystemCommon1Byte),
            x if x == CodeIndexNumber::SysexEnds2Bytes as u8 => Ok(CodeIndexNumber::SysexEnds2Bytes),
            x if x == CodeIndexNumber::SysexEnds3Bytes as u8 => Ok(CodeIndexNumber::SysexEnds3Bytes),
            x if x == CodeIndexNumber::NoteOff as u8 => Ok(CodeIndexNumber::NoteOff),
            x if x == CodeIndexNumber::NoteOn as u8 => Ok(CodeIndexNumber::NoteOn),
            x if x == CodeIndexNumber::PolyKeyPress as u8 => Ok(CodeIndexNumber::PolyKeyPress),
            x if x == CodeIndexNumber::ControlChange as u8 => Ok(CodeIndexNumber::ControlChange),
            x if x == CodeIndexNumber::ProgramChange as u8 => Ok(CodeIndexNumber::ProgramChange),
            x if x == CodeIndexNumber::ChannelPressure as u8 => Ok(CodeIndexNumber::ChannelPressure),
            x if x == CodeIndexNumber::PitchBendChange as u8 => Ok(CodeIndexNumber::PitchBendChange),
            x if x == CodeIndexNumber::SingleByte as u8 => Ok(CodeIndexNumber::SingleByte),
            _ => Err(MidiPacketError::InvalidCodeIndexNumber),
        }
    }
}

impl CodeIndexNumber {
    /// Creates a new number from a MIDI event.
    ///
    /// The detection is based on the content and ignores the slice length.
    fn try_from_event(event: &[u8]) -> Result<Self, MidiPacketError> {
        let Some(status) = event.first() else {
            return Err(MidiPacketError::EmptyEvent);
        };

        if *status < 0xF0 {
            match status & 0xF0 {
                0x80 => Ok(Self::NoteOff),
                0x90 => Ok(Self::NoteOn),
                0xA0 => Ok(Self::PolyKeyPress),
                0xB0 => Ok(Self::ControlChange),
                0xC0 => Ok(Self::ProgramChange),
                0xD0 => Ok(Self::ChannelPressure),
                0xE0 => Ok(Self::PitchBendChange),
                _ => {
                    if event.len() > 1 && event[1] == 0xF7 {
                        Ok(Self::SysexEnds2Bytes)
                    } else if event.len() > 2 && event[2] == 0xF7 {
                        Ok(Self::SysexEnds3Bytes)
                    } else if event.len() == 1 {
                        Ok(Self::SingleByte)
                    } else if event.len() > 2 {
                        Ok(Self::SysexStartsOrContinues)
                    } else {
                        Err(MidiPacketError::InvalidEventLength)
                    }
                }
            }
        } else {
            match status {
                0xF0 => {
                    if event.len() > 1 && event[1] == 0xF7 {
                        Ok(Self::SysexEnds2Bytes)
                    } else if event.len() > 2 && event[2] == 0xF7 {
                        Ok(Self::SysexEnds3Bytes)
                    } else if event.len() > 2 {
                        Ok(Self::SysexStartsOrContinues)
                    } else {
                        Err(MidiPacketError::InvalidEventLength)
                    }
                }
                0xF1 | 0xF3 => Ok(Self::SystemCommon2Bytes),
                0xF2 => Ok(Self::SystemCommon3Bytes),
                0xF6 | 0xF7 => Ok(Self::SystemCommon1Byte),
                0xF8 | 0xF9 | 0xFA | 0xFB | 0xFC | 0xFE | 0xFF => Ok(Self::SingleByte),
                _ => Err(MidiPacketError::InvalidEventStatus),
            }
        }
    }

    /// Returns the length of the event in bytes.
    fn event_len(&self) -> usize {
        match self {
            Self::SystemCommon1Byte | Self::SingleByte => 1,
            Self::SystemCommon2Bytes | Self::SysexEnds2Bytes | Self::ProgramChange | Self::ChannelPressure => 2,
            Self::SystemCommon3Bytes
            | Self::SysexEnds3Bytes
            | Self::SysexStartsOrContinues
            | Self::NoteOff
            | Self::NoteOn
            | Self::PolyKeyPress
            | Self::ControlChange
            | Self::PitchBendChange => 3,

            // These variants are reserved for future use.
            // We assume the maximum length of 3 bytes so that no data can get lost.
            Self::MiscFunction | Self::CableEvents => 3,
        }
    }
}
