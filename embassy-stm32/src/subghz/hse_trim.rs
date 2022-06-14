use super::ValueError;

/// HSE32 load capacitor trimming.
///
/// Argument of [`set_hse_in_trim`] and [`set_hse_out_trim`].
///
/// [`set_hse_in_trim`]: crate::subghz::SubGhz::set_hse_in_trim
/// [`set_hse_out_trim`]: crate::subghz::SubGhz::set_hse_out_trim
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HseTrim {
    val: u8,
}

impl HseTrim {
    /// Maximum capacitor value, ~33.4 pF
    pub const MAX: HseTrim = HseTrim::from_raw(0x2F);

    /// Minimum capacitor value, ~11.3 pF
    pub const MIN: HseTrim = HseTrim::from_raw(0x00);

    /// Power-on-reset capacitor value, ~20.3 pF
    ///
    /// This is the same as `default`.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::HseTrim;
    ///
    /// assert_eq!(HseTrim::POR, HseTrim::default());
    /// ```
    pub const POR: HseTrim = HseTrim::from_raw(0x12);

    /// Create a new [`HseTrim`] structure from a raw value.
    ///
    /// Values greater than the maximum of `0x2F` will be set to the maximum.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::HseTrim;
    ///
    /// assert_eq!(HseTrim::from_raw(0xFF), HseTrim::MAX);
    /// assert_eq!(HseTrim::from_raw(0x2F), HseTrim::MAX);
    /// assert_eq!(HseTrim::from_raw(0x00), HseTrim::MIN);
    /// ```
    pub const fn from_raw(raw: u8) -> HseTrim {
        if raw > 0x2F {
            HseTrim { val: 0x2F }
        } else {
            HseTrim { val: raw }
        }
    }

    /// Create a HSE trim value from farads.
    ///
    /// Values greater than the maximum of 33.4 pF will be set to the maximum.
    /// Values less than the minimum of 11.3 pF will be set to the minimum.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::HseTrim;
    ///
    /// assert!(HseTrim::from_farads(1.0).is_err());
    /// assert!(HseTrim::from_farads(1e-12).is_err());
    /// assert_eq!(HseTrim::from_farads(20.2e-12), Ok(HseTrim::default()));
    /// ```
    pub fn from_farads(farads: f32) -> Result<HseTrim, ValueError<f32>> {
        const MAX: f32 = 33.4E-12;
        const MIN: f32 = 11.3E-12;
        if farads > MAX {
            Err(ValueError::too_high(farads, MAX))
        } else if farads < MIN {
            Err(ValueError::too_low(farads, MIN))
        } else {
            Ok(HseTrim::from_raw(((farads - 11.3e-12) / 0.47e-12) as u8))
        }
    }

    /// Get the capacitance as farads.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::HseTrim;
    ///
    /// assert_eq!((HseTrim::MAX.as_farads() * 10e11) as u8, 33);
    /// assert_eq!((HseTrim::MIN.as_farads() * 10e11) as u8, 11);
    /// ```
    pub fn as_farads(&self) -> f32 {
        (self.val as f32) * 0.47E-12 + 11.3E-12
    }
}

impl From<HseTrim> for u8 {
    fn from(ht: HseTrim) -> Self {
        ht.val
    }
}

impl Default for HseTrim {
    fn default() -> Self {
        Self::POR
    }
}
