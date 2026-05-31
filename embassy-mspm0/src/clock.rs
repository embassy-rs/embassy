//! Clock-tree configuration for MSPM0 devices.
//!
//! `embassy-mspm0` boots with the chip's reset-state clock tree: MCLK is
//! sourced from the internal SYSOSC at 32 MHz. For applications that need
//! tighter frequency accuracy (no ±1 % SYSOSC drift) or higher CPU rates
//! (up to 80 MHz with SYSPLL on supported devices, up to 40 MHz from HFXT)
//! this module brings up the external crystal and switches MCLK over to it.
//!
//! Coverage in this version is intentionally narrow:
//!
//! * **External HFXT crystal** on PA5/PA6 (HFXIN/HFXOUT), 4–48 MHz range.
//! * **MCLK** can stay on SYSOSC (default) or switch to HSCLK ← HFCLK ← HFXT.
//! * SYSPLL, MCLK divider (MDIV), and ULPCLK divider (UDIV) are intentionally
//!   not exposed yet — they belong in follow-up PRs once their behaviour is
//!   fully bench-verified on this family.
//!
//! `embassy-time`'s tick source is LFCLK (32.768 kHz), which is independent
//! of MCLK, so changing MCLK here does not corrupt `embassy_time::Timer`
//! durations.

use crate::pac;
use crate::pac::sysctl::vals;

/// External high-frequency crystal (HFXT) configuration.
///
/// HFXT lives on the dedicated `HFXIN`/`HFXOUT` analog pads (PA5/PA6 on
/// MSPM0G350x packages). Supplying this struct in [`Config::hfxt`] causes
/// `embassy_mspm0::init` to:
///
/// 1. Disable the digital input buffer + pulls on PA5/PA6 so the analog
///    crystal driver can take the pads.
/// 2. Program `SYSCTL.HFCLKCLKCFG` with the matching range and the maximum
///    startup-time window (~16 ms — conservative for cheap crystals).
/// 3. Set `HSCLKEN.HFXTEN = 1` and poll `CLKSTATUS.HFCLKGOOD` until the
///    oscillator is running.
#[derive(Clone, Copy, Debug)]
pub struct HfxtConfig {
    /// Crystal frequency in Hz. Must be in the closed interval
    /// 4 MHz..=48 MHz. The hardware-range bits in `HFXTRSEL` are picked
    /// automatically.
    pub freq_hz: u32,
}

/// Source for `MCLK`, the main system clock that drives the CPU and most
/// peripheral buses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MclkSource {
    /// Internal 32 MHz oscillator. This is the reset default and the safest
    /// choice — no external components, no waiting on a crystal startup.
    Sysosc,
    /// `HSCLK`, which is in turn sourced from `HFCLK` ← HFXT in this version.
    /// Requires [`Config::hfxt`] to be `Some`.
    Hsclk,
}

/// Clock-tree configuration.
///
/// Pass into [`crate::Config::clock`]. The default leaves the chip on its
/// reset-state SYSOSC and does not bring up any external oscillator —
/// matching pre-clock-config behaviour.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// If `Some`, the external HFXT crystal is brought up at boot.
    pub hfxt: Option<HfxtConfig>,
    /// Selector for the MCLK source. Defaults to [`MclkSource::Sysosc`].
    pub mclk: MclkSource,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hfxt: None,
            mclk: MclkSource::Sysosc,
        }
    }
}

/// Apply `cfg` to the SYSCTL clock-tree registers. Called from
/// `embassy_mspm0::init`.
///
/// Order matters: HFXT first (so HFCLK is valid before any later mux can
/// route to it), then MCLK switch (flash wait state set before raising
/// MCLK above the SYSOSC's 32 MHz / WAIT0 envelope).
pub(crate) fn configure(cfg: &Config) {
    if let Some(hfxt) = cfg.hfxt {
        enable_hfxt(&hfxt);
    }

    match cfg.mclk {
        MclkSource::Sysosc => {}
        MclkSource::Hsclk => {
            assert!(
                cfg.hfxt.is_some(),
                "MclkSource::Hsclk requires Config::hfxt to be Some \
                 (SYSPLL is not yet supported by embassy-mspm0)"
            );
            switch_mclk_to_hsclk();
        }
    }
}

fn enable_hfxt(cfg: &HfxtConfig) {
    let s = pac::SYSCTL;

    // HFXIN/HFXOUT are dedicated alternate functions on PA5/PA6. Writing the
    // default (cleared) PINCM disables both the digital input buffer and
    // pull resistors so the analog crystal path is selected. PA5 → PINCM 9,
    // PA6 → PINCM 10. These indices are fixed on G350x and don't go through
    // the gpio_pincm() lookup, which translates pin-number IDs.
    pac::IOMUX.pincm(9).write_value(Default::default());
    pac::IOMUX.pincm(10).write_value(Default::default());

    let range = match cfg.freq_hz {
        4_000_000..=8_000_000 => vals::Hfxtrsel::RANGE4TO8,
        8_000_001..=16_000_000 => vals::Hfxtrsel::RANGE8TO16,
        16_000_001..=32_000_000 => vals::Hfxtrsel::RANGE16TO32,
        32_000_001..=48_000_000 => vals::Hfxtrsel::RANGE32TO48,
        _ => panic!("HFXT freq must be 4..=48 MHz"),
    };

    s.hfclkclkcfg().write(|w| {
        w.set_hfxtrsel(range);
        // ~16 ms startup budget — bench debugging showed a real 40 MHz HC-49S
        // is good in ~1 ms, but cheaper crystals can take several. The
        // generous setting only delays boot, never causes a hang.
        w.set_hfxttime(vals::Hfxttime::MAXSTARTTIME);
        // We poll HFCLKGOOD ourselves; the hardware startup-fault monitor
        // would otherwise need its own ISR plumbing.
        w.set_hfclkfltchk(false);
    });

    s.hsclken().modify(|w| {
        w.set_hfxten(true);
        // useexthfclk = bypass mode (external clock signal, not a crystal).
        // We always drive a real crystal in this version.
        w.set_useexthfclk(false);
    });

    while !s.clkstatus().read().hfclkgood() {}
}

fn switch_mclk_to_hsclk() {
    let s = pac::SYSCTL;

    // Flash needs one wait state above 24 MHz on G350x. Set this BEFORE the
    // MCLK source switch so the CPU never tries to fetch faster than flash
    // can serve.
    s.mclkcfg().modify(|w| {
        w.set_flashwait(vals::Flashwait::WAIT1);
    });

    s.hsclkcfg().modify(|w| w.set_hsclksel(vals::Hsclksel::HFCLKCLK));
    while !s.clkstatus().read().hsclkgood() {}

    s.mclkcfg().modify(|w| w.set_usehsclk(true));
    while s.clkstatus().read().hsclkmux() != vals::Hsclkmux::HSCLK {}
}
