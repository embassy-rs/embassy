/// Operation Errors.
///
/// Returned by [`op_error`].
///
/// [`op_error`]: super::SubGhz::op_error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OpError {
    /// PA ramping failed
    PaRampError = 8,
    /// RF-PLL locking failed
    PllLockError = 6,
    /// HSE32 clock startup failed
    XoscStartError = 5,
    /// Image calibration failed
    ImageCalibrationError = 4,
    /// RF-ADC calibration failed
    AdcCalibrationError = 3,
    /// RF-PLL calibration failed
    PllCalibrationError = 2,
    /// Sub-GHz radio RC 13 MHz oscillator
    RC13MCalibrationError = 1,
    /// Sub-GHz radio RC 64 kHz oscillator
    RC64KCalibrationError = 0,
}

impl OpError {
    /// Get the bitmask for the error.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::OpError;
    ///
    /// assert_eq!(OpError::PaRampError.mask(), 0b1_0000_0000);
    /// assert_eq!(OpError::PllLockError.mask(), 0b0_0100_0000);
    /// assert_eq!(OpError::XoscStartError.mask(), 0b0_0010_0000);
    /// assert_eq!(OpError::ImageCalibrationError.mask(), 0b0_0001_0000);
    /// assert_eq!(OpError::AdcCalibrationError.mask(), 0b0_0000_1000);
    /// assert_eq!(OpError::PllCalibrationError.mask(), 0b0_0000_0100);
    /// assert_eq!(OpError::RC13MCalibrationError.mask(), 0b0_0000_0010);
    /// assert_eq!(OpError::RC64KCalibrationError.mask(), 0b0_0000_0001);
    /// ```
    pub const fn mask(self) -> u16 {
        1 << (self as u8)
    }
}
