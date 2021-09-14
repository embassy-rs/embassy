/// Receiver event which stops the RX timeout timer.
///
/// Used by [`set_rx_timeout_stop`].
///
/// [`set_rx_timeout_stop`]: super::SubGhz::set_rx_timeout_stop
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum RxTimeoutStop {
    /// Receive timeout stopped on synchronization word detection in generic
    /// packet mode or header detection in LoRa packet mode.
    Sync = 0b0,
    /// Receive timeout stopped on preamble detection.
    Preamble = 0b1,
}

impl From<RxTimeoutStop> for u8 {
    fn from(rx_ts: RxTimeoutStop) -> Self {
        rx_ts as u8
    }
}
