use core::fmt::Debug;

use crate::numeric_enum;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MacError {
    Error = 0x01,
    NotImplemented = 0x02,
    NotSupported = 0x03,
    HardwareNotSupported = 0x04,
    Undefined = 0x05,
}

impl From<u8> for MacError {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::Error,
            0x02 => Self::NotImplemented,
            0x03 => Self::NotSupported,
            0x04 => Self::HardwareNotSupported,
            0x05 => Self::Undefined,
            _ => Self::Undefined,
        }
    }
}

numeric_enum! {
    #[repr(u8)]
    #[derive(Debug, Default)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum MacStatus {
        #[default]
        Success = 0x00,
        Failure = 0xFF
    }
}

numeric_enum! {
    #[repr(u8)]
    /// this enum contains all the MAC PIB Ids
    #[derive(Default, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum PibId {
        // PHY
        #[default]
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
}

numeric_enum! {
    #[repr(u8)]
    #[derive(Default, Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum AddressMode {
        #[default]
        NoAddress = 0x00,
        Reserved = 0x01,
        Short = 0x02,
        Extended = 0x03,
}
}

#[derive(Clone, Copy)]
pub union MacAddress {
    pub short: [u8; 2],
    pub extended: [u8; 8],
}

impl Debug for MacAddress {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe {
            write!(
                fmt,
                "MacAddress {{ short: {:?}, extended: {:?} }}",
                self.short, self.extended
            )
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for MacAddress {
    fn format(&self, fmt: defmt::Formatter) {
        unsafe {
            defmt::write!(
                fmt,
                "MacAddress {{ short: {}, extended: {} }}",
                self.short,
                self.extended
            )
        }
    }
}

impl Default for MacAddress {
    fn default() -> Self {
        Self { short: [0, 0] }
    }
}

impl MacAddress {
    pub const BROADCAST: Self = Self { short: [0xFF, 0xFF] };
}

impl TryFrom<&[u8]> for MacAddress {
    type Error = ();

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        const SIZE: usize = 8;
        if buf.len() < SIZE {
            return Err(());
        }

        Ok(Self {
            extended: [buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]],
        })
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GtsCharacteristics {
    pub fields: u8,
}

/// MAC PAN Descriptor which contains the network details of the device from
/// which the beacon is received
#[derive(Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PanDescriptor {
    /// PAN identifier of the coordinator
    pub coord_pan_id: PanId,
    /// Coordinator addressing mode
    pub coord_addr_mode: AddressMode,
    /// The current logical channel occupied by the network
    pub logical_channel: MacChannel,
    /// Coordinator address
    pub coord_addr: MacAddress,
    /// The current channel page occupied by the network
    pub channel_page: u8,
    /// PAN coordinator is accepting GTS requests or not
    pub gts_permit: bool,
    /// Superframe specification as specified in the received beacon frame
    pub superframe_spec: [u8; 2],
    /// The time at which the beacon frame was received, in symbols
    pub time_stamp: [u8; 4],
    /// The LQI at which the network beacon was received
    pub link_quality: u8,
    /// Security level purportedly used by the received beacon frame
    pub security_level: u8,
}

impl TryFrom<&[u8]> for PanDescriptor {
    type Error = ();

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        const SIZE: usize = 22;
        if buf.len() < SIZE {
            return Err(());
        }

        let coord_addr_mode = AddressMode::try_from(buf[2])?;
        let coord_addr = match coord_addr_mode {
            AddressMode::NoAddress => MacAddress { short: [0, 0] },
            AddressMode::Reserved => MacAddress { short: [0, 0] },
            AddressMode::Short => MacAddress {
                short: [buf[4], buf[5]],
            },
            AddressMode::Extended => MacAddress {
                extended: [buf[4], buf[5], buf[6], buf[7], buf[8], buf[9], buf[10], buf[11]],
            },
        };

        Ok(Self {
            coord_pan_id: PanId([buf[0], buf[1]]),
            coord_addr_mode,
            logical_channel: MacChannel::try_from(buf[3])?,
            coord_addr,
            channel_page: buf[12],
            gts_permit: buf[13] != 0,
            superframe_spec: [buf[14], buf[15]],
            time_stamp: [buf[16], buf[17], buf[18], buf[19]],
            link_quality: buf[20],
            security_level: buf[21],
            // 2 byte stuffing
        })
    }
}

numeric_enum! {
    #[repr(u8)]
    #[derive(Default, Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    /// Building wireless applications with STM32WB series MCUs - Application note 13.10.3
    pub enum MacChannel {
        Channel11 = 0x0B,
        Channel12 = 0x0C,
        Channel13 = 0x0D,
        Channel14 = 0x0E,
        Channel15 = 0x0F,
        #[default]
        Channel16 = 0x10,
        Channel17 = 0x11,
        Channel18 = 0x12,
        Channel19 = 0x13,
        Channel20 = 0x14,
        Channel21 = 0x15,
        Channel22 = 0x16,
        Channel23 = 0x17,
        Channel24 = 0x18,
        Channel25 = 0x19,
        Channel26 = 0x1A,
    }
}

#[cfg(not(feature = "defmt"))]
bitflags::bitflags! {
    pub struct Capabilities: u8 {
        /// 1 if the device is capabaleof becoming a PAN coordinator
        const IS_COORDINATOR_CAPABLE = 0b00000001;
        /// 1 if the device is an FFD, 0 if it is an RFD
        const IS_FFD = 0b00000010;
        /// 1 if the device is receiving power from mains, 0 if it is battery-powered
        const IS_MAINS_POWERED = 0b00000100;
        /// 1 if the device does not disable its receiver to conserver power during idle periods
        const RECEIVER_ON_WHEN_IDLE = 0b00001000;
        // 0b00010000 reserved
        // 0b00100000 reserved
        /// 1 if the device is capable of sending and receiving secured MAC frames
        const IS_SECURE = 0b01000000;
        /// 1 if the device wishes the coordinator to allocate a short address as a result of the association
        const ALLOCATE_ADDRESS = 0b10000000;
    }
}

#[cfg(feature = "defmt")]
defmt::bitflags! {
    pub struct Capabilities: u8 {
        /// 1 if the device is capabaleof becoming a PAN coordinator
        const IS_COORDINATOR_CAPABLE = 0b00000001;
        /// 1 if the device is an FFD, 0 if it is an RFD
        const IS_FFD = 0b00000010;
        /// 1 if the device is receiving power from mains, 0 if it is battery-powered
        const IS_MAINS_POWERED = 0b00000100;
        /// 1 if the device does not disable its receiver to conserver power during idle periods
        const RECEIVER_ON_WHEN_IDLE = 0b00001000;
        // 0b00010000 reserved
        // 0b00100000 reserved
        /// 1 if the device is capable of sending and receiving secured MAC frames
        const IS_SECURE = 0b01000000;
        /// 1 if the device wishes the coordinator to allocate a short address as a result of the association
        const ALLOCATE_ADDRESS = 0b10000000;
    }
}

numeric_enum! {
    #[repr(u8)]
    #[derive(Default, Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum KeyIdMode {
        #[default]
        /// the key is determined implicitly from the originator and recipient(s) of the frame
        Implicite = 0x00,
        /// the key is determined explicitly using a 1 bytes key source and a 1 byte key index
        Explicite1Byte = 0x01,
        /// the key is determined explicitly using a 4 bytes key source and a 1 byte key index
        Explicite4Byte = 0x02,
        /// the key is determined explicitly using a 8 bytes key source and a 1 byte key index
        Explicite8Byte = 0x03,
    }
}

numeric_enum! {
    #[repr(u8)]
    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum AssociationStatus {
        /// Association successful
        Success = 0x00,
        /// PAN at capacity
        PanAtCapacity = 0x01,
        /// PAN access denied
        PanAccessDenied = 0x02
    }
}

numeric_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum DisassociationReason {
        /// The coordinator wishes the device to leave the PAN.
        CoordRequested = 0x01,
        /// The device wishes to leave the PAN.
        DeviceRequested = 0x02,
    }
}

numeric_enum! {
    #[repr(u8)]
    #[derive(Default, Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum SecurityLevel {
        /// MAC Unsecured Mode Security
        #[default]
        Unsecure = 0x00,
        /// MAC ACL Mode Security
        AclMode = 0x01,
        /// MAC Secured Mode Security
        Secured = 0x02,
    }
}

numeric_enum! {
    #[repr(u8)]
    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum ScanType {
        EdScan = 0x00,
        Active = 0x01,
        Passive = 0x02,
        Orphan = 0x03
    }
}

/// newtype for Pan Id
#[derive(Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PanId(pub [u8; 2]);

impl PanId {
    pub const BROADCAST: Self = Self([0xFF, 0xFF]);
}
