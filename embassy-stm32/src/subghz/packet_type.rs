/// Packet type definition.
///
/// Argument of [`set_packet_type`]
///
/// [`set_packet_type`]: super::SubGhz::set_packet_type
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PacketType {
    /// FSK (frequency shift keying) generic packet type.
    Fsk = 0,
    /// LoRa (long range) packet type.
    LoRa = 1,
    /// BPSK (binary phase shift keying) packet type.
    Bpsk = 2,
    /// MSK (minimum shift keying) generic packet type.
    Msk = 3,
}

impl PacketType {
    /// Create a new `PacketType` from bits.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::PacketType;
    ///
    /// assert_eq!(PacketType::from_raw(0), Ok(PacketType::Fsk));
    /// assert_eq!(PacketType::from_raw(1), Ok(PacketType::LoRa));
    /// assert_eq!(PacketType::from_raw(2), Ok(PacketType::Bpsk));
    /// assert_eq!(PacketType::from_raw(3), Ok(PacketType::Msk));
    /// // Other values are reserved
    /// assert_eq!(PacketType::from_raw(4), Err(4));
    /// ```
    pub const fn from_raw(bits: u8) -> Result<PacketType, u8> {
        match bits {
            0 => Ok(PacketType::Fsk),
            1 => Ok(PacketType::LoRa),
            2 => Ok(PacketType::Bpsk),
            3 => Ok(PacketType::Msk),
            _ => Err(bits),
        }
    }
}
