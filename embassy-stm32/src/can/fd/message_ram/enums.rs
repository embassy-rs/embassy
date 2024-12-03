// Note: This file is copied and modified from fdcan crate by Richard Meadows

/// Datalength is the message length generalised over
/// the Standard (Classic) and FDCAN message types

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataLength {
    Classic(u8),
    Fdcan(u8),
}
impl DataLength {
    /// Creates a DataLength type
    ///
    /// Uses the byte length and Type of frame as input
    pub fn new(len: u8, ff: FrameFormat) -> DataLength {
        match ff {
            FrameFormat::Classic => match len {
                0..=8 => DataLength::Classic(len),
                _ => panic!("DataLength > 8"),
            },
            FrameFormat::Fdcan => match len {
                0..=64 => DataLength::Fdcan(len),
                _ => panic!("DataLength > 64"),
            },
        }
    }
    /// Specialised function to create classic frames
    pub fn new_classic(len: u8) -> DataLength {
        Self::new(len, FrameFormat::Classic)
    }
    /// Specialised function to create FDCAN frames
    pub fn new_fdcan(len: u8) -> DataLength {
        Self::new(len, FrameFormat::Fdcan)
    }

    /// returns the length in bytes
    pub fn len(&self) -> u8 {
        match self {
            DataLength::Classic(l) | DataLength::Fdcan(l) => *l,
        }
    }

    pub(crate) fn dlc(&self) -> u8 {
        match self {
            DataLength::Classic(l) => *l,
            // See RM0433 Rev 7 Table 475. DLC coding
            DataLength::Fdcan(l) => match l {
                0..=8 => *l,
                9..=12 => 9,
                13..=16 => 10,
                17..=20 => 11,
                21..=24 => 12,
                25..=32 => 13,
                33..=48 => 14,
                49..=64 => 15,
                _ => panic!("DataLength > 64"),
            },
        }
    }
}
impl From<DataLength> for FrameFormat {
    fn from(dl: DataLength) -> FrameFormat {
        match dl {
            DataLength::Classic(_) => FrameFormat::Classic,
            DataLength::Fdcan(_) => FrameFormat::Fdcan,
        }
    }
}

/// Wheter or not to generate an Tx Event
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// Do not generate an Tx Event
    NoEvent,
    /// Generate an Tx Event with a specified ID
    Event(u8),
}

impl From<Event> for EventControl {
    fn from(e: Event) -> Self {
        match e {
            Event::NoEvent => EventControl::DoNotStore,
            Event::Event(_) => EventControl::Store,
        }
    }
}

impl From<Option<u8>> for Event {
    fn from(mm: Option<u8>) -> Self {
        match mm {
            None => Event::NoEvent,
            Some(mm) => Event::Event(mm),
        }
    }
}

impl From<Event> for Option<u8> {
    fn from(e: Event) -> Option<u8> {
        match e {
            Event::NoEvent => None,
            Event::Event(mm) => Some(mm),
        }
    }
}

/// TODO
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorStateIndicator {
    /// TODO
    ErrorActive = 0,
    /// TODO
    ErrorPassive = 1,
}
impl From<ErrorStateIndicator> for bool {
    #[inline(always)]
    fn from(e: ErrorStateIndicator) -> Self {
        e as u8 != 0
    }
}

/// Type of frame, standard (classic) or FdCAN
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrameFormat {
    Classic = 0,
    Fdcan = 1,
}
impl From<FrameFormat> for bool {
    #[inline(always)]
    fn from(e: FrameFormat) -> Self {
        e as u8 != 0
    }
}

/// Type of Id, Standard or Extended
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IdType {
    /// Standard ID
    StandardId = 0,
    /// Extended ID
    ExtendedId = 1,
}
impl From<IdType> for bool {
    #[inline(always)]
    fn from(e: IdType) -> Self {
        e as u8 != 0
    }
}

/// Whether the frame contains data or requests data
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RemoteTransmissionRequest {
    /// Frame contains data
    TransmitDataFrame = 0,
    /// frame does not contain data
    TransmitRemoteFrame = 1,
}
impl From<RemoteTransmissionRequest> for bool {
    #[inline(always)]
    fn from(e: RemoteTransmissionRequest) -> Self {
        e as u8 != 0
    }
}

/// Whether BitRateSwitching should be or was enabled
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BitRateSwitching {
    /// disable bit rate switching
    WithoutBRS = 0,
    /// enable bit rate switching
    WithBRS = 1,
}
impl From<BitRateSwitching> for bool {
    #[inline(always)]
    fn from(e: BitRateSwitching) -> Self {
        e as u8 != 0
    }
}

/// Whether to store transmit Events
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventControl {
    /// do not store an tx event
    DoNotStore,
    /// store transmit events
    Store,
}
impl From<EventControl> for bool {
    #[inline(always)]
    fn from(e: EventControl) -> Self {
        e as u8 != 0
    }
}

/// If an received message matched any filters
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterFrameMatch {
    /// This did match filter <id>
    DidMatch(u8),
    /// This received frame did not match any specific filters
    DidNotMatch,
}

/// Type of filter to be used
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterType {
    /// Filter uses the range between two id's
    RangeFilter = 0b00,
    /// The filter matches on two specific id's (or one ID checked twice)
    DualIdFilter = 0b01,
    /// Filter is using a bitmask
    ClassicFilter = 0b10,
    /// Filter is disabled
    FilterDisabled = 0b11,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterElementConfig {
    /// Filter is disabled
    DisableFilterElement = 0b000,
    /// Store a matching message in FIFO 0
    StoreInFifo0 = 0b001,
    /// Store a matching message in FIFO 1
    StoreInFifo1 = 0b010,
    /// Reject a matching message
    Reject = 0b011,
    /// Flag that a priority message has been received, *But do note store!*??
    SetPriority = 0b100,
    /// Flag and store message in FIFO 0
    SetPriorityAndStoreInFifo0 = 0b101,
    /// Flag and store message in FIFO 1
    SetPriorityAndStoreInFifo1 = 0b110,
    //_Unused = 0b111,
}
