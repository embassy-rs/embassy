//! Enums shared between CAN controller types.

/// Bus error
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BusError {
    /// Bit stuffing error - more than 5 equal bits
    Stuff,
    /// Form error - A fixed format part of a received message has wrong format
    Form,
    /// The message transmitted by the FDCAN was not acknowledged by another node.
    Acknowledge,
    /// Bit0Error: During the transmission of a message the device wanted to send a dominant level
    /// but the monitored bus value was recessive.
    BitRecessive,
    /// Bit1Error: During the transmission of a message the device wanted to send a recessive level
    /// but the monitored bus value was dominant.
    BitDominant,
    /// The CRC check sum of a received message was incorrect. The CRC of an
    /// incoming message does not match with the CRC calculated from the received data.
    Crc,
    /// A software error occured
    Software,
    ///  The FDCAN is in Bus_Off state.
    BusOff,
    ///  The FDCAN is in the Error_Passive state.
    BusPassive,
    ///  At least one of error counter has reached the Error_Warning limit of 96.
    BusWarning,
}

/// Bus error modes.
///
/// Contrary to the `BusError` enum which also includes last-seen acute protocol
/// errors, this enum includes only the mutually exclusive bus error modes.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BusErrorMode {
    /// Error active mode (default). Controller will transmit an active error
    /// frame upon protocol error.
    ErrorActive,
    /// Error passive mode. An error counter exceeded 127. Controller will
    /// transmit a passive error frame upon protocol error.
    ErrorPassive,
    /// Bus off mode. The transmit error counter exceeded 255. Controller is not
    /// participating in bus traffic.
    BusOff,
}

/// Frame Create Errors
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FrameCreateError {
    /// Data in header does not match supplied.
    NotEnoughData,
    /// Invalid data length not 0-8 for Classic packet or valid for FD.
    InvalidDataLength,
    /// Invalid ID.
    InvalidCanId,
}

/// Error returned by `try_read`
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryReadError {
    /// Bus error
    BusError(BusError),
    /// Receive buffer is empty
    Empty,
}

/// Internal Operation
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InternalOperation {
    /// Notify receiver created
    NotifyReceiverCreated,
    /// Notify receiver destroyed
    NotifyReceiverDestroyed,
    /// Notify sender created
    NotifySenderCreated,
    /// Notify sender destroyed
    NotifySenderDestroyed,
}
