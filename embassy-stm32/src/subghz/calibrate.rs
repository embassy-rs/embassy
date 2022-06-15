/// Image calibration.
///
/// Argument of [`calibrate_image`].
///
/// [`calibrate_image`]: crate::subghz::SubGhz::calibrate_image
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CalibrateImage(pub(crate) u8, pub(crate) u8);

impl CalibrateImage {
    /// Image calibration for the 430 - 440 MHz ISM band.
    pub const ISM_430_440: CalibrateImage = CalibrateImage(0x6B, 0x6F);

    /// Image calibration for the 470 - 510 MHz ISM band.
    pub const ISM_470_510: CalibrateImage = CalibrateImage(0x75, 0x81);

    /// Image calibration for the 779 - 787 MHz ISM band.
    pub const ISM_779_787: CalibrateImage = CalibrateImage(0xC1, 0xC5);

    /// Image calibration for the 863 - 870 MHz ISM band.
    pub const ISM_863_870: CalibrateImage = CalibrateImage(0xD7, 0xDB);

    /// Image calibration for the 902 - 928 MHz ISM band.
    pub const ISM_902_928: CalibrateImage = CalibrateImage(0xE1, 0xE9);

    /// Create a new `CalibrateImage` structure from raw values.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::CalibrateImage;
    ///
    /// const CAL: CalibrateImage = CalibrateImage::new(0xE1, 0xE9);
    /// assert_eq!(CAL, CalibrateImage::ISM_902_928);
    /// ```
    pub const fn new(f1: u8, f2: u8) -> CalibrateImage {
        CalibrateImage(f1, f2)
    }

    /// Create a new `CalibrateImage` structure from two frequencies.
    ///
    /// # Arguments
    ///
    /// The units for `freq1` and `freq2` are in MHz.
    ///
    /// # Panics
    ///
    /// * Panics if `freq1` is less than `freq2`.
    /// * Panics if `freq1` or `freq2` is not a multiple of 4MHz.
    /// * Panics if `freq1` or `freq2` is greater than `1020`.
    ///
    /// # Example
    ///
    /// Create an image calibration for the 430 - 440 MHz ISM band.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::CalibrateImage;
    ///
    /// let cal: CalibrateImage = CalibrateImage::from_freq(428, 444);
    /// assert_eq!(cal, CalibrateImage::ISM_430_440);
    /// ```
    pub fn from_freq(freq1: u16, freq2: u16) -> CalibrateImage {
        assert!(freq2 >= freq1);
        assert_eq!(freq1 % 4, 0);
        assert_eq!(freq2 % 4, 0);
        assert!(freq1 <= 1020);
        assert!(freq2 <= 1020);
        CalibrateImage((freq1 / 4) as u8, (freq2 / 4) as u8)
    }
}

impl Default for CalibrateImage {
    fn default() -> Self {
        CalibrateImage::new(0xE1, 0xE9)
    }
}

/// Block calibration.
///
/// Argument of [`calibrate`].
///
/// [`calibrate`]: crate::subghz::SubGhz::calibrate
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Calibrate {
    /// Image calibration
    Image = 1 << 6,
    ///  RF-ADC bulk P calibration
    AdcBulkP = 1 << 5,
    /// RF-ADC bulk N calibration
    AdcBulkN = 1 << 4,
    /// RF-ADC pulse calibration
    AdcPulse = 1 << 3,
    /// RF-PLL calibration
    Pll = 1 << 2,
    /// Sub-GHz radio RC 13 MHz calibration
    Rc13M = 1 << 1,
    /// Sub-GHz radio RC 64 kHz calibration
    Rc64K = 1,
}

impl Calibrate {
    /// Get the bitmask for the block calibration.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::Calibrate;
    ///
    /// assert_eq!(Calibrate::Image.mask(), 0b0100_0000);
    /// assert_eq!(Calibrate::AdcBulkP.mask(), 0b0010_0000);
    /// assert_eq!(Calibrate::AdcBulkN.mask(), 0b0001_0000);
    /// assert_eq!(Calibrate::AdcPulse.mask(), 0b0000_1000);
    /// assert_eq!(Calibrate::Pll.mask(), 0b0000_0100);
    /// assert_eq!(Calibrate::Rc13M.mask(), 0b0000_0010);
    /// assert_eq!(Calibrate::Rc64K.mask(), 0b0000_0001);
    /// ```
    pub const fn mask(self) -> u8 {
        self as u8
    }
}
