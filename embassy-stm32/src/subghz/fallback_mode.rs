/// Fallback mode after successful packet transmission or packet reception.
///
/// Argument of [`set_tx_rx_fallback_mode`].
///
/// [`set_tx_rx_fallback_mode`]: crate::subghz::SubGhz::set_tx_rx_fallback_mode.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum FallbackMode {
    /// Standby mode entry.
    Standby = 0x20,
    /// Standby with HSE32 enabled.
    StandbyHse = 0x30,
    /// Frequency synthesizer entry.
    Fs = 0x40,
}

impl From<FallbackMode> for u8 {
    fn from(fm: FallbackMode) -> Self {
        fm as u8
    }
}

impl Default for FallbackMode {
    /// Default fallback mode after power-on reset.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::FallbackMode;
    ///
    /// assert_eq!(FallbackMode::default(), FallbackMode::Standby);
    /// ```
    fn default() -> Self {
        FallbackMode::Standby
    }
}
