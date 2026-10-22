//! EFM32GG (Series 0) clock tree.
//!
//! Provides the clock [`Config`] and the clock setup that [`crate::init`] runs.

use critical_section::CriticalSection;

use super::{Clocks, Hertz, fill_group_freqs, mux, set_freqs};
use crate::pac::cmu::vals as cmu_vals;
use crate::pac::msc::vals as msc_vals;
use crate::pac::{CMU, DEVINFO, MSC, ROMTABLE};

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

    /// Returns the band for an `HFRCOCTRL.BAND` value, or `None` for a reserved value.
    fn from_vals(v: cmu_vals::HfrcoctrlBand) -> Option<Self> {
        Some(match v {
            cmu_vals::HfrcoctrlBand::_1mhz => HfrcoBand::Freq1M,
            cmu_vals::HfrcoctrlBand::_7mhz => HfrcoBand::Freq7M,
            cmu_vals::HfrcoctrlBand::_11mhz => HfrcoBand::Freq11M,
            cmu_vals::HfrcoctrlBand::_14mhz => HfrcoBand::Freq14M,
            cmu_vals::HfrcoctrlBand::_21mhz => HfrcoBand::Freq21M,
            cmu_vals::HfrcoctrlBand::_28mhz => HfrcoBand::Freq28M,
            _ => return None,
        })
    }

    /// Returns the factory `HFRCOCTRL.TUNING` value of the band.
    ///
    /// Bands 1 to 14 MHz use `DEVINFO.HFRCOCAL0`, and bands 21 and 28 MHz use `DEVINFO.HFRCOCAL1`.
    fn tuning(self) -> u8 {
        let cal0 = || DEVINFO.hfrcocal0().read();
        let cal1 = || DEVINFO.hfrcocal1().read();
        match self {
            HfrcoBand::Freq1M => cal0().band1(),
            HfrcoBand::Freq7M => cal0().band7(),
            HfrcoBand::Freq11M => cal0().band11(),
            HfrcoBand::Freq14M => cal0().band14(),
            HfrcoBand::Freq21M => cal1().band21(),
            HfrcoBand::Freq28M => cal1().band28(),
        }
    }
}

/// Returns the production revision (`DEVINFO.PART.PROD_REV`).
fn prod_rev() -> u8 {
    DEVINFO.part().read().prod_rev()
}

/// Returns the chip revision `(major, minor)` from the ROM table (reference manual 3.6).
fn chip_revision() -> (u8, u8) {
    let major = ROMTABLE.pid0().read().revmajor();
    let minor = (ROMTABLE.pid2().read().revminormsb() << 4) | ROMTABLE.pid3().read().revminorlsb();
    (major, minor)
}

/// Applies erratum CMU_E113 (LFXO start-up at high temperature, EFM32GG errata sheet 2.9).
///
/// The errata sheet lists it for production revision 16 and later on chip revisions D and E. The
/// code checks production revision 16 and later and minor revision 3 (D) and later (reference
/// manual table 3.3), and excludes chip revision 2.4 (Leopard Gecko).
///
/// The errata sheet gives the workaround: clear bit 6 and set bit 4 at 0x400C80C0, CMU offset
/// 0xC0. The metapac has no name for that register.
fn apply_errata_cmu_e113() {
    let (major, minor) = chip_revision();
    if prod_rev() >= 16 && minor >= 3 && !(major == 2 && minor == 4) {
        // SAFETY: Offset 0xC0 is inside the CMU address range 0x400C8000..=0x400C83FF (reference
        // manual 5.2.2), and the read-modify-write changes only bits 4 and 6.
        unsafe {
            let reg = (CMU.as_ptr() as *mut u8).add(0xC0) as *mut u32;
            reg.write_volatile((reg.read_volatile() & !(1 << 6)) | (1 << 4));
        }
    }
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

/// LFACLK and LFBCLK source.
///
/// LFACLK feeds RTC, LETIMER0, LESENSE and LCD. LFBCLK feeds LEUART0 and LEUART1.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LfaSource {
    /// LFRCO, 32.768 kHz, the reset value.
    Lfrco,
    /// LFXO, a 32.768 kHz crystal on LFXTAL_P and LFXTAL_N (PB7 and PB8).
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
    /// LFACLK and LFBCLK source. `init` starts this oscillator.
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
/// Highest HFCLK for EFM32GG.
const MAX_FREQ: u32 = 48_000_000;

/// Sets the flash wait states for `hfcoreclk`.
///
/// Keeps the SCBTP (suppressed conditional branch target prefetch) setting. Needs an unlocked MSC.
fn set_flash_wait_states(hfcoreclk: u32) {
    use msc_vals::Mode;
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

/// Selects `sel` as HFCLK and waits until the CMU reports it.
fn select_hfclk(sel: cmu_vals::Hfclksel) {
    CMU.cmd().write(|w| w.set_hfclksel(sel));
    let selected = || {
        let st = CMU.status().read();
        match sel {
            cmu_vals::Hfclksel::Hfrco => st.hfrcosel(),
            cmu_vals::Hfclksel::Hfxo => st.hfxosel(),
            cmu_vals::Hfclksel::Lfrco => st.lfrcosel(),
            _ => st.lfxosel(),
        }
    };
    while !selected() {}
}

/// Starts LFRCO and waits until it is ready.
fn start_lfrco() {
    CMU.oscencmd().write(|w| w.set_lfrcoen(true));
    while !CMU.status().read().lfrcordy() {}
}

/// Sets the HFRCO band and its factory tuning while HFCLK runs from HFRCO.
///
/// Flash wait states must be at their maximum, so flash reads work at any band.
fn set_hfrco_band(band: HfrcoBand) {
    let current = HfrcoBand::from_vals(CMU.hfrcoctrl().read().band());
    if current == Some(band) {
        // A bootloader can have changed the tuning.
        CMU.hfrcoctrl().modify(|w| w.set_tuning(band.tuning()));
        return;
    }
    // CMU_E115 (EFM32GG errata sheet 2.10): switching to or from the 1 MHz band while HFRCO is
    // HFCLK can glitch the clock. HFCLK runs from LFRCO during the switch.
    let detour = band == HfrcoBand::Freq1M || current == Some(HfrcoBand::Freq1M);
    let lfrco_was_on = CMU.status().read().lfrcoens();
    if detour {
        start_lfrco();
        select_hfclk(cmu_vals::Hfclksel::Lfrco);
    }
    CMU.hfrcoctrl().modify(|w| {
        w.set_band(band.to_vals());
        w.set_tuning(band.tuning());
    });
    while !CMU.status().read().hfrcordy() {}
    if detour {
        select_hfclk(cmu_vals::Hfclksel::Hfrco);
        if !lfrco_was_on {
            CMU.oscencmd().write(|w| w.set_lfrcodis(true));
        }
    }
}

pub(crate) fn init_clocks(_cs: CriticalSection, config: Config) {
    let hfclk = match config.hfclk {
        HfclkSource::Hfrco(band) => band.frequency(),
        HfclkSource::Hfxo { freq } => {
            cmu_assert!((4_000_000..=MAX_FREQ).contains(&freq.0));
            freq
        }
    };
    let hfcoreclk = hfclk;
    let hfperclk = Hertz(hfclk.0 / config.hfperclk_div.divisor());
    cmu_assert!(hfclk.0 <= MAX_FREQ);

    // A bootloader can leave interrupts enabled or pending, and SysTick running. Their vectors go
    // to `DefaultHandler`, so disable them all.
    // SAFETY: The NVIC and SysTick registers are always mapped. Drivers get their peripherals from
    // the `Peripherals` that `init` returns, so no driver has enabled an interrupt yet.
    unsafe {
        let nvic = &*cortex_m::peripheral::NVIC::PTR;
        for i in 0..nvic.icer.len() {
            nvic.icer[i].write(u32::MAX);
            nvic.icpr[i].write(u32::MAX);
        }
        (*cortex_m::peripheral::SYST::PTR).csr.write(0);
    }

    // A bootloader can set `MSC_READCTRL.AIDIS` and then write flash, which leaves stale cache
    // lines (reference manual 7.3.4.6). MSC_LOCK does not cover MSC_CMD (7.5.12).
    MSC.cmd().write(|w| w.set_invcache(true));

    // A bootloader can have locked the CMU or the MSC.
    CMU.lock().write(|w| w.set_lockkey(cmu_vals::Lockkey::Unlock));
    MSC.lock().write(|w| w.set_lockkey(msc_vals::LockLockkey::Unlock));
    // A set `REGFREEZE` holds all LF register writes, and the `SYNCBUSY` waits never end.
    CMU.freeze().write(|w| w.set_regfreeze(false));

    // A bootloader can leave HFXO at 48 MHz as HFCLK. These settings are safe for any HFCLK up to
    // 48 MHz, so lower them only after the last switch.
    set_flash_wait_states(MAX_FREQ);
    set_hfle(MAX_FREQ);

    // HFRCO is at most 28 MHz, so HFCLK and HFCORECLK need no divider.
    CMU.oscencmd().write(|w| w.set_hfrcoen(true));
    while !CMU.status().read().hfrcordy() {}
    select_hfclk(cmu_vals::Hfclksel::Hfrco);
    CMU.ctrl().modify(|w| w.set_hfclkdiv(0));
    CMU.hfcoreclkdiv()
        .modify(|w| w.set_hfcoreclkdiv(cmu_vals::Hfcoreclkdiv::Hfclk));

    // HFXO settings can change only while HFXO is off.
    if CMU.status().read().hfxoens() {
        CMU.oscencmd().write(|w| w.set_hfxodis(true));
        while CMU.status().read().hfxoens() {}
    }

    match config.hfclk {
        HfclkSource::Hfrco(band) => set_hfrco_band(band),
        HfclkSource::Hfxo { freq } => {
            // Above 32 MHz, the HFXO buffer needs higher bias current (RM table 11.1).
            CMU.ctrl().modify(|w| {
                w.set_hfxomode(cmu_vals::Hfxomode::Xtal);
                w.set_hfxobufcur(if freq.0 > MAX_FREQ_1WS {
                    cmu_vals::Hfxobufcur::Boostabove32mhz
                } else {
                    cmu_vals::Hfxobufcur::Boostupto32mhz
                });
            });
            CMU.oscencmd().write(|w| w.set_hfxoen(true));
            while !CMU.status().read().hfxordy() {}
            select_hfclk(cmu_vals::Hfclksel::Hfxo);
        }
    }

    // HFPERCLK runs only while HFPERCLKEN is set.
    CMU.hfperclkdiv().modify(|w| {
        w.set_hfperclkdiv(config.hfperclk_div.to_vals());
        w.set_hfperclken(true);
    });

    set_flash_wait_states(hfcoreclk.0);
    set_hfle(hfcoreclk.0);
    // Any value other than the unlock key locks the MSC.
    MSC.lock().write(|w| w.set_lockkey(msc_vals::LockLockkey::Locked));

    // GPIO is on HFPERCLK.
    super::enable::<crate::peripherals::GPIO>();

    // The LF source must be ready before the select (reference manual, 11.3.2).
    match config.lfa {
        LfaSource::Lfrco => start_lfrco(),
        LfaSource::Lfxo => {
            // A bootloader can have started LFXO with other settings, which must not change while
            // LFXO runs. So stop LFXO first.
            if CMU.status().read().lfxoens() {
                CMU.lfclksel().write(|w| {
                    w.set_lfa(cmu_vals::Lfa::Disabled);
                    w.set_lfb(cmu_vals::Lfb::Disabled);
                });
                CMU.oscencmd().write(|w| w.set_lfxodis(true));
                while CMU.status().read().lfxoens() {}
            }
            apply_errata_cmu_e113();
            // CMU_E111 (EFM32GG errata sheet 3.8): before production revision 15, the reset values
            // of LFXOBUFCUR and LFXOBOOST are wrong. Set LFXOBUFCUR to 1 and LFXOBOOST to 0 (70 %).
            let cmu_e111 = prod_rev() < 15;
            // LFXOMODE and LFXOTIMEOUT get their reset values (reference manual 11.5.1, CMU_CTRL).
            CMU.ctrl().modify(|w| {
                w.set_lfxomode(cmu_vals::Lfxomode::Xtal);
                // 100 % boost on later chips.
                w.set_lfxoboost(!cmu_e111);
                // CMU_E112 (errata sheet 2.8): later chips must keep LFXOBUFCUR at 0.
                w.set_lfxobufcur(cmu_e111);
                w.set_lfxotimeout(cmu_vals::Lfxotimeout::_32kcycles);
            });
            CMU.oscencmd().write(|w| w.set_lfxoen(true));
            while !CMU.status().read().lfxordy() {}
        }
    }
    let (lfa, lfb) = match config.lfa {
        LfaSource::Lfrco => (cmu_vals::Lfa::Lfrco, cmu_vals::Lfb::Lfrco),
        LfaSource::Lfxo => (cmu_vals::Lfa::Lfxo, cmu_vals::Lfb::Lfxo),
    };
    // A set `LFAE` or `LFBE` selects ULFRCO instead.
    CMU.lfclksel().write(|w| {
        w.set_lfa(lfa);
        w.set_lfb(lfb);
        w.set_lfae(false);
        w.set_lfbe(false);
    });
    // Free the LFXO pins when the config does not use LFXO.
    if config.lfa != LfaSource::Lfxo && CMU.status().read().lfxoens() {
        CMU.oscencmd().write(|w| w.set_lfxodis(true));
    }

    config.mux.init();

    let mut clocks = Clocks::default();
    let status = CMU.status().read();
    clocks.hfrco = HfrcoBand::from_vals(CMU.hfrcoctrl().read().band())
        .filter(|_| status.hfrcoens())
        .map(HfrcoBand::frequency);
    clocks.hfxo = match config.hfclk {
        HfclkSource::Hfxo { freq } => Some(freq),
        HfclkSource::Hfrco(_) => None,
    };
    clocks.lfrco = status.lfrcoens().then_some(LFRCO_FREQ);
    clocks.lfxo = (config.lfa == LfaSource::Lfxo).then_some(LFXO_FREQ);
    clocks.hfclk = Some(hfclk);
    clocks.hfcoreclk = Some(hfcoreclk);
    clocks.hfperclk = Some(hfperclk);
    clocks.lfaclk = match config.lfa {
        LfaSource::Lfrco => Some(LFRCO_FREQ),
        LfaSource::Lfxo => Some(LFXO_FREQ),
    };
    clocks.lfbclk = clocks.lfaclk;
    fill_group_freqs(&mut clocks);

    unsafe { set_freqs(clocks) };
}
