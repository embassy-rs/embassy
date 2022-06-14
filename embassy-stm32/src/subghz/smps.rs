/// SMPS maximum drive capability.
///
/// Argument of [`set_smps_drv`](super::SubGhz::set_smps_drv).
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SmpsDrv {
    /// 20 mA
    Milli20 = 0x0,
    /// 40 mA
    Milli40 = 0x1,
    /// 60 mA
    Milli60 = 0x2,
    /// 100 mA (default)
    Milli100 = 0x3,
}

impl SmpsDrv {
    /// Get the SMPS drive value as milliamps.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::SmpsDrv;
    ///
    /// assert_eq!(SmpsDrv::Milli20.as_milliamps(), 20);
    /// assert_eq!(SmpsDrv::Milli40.as_milliamps(), 40);
    /// assert_eq!(SmpsDrv::Milli60.as_milliamps(), 60);
    /// assert_eq!(SmpsDrv::Milli100.as_milliamps(), 100);
    /// ```
    pub const fn as_milliamps(&self) -> u8 {
        match self {
            SmpsDrv::Milli20 => 20,
            SmpsDrv::Milli40 => 40,
            SmpsDrv::Milli60 => 60,
            SmpsDrv::Milli100 => 100,
        }
    }
}

impl Default for SmpsDrv {
    fn default() -> Self {
        SmpsDrv::Milli100
    }
}
