//! HCI types, constants, and opcodes

/// HCI Packet types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    Command = 0x01,
    AclData = 0x02,
    SyncData = 0x03,
    Event = 0x04,
}

/// HCI Opcode Group Field (OGF)
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeGroup {
    LinkControl = 0x01,
    LinkPolicy = 0x02,
    ControllerAndBaseband = 0x03,
    InformationalParameters = 0x04,
    StatusParameters = 0x05,
    Testing = 0x06,
    LeController = 0x08,
    VendorSpecific = 0x3F,
}

/// Construct HCI opcode from OGF and OCF
pub const fn opcode(ogf: u16, ocf: u16) -> u16 {
    (ogf << 10) | ocf
}

/// HCI Command Opcodes - Link Control
pub mod link_control {
    use super::*;

    pub const DISCONNECT: u16 = opcode(OpcodeGroup::LinkControl as u16, 0x0006);
}

/// HCI Command Opcodes - Controller & Baseband
pub mod controller {
    use super::*;

    pub const RESET: u16 = opcode(OpcodeGroup::ControllerAndBaseband as u16, 0x0003);
    pub const SET_EVENT_MASK: u16 = opcode(OpcodeGroup::ControllerAndBaseband as u16, 0x0001);
    pub const READ_TRANSMIT_POWER_LEVEL: u16 = opcode(OpcodeGroup::ControllerAndBaseband as u16, 0x002D);
}

/// HCI Command Opcodes - Informational Parameters
pub mod info {
    use super::*;

    pub const READ_LOCAL_VERSION: u16 = opcode(OpcodeGroup::InformationalParameters as u16, 0x0001);
    pub const READ_LOCAL_SUPPORTED_COMMANDS: u16 = opcode(OpcodeGroup::InformationalParameters as u16, 0x0002);
    pub const READ_LOCAL_SUPPORTED_FEATURES: u16 = opcode(OpcodeGroup::InformationalParameters as u16, 0x0003);
    pub const READ_BD_ADDR: u16 = opcode(OpcodeGroup::InformationalParameters as u16, 0x0009);
}

/// HCI Command Opcodes - LE Controller
pub mod le {
    use super::*;

    pub const SET_EVENT_MASK: u16 = opcode(OpcodeGroup::LeController as u16, 0x0001);
    pub const READ_BUFFER_SIZE: u16 = opcode(OpcodeGroup::LeController as u16, 0x0002);
    pub const READ_LOCAL_SUPPORTED_FEATURES: u16 = opcode(OpcodeGroup::LeController as u16, 0x0003);
    pub const SET_RANDOM_ADDRESS: u16 = opcode(OpcodeGroup::LeController as u16, 0x0005);
    pub const SET_ADVERTISING_PARAMETERS: u16 = opcode(OpcodeGroup::LeController as u16, 0x0006);
    pub const READ_ADVERTISING_CHANNEL_TX_POWER: u16 = opcode(OpcodeGroup::LeController as u16, 0x0007);
    pub const SET_ADVERTISING_DATA: u16 = opcode(OpcodeGroup::LeController as u16, 0x0008);
    pub const SET_SCAN_RESPONSE_DATA: u16 = opcode(OpcodeGroup::LeController as u16, 0x0009);
    pub const SET_ADVERTISE_ENABLE: u16 = opcode(OpcodeGroup::LeController as u16, 0x000A);
    pub const SET_SCAN_PARAMETERS: u16 = opcode(OpcodeGroup::LeController as u16, 0x000B);
    pub const SET_SCAN_ENABLE: u16 = opcode(OpcodeGroup::LeController as u16, 0x000C);
    pub const CREATE_CONNECTION: u16 = opcode(OpcodeGroup::LeController as u16, 0x000D);
    pub const CREATE_CONNECTION_CANCEL: u16 = opcode(OpcodeGroup::LeController as u16, 0x000E);
    pub const READ_WHITE_LIST_SIZE: u16 = opcode(OpcodeGroup::LeController as u16, 0x000F);
    pub const CLEAR_WHITE_LIST: u16 = opcode(OpcodeGroup::LeController as u16, 0x0010);
    pub const ADD_DEVICE_TO_WHITE_LIST: u16 = opcode(OpcodeGroup::LeController as u16, 0x0011);
    pub const REMOVE_DEVICE_FROM_WHITE_LIST: u16 = opcode(OpcodeGroup::LeController as u16, 0x0012);
    pub const CONNECTION_UPDATE: u16 = opcode(OpcodeGroup::LeController as u16, 0x0013);
    pub const READ_REMOTE_FEATURES: u16 = opcode(OpcodeGroup::LeController as u16, 0x0016);
    pub const ENCRYPT: u16 = opcode(OpcodeGroup::LeController as u16, 0x0017);
    pub const RAND: u16 = opcode(OpcodeGroup::LeController as u16, 0x0018);
    pub const START_ENCRYPTION: u16 = opcode(OpcodeGroup::LeController as u16, 0x0019);
    pub const LONG_TERM_KEY_REQUEST_REPLY: u16 = opcode(OpcodeGroup::LeController as u16, 0x001A);
    pub const LONG_TERM_KEY_REQUEST_NEGATIVE_REPLY: u16 = opcode(OpcodeGroup::LeController as u16, 0x001B);
    pub const READ_SUPPORTED_STATES: u16 = opcode(OpcodeGroup::LeController as u16, 0x001C);
}

/// HCI Status Codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Success = 0x00,
    UnknownCommand = 0x01,
    UnknownConnectionId = 0x02,
    HardwareFailure = 0x03,
    PageTimeout = 0x04,
    AuthenticationFailure = 0x05,
    PinOrKeyMissing = 0x06,
    MemoryCapacityExceeded = 0x07,
    ConnectionTimeout = 0x08,
    ConnectionLimitExceeded = 0x09,
    InvalidHciCommandParameters = 0x12,
    RemoteUserTerminatedConnection = 0x13,
    ConnectionTerminatedByLocalHost = 0x16,
    UnsupportedRemoteFeature = 0x1A,
    InvalidLmpParameters = 0x1E,
    UnspecifiedError = 0x1F,
    UnsupportedLmpParameterValue = 0x20,
    RoleChangeNotAllowed = 0x21,
    LmpResponseTimeout = 0x22,
    ControllerBusy = 0x3A,
    UnacceptableConnectionParameters = 0x3B,
    AdvertisingTimeout = 0x3C,
    ConnectionTerminatedDueToMicFailure = 0x3D,
    ConnectionFailedToBeEstablished = 0x3E,
}

impl Status {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x00 => Status::Success,
            0x01 => Status::UnknownCommand,
            0x02 => Status::UnknownConnectionId,
            0x03 => Status::HardwareFailure,
            0x04 => Status::PageTimeout,
            0x05 => Status::AuthenticationFailure,
            0x06 => Status::PinOrKeyMissing,
            0x07 => Status::MemoryCapacityExceeded,
            0x08 => Status::ConnectionTimeout,
            0x09 => Status::ConnectionLimitExceeded,
            0x12 => Status::InvalidHciCommandParameters,
            0x13 => Status::RemoteUserTerminatedConnection,
            0x16 => Status::ConnectionTerminatedByLocalHost,
            0x1A => Status::UnsupportedRemoteFeature,
            0x1E => Status::InvalidLmpParameters,
            0x1F => Status::UnspecifiedError,
            0x20 => Status::UnsupportedLmpParameterValue,
            0x21 => Status::RoleChangeNotAllowed,
            0x22 => Status::LmpResponseTimeout,
            0x3A => Status::ControllerBusy,
            0x3B => Status::UnacceptableConnectionParameters,
            0x3C => Status::AdvertisingTimeout,
            0x3D => Status::ConnectionTerminatedDueToMicFailure,
            0x3E => Status::ConnectionFailedToBeEstablished,
            _ => Status::UnspecifiedError,
        }
    }
}

/// BLE Address Type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddressType {
    Public = 0x00,
    Random = 0x01,
    PublicIdentity = 0x02,
    RandomIdentity = 0x03,
}

/// BLE Address (6 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Address(pub [u8; 6]);

impl Address {
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }
}

/// Connection Handle (12-bit value)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Handle(pub u16);

impl Handle {
    pub const fn new(handle: u16) -> Self {
        Self(handle & 0x0FFF)
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

/// Advertising Type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvType {
    /// Connectable and scannable undirected advertising
    ConnectableUndirected = 0x00,
    /// Connectable high duty cycle directed advertising
    ConnectableDirectedHighDutyCycle = 0x01,
    /// Scannable undirected advertising
    ScannableUndirected = 0x02,
    /// Non-connectable undirected advertising
    NonConnectableUndirected = 0x03,
    /// Connectable low duty cycle directed advertising
    ConnectableDirectedLowDutyCycle = 0x04,
}

/// Own Address Type for advertising/scanning
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OwnAddressType {
    Public = 0x00,
    Random = 0x01,
    ResolvableOrPublic = 0x02,
    ResolvableOrRandom = 0x03,
}

/// Advertising Filter Policy
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdvFilterPolicy {
    /// Process scan and connection requests from all devices
    All = 0x00,
    /// Process connection requests from all devices and scan requests only from white list
    ConnAllScanWhiteList = 0x01,
    /// Process scan requests from all devices and connection requests only from white list
    ScanAllConnWhiteList = 0x02,
    /// Process scan and connection requests only from white list
    WhiteListOnly = 0x03,
}
