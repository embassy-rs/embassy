/// RX gain power modes.
///
/// Argument of [`set_rx_gain`].
///
/// [`set_rx_gain`]: super::SubGhz::set_rx_gain
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PMode {
    /// Power saving mode.
    ///
    /// Reduces sensitivity.
    #[allow(clippy::identity_op)]
    PowerSaving = (0x25 << 2) | 0b00,
    /// Boost mode level 1.
    ///
    /// Improves sensitivity at detriment of power consumption.
    Boost1 = (0x25 << 2) | 0b01,
    /// Boost mode level 2.
    ///
    /// Improves a set further sensitivity at detriment of power consumption.
    Boost2 = (0x25 << 2) | 0b10,
    /// Boost mode.
    ///
    /// Best receiver sensitivity.
    Boost = (0x25 << 2) | 0b11,
}
