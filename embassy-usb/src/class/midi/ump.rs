//! Universal MIDI Packet (UMP) processing for MIDI 2.0.

use core::fmt;

/// Maximum number of 32-bit words in a single UMP packet.
pub const MAX_UMP_WORDS: usize = 4;
/// Maximum number of bytes in a single UMP packet (16 bytes = 128 bits).
pub const MAX_UMP_BYTES: usize = MAX_UMP_WORDS * 4;

/// Universal MIDI Packet error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UmpError {
    /// Provided buffer is too small to contain or serialize the UMP packet.
    BufferTooSmall,
    /// Unexpected end of input while decoding a packet.
    UnexpectedEof,
    /// Invalid packet length or invalid word count.
    InvalidLength,
}

impl fmt::Display for UmpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferTooSmall => write!(f, "buffer too small for UMP packet"),
            Self::UnexpectedEof => write!(f, "unexpected end of input for UMP packet"),
            Self::InvalidLength => write!(f, "invalid UMP packet length"),
        }
    }
}

/// UMP Message Types (MT, 4 bits, Table 1 in Universal MIDI Packet specification).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MessageType {
    /// 0x0: 32-bit Utility Messages (NOP, Timestamps)
    Utility = 0x0,
    /// 0x1: 32-bit System Real Time and System Common Messages
    System = 0x1,
    /// 0x2: 32-bit MIDI 1.0 Channel Voice Messages
    Midi1ChannelVoice = 0x2,
    /// 0x3: 64-bit Data Messages (SysEx 7-bit)
    Data64 = 0x3,
    /// 0x4: 64-bit MIDI 2.0 Channel Voice Messages
    Midi2ChannelVoice = 0x4,
    /// 0x5: 128-bit Data Messages (SysEx 8-bit & Mixed Data Sets)
    Data128 = 0x5,
    /// 0x6: 32-bit Reserved
    Reserved6 = 0x6,
    /// 0x7: 32-bit Reserved
    Reserved7 = 0x7,
    /// 0x8: 64-bit Reserved
    Reserved8 = 0x8,
    /// 0x9: 64-bit Reserved
    Reserved9 = 0x9,
    /// 0xA: 64-bit Reserved
    ReservedA = 0xA,
    /// 0xB: 96-bit Reserved
    ReservedB = 0xB,
    /// 0xC: 96-bit Reserved
    ReservedC = 0xC,
    /// 0xD: 128-bit Flex Data Messages
    FlexData = 0xD,
    /// 0xE: 128-bit Reserved
    ReservedE = 0xE,
    /// 0xF: 128-bit Stream Messages
    Stream = 0xF,
}

impl MessageType {
    /// Return the message type from a 4-bit nibble.
    pub const fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0x0 => Self::Utility,
            0x1 => Self::System,
            0x2 => Self::Midi1ChannelVoice,
            0x3 => Self::Data64,
            0x4 => Self::Midi2ChannelVoice,
            0x5 => Self::Data128,
            0x6 => Self::Reserved6,
            0x7 => Self::Reserved7,
            0x8 => Self::Reserved8,
            0x9 => Self::Reserved9,
            0xA => Self::ReservedA,
            0xB => Self::ReservedB,
            0xC => Self::ReservedC,
            0xD => Self::FlexData,
            0xE => Self::ReservedE,
            _ => Self::Stream,
        }
    }

    /// Number of 32-bit words for this message type.
    pub const fn word_count(self) -> usize {
        match self as u8 {
            0x0 | 0x1 | 0x2 | 0x6 | 0x7 => 1,
            0x3 | 0x4 | 0x8 | 0x9 | 0xA => 2,
            0xB | 0xC => 3,
            0x5 | 0xD | 0xE | 0xF => 4,
            _ => 1,
        }
    }

    /// Number of bytes on the wire for this message type.
    pub const fn byte_count(self) -> usize {
        self.word_count() * 4
    }
}

/// Universal MIDI Packet (UMP).
///
/// Contains between 1 and 4 32-bit words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UmpPacket {
    words: [u32; MAX_UMP_WORDS],
    len: u8,
}

impl UmpPacket {
    /// Create a single-word (32-bit) UMP packet.
    pub const fn from_u32(w0: u32) -> Result<Self, UmpError> {
        Self::checked([w0, 0, 0, 0], 1)
    }

    /// Create a two-word (64-bit) UMP packet.
    pub const fn from_u32_pair(w0: u32, w1: u32) -> Result<Self, UmpError> {
        Self::checked([w0, w1, 0, 0], 2)
    }

    /// Create a three-word (96-bit) UMP packet.
    pub const fn from_u32_triple(w0: u32, w1: u32, w2: u32) -> Result<Self, UmpError> {
        Self::checked([w0, w1, w2, 0], 3)
    }

    /// Create a four-word (128-bit) UMP packet.
    pub const fn from_u32_quad(w0: u32, w1: u32, w2: u32, w3: u32) -> Result<Self, UmpError> {
        Self::checked([w0, w1, w2, w3], 4)
    }

    // The word count is implied by the message type, so a mismatch would desync the stream.
    const fn checked(words: [u32; MAX_UMP_WORDS], len: usize) -> Result<Self, UmpError> {
        if MessageType::from_nibble((words[0] >> 28) as u8).word_count() != len {
            return Err(UmpError::InvalidLength);
        }
        Ok(Self { words, len: len as u8 })
    }

    /// Create a UMP packet from a slice of words, verifying word length against the message type.
    pub fn from_words(words: &[u32]) -> Result<Self, UmpError> {
        if words.is_empty() || words.len() > MAX_UMP_WORDS {
            return Err(UmpError::InvalidLength);
        }

        let mut raw = [0u32; MAX_UMP_WORDS];
        raw[..words.len()].copy_from_slice(words);
        Self::checked(raw, words.len())
    }

    /// Returns the message type of the packet.
    #[inline]
    pub fn message_type(&self) -> MessageType {
        MessageType::from_nibble((self.words[0] >> 28) as u8)
    }

    /// Returns the group index (0..15), or `None` for Utility and Stream messages, which have none.
    #[inline]
    pub fn group(&self) -> Option<u8> {
        match self.message_type() {
            MessageType::Utility | MessageType::Stream => None,
            _ => Some(((self.words[0] >> 24) & 0x0F) as u8),
        }
    }

    /// Whether the packet is accepted by `mask` (bit `n` = group `n`); groupless packets always are.
    #[inline]
    pub fn in_groups(&self, mask: u16) -> bool {
        self.group().is_none_or(|group| mask & (1 << group) != 0)
    }

    /// Returns the status byte of the first word (bits 16..23).
    #[inline]
    pub fn status(&self) -> u8 {
        ((self.words[0] >> 16) & 0xFF) as u8
    }

    /// Returns the channel (0..15), or `None` unless this is a channel voice message.
    #[inline]
    pub fn channel(&self) -> Option<u8> {
        matches!(
            self.message_type(),
            MessageType::Midi1ChannelVoice | MessageType::Midi2ChannelVoice
        )
        .then(|| ((self.words[0] >> 16) & 0x0F) as u8)
    }

    /// Returns the slice of 32-bit words forming this packet.
    #[inline]
    pub fn words(&self) -> &[u32] {
        &self.words[..self.len as usize]
    }

    /// Returns the number of 32-bit words in this packet.
    #[inline]
    pub fn word_count(&self) -> usize {
        self.len as usize
    }

    /// Returns the byte length of this packet when serialized.
    #[inline]
    pub fn byte_count(&self) -> usize {
        self.len as usize * 4
    }

    /// Try to decode a UMP packet from bytes (little-endian 32-bit words).
    ///
    /// Returns the decoded packet and the number of bytes consumed.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<(Self, usize), UmpError> {
        if bytes.len() < 4 {
            return Err(UmpError::UnexpectedEof);
        }

        let w0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let mt = MessageType::from_nibble((w0 >> 28) as u8);
        let word_count = mt.word_count();
        let byte_count = word_count * 4;

        if bytes.len() < byte_count {
            return Err(UmpError::UnexpectedEof);
        }

        let mut words = [0u32; MAX_UMP_WORDS];
        words[0] = w0;

        for i in 1..word_count {
            let offset = i * 4;
            words[i] = u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
        }

        Ok((
            Self {
                words,
                len: word_count as u8,
            },
            byte_count,
        ))
    }

    /// Serialize the UMP packet to bytes in little-endian byte order.
    ///
    /// Returns the number of bytes written.
    pub fn to_bytes(&self, out: &mut [u8]) -> Result<usize, UmpError> {
        let byte_count = self.byte_count();
        if out.len() < byte_count {
            return Err(UmpError::BufferTooSmall);
        }

        for (i, &word) in self.words().iter().enumerate() {
            let bytes = word.to_le_bytes();
            out[i * 4..i * 4 + 4].copy_from_slice(&bytes);
        }

        Ok(byte_count)
    }
}

/// A 32-bit UMP: the word's message type is known to have exactly one word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ump32(u32);

impl Ump32 {
    /// Create a packet from `word`, rejecting message types that need more words.
    pub const fn new(word: u32) -> Result<Self, UmpError> {
        match UmpPacket::from_u32(word) {
            Ok(_) => Ok(Self(word)),
            Err(error) => Err(error),
        }
    }

    /// Returns the packet word.
    pub const fn word(self) -> u32 {
        self.0
    }
}

impl From<Ump32> for UmpPacket {
    fn from(packet: Ump32) -> Self {
        Self {
            words: [packet.0, 0, 0, 0],
            len: 1,
        }
    }
}

/// Zero-copy iterator over Universal MIDI Packets contained within a USB data buffer.
#[derive(Clone, Debug)]
pub struct UmpReader<'a> {
    data: &'a [u8],
    failed: bool,
}

impl<'a> UmpReader<'a> {
    /// Creates a new `UmpReader` from a byte slice.
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data, failed: false }
    }

    /// Returns the unparsed bytes, including a partial packet at the end.
    pub fn remaining(&self) -> &'a [u8] {
        self.data
    }
}

impl<'a> Iterator for UmpReader<'a> {
    type Item = Result<UmpPacket, UmpError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed || self.data.is_empty() {
            return None;
        }

        match UmpPacket::try_from_bytes(self.data) {
            Ok((packet, consumed)) => {
                self.data = &self.data[consumed..];
                Some(Ok(packet))
            }
            Err(e) => {
                self.failed = true;
                Some(Err(e))
            }
        }
    }
}

/// Sequential serializer for Universal MIDI Packets into a byte buffer.
#[derive(Debug)]
pub struct UmpWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> UmpWriter<'a> {
    /// Creates a new `UmpWriter` writing into `buf`.
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    /// Writes one UMP packet into the buffer.
    pub fn write_packet(&mut self, packet: &UmpPacket) -> Result<(), UmpError> {
        let n = packet.to_bytes(&mut self.buf[self.offset..])?;
        self.offset += n;
        Ok(())
    }

    /// Returns the number of bytes written so far.
    pub fn written_len(&self) -> usize {
        self.offset
    }

    /// Returns the slice of written bytes.
    pub fn written(&self) -> &[u8] {
        &self.buf[..self.offset]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_types_word_counts() {
        assert_eq!(MessageType::Utility.word_count(), 1);
        assert_eq!(MessageType::System.word_count(), 1);
        assert_eq!(MessageType::Midi1ChannelVoice.word_count(), 1);
        assert_eq!(MessageType::Data64.word_count(), 2);
        assert_eq!(MessageType::Midi2ChannelVoice.word_count(), 2);
        assert_eq!(MessageType::Data128.word_count(), 4);
        assert_eq!(MessageType::ReservedB.word_count(), 3);
        assert_eq!(MessageType::FlexData.word_count(), 4);
        assert_eq!(MessageType::Stream.word_count(), 4);
    }

    #[test]
    fn test_ump_roundtrip_32bit() {
        // MT = 0x2 (MIDI 1.0 Channel Voice), Group = 3, Note On Ch 0, Note 60, Vel 100
        let w0 = (0x2 << 28) | (3 << 24) | (0x90 << 16) | (60 << 8) | 100;
        let packet = UmpPacket::from_u32(w0).unwrap();

        assert_eq!(packet.message_type(), MessageType::Midi1ChannelVoice);
        assert_eq!(packet.group(), Some(3));
        assert_eq!(packet.channel(), Some(0));
        assert_eq!(packet.word_count(), 1);
        assert_eq!(packet.byte_count(), 4);

        let mut buf = [0u8; 8];
        let written = packet.to_bytes(&mut buf).unwrap();
        assert_eq!(written, 4);
        assert_eq!(&buf[..4], &w0.to_le_bytes());

        let (decoded, consumed) = UmpPacket::try_from_bytes(&buf[..written]).unwrap();
        assert_eq!(consumed, 4);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn test_ump_roundtrip_64bit() {
        // MT = 0x4 (MIDI 2.0 Channel Voice)
        let w0 = (0x4 << 28) | (1 << 24) | (0x90 << 16) | (60 << 8) | 0x00;
        let w1 = 0x8000_0000;
        let packet = UmpPacket::from_u32_pair(w0, w1).unwrap();

        assert_eq!(packet.message_type(), MessageType::Midi2ChannelVoice);
        assert_eq!(packet.group(), Some(1));
        assert_eq!(packet.word_count(), 2);
        assert_eq!(packet.byte_count(), 8);

        let mut buf = [0u8; 8];
        let written = packet.to_bytes(&mut buf).unwrap();
        assert_eq!(written, 8);

        let (decoded, consumed) = UmpPacket::try_from_bytes(&buf).unwrap();
        assert_eq!(consumed, 8);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn test_ump_reader_multiple_packets() {
        let p1 = UmpPacket::from_u32((0x2 << 28) | (0 << 24) | (0x90 << 16) | 0x3C64).unwrap();
        let p2 = UmpPacket::from_u32_pair((0x4 << 28) | (2 << 24) | (0x90 << 16), 0x1234_5678).unwrap();

        let mut buffer = [0u8; 32];
        let mut writer = UmpWriter::new(&mut buffer);
        writer.write_packet(&p1).unwrap();
        writer.write_packet(&p2).unwrap();

        let reader = UmpReader::new(writer.written());
        let packets: heapless::Vec<UmpPacket, 4> = reader.map(|r| r.unwrap()).collect();

        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0], p1);
        assert_eq!(packets[1], p2);
    }

    #[test]
    fn constructors_check_word_count_against_message_type() {
        // MT 4 is a 64-bit message.
        assert_eq!(UmpPacket::from_u32(0x4090_3c00), Err(UmpError::InvalidLength));
        assert!(UmpPacket::from_u32_pair(0x4090_3c00, 0).is_ok());
    }

    #[test]
    fn utility_and_stream_packets_have_no_group() {
        let stream = UmpPacket::from_u32_quad(0xf401_0000, 0, 0, 0).unwrap();
        let utility = UmpPacket::from_u32(0x0020_0000).unwrap();
        let voice = UmpPacket::from_u32(0x2390_3c7f).unwrap();
        assert_eq!((stream.group(), utility.group(), voice.group()), (None, None, Some(3)));
        assert!(stream.in_groups(0) && utility.in_groups(0));
        assert!(voice.in_groups(1 << 3) && !voice.in_groups(1));
    }

    #[test]
    fn reader_keeps_partial_tail() {
        // A 32-bit packet, then a 64-bit one cut after its first word.
        let mut data = [0u8; 8];
        data[..4].copy_from_slice(&0x2090_3c7fu32.to_le_bytes());
        data[4..].copy_from_slice(&0x4090_3c00u32.to_le_bytes());
        let mut reader = UmpReader::new(&data);
        assert!(reader.next().unwrap().is_ok());
        assert_eq!(reader.next(), Some(Err(UmpError::UnexpectedEof)));
        assert_eq!(reader.next(), None);
        assert_eq!(reader.remaining(), &data[4..]);
    }

    #[test]
    fn ump32_accepts_only_one_word_message_types() {
        let packet = Ump32::new(0x2a90_3c7f).unwrap();
        assert_eq!(UmpPacket::from(packet), UmpPacket::from_u32(0x2a90_3c7f).unwrap());
        assert_eq!(Ump32::new(0x4a90_3c7f), Err(UmpError::InvalidLength));
    }
}
