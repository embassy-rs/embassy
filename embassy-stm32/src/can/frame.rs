//! Definition for CAN Frames
use bit_field::BitField;

/// CAN Header, without meta data
#[derive(Debug, Copy, Clone)]
pub struct Header {
    id: embedded_can::Id,
    len: u8,
    flags: u8,
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
}

/// Trait for FDCAN frame types, providing ability to construct from a Header
/// and to retrieve the Header from a frame
pub trait CanHeader: Sized {
    /// Construct frame from header and payload
    fn from_header(header: Header, data: &[u8]) -> Option<Self>;

    /// Get this frame's header struct
    fn header(&self) -> &Header;
}

/// Payload of a classic CAN data frame.
///
/// Contains 0 to 8 Bytes of data.
#[derive(Debug, Copy, Clone)]
pub struct ClassicData {
    pub(crate) bytes: [u8; 8],
}

impl ClassicData {
    /// Creates a data payload from a raw byte slice.
    ///
    /// Returns `None` if `data` is more than 64 bytes (which is the maximum) or
    /// cannot be represented with an FDCAN DLC.
    pub fn new(data: &[u8]) -> Option<Self> {
        if !FdData::is_valid_len(data.len()) {
            return None;
        }

        let mut bytes = [0; 8];
        bytes[..data.len()].copy_from_slice(data);

        Some(Self { bytes })
    }

    /// Raw read access to data.
    pub fn raw(&self) -> &[u8] {
        &self.bytes
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

/// Frame with up to 8 bytes of data payload as per Classic CAN
#[derive(Debug, Copy, Clone)]
pub struct ClassicFrame {
    can_header: Header,
    data: ClassicData,
}

impl ClassicFrame {
    pub(crate) const MAX_DATA_LEN: usize = 8;

    /// Create a new CAN classic Frame
    pub fn new(can_header: Header, data: ClassicData) -> ClassicFrame {
        ClassicFrame { can_header, data }
    }

    /// Create new extended frame
    pub fn new_extended(raw_id: u32, raw_data: &[u8]) -> Option<Self> {
        if let Some(id) = embedded_can::ExtendedId::new(raw_id) {
            match ClassicData::new(raw_data) {
                Some(data) => Some(ClassicFrame::new(
                    Header::new(id.into(), raw_data.len() as u8, false),
                    data,
                )),
                None => None,
            }
        } else {
            None
        }
    }

    /// Create new standard frame
    pub fn new_standard(raw_id: u16, raw_data: &[u8]) -> Option<Self> {
        if let Some(id) = embedded_can::StandardId::new(raw_id) {
            match ClassicData::new(raw_data) {
                Some(data) => Some(ClassicFrame::new(
                    Header::new(id.into(), raw_data.len() as u8, false),
                    data,
                )),
                None => None,
            }
        } else {
            None
        }
    }

    /// Create new remote frame
    pub fn new_remote(id: impl Into<embedded_can::Id>, len: usize) -> Option<Self> {
        if len <= 8usize {
            Some(ClassicFrame::new(
                Header::new(id.into(), len as u8, true),
                ClassicData::empty(),
            ))
        } else {
            None
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
}

impl embedded_can::Frame for ClassicFrame {
    fn new(id: impl Into<embedded_can::Id>, raw_data: &[u8]) -> Option<Self> {
        match ClassicData::new(raw_data) {
            Some(data) => Some(ClassicFrame::new(
                Header::new(id.into(), raw_data.len() as u8, false),
                data,
            )),
            None => None,
        }
    }
    fn new_remote(id: impl Into<embedded_can::Id>, len: usize) -> Option<Self> {
        if len <= 8 {
            Some(ClassicFrame::new(
                Header::new(id.into(), len as u8, true),
                ClassicData::empty(),
            ))
        } else {
            None
        }
    }
    fn is_extended(&self) -> bool {
        match self.can_header.id {
            embedded_can::Id::Extended(_) => true,
            embedded_can::Id::Standard(_) => true,
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

impl CanHeader for ClassicFrame {
    fn from_header(header: Header, data: &[u8]) -> Option<Self> {
        Some(Self::new(header, ClassicData::new(data)?))
    }

    fn header(&self) -> &Header {
        self.header()
    }
}

/// Payload of a (FD)CAN data frame.
///
/// Contains 0 to 64 Bytes of data.
#[derive(Debug, Copy, Clone)]
pub struct FdData {
    pub(crate) bytes: [u8; 64],
}

impl FdData {
    /// Creates a data payload from a raw byte slice.
    ///
    /// Returns `None` if `data` is more than 64 bytes (which is the maximum) or
    /// cannot be represented with an FDCAN DLC.
    pub fn new(data: &[u8]) -> Option<Self> {
        if !FdData::is_valid_len(data.len()) {
            return None;
        }

        let mut bytes = [0; 64];
        bytes[..data.len()].copy_from_slice(data);

        Some(Self { bytes })
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
        Self { bytes: [0; 64] }
    }
}

/// Frame with up to 8 bytes of data payload as per Fd CAN
#[derive(Debug, Copy, Clone)]
pub struct FdFrame {
    can_header: Header,
    data: FdData,
}

impl FdFrame {
    pub(crate) const MAX_DATA_LEN: usize = 64;

    /// Create a new CAN classic Frame
    pub fn new(can_header: Header, data: FdData) -> FdFrame {
        FdFrame { can_header, data }
    }

    /// Create new extended frame
    pub fn new_extended(raw_id: u32, raw_data: &[u8]) -> Option<Self> {
        if let Some(id) = embedded_can::ExtendedId::new(raw_id) {
            match FdData::new(raw_data) {
                Some(data) => Some(FdFrame::new(Header::new(id.into(), raw_data.len() as u8, false), data)),
                None => None,
            }
        } else {
            None
        }
    }

    /// Create new standard frame
    pub fn new_standard(raw_id: u16, raw_data: &[u8]) -> Option<Self> {
        if let Some(id) = embedded_can::StandardId::new(raw_id) {
            match FdData::new(raw_data) {
                Some(data) => Some(FdFrame::new(Header::new(id.into(), raw_data.len() as u8, false), data)),
                None => None,
            }
        } else {
            None
        }
    }

    /// Create new remote frame
    pub fn new_remote(id: impl Into<embedded_can::Id>, len: usize) -> Option<Self> {
        if len <= 8 {
            Some(FdFrame::new(Header::new(id.into(), len as u8, true), FdData::empty()))
        } else {
            None
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
}

impl embedded_can::Frame for FdFrame {
    fn new(id: impl Into<embedded_can::Id>, raw_data: &[u8]) -> Option<Self> {
        match FdData::new(raw_data) {
            Some(data) => Some(FdFrame::new(
                Header::new_fd(id.into(), raw_data.len() as u8, false, true),
                data,
            )),
            None => None,
        }
    }
    fn new_remote(id: impl Into<embedded_can::Id>, len: usize) -> Option<Self> {
        if len <= 8 {
            Some(FdFrame::new(
                Header::new_fd(id.into(), len as u8, true, true),
                FdData::empty(),
            ))
        } else {
            None
        }
    }
    fn is_extended(&self) -> bool {
        match self.can_header.id {
            embedded_can::Id::Extended(_) => true,
            embedded_can::Id::Standard(_) => true,
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
        &self.data.raw()
    }
}

impl CanHeader for FdFrame {
    fn from_header(header: Header, data: &[u8]) -> Option<Self> {
        Some(Self::new(header, FdData::new(data)?))
    }

    fn header(&self) -> &Header {
        self.header()
    }
}
