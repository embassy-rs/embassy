/// Clock in standby mode.
///
/// Used by [`set_standby`].
///
/// [`set_standby`]: super::SubGhz::set_standby
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum StandbyClk {
    /// RC 13 MHz used in standby mode.
    Rc = 0b0,
    /// HSE32 used in standby mode.
    Hse = 0b1,
}

impl From<StandbyClk> for u8 {
    fn from(sc: StandbyClk) -> Self {
        sc as u8
    }
}
