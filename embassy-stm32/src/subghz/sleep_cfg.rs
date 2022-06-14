/// Startup configurations when exiting sleep mode.
///
/// Argument of [`SleepCfg::set_startup`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Startup {
    /// Cold startup when exiting Sleep mode, configuration registers reset.
    Cold = 0,
    /// Warm startup when exiting Sleep mode,
    /// configuration registers kept in retention.
    ///
    /// **Note:** Only the configuration of the activated modem,
    /// before going to sleep mode, is retained.
    /// The configuration of the other modes is lost and must be re-configured
    /// when exiting sleep mode.
    Warm = 1,
}

impl Default for Startup {
    fn default() -> Self {
        Startup::Warm
    }
}

/// Sleep configuration.
///
/// Argument of [`set_sleep`].
///
/// [`set_sleep`]: super::SubGhz::set_sleep
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SleepCfg(u8);

impl SleepCfg {
    /// Create a new `SleepCfg` structure.
    ///
    /// This is the same as `default`, but in a `const` function.
    ///
    /// The defaults are a warm startup, with RTC wakeup enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::SleepCfg;
    ///
    /// const SLEEP_CFG: SleepCfg = SleepCfg::new();
    /// assert_eq!(SLEEP_CFG, SleepCfg::default());
    /// # assert_eq!(u8::from(SLEEP_CFG), 0b101);
    /// ```
    pub const fn new() -> SleepCfg {
        SleepCfg(0)
            .set_startup(Startup::Warm)
            .set_rtc_wakeup_en(true)
    }

    /// Set the startup mode.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{SleepCfg, Startup};
    ///
    /// const SLEEP_CFG: SleepCfg = SleepCfg::new().set_startup(Startup::Cold);
    /// # assert_eq!(u8::from(SLEEP_CFG), 0b001);
    /// # assert_eq!(u8::from(SLEEP_CFG.set_startup(Startup::Warm)), 0b101);
    /// ```
    pub const fn set_startup(mut self, startup: Startup) -> SleepCfg {
        if startup as u8 == 1 {
            self.0 |= 1 << 2
        } else {
            self.0 &= !(1 << 2)
        }
        self
    }

    /// Set the RTC wakeup enable.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::SleepCfg;
    ///
    /// const SLEEP_CFG: SleepCfg = SleepCfg::new().set_rtc_wakeup_en(false);
    /// # assert_eq!(u8::from(SLEEP_CFG), 0b100);
    /// # assert_eq!(u8::from(SLEEP_CFG.set_rtc_wakeup_en(true)), 0b101);
    /// ```
    #[must_use = "set_rtc_wakeup_en returns a modified SleepCfg"]
    pub const fn set_rtc_wakeup_en(mut self, en: bool) -> SleepCfg {
        if en {
            self.0 |= 0b1
        } else {
            self.0 &= !0b1
        }
        self
    }
}

impl From<SleepCfg> for u8 {
    fn from(sc: SleepCfg) -> Self {
        sc.0
    }
}

impl Default for SleepCfg {
    fn default() -> Self {
        Self::new()
    }
}
