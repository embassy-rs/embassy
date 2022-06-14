use super::Status;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LoRaStats;

impl LoRaStats {
    pub const fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FskStats;

impl FskStats {
    pub const fn new() -> Self {
        Self {}
    }
}

/// Packet statistics.
///
/// Returned by [`fsk_stats`] and [`lora_stats`].
///
/// [`fsk_stats`]: super::SubGhz::fsk_stats
/// [`lora_stats`]: super::SubGhz::lora_stats
#[derive(Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Stats<ModType> {
    status: Status,
    pkt_rx: u16,
    pkt_crc: u16,
    pkt_len_or_hdr_err: u16,
    ty: ModType,
}

impl<ModType> Stats<ModType> {
    const fn from_buf(buf: [u8; 7], ty: ModType) -> Stats<ModType> {
        Stats {
            status: Status::from_raw(buf[0]),
            pkt_rx: u16::from_be_bytes([buf[1], buf[2]]),
            pkt_crc: u16::from_be_bytes([buf[3], buf[4]]),
            pkt_len_or_hdr_err: u16::from_be_bytes([buf[5], buf[6]]),
            ty,
        }
    }

    /// Get the radio status returned with the packet statistics.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CmdStatus, FskStats, Stats, StatusMode};
    ///
    /// let example_data_from_radio: [u8; 7] = [0x54, 0, 0, 0, 0, 0, 0];
    /// let stats: Stats<FskStats> = Stats::from_raw_fsk(example_data_from_radio);
    /// assert_eq!(stats.status().mode(), Ok(StatusMode::Rx));
    /// assert_eq!(stats.status().cmd(), Ok(CmdStatus::Avaliable));
    /// ```
    pub const fn status(&self) -> Status {
        self.status
    }

    /// Number of packets received.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{FskStats, Stats};
    ///
    /// let example_data_from_radio: [u8; 7] = [0x54, 0, 3, 0, 0, 0, 0];
    /// let stats: Stats<FskStats> = Stats::from_raw_fsk(example_data_from_radio);
    /// assert_eq!(stats.pkt_rx(), 3);
    /// ```
    pub const fn pkt_rx(&self) -> u16 {
        self.pkt_rx
    }

    /// Number of packets received with a payload CRC error
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{LoRaStats, Stats};
    ///
    /// let example_data_from_radio: [u8; 7] = [0x54, 0, 0, 0, 1, 0, 0];
    /// let stats: Stats<LoRaStats> = Stats::from_raw_lora(example_data_from_radio);
    /// assert_eq!(stats.pkt_crc(), 1);
    /// ```
    pub const fn pkt_crc(&self) -> u16 {
        self.pkt_crc
    }
}

impl Stats<FskStats> {
    /// Create a new FSK packet statistics structure from a raw buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{FskStats, Stats};
    ///
    /// let example_data_from_radio: [u8; 7] = [0x54, 0, 0, 0, 0, 0, 0];
    /// let stats: Stats<FskStats> = Stats::from_raw_fsk(example_data_from_radio);
    /// ```
    pub const fn from_raw_fsk(buf: [u8; 7]) -> Stats<FskStats> {
        Self::from_buf(buf, FskStats::new())
    }

    /// Number of packets received with a payload length error.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{FskStats, Stats};
    ///
    /// let example_data_from_radio: [u8; 7] = [0x54, 0, 0, 0, 0, 0, 1];
    /// let stats: Stats<FskStats> = Stats::from_raw_fsk(example_data_from_radio);
    /// assert_eq!(stats.pkt_len_err(), 1);
    /// ```
    pub const fn pkt_len_err(&self) -> u16 {
        self.pkt_len_or_hdr_err
    }
}

impl Stats<LoRaStats> {
    /// Create a new LoRa packet statistics structure from a raw buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{LoRaStats, Stats};
    ///
    /// let example_data_from_radio: [u8; 7] = [0x54, 0, 0, 0, 0, 0, 0];
    /// let stats: Stats<LoRaStats> = Stats::from_raw_lora(example_data_from_radio);
    /// ```
    pub const fn from_raw_lora(buf: [u8; 7]) -> Stats<LoRaStats> {
        Self::from_buf(buf, LoRaStats::new())
    }

    /// Number of packets received with a header CRC error.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{LoRaStats, Stats};
    ///
    /// let example_data_from_radio: [u8; 7] = [0x54, 0, 0, 0, 0, 0, 1];
    /// let stats: Stats<LoRaStats> = Stats::from_raw_lora(example_data_from_radio);
    /// assert_eq!(stats.pkt_hdr_err(), 1);
    /// ```
    pub const fn pkt_hdr_err(&self) -> u16 {
        self.pkt_len_or_hdr_err
    }
}

impl core::fmt::Debug for Stats<FskStats> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Stats")
            .field("status", &self.status())
            .field("pkt_rx", &self.pkt_rx())
            .field("pkt_crc", &self.pkt_crc())
            .field("pkt_len_err", &self.pkt_len_err())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::super::{CmdStatus, LoRaStats, Stats, StatusMode};

    #[test]
    fn mixed() {
        let example_data_from_radio: [u8; 7] = [0x54, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let stats: Stats<LoRaStats> = Stats::from_raw_lora(example_data_from_radio);
        assert_eq!(stats.status().mode(), Ok(StatusMode::Rx));
        assert_eq!(stats.status().cmd(), Ok(CmdStatus::Avaliable));
        assert_eq!(stats.pkt_rx(), 0x0102);
        assert_eq!(stats.pkt_crc(), 0x0304);
        assert_eq!(stats.pkt_hdr_err(), 0x0506);
    }
}
