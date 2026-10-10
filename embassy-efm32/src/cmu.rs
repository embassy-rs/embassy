//! Clock tree (CMU) configuration.
//!
//! This is a minimal bring-up: it runs the core and all `HFPER` peripherals (GPIO, USARTn,
//! TIMERn, ...) from the high-frequency RC oscillator (HFRCO), and, with a time driver feature,
//! brings up the low-frequency branch (`LFA`, from the LFRCO or a 32.768 kHz crystal) used by the
//! RTC-based time driver.
//! Running from an external high-frequency crystal (HFXO) is not implemented yet.
//!
//! The register-level implementation lives in the chip family module (`chips/*.rs`).

use crate::chip;
pub(crate) use crate::chip::PeripheralClock;
use crate::time::Hertz;

/// HFRCO frequency band.
///
/// This sets the frequency of the core clock, `HFCORECLK` and `HFPERCLK`: the HFRCO is always
/// used as the `HFCLK` source, with the `HFCORECLKDIV`/`HFPERCLKDIV` prescalers set to 1.
///
/// The bands are named as in the reference manual. On current silicon (production revision 19 and
/// later), the 7 MHz and 1 MHz bands actually run at 6.6 MHz and 1.2 MHz; [`clocks`] returns the
/// real frequency for the chip at hand.
///
/// The HFRCO is factory calibrated, but not a precision oscillator: the data sheet gives e.g.
/// 27.5 to 28.5 MHz for the 28 MHz band at 25 °C and 3.0 V, and it drifts further over
/// temperature and supply voltage.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HfrcoBand {
    /// 1 MHz (1.2 MHz on production revision 19 and later).
    Band1Mhz,
    /// 7 MHz (6.6 MHz on production revision 19 and later).
    Band7Mhz,
    /// 11 MHz.
    Band11Mhz,
    /// 14 MHz (reset default).
    #[default]
    Band14Mhz,
    /// 21 MHz.
    Band21Mhz,
    /// 28 MHz.
    Band28Mhz,
}

/// Clock tree configuration.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// HFRCO frequency band. Defaults to [`HfrcoBand::Band14Mhz`], the hardware reset default.
    pub hfrco_band: HfrcoBand,
    /// Source of the `LFA` branch, which clocks the RTC time driver. Defaults to
    /// [`LfClockSource::Lfrco`], which needs no external components.
    ///
    /// Only available with a time driver feature: nothing else uses the `LFA` branch yet.
    #[cfg(feature = "_time-driver")]
    pub lfa_source: LfClockSource,
}

/// Low-frequency clock source.
#[cfg(feature = "_time-driver")]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LfClockSource {
    /// The internal 32.768 kHz RC oscillator. Only accurate to a few percent.
    #[default]
    Lfrco,
    /// An external 32.768 kHz crystal on the chip's `LFXTAL_P`/`LFXTAL_N` pins (PB7/PB8).
    ///
    /// Not available with the `lfxo-as-gpio` feature, which hands these pins out as GPIOs instead.
    #[cfg(not(feature = "lfxo-as-gpio"))]
    Lfxo,
}

/// The frequencies of the clock tree, as configured by [`crate::init`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Clocks {
    /// `HFCLK` / core clock frequency.
    pub hfclk: Hertz,
    /// `HFPERCLK` frequency. Clocks USARTn, TIMERn, ADC0, I2Cn, etc.
    pub hfperclk: Hertz,
}

// The reset state, until `init` has run.
static mut CLOCKS: Clocks = Clocks {
    hfclk: Hertz::mhz(14),
    hfperclk: Hertz::mhz(14),
};

/// Returns the clock tree frequencies configured at [`crate::init`] time.
pub fn clocks() -> Clocks {
    unsafe { CLOCKS }
}

pub(crate) fn init(config: Config) {
    let clocks = chip::init_clocks(&config);
    unsafe { CLOCKS = clocks };
}

/// Enable the `HFPERCLK` gate of a peripheral.
pub(crate) fn enable(clock: PeripheralClock) {
    chip::set_peripheral_clock(clock, true);
}

/// Disable the `HFPERCLK` gate of a peripheral.
pub(crate) fn disable(clock: PeripheralClock) {
    chip::set_peripheral_clock(clock, false);
}
