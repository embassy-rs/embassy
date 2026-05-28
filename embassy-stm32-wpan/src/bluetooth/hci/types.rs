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

/// DTM packet payload pattern.                                                                                    
/// BLE Core Spec v6.0, Vol 4, Part E, Section 7.8.29, page 2547.                                                  
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DtmPacketPayload {
    /// PRBS9 sequence as described in [Vol 6] Part F, Section 4.1.5                                               
    Prbs9 = 0x00,
    /// Repeated '11110000' sequence as described in [Vol 6] Part F, Section 4.1.5
    Repeated11110000 = 0x01,
    /// Repeated '10101010' sequence as described in [Vol 6] Part F, Section 4.1.5
    Repeated10101010 = 0x02,
    /// PRBS15 sequence as described in [Vol 6] Part F, Section 4.1.5
    Prbs15 = 0x03,
    /// Repeated '11111111' sequence
    AllOnes = 0x04,
    /// Repeated '00000000' sequence
    AllZeros = 0x05,
    /// Repeated '00001111' sequence
    Repeated00001111 = 0x06,
    /// Repeated '01010101' sequence                                                                               
    Repeated01010101 = 0x07,
}

/// PHY for DTM transmitter test.                                                                                  
/// BLE Core Spec v6.0, Vol 4, Part E, Section 7.8.29, page 2548.                                                  
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DtmTxPhy {
    /// LE 1M PHY                                                                                                  
    Le1M = 0x01,
    /// LE 2M PHY                                                                                                  
    Le2M = 0x02,
    /// LE Coded PHY with S=8 data coding (125 kbps)                                                               
    LeCodedS8 = 0x03,
    /// LE Coded PHY with S=2 data coding (500 kbps)
    LeCodedS2 = 0x04,
}

/// PHY for DTM receiver test.                                                                                     
/// BLE Core Spec v6.0, Vol 4, Part E, Section 7.8.28, page 2542.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DtmRxPhy {
    /// LE 1M PHY
    Le1M = 0x01,
    /// LE 2M PHY
    Le2M = 0x02,
    /// LE Coded PHY
    LeCoded = 0x03,
}
