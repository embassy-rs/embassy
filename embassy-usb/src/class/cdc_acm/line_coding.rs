//! CDC line coding (UART parameters), shared by the device and host classes.

use core::mem;

/// Number of stop bits for LineCoding
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StopBits {
    /// 1 stop bit
    One = 0,

    /// 1.5 stop bits
    OnePointFive = 1,

    /// 2 stop bits
    Two = 2,
}

impl From<u8> for StopBits {
    fn from(value: u8) -> Self {
        if value <= 2 {
            unsafe { mem::transmute::<u8, StopBits>(value) }
        } else {
            StopBits::One
        }
    }
}

/// Parity for LineCoding
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ParityType {
    /// No parity bit.
    None = 0,
    /// Parity bit is 1 if the amount of `1` bits in the data byte is odd.
    Odd = 1,
    /// Parity bit is 1 if the amount of `1` bits in the data byte is even.
    Even = 2,
    /// Parity bit is always 1
    Mark = 3,
    /// Parity bit is always 0
    Space = 4,
}

impl From<u8> for ParityType {
    fn from(value: u8) -> Self {
        if value <= 4 {
            unsafe { mem::transmute::<u8, ParityType>(value) }
        } else {
            ParityType::None
        }
    }
}

/// Line coding parameters
///
/// This is provided by the host for specifying the standard UART parameters such as baud rate. Can
/// be ignored if you don't plan to interface with a physical UART.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LineCoding {
    stop_bits: StopBits,
    data_bits: u8,
    parity_type: ParityType,
    data_rate: u32,
}

impl LineCoding {
    /// Creates line coding parameters.
    pub const fn new(data_rate: u32, stop_bits: StopBits, parity_type: ParityType, data_bits: u8) -> Self {
        Self {
            stop_bits,
            data_bits,
            parity_type,
            data_rate,
        }
    }

    /// Decodes the 7-byte `SET_LINE_CODING`/`GET_LINE_CODING` payload (CDC PSTN 1.2 Table 17).
    ///
    /// Out-of-range stop bits and parity fall back to one stop bit and no parity.
    pub fn from_bytes(bytes: [u8; 7]) -> Self {
        Self {
            data_rate: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            stop_bits: bytes[4].into(),
            parity_type: bytes[5].into(),
            data_bits: bytes[6],
        }
    }

    /// Encodes the 7-byte `SET_LINE_CODING`/`GET_LINE_CODING` payload.
    pub const fn to_bytes(&self) -> [u8; 7] {
        let rate = self.data_rate.to_le_bytes();
        [
            rate[0],
            rate[1],
            rate[2],
            rate[3],
            self.stop_bits as u8,
            self.parity_type as u8,
            self.data_bits,
        ]
    }

    /// Gets the number of stop bits for UART communication.
    pub fn stop_bits(&self) -> StopBits {
        self.stop_bits
    }

    /// Gets the number of data bits for UART communication.
    pub const fn data_bits(&self) -> u8 {
        self.data_bits
    }

    /// Gets the parity type for UART communication.
    pub const fn parity_type(&self) -> ParityType {
        self.parity_type
    }

    /// Gets the data rate in bits per second for UART communication.
    pub const fn data_rate(&self) -> u32 {
        self.data_rate
    }
}

impl LineCoding {
    pub(crate) const DEFAULT: Self = Self::new(8_000, StopBits::One, ParityType::None, 8);
}

impl Default for LineCoding {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_and_decodes_wire_format() {
        let coding = LineCoding::new(115_200, StopBits::Two, ParityType::Even, 7);
        let bytes = [0x00, 0xc2, 0x01, 0x00, 2, 2, 7];
        assert_eq!(coding.to_bytes(), bytes);
        assert_eq!(LineCoding::from_bytes(bytes), coding);
    }

    #[test]
    fn decodes_reserved_fields_to_defaults() {
        let coding = LineCoding::from_bytes([0x40, 0x1f, 0, 0, 3, 5, 8]);
        assert_eq!(coding.stop_bits(), StopBits::One);
        assert_eq!(coding.parity_type(), ParityType::None);
        assert_eq!(coding.data_rate(), 8_000);
    }
}
