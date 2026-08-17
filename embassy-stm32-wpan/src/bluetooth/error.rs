//! BLE Error types

use stm32wb_hci::host::Error as HostError;
use stm32wb_hci::vendor::command::gap::Error as GapError;
use stm32wb_hci::vendor::command::gatt::Error as GattError;
use stm32wb_hci::vendor::command::hal::Error as HalError;

use super::hci::types::Status;

/// BLE Stack Errors
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BleError {
    /// BLE stack not initialized or runner not initialized
    NotInitialized,

    /// BLE stack initialization failed
    InitializationFailed,

    /// HCI command failed with status code
    CommandFailed(Status),

    /// Operation timed out
    Timeout,

    /// Invalid parameter provided
    InvalidParameter,

    /// Buffer full (event queue, command queue, etc.)
    BufferFull,

    /// Hardware error
    HardwareError(u8),

    /// Connection error
    ConnectionError,

    // Controller Host Error
    HostError(HostError),

    // Controller Gatt Error
    GattError(GattError),

    // Controller Gap Error
    GapError(GapError),

    // Controller Hal Error
    HalError(HalError),

    /// Unknown or unspecified error
    Unknown,
}

impl From<HostError> for BleError {
    fn from(err: HostError) -> Self {
        Self::HostError(err)
    }
}

impl From<GattError> for BleError {
    fn from(err: GattError) -> Self {
        Self::GattError(err)
    }
}

impl From<GapError> for BleError {
    fn from(err: GapError) -> Self {
        Self::GapError(err)
    }
}

impl From<HalError> for BleError {
    fn from(err: HalError) -> Self {
        Self::HalError(err)
    }
}

impl From<Status> for BleError {
    fn from(status: Status) -> Self {
        match status {
            Status::Success => BleError::Unknown, // Shouldn't convert success to error
            Status::HardwareFailure => BleError::HardwareError(0x03),
            Status::InvalidHciCommandParameters => BleError::InvalidParameter,
            Status::ConnectionTimeout => BleError::Timeout,
            Status::ConnectionLimitExceeded => BleError::ConnectionError,
            Status::ConnectionFailedToBeEstablished => BleError::ConnectionError,
            _ => BleError::CommandFailed(status),
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Status {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Status::Success => defmt::write!(fmt, "Success"),
            Status::UnknownCommand => defmt::write!(fmt, "UnknownCommand"),
            Status::UnknownConnectionId => defmt::write!(fmt, "UnknownConnectionId"),
            Status::HardwareFailure => defmt::write!(fmt, "HardwareFailure"),
            Status::PageTimeout => defmt::write!(fmt, "PageTimeout"),
            Status::AuthenticationFailure => defmt::write!(fmt, "AuthenticationFailure"),
            Status::PinOrKeyMissing => defmt::write!(fmt, "PinOrKeyMissing"),
            Status::MemoryCapacityExceeded => defmt::write!(fmt, "MemoryCapacityExceeded"),
            Status::ConnectionTimeout => defmt::write!(fmt, "ConnectionTimeout"),
            Status::ConnectionLimitExceeded => defmt::write!(fmt, "ConnectionLimitExceeded"),
            Status::InvalidHciCommandParameters => defmt::write!(fmt, "InvalidHciCommandParameters"),
            Status::RemoteUserTerminatedConnection => defmt::write!(fmt, "RemoteUserTerminatedConnection"),
            Status::ConnectionTerminatedByLocalHost => defmt::write!(fmt, "ConnectionTerminatedByLocalHost"),
            Status::UnsupportedRemoteFeature => defmt::write!(fmt, "UnsupportedRemoteFeature"),
            Status::InvalidLmpParameters => defmt::write!(fmt, "InvalidLmpParameters"),
            Status::UnspecifiedError => defmt::write!(fmt, "UnspecifiedError"),
            Status::UnsupportedLmpParameterValue => defmt::write!(fmt, "UnsupportedLmpParameterValue"),
            Status::RoleChangeNotAllowed => defmt::write!(fmt, "RoleChangeNotAllowed"),
            Status::LmpResponseTimeout => defmt::write!(fmt, "LmpResponseTimeout"),
            Status::ControllerBusy => defmt::write!(fmt, "ControllerBusy"),
            Status::UnacceptableConnectionParameters => defmt::write!(fmt, "UnacceptableConnectionParameters"),
            Status::AdvertisingTimeout => defmt::write!(fmt, "AdvertisingTimeout"),
            Status::ConnectionTerminatedDueToMicFailure => defmt::write!(fmt, "ConnectionTerminatedDueToMicFailure"),
            Status::ConnectionFailedToBeEstablished => defmt::write!(fmt, "ConnectionFailedToBeEstablished"),
        }
    }
}
