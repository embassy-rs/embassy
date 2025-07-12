//! Definition for CAN Frames
use bit_field::BitField;

use crate::can::enums::FrameCreateError;

/// Calculate proper timestamp when available.
#[cfg(feature = "time")]
pub type Timestamp = embassy_time::Instant;

/// Raw register timestamp
#[cfg(not(feature = "time"))]
pub type Timestamp = u16;

/// CAN Header, without meta data
#[derive(Debug, Copy, Clone)]
pub struct Header {
    id: embedded_can::Id,
    len: u8,
    flags: u8,
}

#[cfg(feature = "defmt")]
impl defmt::Format for Header {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        match self.id() {
            embedded_can::Id::Standard(id) => {
                defmt::write!(fmt, "Can Standard ID={:x} len={}", id.as_raw(), self.len,)
            }
            embedded_can::Id::Extended(id) => {
                defmt::write!(fmt, "Can Extended ID={:x} len={}", id.as_raw(), self.len,)
            }
        }
    }
}

impl Header {
    const FLAG_RTR: usize = 0; // Remote
    const FLAG_FDCAN: usize = 1; // FDCan vs Classic CAN
    const FLAG_BRS: usize = 2; // Bit-rate switching, ignored for Classic CAN

    /// Create new CAN Header
    pub fn new(id: embedded_can::Id, len: u8, rtr: bool) -> Header {
        let mut flags = 0u8;
        flags.set_bit(Self::FLAG_RTR, rtr);
        Header { id, len, flags }
    }

    /// Create new CAN FD Header
    pub fn new_fd(id: embedded_can::Id, len: u8, rtr: bool, brs: bool) -> Header {
        let mut flags = 0u8;
        flags.set_bit(Self::FLAG_RTR, rtr);
        flags.set_bit(Self::FLAG_FDCAN, true);
        flags.set_bit(Self::FLAG_BRS, brs);
        Header { id, len, flags }
    }

    /// Return ID
    pub fn id(&self) -> &embedded_can::Id {
        &self.id
    }

    /// Return length as u8
    pub fn len(&self) -> u8 {
        self.len
    }

    /// Is remote frame
    pub fn rtr(&self) -> bool {
        self.flags.get_bit(Self::FLAG_RTR)
    }

    /// Request/is FDCAN frame
    pub fn fdcan(&self) -> bool {
        self.flags.get_bit(Self::FLAG_FDCAN)
    }

    /// Request/is Flexible Data Rate
    pub fn bit_rate_switching(&self) -> bool {
        self.flags.get_bit(Self::FLAG_BRS)
    }

    /// Get priority of frame
    pub(crate) fn priority(&self) -> u32 {
        match self.id() {
            embedded_can::Id::Standard(id) => (id.as_raw() as u32) << 18,
            embedded_can::Id::Extended(id) => id.as_raw(),
        }
    }
}

/// Trait for FDCAN frame types, providing ability to construct from a Header
/// and to retrieve the Header from a frame
pub trait CanHeader: Sized {
    /// Construct frame from header and payload
    fn from_header(header: Header, data: &[u8]) -> Result<Self, FrameCreateError>;

    /// Get this frame's header struct
    fn header(&self) -> &Header;
}

/// Payload of a classic CAN data frame.
///
/// Contains 0 to 8 Bytes of data.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ClassicData {
    pub(crate) bytes: [u8; 8],
}

impl ClassicData {
    /// Creates a data payload from a raw byte slice.
    ///
    /// Returns `FrameCreateError` if `data` is more than 8 bytes (which is the maximum).
    pub fn new(data: &[u8]) -> Result<Self, FrameCreateError> {
        if data.len() > 8 {
            return Err(FrameCreateError::InvalidDataLength);
        }

        let mut bytes = [0; 8];
        bytes[..data.len()].copy_from_slice(data);

        Ok(Self { bytes })
    }

    /// Raw read access to data.
    pub fn raw(&self) -> &[u8] {
        &self.bytes
    }

    /// Raw mutable read access to data.
    pub fn raw_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    /// Checks if the length can be encoded in FDCAN DLC field.
    pub const fn is_valid_len(len: usize) -> bool {
        match len {
            0..=8 => true,
            _ => false,
        }
    }

    /// Creates an empty data payload containing 0 bytes.
    #[inline]
    pub const fn empty() -> Self {
        Self { bytes: [0; 8] }
    }
}

/// Frame with up to 8 bytes of data payload as per Classic(non-FD) CAN
/// For CAN-FD support use FdFrame
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Frame {
    can_header: Header,
    data: ClassicData,
}

impl Frame {
    /// Create a new CAN classic Frame
    pub fn new(can_header: Header, raw_data: &[u8]) -> Result<Self, FrameCreateError> {
        let data = ClassicData::new(raw_data)?;
        Ok(Frame { can_header, data: data })
    }

    /// Creates a new data frame.
    pub fn new_data(id: impl Into<embedded_can::Id>, data: &[u8]) -> Result<Self, FrameCreateError> {
        let eid: embedded_can::Id = id.into();
        let header = Header::new(eid, data.len() as u8, false);
        Self::new(header, data)
    }

    /// Create new extended frame
    pub fn new_extended(raw_id: u32, raw_data: &[u8]) -> Result<Self, FrameCreateError> {
        if let Some(id) = embedded_can::ExtendedId::new(raw_id) {
            Self::new(Header::new(id.into(), raw_data.len() as u8, false), raw_data)
        } else {
            Err(FrameCreateError::InvalidCanId)
        }
    }

    /// Create new standard frame
    pub fn new_standard(raw_id: u16, raw_data: &[u8]) -> Result<Self, FrameCreateError> {
        if let Some(id) = embedded_can::StandardId::new(raw_id) {
            Self::new(Header::new(id.into(), raw_data.len() as u8, false), raw_data)
        } else {
            Err(FrameCreateError::InvalidCanId)
        }
    }

    /// Create new remote frame
    pub fn new_remote(id: impl Into<embedded_can::Id>, len: usize) -> Result<Self, FrameCreateError> {
        if len <= 8usize {
            Self::new(Header::new(id.into(), len as u8, true), &[0; 8])
        } else {
            Err(FrameCreateError::InvalidDataLength)
        }
    }

    /// Get reference to data
    pub fn header(&self) -> &Header {
        &self.can_header
    }

    /// Return ID
    pub fn id(&self) -> &embedded_can::Id {
        &self.can_header.id
    }

    /// Get reference to data
    pub fn data(&self) -> &[u8] {
        &self.data.raw()[..self.can_header.len as usize]
    }

    /// Get reference to underlying 8-byte raw data buffer, some bytes on the tail might be undefined.
    pub fn raw_data(&self) -> &[u8] {
        self.data.raw()
    }

    /// Get mutable reference to data
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data.raw_mut()[..self.can_header.len as usize]
    }

    /// Get priority of frame
    pub fn priority(&self) -> u32 {
        self.header().priority()
    }
}

impl embedded_can::Frame for Frame {
    fn new(id: impl Into<embedded_can::Id>, raw_data: &[u8]) -> Option<Self> {
        let frameopt = Frame::new(Header::new(id.into(), raw_data.len() as u8, false), raw_data);
        match frameopt {
            Ok(frame) => Some(frame),
            Err(_) => None,
        }
    }
    fn new_remote(id: impl Into<embedded_can::Id>, len: usize) -> Option<Self> {
        if len <= 8 {
            let frameopt = Frame::new(Header::new(id.into(), len as u8, true), &[0; 8]);
            match frameopt {
                Ok(frame) => Some(frame),
                Err(_) => None,
            }
        } else {
            None
        }
    }
    fn is_extended(&self) -> bool {
        match self.can_header.id {
            embedded_can::Id::Extended(_) => true,
            embedded_can::Id::Standard(_) => false,
        }
    }
    fn is_remote_frame(&self) -> bool {
        self.can_header.rtr()
    }
    fn id(&self) -> embedded_can::Id {
        self.can_header.id
    }
    fn dlc(&self) -> usize {
        self.can_header.len as usize
    }
    fn data(&self) -> &[u8] {
        &self.data()
    }
}

impl CanHeader for Frame {
    fn from_header(header: Header, data: &[u8]) -> Result<Self, FrameCreateError> {
        Self::new(header, data)
    }

    fn header(&self) -> &Header {
        self.header()
    }
}

/// Contains CAN frame and additional metadata.
///
/// Timestamp is available if `time` feature is enabled.
/// For CAN-FD support use FdEnvelope
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Envelope {
    /// Reception time.
    pub ts: Timestamp,
    /// The actual CAN frame.
    pub frame: Frame,
}

impl Envelope {
    /// Convert into a tuple
    pub fn parts(self) -> (Frame, Timestamp) {
        (self.frame, self.ts)
    }
}

/// Payload of a (FD)CAN data frame.
///
/// Contains 0 to 64 Bytes of data.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FdData {
    pub(crate) bytes: [u8; 64],
}

impl FdData {
    /// Creates a data payload from a raw byte slice.
    ///
    /// Returns `None` if `data` is more than 64 bytes (which is the maximum) or
    /// cannot be represented with an FDCAN DLC.
    pub fn new(data: &[u8]) -> Result<Self, FrameCreateError> {
        if !FdData::is_valid_len(data.len()) {
            return Err(FrameCreateError::InvalidDataLength);
        }

        let mut bytes = [0; 64];
        bytes[..data.len()].copy_from_slice(data);

        Ok(Self { bytes })
    }

    /// Raw read access to data.
    pub fn raw(&self) -> &[u8] {
        &self.bytes
    }

    /// Raw mutable read access to data.
    pub fn raw_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    /// Checks if the length can be encoded in FDCAN DLC field.
    pub const fn is_valid_len(len: usize) -> bool {
        match len {
            0..=8 => true,
            12 => true,
            16 => true,
            20 => true,
            24 => true,
            32 => true,
            48 => true,
            64 => true,
            _ => false,
        }
    }

    /// Creates an empty data payload containing 0 bytes.
    #[inline]
    pub const fn empty() -> Self {
        Self { bytes: [0; 64] }
    }
}

/// Frame with up to 8 bytes of data payload as per Fd CAN
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FdFrame {
    can_header: Header,
    data: FdData,
}

impl FdFrame {
    /// Create a new CAN classic Frame
    pub fn new(can_header: Header, raw_data: &[u8]) -> Result<Self, FrameCreateError> {
        let data = FdData::new(raw_data)?;
        Ok(FdFrame { can_header, data })
    }

    /// Create new extended frame
    pub fn new_extended(raw_id: u32, raw_data: &[u8]) -> Result<Self, FrameCreateError> {
        if let Some(id) = embedded_can::ExtendedId::new(raw_id) {
            Self::new(Header::new(id.into(), raw_data.len() as u8, false), raw_data)
        } else {
            Err(FrameCreateError::InvalidCanId)
        }
    }

    /// Create new standard frame
    pub fn new_standard(raw_id: u16, raw_data: &[u8]) -> Result<Self, FrameCreateError> {
        if let Some(id) = embedded_can::StandardId::new(raw_id) {
            Self::new(Header::new(id.into(), raw_data.len() as u8, false), raw_data)
        } else {
            Err(FrameCreateError::InvalidCanId)
        }
    }

    /// Create new remote frame
    pub fn new_remote(id: impl Into<embedded_can::Id>, len: usize) -> Result<Self, FrameCreateError> {
        if len <= 8 {
            Self::new(Header::new(id.into(), len as u8, true), &[0; 8])
        } else {
            Err(FrameCreateError::InvalidDataLength)
        }
    }

    /// Get reference to data
    pub fn header(&self) -> &Header {
        &self.can_header
    }

    /// Return ID
    pub fn id(&self) -> &embedded_can::Id {
        &self.can_header.id
    }

    /// Get reference to data
    pub fn data(&self) -> &[u8] {
        &self.data.raw()[..self.can_header.len as usize]
    }

    /// Get mutable reference to data
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data.raw_mut()[..self.can_header.len as usize]
    }
}

impl embedded_can::Frame for FdFrame {
    fn new(id: impl Into<embedded_can::Id>, raw_data: &[u8]) -> Option<Self> {
        match FdFrame::new(Header::new_fd(id.into(), raw_data.len() as u8, false, true), raw_data) {
            Ok(frame) => Some(frame),
            Err(_) => None,
        }
    }
    fn new_remote(id: impl Into<embedded_can::Id>, len: usize) -> Option<Self> {
        if len <= 8 {
            match FdFrame::new(Header::new_fd(id.into(), len as u8, true, true), &[0; 64]) {
                Ok(frame) => Some(frame),
                Err(_) => None,
            }
        } else {
            None
        }
    }
    fn is_extended(&self) -> bool {
        match self.can_header.id {
            embedded_can::Id::Extended(_) => true,
            embedded_can::Id::Standard(_) => false,
        }
    }
    fn is_remote_frame(&self) -> bool {
        self.can_header.rtr()
    }
    fn id(&self) -> embedded_can::Id {
        self.can_header.id
    }
    // Returns length in bytes even for CANFD packets which embedded-can does not really mention.
    fn dlc(&self) -> usize {
        self.can_header.len as usize
    }
    fn data(&self) -> &[u8] {
        &self.data()
    }
}

impl CanHeader for FdFrame {
    fn from_header(header: Header, data: &[u8]) -> Result<Self, FrameCreateError> {
        Self::new(header, data)
    }

    fn header(&self) -> &Header {
        self.header()
    }
}

/// Contains CAN FD frame and additional metadata.
///
/// Timestamp is available if `time` feature is enabled.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FdEnvelope {
    /// Reception time.
    pub ts: Timestamp,

    /// The actual CAN frame.
    pub frame: FdFrame,
}

impl FdEnvelope {
    /// Convert into a tuple
    pub fn parts(self) -> (FdFrame, Timestamp) {
        (self.frame, self.ts)
    }
}
