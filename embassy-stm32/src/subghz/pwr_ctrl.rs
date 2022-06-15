/// Power-supply current limit.
///
/// Argument of [`PwrCtrl::set_current_lim`].
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum CurrentLim {
    /// 25 mA
    Milli25 = 0x0,
    /// 50 mA (default)
    Milli50 = 0x1,
    /// 100 mA
    Milli100 = 0x2,
    /// 200 mA
    Milli200 = 0x3,
}

impl CurrentLim {
    /// Get the SMPS drive value as milliamps.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::CurrentLim;
    ///
    /// assert_eq!(CurrentLim::Milli25.as_milliamps(), 25);
    /// assert_eq!(CurrentLim::Milli50.as_milliamps(), 50);
    /// assert_eq!(CurrentLim::Milli100.as_milliamps(), 100);
    /// assert_eq!(CurrentLim::Milli200.as_milliamps(), 200);
    /// ```
    pub const fn as_milliamps(&self) -> u8 {
        match self {
            CurrentLim::Milli25 => 25,
            CurrentLim::Milli50 => 50,
            CurrentLim::Milli100 => 100,
            CurrentLim::Milli200 => 200,
        }
    }
}

impl Default for CurrentLim {
    fn default() -> Self {
        CurrentLim::Milli50
    }
}

/// Power control.
///
/// Argument of [`set_bit_sync`](super::SubGhz::set_bit_sync).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PwrCtrl {
    val: u8,
}

impl PwrCtrl {
    /// Power control register reset value.
    pub const RESET: PwrCtrl = PwrCtrl { val: 0x50 };

    /// Create a new [`PwrCtrl`] structure from a raw value.
    ///
    /// Reserved bits will be masked.
    pub const fn from_raw(raw: u8) -> Self {
        Self { val: raw & 0x70 }
    }

    /// Get the raw value of the [`PwrCtrl`] register.
    pub const fn as_bits(&self) -> u8 {
        self.val
    }

    /// Set the current limiter enable.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::PwrCtrl;
    ///
    /// const PWR_CTRL: PwrCtrl = PwrCtrl::RESET.set_current_lim_en(true);
    /// # assert_eq!(u8::from(PWR_CTRL), 0x50u8);
    /// ```
    #[must_use = "set_current_lim_en returns a modified PwrCtrl"]
    pub const fn set_current_lim_en(mut self, en: bool) -> PwrCtrl {
        if en {
            self.val |= 1 << 6;
        } else {
            self.val &= !(1 << 6);
        }
        self
    }

    /// Returns `true` if current limiting is enabled
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::PwrCtrl;
    ///
    /// let pc: PwrCtrl = PwrCtrl::RESET;
    /// assert_eq!(pc.current_limit_en(), true);
    /// let pc: PwrCtrl = pc.set_current_lim_en(false);
    /// assert_eq!(pc.current_limit_en(), false);
    /// let pc: PwrCtrl = pc.set_current_lim_en(true);
    /// assert_eq!(pc.current_limit_en(), true);
    /// ```
    pub const fn current_limit_en(&self) -> bool {
        self.val & (1 << 6) != 0
    }

    /// Set the current limit.
    #[must_use = "set_current_lim returns a modified PwrCtrl"]
    pub const fn set_current_lim(mut self, lim: CurrentLim) -> PwrCtrl {
        self.val &= !(0x30);
        self.val |= (lim as u8) << 4;
        self
    }

    /// Get the current limit.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CurrentLim, PwrCtrl};
    ///
    /// let pc: PwrCtrl = PwrCtrl::RESET;
    /// assert_eq!(pc.current_lim(), CurrentLim::Milli50);
    ///
    /// let pc: PwrCtrl = pc.set_current_lim(CurrentLim::Milli25);
    /// assert_eq!(pc.current_lim(), CurrentLim::Milli25);
    ///
    /// let pc: PwrCtrl = pc.set_current_lim(CurrentLim::Milli50);
    /// assert_eq!(pc.current_lim(), CurrentLim::Milli50);
    ///
    /// let pc: PwrCtrl = pc.set_current_lim(CurrentLim::Milli100);
    /// assert_eq!(pc.current_lim(), CurrentLim::Milli100);
    ///
    /// let pc: PwrCtrl = pc.set_current_lim(CurrentLim::Milli200);
    /// assert_eq!(pc.current_lim(), CurrentLim::Milli200);
    /// ```
    pub const fn current_lim(&self) -> CurrentLim {
        match (self.val >> 4) & 0b11 {
            0x0 => CurrentLim::Milli25,
            0x1 => CurrentLim::Milli50,
            0x2 => CurrentLim::Milli100,
            _ => CurrentLim::Milli200,
        }
    }
}

impl From<PwrCtrl> for u8 {
    fn from(bs: PwrCtrl) -> Self {
        bs.val
    }
}

impl Default for PwrCtrl {
    fn default() -> Self {
        Self::RESET
    }
}
