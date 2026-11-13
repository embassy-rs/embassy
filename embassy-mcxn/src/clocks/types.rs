//! Clock state tracking.

/// Errors that can occur while configuring or querying clocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ClockError {
    /// [`super::init()`] has already been called.
    AlreadyInitialized,
    /// `main_clk` is not sourced from `fro_hf`, its reset default.
    ///
    /// [`super::init()`] does not switch the main clock source, so it cannot
    /// describe the clock tree once something else has.
    UnsupportedMainClockSource,
}

impl core::fmt::Display for ClockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AlreadyInitialized => f.write_str("clocks already initialized"),
            Self::UnsupportedMainClockSource => f.write_str("main_clk is not sourced from fro_hf"),
        }
    }
}

impl core::error::Error for ClockError {}

/// State of the core system clocks, as observed after [`super::init()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Clocks {
    /// Frequency of `main_clk`, in Hz.
    ///
    /// See RM "Chapter 33 Clocking" - the main clock feeds the CPU and AHB
    /// bus (via `AHBCLKDIV`), and most peripherals derive their clock from
    /// it (directly, or via a peripheral-specific mux/divider).
    pub main_clk_hz: u32,

    /// Frequency of `system_clk` (`main_clk` divided by `AHBCLKDIV`), in Hz.
    pub system_clk_hz: u32,
}
