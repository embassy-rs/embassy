#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MacStatus {
    Success = 0x00,
    Error = 0x01,
    NotImplemented = 0x02,
    NotSupported = 0x03,
    HardwareNotSupported = 0x04,
    Undefined = 0x05,
}

impl TryFrom<u8> for MacStatus {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, <MacStatus as TryFrom<u8>>::Error> {
        match value {
            0x00 => Ok(Self::Success),
            0x01 => Ok(Self::Error),
            0x02 => Ok(Self::NotImplemented),
            0x03 => Ok(Self::NotSupported),
            0x04 => Ok(Self::HardwareNotSupported),
            0x05 => Ok(Self::Undefined),
            _ => Err(()),
        }
    }
}

/// this enum contains all the MAC PIB Ids
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PibId {
    // PHY
    CurrentChannel = 0x00,
    ChannelsSupported = 0x01,
    TransmitPower = 0x02,
    CCAMode = 0x03,
    CurrentPage = 0x04,
    MaxFrameDuration = 0x05,
    SHRDuration = 0x06,
    SymbolsPerOctet = 0x07,

    // MAC
    AckWaitDuration = 0x40,
    AssociationPermit = 0x41,
    AutoRequest = 0x42,
    BeaconPayload = 0x45,
    BeaconPayloadLength = 0x46,
    BeaconOrder = 0x47,
    Bsn = 0x49,
    CoordExtendedAdddress = 0x4A,
    CoordShortAddress = 0x4B,
    Dsn = 0x4C,
    MaxFrameTotalWaitTime = 0x58,
    MaxFrameRetries = 0x59,
    PanId = 0x50,
    ResponseWaitTime = 0x5A,
    RxOnWhenIdle = 0x52,
    SecurityEnabled = 0x5D,
    ShortAddress = 0x53,
    SuperframeOrder = 0x54,
    TimestampSupported = 0x5C,
    TransactionPersistenceTime = 0x55,
    MaxBe = 0x57,
    LifsPeriod = 0x5E,
    SifsPeriod = 0x5F,
    MaxCsmaBackoffs = 0x4E,
    MinBe = 0x4F,
    PanCoordinator = 0x10,
    AssocPanCoordinator = 0x11,
    ExtendedAddress = 0x6F,
    AclEntryDescriptor = 0x70,
    AclEntryDescriptorSize = 0x71,
    DefaultSecurity = 0x72,
    DefaultSecurityMaterialLength = 0x73,
    DefaultSecurityMaterial = 0x74,
    DefaultSecuritySuite = 0x75,
    SecurityMode = 0x76,
    CurrentAclEntries = 0x80,
    DefaultSecurityExtendedAddress = 0x81,
    AssociatedPanCoordinator = 0x56,
    PromiscuousMode = 0x51,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddressMode {
    NoAddress = 0x00,
    Reserved = 0x01,
    Short = 0x02,
    Extended = 0x03,
}

pub union MacAddress {
    pub short: [u8; 2],
    pub extended: [u8; 8],
}

pub struct GtsCharacteristics {
    pub fields: u8,
}

/// MAC PAN Descriptor which contains the network details of the device from
/// which the beacon is received
pub struct PanDescriptor {
    /// PAN identifier of the coordinator
    pub a_coord_pan_id: [u8; 2],
    /// Coordinator addressing mode
    pub coord_addr_mode: AddressMode,
    /// The current logical channel occupied by the network
    pub logical_channel: u8,
    /// Coordinator address
    pub coord_addr: MacAddress,
    /// The current channel page occupied by the network
    pub channel_page: u8,
    /// PAN coordinator is accepting GTS requests or not
    pub gts_permit: bool,
    /// Superframe specification as specified in the received beacon frame
    pub a_superframe_spec: [u8; 2],
    /// The time at which the beacon frame was received, in symbols
    pub a_time_stamp: [u8; 4],
    /// The LQI at which the network beacon was received
    pub link_quality: u8,
    /// Security level purportedly used by the received beacon frame
    pub security_level: u8,
    /// Byte Stuffing to keep 32 bit alignment
    pub a_stuffing: [u8; 2],
}
