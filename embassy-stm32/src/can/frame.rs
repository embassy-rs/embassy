//! Definition for CAN Frames
use core::marker::PhantomData;

use bit_field::BitField;

use crate::can::enums::FrameCreateError;

use super::common::{CanMode, Classic, Fd};

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
    /// RTR (remote transmit request) flag in CAN frame
    const FLAG_RTR: usize = 0;
    /// FDCan vs Classic CAN
    const FLAG_FDCAN: usize = 1;
    /// Bit-rate switching, ignored for Classic CAN
    const FLAG_BRS: usize = 2;
    /// ESI recessive in CAN FD message.
    /// ORed with error passive flag before transmission.
    /// By spec, an error active mode may transmit ESI resessive,
    /// but an error passive node will always transmit ESI resessive.
    const FLAG_ESI: usize = 3;

    /// Create new CAN Header
    pub fn new(id: embedded_can::Id, len: u8, rtr: bool) -> Header {
        let mut flags = 0u8;
        flags.set_bit(Self::FLAG_RTR, rtr);
        Header { id, len, flags }
    }

    /// Sets the CAN FD flags for the header
    pub fn set_can_fd(mut self, flag: bool, brs: bool) -> Header {
        self.flags.set_bit(Self::FLAG_FDCAN, flag);
        self.flags.set_bit(Self::FLAG_BRS, brs);
        self
    }

    /// Sets the error passive indicator bit
    pub fn set_esi(mut self, flag: bool) -> Header {
        self.flags.set_bit(Self::FLAG_ESI, flag);
        self
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

    /// Has error passive indicator bit set
    pub fn esi(&self) -> bool {
        self.flags.get_bit(Self::FLAG_ESI)
    }

    /// Request is FDCAN frame
    pub fn fdcan(&self) -> bool {
        self.flags.get_bit(Self::FLAG_FDCAN)
    }

    /// Request is Flexible Data Rate
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

pub(crate) trait CanData: Sized {
    const MAX_DATA_LEN: usize;
    fn new(data: &[u8]) -> Result<Self, FrameCreateError>;
    fn is_valid_len(len: usize) -> bool;
    fn raw(&self) -> &[u8];
}

/// Payload of a classic CAN data frame.
///
/// Contains 0 to 8 Bytes of data.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Data {
    pub(crate) bytes: [u8; 8],
}

impl Data {
    pub(crate) const MAX_DATA_LEN: usize = 8;

    /// Creates a data payload from a raw byte slice.
    ///
    /// Returns `None` if `data` is more than 64 bytes (which is the maximum) or
    /// cannot be represented with an FDCAN DLC.
    pub fn new(data: &[u8]) -> Result<Self, FrameCreateError> {
        if data.len() > Self::MAX_DATA_LEN {
            return Err(FrameCreateError::InvalidDataLength);
        }

        let mut bytes = [0; Self::MAX_DATA_LEN];
        bytes[..data.len()].copy_from_slice(data);

        Ok(Self { bytes })
    }

    /// Raw read access to data.
    pub fn raw(&self) -> &[u8] {
        &self.bytes
    }

    /// Checks if the length can be encoded in FDCAN DLC field.
    pub const fn is_valid_len(len: usize) -> bool {
        match len {
            0..=Self::MAX_DATA_LEN => true,
            _ => false,
        }
    }

    /// Creates an empty data payload containing 0 bytes.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            bytes: [0; Self::MAX_DATA_LEN],
        }
    }
}

impl CanData for Data {
    const MAX_DATA_LEN: usize = 8;

    fn new(data: &[u8]) -> Result<Self, FrameCreateError> {
        Self::new(data)
    }

    fn is_valid_len(len: usize) -> bool {
        Self::is_valid_len(len)
    }

    fn raw(&self) -> &[u8] {
        Self::raw(&self)
    }
}

/// Payload of a (FD)CAN data frame.
///
/// Contains 0 to 64 Bytes of data.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FdData {
    pub(crate) bytes: [u8; Self::MAX_DATA_LEN],
}

impl FdData {
    pub(crate) const MAX_DATA_LEN: usize = 64;

    /// Creates a data payload from a raw byte slice.
    ///
    /// Returns `None` if `data` is more than 64 bytes (which is the maximum) or
    /// cannot be represented with an FDCAN DLC.
    pub fn new(data: &[u8]) -> Result<Self, FrameCreateError> {
        if !FdData::is_valid_len(data.len()) {
            return Err(FrameCreateError::InvalidDataLength);
        }

        let mut bytes = [0; Self::MAX_DATA_LEN];
        bytes[..data.len()].copy_from_slice(data);

        Ok(Self { bytes })
    }

    /// Raw read access to data.
    pub fn raw(&self) -> &[u8] {
        &self.bytes
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
        Self {
            bytes: [0; Self::MAX_DATA_LEN],
        }
    }
}

impl CanData for FdData {
    const MAX_DATA_LEN: usize = 64;

    fn new(data: &[u8]) -> Result<Self, FrameCreateError> {
        Self::new(data)
    }

    fn is_valid_len(len: usize) -> bool {
        Self::is_valid_len(len)
    }

    fn raw(&self) -> &[u8] {
        Self::raw(&self)
    }
}

/// Payload of a CAN frame.
/// Max payload size depends on the CanMode type parameter.
/// See documentation for `Frame` and `FdFrame` type aliases.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BaseFrame<M: CanMode> {
    _phantom: PhantomData<M>,
    can_header: Header,
    data: M::Data,
}

/// Payload of a CAN Classic frame.
/// Contains 0 to 8 Bytes of data.
pub type Frame = BaseFrame<Classic>;

/// Payload of a (FD)CAN frame.
/// Contains 0 to 64 Bytes of data.
pub type FdFrame = BaseFrame<Fd>;

impl<M: CanMode> BaseFrame<M> {
    /// Create a new CAN classic Frame
    pub fn new(can_header: Header, raw_data: &[u8]) -> Result<Self, FrameCreateError> {
        Ok(BaseFrame {
            _phantom: PhantomData,
            can_header,
            data: M::Data::new(raw_data)?,
        })
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
        &self.data.raw()
    }

    /// Get priority of frame
    pub fn priority(&self) -> u32 {
        self.header().priority()
    }
}

impl<M: CanMode> embedded_can::Frame for BaseFrame<M> {
    fn new(id: impl Into<embedded_can::Id>, raw_data: &[u8]) -> Option<Self> {
        let frameopt = BaseFrame::new(Header::new(id.into(), raw_data.len() as u8, false), raw_data);
        match frameopt {
            Ok(frame) => Some(frame),
            Err(_) => None,
        }
    }
    fn new_remote(id: impl Into<embedded_can::Id>, len: usize) -> Option<Self> {
        if M::Data::is_valid_len(len) {
            let frameopt = BaseFrame::new(Header::new(id.into(), len as u8, true), &[]);
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
        &self.data.raw()
    }
}

impl<M: CanMode> CanHeader for BaseFrame<M> {
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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BaseEnvelope<M: CanMode> {
    /// Reception time.
    pub ts: Timestamp,
    /// The actual CAN frame.
    pub frame: BaseFrame<M>,
}

/// Contains CAN frame and additional metadata.
///
/// Timestamp is available if `time` feature is enabled.
/// For CAN-FD support use FdEnvelope
pub type Envelope = BaseEnvelope<Classic>;

/// Contains CAN FD frame and additional metadata.
///
/// Timestamp is available if `time` feature is enabled.
pub type FdEnvelope = BaseEnvelope<Fd>;

impl<M: CanMode> BaseEnvelope<M> {
    /// Convert into a tuple
    pub fn parts(self) -> (BaseFrame<M>, Timestamp) {
        (self.frame, self.ts)
    }
}
