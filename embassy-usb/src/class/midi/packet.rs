//! Packet processing for MIDI 1.0 events.

use core::slice::ChunksExact;

use super::MAX_MIDI_JACKS;

/// Size of one event packet in bytes.
pub const MIDI_PACKET_SIZE: usize = 4;

/// Packet containing the encoded event.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiPacket {
    /// Packet data.
    raw: [u8; MIDI_PACKET_SIZE],
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

impl From<[u8; MIDI_PACKET_SIZE]> for MidiPacket {
    fn from(bytes: [u8; MIDI_PACKET_SIZE]) -> Self {
        Self::new(bytes)
    }
}

impl MidiPacket {
    /// Creates a new packet from bytes.
    pub const fn new(bytes: [u8; MIDI_PACKET_SIZE]) -> Self {
        Self { raw: bytes }
    }

    /// Returns the cable number.
    pub const fn cable_number(&self) -> u8 {
        self.raw[0] >> 4
    }

    /// Returns the Code Index Number from the packet header.
    pub const fn cin(&self) -> u8 {
        self.raw[0] & 0x0F
    }

    #[inline]
    const fn cin_kind(&self) -> CodeIndexNumber {
        CodeIndexNumber::from_nibble(self.cin())
    }

    /// Returns the event bytes, or an empty slice when the packet uses a reserved CIN.
    pub fn event(&self) -> &[u8] {
        let cin = self.cin_kind();
        if cin.is_reserved() {
            &[]
        } else {
            &self.raw[1..1 + cin.event_len()]
        }
    }

    /// Returns a tuple of (cable_no, event).
    pub fn decode(&self) -> (u8, &[u8]) {
        (self.cable_number(), self.event())
    }

    /// Returns a reference to the packet bytes.
    pub const fn as_bytes(&self) -> &[u8; MIDI_PACKET_SIZE] {
        &self.raw
    }

    /// Returns the packet bytes as owned array.
    pub const fn to_bytes(&self) -> [u8; MIDI_PACKET_SIZE] {
        self.raw
    }

    /// Tries to create a packet from an event.
    pub fn try_encode(cable_number: u8, event: &[u8]) -> Result<Self, MidiPacketError> {
        if cable_number >= MAX_MIDI_JACKS {
            return Err(MidiPacketError::InvalidCableNumber);
        }

        let cin = CodeIndexNumber::try_from_event(event)?;
        let event_len = cin.event_len();

        if event.len() != event_len {
            return Err(MidiPacketError::InvalidEventLength);
        }

        let invalid_data = match cin {
            // The first byte is either F0 or a data byte (checked while selecting
            // the CIN), and the final byte is F7. A three-byte ending still has
            // one data byte in the middle that must not have its status bit set.
            CodeIndexNumber::SysexEnds2Bytes => false,
            CodeIndexNumber::SysexEnds3Bytes => event[1] & 0x80 != 0,
            _ => event[1..].iter().any(|byte| byte & 0x80 != 0),
        };
        if invalid_data {
            return Err(MidiPacketError::InvalidEventData);
        }

        let mut raw = [0; 4];
        raw[0] = cable_number << 4 | cin as u8;
        raw[1..1 + event_len].copy_from_slice(&event[..event_len]);

        Ok(Self { raw })
    }

    /// Tries to create a packet from an event with relaxed length checking.
    /// The event slice can be longer than expected, additional bytes are ignored.
    pub fn try_encode_relaxed(cable_number: u8, event: &[u8]) -> Result<Self, MidiPacketError> {
        if cable_number >= MAX_MIDI_JACKS {
            return Err(MidiPacketError::InvalidCableNumber);
        }

        let cin = CodeIndexNumber::try_from_event(event)?;
        let event_len = cin.event_len();

        if event.len() < event_len {
            return Err(MidiPacketError::InvalidEventLength);
        }

        let invalid_data = match cin {
            // The first byte is either F0 or a data byte (checked while selecting
            // the CIN), and the final byte is F7. A three-byte ending still has
            // one data byte in the middle that must not have its status bit set.
            CodeIndexNumber::SysexEnds2Bytes => false,
            CodeIndexNumber::SysexEnds3Bytes => event[1] & 0x80 != 0,
            _ => event[1..].iter().any(|byte| byte & 0x80 != 0),
        };
        if invalid_data {
            return Err(MidiPacketError::InvalidEventData);
        }

        let mut raw = [0; 4];
        raw[0] = cable_number << 4 | cin as u8;
        raw[1..1 + event_len].copy_from_slice(&event[..event_len]);

        Ok(Self { raw })
    }

    /// Checks if the event is part of a SysEx message.
    pub fn is_sysex(&self) -> bool {
        match self.cin_kind() {
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

/// Writer for accumulating MIDI packets in a USB bulk transfer buffer.
pub struct MidiPacketWriter<'a> {
    buf: &'a mut [u8],
    position: usize,
}

impl<'a> MidiPacketWriter<'a> {
    /// Creates a writer that appends packets to `buf`.
    pub const fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, position: 0 }
    }

    /// Appends one packet to the transfer buffer.
    pub fn write(&mut self, packet: MidiPacket) -> Result<(), MidiPacketError> {
        let end = self
            .position
            .checked_add(MIDI_PACKET_SIZE)
            .ok_or(MidiPacketError::BufferOverflow)?;
        let destination = self
            .buf
            .get_mut(self.position..end)
            .ok_or(MidiPacketError::BufferOverflow)?;
        destination.copy_from_slice(packet.as_bytes());
        self.position = end;
        Ok(())
    }

    /// Returns the number of bytes written.
    pub const fn position(&self) -> usize {
        self.position
    }

    /// Returns the populated part of the transfer buffer.
    pub fn into_buf(self) -> &'a mut [u8] {
        &mut self.buf[..self.position]
    }
}

/// Reader for iterating over MIDI packets in a USB bulk transfer.
#[derive(Clone)]
pub struct MidiPacketReader<'a> {
    chunks: ChunksExact<'a, u8>,
}

impl<'a> MidiPacketReader<'a> {
    /// Creates a reader over a non-empty transfer containing complete packets.
    pub fn new(data: &'a [u8]) -> Result<Self, MidiPacketError> {
        if data.is_empty() || !data.len().is_multiple_of(MIDI_PACKET_SIZE) {
            return Err(MidiPacketError::InvalidPacketLength);
        }

        Ok(Self {
            chunks: data.chunks_exact(MIDI_PACKET_SIZE),
        })
    }
}

impl Iterator for MidiPacketReader<'_> {
    type Item = MidiPacket;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.chunks.next()?;
        Some(MidiPacket::new([chunk[0], chunk[1], chunk[2], chunk[3]]))
    }
}

/// Packet errors.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MidiPacketError {
    /// Invalid packet.
    InvalidPacket,

    /// Cable number is out of the allowed range.
    InvalidCableNumber,

    /// Event does not contain any data.
    EmptyEvent,

    /// Status byte of the event is invalid.
    InvalidEventStatus,

    /// One or more data bytes of the event are invalid.
    InvalidEventData,

    /// Event length does not match its status byte.
    InvalidEventLength,

    /// Packet does not have a length of 4 bytes.
    InvalidPacketLength,

    /// Transfer buffer does not have room for another packet.
    BufferOverflow,
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
    const fn event_len(&self) -> usize {
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

    /// Creates a CIN from the low nibble of a USB-MIDI packet header.
    const fn from_nibble(value: u8) -> Self {
        match value & 0x0f {
            0x0 => Self::MiscFunction,
            0x1 => Self::CableEvents,
            0x2 => Self::SystemCommon2Bytes,
            0x3 => Self::SystemCommon3Bytes,
            0x4 => Self::SysexStartsOrContinues,
            0x5 => Self::SystemCommon1Byte,
            0x6 => Self::SysexEnds2Bytes,
            0x7 => Self::SysexEnds3Bytes,
            0x8 => Self::NoteOff,
            0x9 => Self::NoteOn,
            0xa => Self::PolyKeyPress,
            0xb => Self::ControlChange,
            0xc => Self::ProgramChange,
            0xd => Self::ChannelPressure,
            0xe => Self::PitchBendChange,
            _ => Self::SingleByte,
        }
    }

    /// Returns whether the CIN is reserved for future use.
    const fn is_reserved(&self) -> bool {
        matches!(self, Self::MiscFunction | Self::CableEvents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod decode {
        use super::*;

        macro_rules! decode_packet_test {
            ($($id:ident: $value:expr,)*) => {
                $(
                    #[test]
                    fn $id() {
                        let (raw, expected) = $value;
                        let expected = (
                            expected.0,
                            expected.1.as_slice(),
                            expected.2, expected.3,
                            expected.4
                        );
                        let packet = MidiPacket::try_from(raw.as_slice()).unwrap();
                        let decoded = (
                            packet.cable_number(),
                            packet.event(),
                            packet.is_sysex(),
                            packet.is_sysex_start(),
                            packet.is_sysex_end()
                        );
                        assert_eq!(decoded, expected);
                    }
                )*
            }
        }

        decode_packet_test! {
            note_off: ([0x18, 0x80, 60, 64], (1, [0x80, 60, 64], false, false, false)),
            note_on: ([0x49, 0x92, 48, 20], (4, [0x92, 48, 20], false, false, false)),
            poly_key_press: ([0x0A, 0xA0, 15, 74], (0, [0xA0, 15, 74], false, false, false)),
            control_change: ([0x0B, 0xB0, 64, 127], (0, [0xB0, 64, 127], false, false, false)),
            program_change: ([0x0C, 0xC0, 5, 0], (0, [0xC0, 5], false, false, false)),
            channel_pressure: ([0x0D, 0xD0, 85, 0], (0, [0xD0, 85], false, false, false)),
            pitch_bend: ([0x0E, 0xE0, 40, 120], (0, [0xE0, 40, 120], false, false, false)),
            mtc_quarter_frame: ([0x02, 0xF1, 27, 0], (0, [0xF1, 27], false, false, false)),
            song_position_pointer: ([0x03, 0xF2, 38, 17], (0, [0xF2, 38, 17], false, false, false)),
            song_select: ([0x02, 0xF3, 2, 0], (0, [0xF3, 2], false, false, false)),
            tune_request: ([0x05, 0xF6, 0, 0], (0, [0xF6], false, false, false)),
            timing_clock: ([0x0F, 0xF8, 0, 0], (0, [0xF8], false, false, false)),
            tick: ([0x0F, 0xF9, 0, 0], (0, [0xF9], false, false, false)),
            start: ([0x0F, 0xFA, 0, 0], (0, [0xFA], false, false, false)),
            continue_: ([0x0F, 0xFB, 0, 0], (0, [0xFB], false, false, false)),
            stop: ([0x0F, 0xFC, 0, 0], (0, [0xFC], false, false, false)),
            active_sensing: ([0x0F, 0xFE, 0, 0], (0, [0xFE], false, false, false)),
            system_reset: ([0x0F, 0xFF, 0, 0], (0, [0xFF], false, false, false)),
            sysex_starts: ([0x04, 0xF0, 1, 2], (0, [0xF0, 1, 2], true, true, false)),
            sysex_continues_1byte: ([0x0F, 1, 0, 0], (0, [1], true, false, false)),
            sysex_continues_3bytes: ([0x04, 1, 2, 3], (0, [1, 2, 3], true, false, false)),
            sysex_ends_1byte: ([0x05, 0xF7, 0, 0], (0, [0xF7], true, false, true)),
            sysex_ends_2bytes: ([0x06, 1, 0xF7, 0], (0, [1, 0xF7], true, false, true)),
            sysex_ends_3bytes: ([0x07, 1, 2, 0xF7], (0, [1, 2, 0xF7], true, false, true)),
            sysex_2bytes: ([0x06, 0xF0, 0xF7, 0], (0, [0xF0, 0xF7], true, true, true)),
            sysex_3bytes: ([0x07, 0xF0, 1, 0xF7], (0, [0xF0, 1, 0xF7], true, true, true)),
            undefined_f4: ([0x02, 0xF4, 1, 0], (0, [0xF4, 1], false, false, false)),
            undefined_f5: ([0x03, 0xF5, 1, 2], (0, [0xF5, 1, 2], false, false, false)),
            reserved_cable_event: ([0x01, 1, 2, 3], (0, [], false, false, false)),
            empty: ([0x00, 0, 0, 0], (0, [], false, false, false)),
        }
    }

    mod encode {
        use super::*;

        macro_rules! encode_packet_test {
            ($($id:ident: $value:expr,)*) => {
                $(
                    #[test]
                    fn $id() {
                        let ((cable, payload), expected) = $value;
                        let payload = payload.as_slice();
                        let encoded = MidiPacket::try_encode(cable, payload);
                        let expected: Result<[u8; 4], MidiPacketError> = expected;
                        assert_eq!(encoded, expected.map(
                            |v| MidiPacket::try_from(v.as_slice()).unwrap())
                        );
                    }
                )*
            }
        }

        macro_rules! encode_packet_relaxed_test {
            ($($id:ident: $value:expr,)*) => {
                $(
                    #[test]
                    fn $id() {
                        let ((cable, payload), expected) = $value;
                        let payload = payload.as_slice();
                        let encoded = MidiPacket::try_encode_relaxed(cable, payload);
                        let expected: Result<[u8; 4], MidiPacketError> = expected;
                        assert_eq!(encoded, expected.map(
                            |v| MidiPacket::try_from(v.as_slice()).unwrap())
                        );
                    }
                )*
            }
        }

        encode_packet_test! {
            note_off: ((3, [0x80, 33, 75]), Ok([0x38, 0x80, 33, 75])),
            note_on: ((2, [0x96, 67, 14]), Ok([0x29, 0x96, 67, 14])),
            poly_key_press: ((0, [0xA0, 48, 72]), Ok([0x0A, 0xA0, 48, 72])),
            control_change: ((0, [0xB0, 10, 127]), Ok([0x0B, 0xB0, 10, 127])),
            program_change: ((0, [0xC0, 36]), Ok([0x0C, 0xC0, 36, 0])),
            channel_pressure: ((0, [0xD0, 115]), Ok([0x0D, 0xD0, 115, 0])),
            pitch_bend: ((0, [0xE0, 93, 46]), Ok([0x0E, 0xE0, 93, 46])),
            mtc_quarter_frame: ((0, [0xF1, 102]), Ok([0x02, 0xF1, 102, 0])),
            song_position_pointer: ((0, [0xF2, 42, 74]), Ok([0x03, 0xF2, 42, 74])),
            song_select: ((0, [0xF3, 24]), Ok([0x02, 0xF3, 24, 0])),
            tune_request: ((0, [0xF6]), Ok([0x05, 0xF6, 0, 0])),
            timing_clock: ((0, [0xF8]), Ok([0x0F, 0xF8, 0, 0])),
            tick: ((0, [0xF9]), Ok([0x0F, 0xF9, 0, 0])),
            start: ((0, [0xFA]), Ok([0x0F, 0xFA, 0, 0])),
            continue_: ((0, [0xFB]), Ok([0x0F, 0xFB, 0, 0])),
            stop: ((0, [0xFC]), Ok([0x0F, 0xFC, 0, 0])),
            active_sensing: ((0, [0xFE]), Ok([0x0F, 0xFE, 0, 0])),
            system_reset: ((0, [0xFF]), Ok([0x0F, 0xFF, 0, 0])),
            sysex_starts: ((0, [0xF0, 1, 2]), Ok([0x04, 0xF0, 1, 2])),
            sysex_starts_1byte: ((0, [0xF0]), Err(MidiPacketError::InvalidEventLength)),
            sysex_starts_2bytes: ((0, [0xF0, 1]), Err(MidiPacketError::InvalidEventLength)),
            sysex_continues_1byte: ((0, [1]), Ok([0x0F, 1, 0, 0])),
            sysex_continues_2bytes: ((0, [1, 2]), Err(MidiPacketError::InvalidEventLength)),
            sysex_continues_3bytes: ((0, [1, 2, 3]), Ok([0x04, 1, 2, 3])),
            sysex_ends_1byte: ((0, [0xF7]), Ok([0x05, 0xF7, 0, 0])),
            sysex_ends_2bytes: ((0, [1, 0xF7]), Ok([0x06, 1, 0xF7, 0])),
            sysex_ends_3bytes: ((0, [1, 2, 0xF7]), Ok([0x07, 1, 2, 0xF7])),
            sysex_2bytes: ((0, [0xF0, 0xF7]), Ok([0x06, 0xF0, 0xF7, 0])),
            sysex_3bytes: ((0, [0xF0, 1, 0xF7]), Ok([0x07, 0xF0, 1, 0xF7])),
            undefined_f4: ((0, [0xF4]), Err(MidiPacketError::InvalidEventStatus)),
            undefined_f5: ((0, [0xF5]), Err(MidiPacketError::InvalidEventStatus)),
            note_off_missing_1byte: ((3, [0x80, 26]), Err(MidiPacketError::InvalidEventLength)),
            note_off_missing_2bytes: ((3, [0x80]), Err(MidiPacketError::InvalidEventLength)),
            note_off_invalid_data: ((3, [0x80, 33, 128]), Err(MidiPacketError::InvalidEventData)),
            invalid_cable: ((16, [0x90, 33, 75]), Err(MidiPacketError::InvalidCableNumber)),
            empty: ((0, []), Err(MidiPacketError::EmptyEvent)),
        }

        encode_packet_relaxed_test! {
            program_change_relaxed: ((0, [0xC0, 36, 54]), Ok([0x0C, 0xC0, 36, 0])),
            channel_pressure_relaxed: ((0, [0xD0, 115, 27]), Ok([0x0D, 0xD0, 115, 0])),
            mtc_quarter_frame_relaxed: ((0, [0xF1, 102, 46, 7]), Ok([0x02, 0xF1, 102, 0])),
            song_select_relaxed: ((0, [0xF3, 24, 96]), Ok([0x02, 0xF3, 24, 0])),
            tune_request_relaxed: ((0, [0xF6, 67, 72]), Ok([0x05, 0xF6, 0, 0])),
            timing_clock_relaxed: ((0, [0xF8, 38, 126]), Ok([0x0F, 0xF8, 0, 0])),
        }
    }

    #[test]
    fn rejects_invalid_packet_slice_lengths() {
        for raw in [&[][..], &[0; 3], &[0; 5]] {
            assert_eq!(MidiPacket::try_from(raw), Err(MidiPacketError::InvalidPacket));
        }
    }

    #[test]
    fn iterates_complete_packets_from_bulk_transaction() {
        let data = [0x09, 0x90, 60, 100, 0x08, 0x80, 60, 0];
        let packets: heapless::Vec<_, 2> = MidiPacketReader::new(&data).unwrap().collect();
        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0].event(), &[0x90, 60, 100]);
        assert_eq!(packets[1].event(), &[0x80, 60, 0]);
    }

    #[test]
    fn rejects_empty_and_partial_bulk_transactions() {
        assert!(matches!(
            MidiPacketReader::new(&[]),
            Err(MidiPacketError::InvalidPacketLength)
        ));
        assert!(matches!(
            MidiPacketReader::new(&[0; 7]),
            Err(MidiPacketError::InvalidPacketLength)
        ));
    }

    #[test]
    fn writes_packets_to_bulk_transfer_buffer() {
        let mut buffer = [0; 8];
        let mut writer = MidiPacketWriter::new(&mut buffer);
        writer.write(MidiPacket::new([0x09, 0x90, 60, 100])).unwrap();
        writer.write(MidiPacket::new([0x08, 0x80, 60, 0])).unwrap();
        assert_eq!(writer.position(), 8);
        assert_eq!(writer.into_buf(), &[0x09, 0x90, 60, 100, 0x08, 0x80, 60, 0]);
    }

    #[test]
    fn packet_writer_reports_buffer_overflow() {
        let mut buffer = [0; 7];
        let mut writer = MidiPacketWriter::new(&mut buffer);
        writer.write(MidiPacket::new([0x09, 0x90, 60, 100])).unwrap();
        assert_eq!(
            writer.write(MidiPacket::new([0x08, 0x80, 60, 0])),
            Err(MidiPacketError::BufferOverflow)
        );
        assert_eq!(writer.position(), 4);
    }
}
