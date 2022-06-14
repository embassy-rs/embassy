use super::{Ratio, Status};

/// (G)FSK packet status.
///
/// Returned by [`fsk_packet_status`].
///
/// [`fsk_packet_status`]: super::SubGhz::fsk_packet_status
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FskPacketStatus {
    buf: [u8; 4],
}

impl From<[u8; 4]> for FskPacketStatus {
    fn from(buf: [u8; 4]) -> Self {
        FskPacketStatus { buf }
    }
}

impl FskPacketStatus {
    /// Get the status.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CmdStatus, FskPacketStatus, Status, StatusMode};
    ///
    /// let example_data_from_radio: [u8; 4] = [0x54, 0, 0, 0];
    /// let pkt_status: FskPacketStatus = FskPacketStatus::from(example_data_from_radio);
    /// let status: Status = pkt_status.status();
    /// assert_eq!(status.mode(), Ok(StatusMode::Rx));
    /// assert_eq!(status.cmd(), Ok(CmdStatus::Avaliable));
    /// ```
    pub const fn status(&self) -> Status {
        Status::from_raw(self.buf[0])
    }

    /// Returns `true` if a preamble error occurred.
    pub const fn preamble_err(&self) -> bool {
        (self.buf[1] & (1 << 7)) != 0
    }

    /// Returns `true` if a synchronization error occurred.
    pub const fn sync_err(&self) -> bool {
        (self.buf[1] & (1 << 6)) != 0
    }

    /// Returns `true` if an address error occurred.
    pub const fn addr_err(&self) -> bool {
        (self.buf[1] & (1 << 5)) != 0
    }

    /// Returns `true` if an CRC error occurred.
    pub const fn crc_err(&self) -> bool {
        (self.buf[1] & (1 << 4)) != 0
    }

    /// Returns `true` if a length error occurred.
    pub const fn length_err(&self) -> bool {
        (self.buf[1] & (1 << 3)) != 0
    }

    /// Returns `true` if an abort error occurred.
    pub const fn abort_err(&self) -> bool {
        (self.buf[1] & (1 << 2)) != 0
    }

    /// Returns `true` if a packet is received.
    pub const fn pkt_received(&self) -> bool {
        (self.buf[1] & (1 << 1)) != 0
    }

    /// Returns `true` when a packet has been sent.
    pub const fn pkt_sent(&self) -> bool {
        (self.buf[1] & 1) != 0
    }

    /// Returns `true` if any error occurred.
    pub const fn any_err(&self) -> bool {
        (self.buf[1] & 0xFC) != 0
    }

    /// RSSI level when the synchronization address is detected.
    ///
    /// Units are in dBm.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::{subghz::FskPacketStatus, Ratio};
    ///
    /// let example_data_from_radio: [u8; 4] = [0, 0, 80, 0];
    /// let pkt_status: FskPacketStatus = FskPacketStatus::from(example_data_from_radio);
    /// assert_eq!(pkt_status.rssi_sync().to_integer(), -40);
    /// ```
    pub fn rssi_sync(&self) -> Ratio<i16> {
        Ratio::new_raw(i16::from(self.buf[2]), -2)
    }

    /// Return the RSSI level over the received packet.
    ///
    /// Units are in dBm.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::{subghz::FskPacketStatus, Ratio};
    ///
    /// let example_data_from_radio: [u8; 4] = [0, 0, 0, 100];
    /// let pkt_status: FskPacketStatus = FskPacketStatus::from(example_data_from_radio);
    /// assert_eq!(pkt_status.rssi_avg().to_integer(), -50);
    /// ```
    pub fn rssi_avg(&self) -> Ratio<i16> {
        Ratio::new_raw(i16::from(self.buf[3]), -2)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for FskPacketStatus {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            r#"FskPacketStatus {{
    status: {},
    preamble_err: {},
    sync_err: {},
    addr_err: {},
    crc_err: {},
    length_err: {},
    abort_err: {},
    pkt_received: {},
    pkt_sent: {},
    rssi_sync: {},
    rssi_avg: {},
}}"#,
            self.status(),
            self.preamble_err(),
            self.sync_err(),
            self.addr_err(),
            self.crc_err(),
            self.length_err(),
            self.abort_err(),
            self.pkt_received(),
            self.pkt_sent(),
            self.rssi_sync().to_integer(),
            self.rssi_avg().to_integer()
        )
    }
}

impl core::fmt::Debug for FskPacketStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FskPacketStatus")
            .field("status", &self.status())
            .field("preamble_err", &self.preamble_err())
            .field("sync_err", &self.sync_err())
            .field("addr_err", &self.addr_err())
            .field("crc_err", &self.crc_err())
            .field("length_err", &self.length_err())
            .field("abort_err", &self.abort_err())
            .field("pkt_received", &self.pkt_received())
            .field("pkt_sent", &self.pkt_sent())
            .field("rssi_sync", &self.rssi_sync().to_integer())
            .field("rssi_avg", &self.rssi_avg().to_integer())
            .finish()
    }
}

/// (G)FSK packet status.
///
/// Returned by [`lora_packet_status`].
///
/// [`lora_packet_status`]: super::SubGhz::lora_packet_status
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LoRaPacketStatus {
    buf: [u8; 4],
}

impl From<[u8; 4]> for LoRaPacketStatus {
    fn from(buf: [u8; 4]) -> Self {
        LoRaPacketStatus { buf }
    }
}

impl LoRaPacketStatus {
    /// Get the status.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CmdStatus, LoRaPacketStatus, Status, StatusMode};
    ///
    /// let example_data_from_radio: [u8; 4] = [0x54, 0, 0, 0];
    /// let pkt_status: LoRaPacketStatus = LoRaPacketStatus::from(example_data_from_radio);
    /// let status: Status = pkt_status.status();
    /// assert_eq!(status.mode(), Ok(StatusMode::Rx));
    /// assert_eq!(status.cmd(), Ok(CmdStatus::Avaliable));
    /// ```
    pub const fn status(&self) -> Status {
        Status::from_raw(self.buf[0])
    }

    /// Average RSSI level over the received packet.
    ///
    /// Units are in dBm.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::{subghz::LoRaPacketStatus, Ratio};
    ///
    /// let example_data_from_radio: [u8; 4] = [0, 80, 0, 0];
    /// let pkt_status: LoRaPacketStatus = LoRaPacketStatus::from(example_data_from_radio);
    /// assert_eq!(pkt_status.rssi_pkt().to_integer(), -40);
    /// ```
    pub fn rssi_pkt(&self) -> Ratio<i16> {
        Ratio::new_raw(i16::from(self.buf[1]), -2)
    }

    /// Estimation of SNR over the received packet.
    ///
    /// Units are in dB.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::{subghz::LoRaPacketStatus, Ratio};
    ///
    /// let example_data_from_radio: [u8; 4] = [0, 0, 40, 0];
    /// let pkt_status: LoRaPacketStatus = LoRaPacketStatus::from(example_data_from_radio);
    /// assert_eq!(pkt_status.snr_pkt().to_integer(), 10);
    /// ```
    pub fn snr_pkt(&self) -> Ratio<i16> {
        Ratio::new_raw(i16::from(self.buf[2]), 4)
    }

    /// Estimation of RSSI level of the LoRa signal after despreading.
    ///
    /// Units are in dBm.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::{subghz::LoRaPacketStatus, Ratio};
    ///
    /// let example_data_from_radio: [u8; 4] = [0, 0, 0, 80];
    /// let pkt_status: LoRaPacketStatus = LoRaPacketStatus::from(example_data_from_radio);
    /// assert_eq!(pkt_status.signal_rssi_pkt().to_integer(), -40);
    /// ```
    pub fn signal_rssi_pkt(&self) -> Ratio<i16> {
        Ratio::new_raw(i16::from(self.buf[3]), -2)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for LoRaPacketStatus {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            r#"LoRaPacketStatus {{
    status: {},
    rssi_pkt: {},
    snr_pkt: {},
    signal_rssi_pkt: {},
}}"#,
            self.status(),
            self.rssi_pkt().to_integer(),
            self.snr_pkt().to_integer(),
            self.signal_rssi_pkt().to_integer(),
        )
    }
}

impl core::fmt::Debug for LoRaPacketStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LoRaPacketStatus")
            .field("status", &self.status())
            .field("rssi_pkt", &self.rssi_pkt().to_integer())
            .field("snr_pkt", &self.snr_pkt().to_integer())
            .field("signal_rssi_pkt", &self.signal_rssi_pkt().to_integer())
            .finish()
    }
}
