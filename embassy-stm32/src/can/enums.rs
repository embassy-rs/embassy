//! Enums shared between CAN controller types.
use core::convert::TryFrom;

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
impl TryFrom<u8> for BusError {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            //0b000 => None,
            0b001 => Ok(Self::Stuff),
            0b010 => Ok(Self::Form),
            0b011 => Ok(Self::Acknowledge),
            0b100 => Ok(Self::BitRecessive),
            0b101 => Ok(Self::BitDominant),
            0b110 => Ok(Self::Crc),
            //0b111 => Ok(Self::NoError),
            _ => Err(()),
        }
    }
}
