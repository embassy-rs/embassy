//! EFM32GG (Series 0) clock tree.
//!
//! After reset, HFRCO runs in the 14 MHz band and is HFCLK. HFPERCLK is
//! HFCLK / 1, and the LF oscillators are off. [`Config::new`] is this state.
//!
//! The sequences follow the EFM32GG reference manual. They are
//! cross-checked with emlib `em_cmu.c`.

use critical_section::CriticalSection;

use super::{Clocks, Hertz, fill_group_freqs, mux, set_freqs};
use crate::pac::cmu::vals as cmu_vals;
use crate::pac::msc::vals as msc_vals;
use crate::pac::{CMU, MSC};

/// LFRCO frequency, 32.768 kHz.
pub const LFRCO_FREQ: Hertz = Hertz(32_768);
/// LFXO frequency. EFM32GG boards use a 32.768 kHz LF crystal.
pub const LFXO_FREQ: Hertz = Hertz(32_768);

/// HFRCO frequency band.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HfrcoBand {
    /// 1 MHz band.
    Freq1M,
    /// 7 MHz band.
    Freq7M,
    /// 11 MHz.
    Freq11M,
    /// 14 MHz. Selected after reset.
    Freq14M,
    /// 21 MHz.
    Freq21M,
    /// 28 MHz.
    Freq28M,
}

impl HfrcoBand {
    /// Returns the band frequency on this chip.
    ///
    /// From production revision 19 (`DEVINFO.PART.PROD_REV`), the 1 MHz band runs at 1.2 MHz and
    /// the 7 MHz band at 6.6 MHz (datasheet table 4.12).
    pub fn frequency(self) -> Hertz {
        let new_rev = prod_rev() >= 19;
        match self {
            HfrcoBand::Freq1M if new_rev => Hertz(1_200_000),
            HfrcoBand::Freq1M => Hertz::mhz(1),
            HfrcoBand::Freq7M if new_rev => Hertz(6_600_000),
            HfrcoBand::Freq7M => Hertz::mhz(7),
            HfrcoBand::Freq11M => Hertz::mhz(11),
            HfrcoBand::Freq14M => Hertz::mhz(14),
            HfrcoBand::Freq21M => Hertz::mhz(21),
            HfrcoBand::Freq28M => Hertz::mhz(28),
        }
    }

    fn to_vals(self) -> cmu_vals::HfrcoctrlBand {
        match self {
            HfrcoBand::Freq1M => cmu_vals::HfrcoctrlBand::_1mhz,
            HfrcoBand::Freq7M => cmu_vals::HfrcoctrlBand::_7mhz,
            HfrcoBand::Freq11M => cmu_vals::HfrcoctrlBand::_11mhz,
            HfrcoBand::Freq14M => cmu_vals::HfrcoctrlBand::_14mhz,
            HfrcoBand::Freq21M => cmu_vals::HfrcoctrlBand::_21mhz,
            HfrcoBand::Freq28M => cmu_vals::HfrcoctrlBand::_28mhz,
        }
    }

    /// Factory `HFRCOCTRL.TUNING` value of the band, from DEVINFO.
    ///
    /// The metapac has no Series 0 DEVINFO block, so this reads the
    /// addresses of `efm32gg_devinfo.h`: `HFRCOCAL0` at `0x0FE081DC` holds
    /// the 1, 7, 11 and 14 MHz bytes, and `HFRCOCAL1` at `0x0FE081E0` holds
    /// the 21 and 28 MHz bytes.
    fn tuning(self) -> u8 {
        const HFRCOCAL0: *const u32 = 0x0FE0_81DC as *const u32;
        const HFRCOCAL1: *const u32 = 0x0FE0_81E0 as *const u32;
        let (reg, byte) = match self {
            HfrcoBand::Freq1M => (HFRCOCAL0, 0),
            HfrcoBand::Freq7M => (HFRCOCAL0, 1),
            HfrcoBand::Freq11M => (HFRCOCAL0, 2),
            HfrcoBand::Freq14M => (HFRCOCAL0, 3),
            HfrcoBand::Freq21M => (HFRCOCAL1, 0),
            HfrcoBand::Freq28M => (HFRCOCAL1, 1),
        };
        // SAFETY: DEVINFO is a read-only flash page that is always mapped.
        let word = unsafe { core::ptr::read_volatile(reg) };
        (word >> (8 * byte)) as u8
    }
}

/// Production revision, `DEVINFO.PART` bits 31:24.
///
/// The metapac has no Series 0 DEVINFO block, so this reads `PART` at
/// `0x0FE081FC` (base `0x0FE081B0` plus offset `0x4C`, from
/// `efm32gg_devinfo.h`).
fn prod_rev() -> u8 {
    const PART: *const u32 = 0x0FE0_81FC as *const u32;
    // SAFETY: DEVINFO is a read-only flash page that is always mapped.
    (unsafe { core::ptr::read_volatile(PART) } >> 24) as u8
}

/// HFCLK source.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HfclkSource {
    /// HFRCO in this band.
    Hfrco(HfrcoBand),
    /// HFXO crystal, 4 MHz to 48 MHz.
    Hfxo {
        /// Crystal frequency.
        freq: Hertz,
    },
}

/// LFACLK source. LFACLK clocks the RTC, LETIMER0, LESENSE and LCD.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LfaSource {
    /// LFRCO, 32.768 kHz, the reset value.
    Lfrco,
    /// LFXO, a 32.768 kHz crystal.
    Lfxo,
}

/// HFPERCLK divider: HFPERCLK = HFCLK / divider.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HfperclkDiv {
    /// HFCLK / 1, the reset value.
    Div1,
    /// HFCLK / 2.
    Div2,
    /// HFCLK / 4.
    Div4,
    /// HFCLK / 8.
    Div8,
    /// HFCLK / 16.
    Div16,
    /// HFCLK / 32.
    Div32,
    /// HFCLK / 64.
    Div64,
    /// HFCLK / 128.
    Div128,
    /// HFCLK / 256.
    Div256,
    /// HFCLK / 512.
    Div512,
}

impl HfperclkDiv {
    /// Returns the divider.
    pub const fn divisor(self) -> u32 {
        1 << (self as u32)
    }

    fn to_vals(self) -> cmu_vals::Hfperclkdiv {
        match self {
            HfperclkDiv::Div1 => cmu_vals::Hfperclkdiv::Hfclk,
            HfperclkDiv::Div2 => cmu_vals::Hfperclkdiv::Hfclk2,
            HfperclkDiv::Div4 => cmu_vals::Hfperclkdiv::Hfclk4,
            HfperclkDiv::Div8 => cmu_vals::Hfperclkdiv::Hfclk8,
            HfperclkDiv::Div16 => cmu_vals::Hfperclkdiv::Hfclk16,
            HfperclkDiv::Div32 => cmu_vals::Hfperclkdiv::Hfclk32,
            HfperclkDiv::Div64 => cmu_vals::Hfperclkdiv::Hfclk64,
            HfperclkDiv::Div128 => cmu_vals::Hfperclkdiv::Hfclk128,
            HfperclkDiv::Div256 => cmu_vals::Hfperclkdiv::Hfclk256,
            HfperclkDiv::Div512 => cmu_vals::Hfperclkdiv::Hfclk512,
        }
    }
}

/// Clock tree configuration for [`init_clocks`].
///
/// [`Config::new`] returns the reset state as a starting point.
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// HFCLK source. HFCORECLK equals HFCLK.
    pub hfclk: HfclkSource,
    /// HFPERCLK = HFCLK / `hfperclk_div`.
    pub hfperclk_div: HfperclkDiv,
    /// LFACLK source. `init` starts this oscillator.
    pub lfa: LfaSource,
    /// Peripheral clock selects (`pcnt0`, ...), generated from chip metadata.
    pub mux: mux::ClockMux,
}

impl Config {
    /// Returns the reset state.
    ///
    /// HFRCO at 14 MHz drives HFCLK, HFPERCLK is undivided, and LFRCO feeds LFACLK. Every `mux`
    /// field is `None`, so `init` writes no peripheral clock select.
    pub const fn new() -> Self {
        Self {
            hfclk: HfclkSource::Hfrco(HfrcoBand::Freq14M),
            hfperclk_div: HfperclkDiv::Div1,
            lfa: LfaSource::Lfrco,
            mux: mux::ClockMux::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// HFCORECLK limit for 0 flash wait states.
const MAX_FREQ_0WS: u32 = 16_000_000;
/// HFCORECLK limit for 1 flash wait state.
const MAX_FREQ_1WS: u32 = 32_000_000;
/// HFCORECLK limit for the LE interface without `CTRL.HFLE`.
const MAX_FREQ_HFLE: u32 = 32_000_000;

/// Set the flash wait states for `hfcoreclk`. Keep the SCBTP (suppressed
/// conditional branch target prefetch) setting.
fn set_flash_wait_states(hfcoreclk: u32) {
    use msc_vals::Mode;
    // The MSC registers are unlocked after reset. Unlock them in case a
    // bootloader locked them. The metapac `LockLockkey` enum has only the
    // read values, so the unlock key 0x1B71 is written as raw bits.
    MSC.lock()
        .write(|w| w.set_lockkey(msc_vals::LockLockkey::from_bits(0x1B71)));
    MSC.readctrl().modify(|w| {
        let scbtp = matches!(w.mode(), Mode::Ws0scbtp | Mode::Ws1scbtp | Mode::Ws2scbtp);
        let mode = match (hfcoreclk, scbtp) {
            (0..=MAX_FREQ_0WS, false) => Mode::Ws0,
            (0..=MAX_FREQ_0WS, true) => Mode::Ws0scbtp,
            (0..=MAX_FREQ_1WS, false) => Mode::Ws1,
            (0..=MAX_FREQ_1WS, true) => Mode::Ws1scbtp,
            (_, false) => Mode::Ws2,
            (_, true) => Mode::Ws2scbtp,
        };
        w.set_mode(mode);
    });
}

/// Sets the LE interface for `hfcoreclk`.
///
/// Above 32 MHz, the LE interface needs a wait state (`CTRL.HFLE`) and HFCORECLKLE = HFCORECLK / 4
/// (`HFCORECLKDIV.HFCORECLKLEDIV`). Otherwise HFCORECLKLE = HFCORECLK / 2.
fn set_hfle(hfcoreclk: u32) {
    let hfle = hfcoreclk > MAX_FREQ_HFLE;
    CMU.ctrl().modify(|w| w.set_hfle(hfle));
    CMU.hfcoreclkdiv().modify(|w| w.set_hfcoreclklediv(hfle));
}

pub(crate) fn init_clocks(_cs: CriticalSection, config: Config) {
    let hfclk = match config.hfclk {
        HfclkSource::Hfrco(band) => band.frequency(),
        HfclkSource::Hfxo { freq } => {
            cmu_assert!((4_000_000..=48_000_000).contains(&freq.0));
            freq
        }
    };
    let hfcoreclk = hfclk;
    let hfperclk = Hertz(hfclk.0 / config.hfperclk_div.divisor());
    cmu_assert!(hfclk.0 <= 48_000_000);

    // Use the slowest flash timing and the LE wait state while HFCLK
    // changes. Lower them only after the switch.
    set_flash_wait_states(u32::MAX);
    set_hfle(u32::MAX);

    match config.hfclk {
        HfclkSource::Hfrco(band) => {
            // HFRCO is on after reset. Set the band and its factory tuning
            // in one write.
            CMU.oscencmd().write(|w| w.set_hfrcoen(true));
            while !CMU.status().read().hfrcordy() {}
            CMU.hfrcoctrl().modify(|w| {
                w.set_band(band.to_vals());
                w.set_tuning(band.tuning());
            });
            CMU.cmd().write(|w| w.set_hfclksel(cmu_vals::Hfclksel::Hfrco));
            while !CMU.status().read().hfrcosel() {}
        }
        HfclkSource::Hfxo { freq } => {
            // Crystal mode. Above 32 MHz, the HFXO buffer needs the higher
            // bias current. HFXOBOOST stays at its reset value (100 %).
            CMU.ctrl().modify(|w| {
                w.set_hfxomode(cmu_vals::Hfxomode::Xtal);
                w.set_hfxobufcur(if freq.0 > 32_000_000 {
                    cmu_vals::Hfxobufcur::Boostabove32mhz
                } else {
                    cmu_vals::Hfxobufcur::Boostupto32mhz
                });
            });
            CMU.oscencmd().write(|w| w.set_hfxoen(true));
            while !CMU.status().read().hfxordy() {}
            CMU.cmd().write(|w| w.set_hfclksel(cmu_vals::Hfclksel::Hfxo));
            while !CMU.status().read().hfxosel() {}
        }
    }

    // HFCORECLK = HFCLK. HFPERCLK = HFCLK / div, and HFPERCLKEN must be 1.
    CMU.hfcoreclkdiv()
        .modify(|w| w.set_hfcoreclkdiv(cmu_vals::Hfcoreclkdiv::Hfclk));
    CMU.hfperclkdiv().modify(|w| {
        w.set_hfperclkdiv(config.hfperclk_div.to_vals());
        w.set_hfperclken(true);
    });

    set_flash_wait_states(hfcoreclk.0);
    set_hfle(hfcoreclk.0);

    // GPIO is on HFPERCLK.
    super::enable::<crate::peripherals::GPIO>();

    // The LE interface clock must be on before any access to an LE
    // peripheral (RTC, LETIMER, LEUART, ...) or to the LF clock registers.
    CMU.hfcoreclken0().modify(|w| w.set_le(true));

    // Start the LFACLK oscillator and select it.
    let (lfrco_on, lfxo_on) = match config.lfa {
        LfaSource::Lfrco => {
            CMU.oscencmd().write(|w| w.set_lfrcoen(true));
            while !CMU.status().read().lfrcordy() {}
            CMU.lfclksel().modify(|w| {
                w.set_lfa(cmu_vals::Lfa::Lfrco);
                w.set_lfae(false);
            });
            (true, false)
        }
        LfaSource::Lfxo => {
            CMU.ctrl().modify(|w| w.set_lfxomode(cmu_vals::Lfxomode::Xtal));
            CMU.oscencmd().write(|w| w.set_lfxoen(true));
            while !CMU.status().read().lfxordy() {}
            CMU.lfclksel().modify(|w| {
                w.set_lfa(cmu_vals::Lfa::Lfxo);
                w.set_lfae(false);
            });
            (CMU.status().read().lfrcoens(), true)
        }
    };

    config.mux.init();

    let mut clocks = Clocks::default();
    // HFRCO stays on. A HFXO HFCLK leaves it in the reset band.
    clocks.hfrco = Some(match config.hfclk {
        HfclkSource::Hfrco(band) => band.frequency(),
        HfclkSource::Hfxo { .. } => HfrcoBand::Freq14M.frequency(),
    });
    clocks.hfxo = match config.hfclk {
        HfclkSource::Hfxo { freq } => Some(freq),
        HfclkSource::Hfrco(_) => None,
    };
    clocks.lfrco = lfrco_on.then_some(LFRCO_FREQ);
    clocks.lfxo = lfxo_on.then_some(LFXO_FREQ);
    clocks.hfclk = Some(hfclk);
    clocks.hfcoreclk = Some(hfcoreclk);
    clocks.hfperclk = Some(hfperclk);
    clocks.lfaclk = match config.lfa {
        LfaSource::Lfrco => clocks.lfrco,
        LfaSource::Lfxo => clocks.lfxo,
    };
    // HFCORECLKLE is HFCORECLK / 2, or / 4 with HFLE. The LFB select
    // `HFCORECLKLEDIV2` is this clock (RM 11.3.4).
    let hfcoreclkle = Hertz(hfcoreclk.0 / if hfcoreclk.0 > MAX_FREQ_HFLE { 4 } else { 2 });
    // LFBE selects ULFRCO, which `Clocks` does not track.
    let lfclksel = CMU.lfclksel().read();
    clocks.lfbclk = match lfclksel.lfb() {
        _ if lfclksel.lfbe() => None,
        cmu_vals::Lfb::Lfrco => clocks.lfrco,
        cmu_vals::Lfb::Lfxo => clocks.lfxo,
        cmu_vals::Lfb::Hfcoreclklediv2 => Some(hfcoreclkle),
        cmu_vals::Lfb::Disabled => None,
    };
    fill_group_freqs(&mut clocks);

    unsafe { set_freqs(clocks) };
}
