//! Clock Management Unit (CMU) for EFR32 Series 2.

use core::mem::MaybeUninit;

pub use cmu_mod::vals::{
    Em01grpaclkctrlClksel as Em01GrpAClkSource, Em4grpaclkctrlClksel as Em4GrpAClkSource,
    Em23grpaclkctrlClksel as Em23GrpAClkSource, Hclkpresc as HclkPrescaler, IadcclkctrlClksel as IadcClkSource,
    Pclkpresc as PclkPrescaler, SysclkctrlClksel as SysclkSource, Wdog0clkctrlClksel as Wdog0ClkSource,
};
// These clock branches don't exist on MG22 (config 2): no EM01GRPCCLK split,
// no SYSRTC0, no second WDOG, no EUSART, no TRACE clock select.
#[cfg(not(silabs_series_2_config = "2"))]
pub use cmu_mod::vals::{
    Em01grpcclkctrlClksel as Em01GrpCClkSource, Eusart0clkctrlClksel as Eusart0ClkSource,
    Sysrtc0clkctrlClksel as Sysrtc0ClkSource, TraceclkctrlClksel as TraceClkSource,
    Wdog1clkctrlClksel as Wdog1ClkSource,
};
use critical_section::CriticalSection;

use crate::pac::CMU;
// `_mod` marks the metapac register modules, so they do not read like the `hfxo`, `lfxo` and
// `msc` fields and variables.
use crate::pac::{cmu as cmu_mod, hfrco as hfrco_mod, hfxo as hfxo_mod, lfxo as lfxo_mod, msc as msc_mod};
pub use crate::time::Hertz;

/// FSRCO frequency, 20 MHz.
pub const FSRCO_FREQ: Hertz = Hertz::mhz(20);

pub(crate) trait SealedCmuPeripheral {
    fn enable_bus_clock();
}

/// A peripheral with a CMU bus-clock gate. `build.rs` implements it for
/// every peripheral whose gate is in the metapac metadata.
#[allow(private_bounds)]
pub trait CmuPeripheral: SealedCmuPeripheral + 'static {}

/// Turns on bus clock for peripheral `T`.
///
/// This is one write to the gate register's SET alias, so it needs no
/// critical section.
pub fn enable<T: CmuPeripheral>() {
    T::enable_bus_clock();
}
/// LFRCO frequency, 32.768 kHz.
pub const LFRCO_FREQ: Hertz = Hertz(32_768);
/// ULFRCO frequency, about 1 kHz.
pub const ULFRCO_FREQ: Hertz = Hertz(1_000);
/// HFRCODPLL reset frequency, 19 MHz.
pub const HFRCODPLL_RESET_FREQ: Hertz = Hertz::mhz(19);
/// HFRCOEM23 reset frequency, 19 MHz.
pub const HFRCOEM23_RESET_FREQ: Hertz = Hertz::mhz(19);
/// LFXO frequency. EFR32 boards use a 32.768 kHz LF crystal.
pub const LFXO_FREQ: Hertz = Hertz(32_768);

/// HFXO crystal driver mode.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HfxoMode {
    /// Two-pin crystal on XI/XO.
    Xtal,
    /// External clock on XI.
    ExtClk,
    /// External clock with peak detector.
    ExtClkPkDet,
}

/// HFXO load capacitance trim source.
///
/// 1. **DEVINFO.MODXOCAL** (`HFXOCTUNEXIANA`) — set by Silicon Labs at
///    module manufacturing time. Used for module parts (BGM/MGM/etc.)
///    where Silicon Labs already characterised the assembled module.
///    Selected when `DEVINFO.MODULEINFO.HFXOCALVAL` reads as valid.
/// 2. **USERDATA flash page** at `0x0FE00100` — written by the board
///    manufacturer during board-level crystal trimming. Valid if the
///    16-bit value is `<= 0xFF` (unprogrammed flash reads `0xFFFF`).
/// 3. **Caller-supplied default** — used when neither token is valid,
///    e.g. on development boards without per-unit trimming.
///
/// Use [`HfxoCtune::Auto`] for the standard chain.
/// Use [`HfxoCtune::Fixed`] to override factory tokens (testing, custom characterisation).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HfxoCtune {
    /// Run the factory-trim resolution chain. The `default` is the
    /// step-3 fallback (typically the `SL_CLOCK_MANAGER_HFXO_CTUNE`
    /// value from a matching SLCP config — e.g. `140` for brd2713a).
    Auto { default: u8 },
    /// Uses this value and ignores DEVINFO and USERDATA.
    Fixed(u8),
}

impl HfxoCtune {
    /// Resolve to a concrete CTUNE byte. Performs DEVINFO/USERDATA reads
    /// for [`HfxoCtune::Auto`] via [`resolve_hfxo_ctune_auto`].
    ///
    /// Safe: the DEVINFO peripheral is memory-mapped read-only chip
    /// info, and the USERDATA flash page is a fixed-address read.
    pub fn resolve(self) -> u8 {
        match self {
            HfxoCtune::Fixed(v) => v,
            HfxoCtune::Auto { default } => resolve_hfxo_ctune_auto(default),
        }
    }
}

/// Run the factory-trim resolution chain nd return the resolved value.
///
/// Resolution order:
/// 1. **DEVINFO module trim** — `MODULEINFO.HFXOCALVAL` reads `false`
///    when a valid factory trim is present in `MODXOCAL.HFXOCTUNEXIANA`
///    (Silicon Labs module parts: BGM/MGM/…).
/// 2. **USERDATA flash page** — 16-bit value at `0x0FE00100`. Valid if
///    `<= 0xFF` (an 8-bit value zero-extended to 16 bits). Unprogrammed
///    flash reads back `0xFFFF`, which fails the check.
/// 3. **Caller-supplied default** — used when neither token is valid.
fn resolve_hfxo_ctune_auto(default: u8) -> u8 {
    let devinfo_valid = !crate::pac::DEVINFO.moduleinfo().read().hfxocalval();
    let userdata_raw = unsafe { core::ptr::read_volatile(0x0FE00100 as *const u16) };

    if devinfo_valid {
        crate::pac::DEVINFO.modxocal().read().hfxoctunexiana()
    } else if userdata_raw <= 0xFF {
        userdata_raw as u8
    } else {
        default
    }
}

/// HFXO oscillator configuration.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HfxoConfig {
    /// Crystal frequency, 38 MHz to 40 MHz.
    pub freq: Hertz,
    pub mode: HfxoMode,
    pub ctune: HfxoCtune,
}

/// HFRCODPLL frequency band.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HfrcoBand {
    Freq1M,
    Freq2M,
    Freq4M,
    Freq7M,
    Freq13M,
    Freq16M,
    /// Reset default.
    Freq19M,
    Freq26M,
    Freq32M,
    Freq38M,
    Freq48M,
    Freq56M,
    Freq64M,
    Freq80M,
}

impl HfrcoBand {
    pub const fn frequency(self) -> Hertz {
        match self {
            HfrcoBand::Freq1M => Hertz::mhz(1),
            HfrcoBand::Freq2M => Hertz::mhz(2),
            HfrcoBand::Freq4M => Hertz::mhz(4),
            HfrcoBand::Freq7M => Hertz::mhz(7),
            HfrcoBand::Freq13M => Hertz::mhz(13),
            HfrcoBand::Freq16M => Hertz::mhz(16),
            HfrcoBand::Freq19M => Hertz::mhz(19),
            HfrcoBand::Freq26M => Hertz::mhz(26),
            HfrcoBand::Freq32M => Hertz::mhz(32),
            HfrcoBand::Freq38M => Hertz::mhz(38),
            HfrcoBand::Freq48M => Hertz::mhz(48),
            HfrcoBand::Freq56M => Hertz::mhz(56),
            HfrcoBand::Freq64M => Hertz::mhz(64),
            HfrcoBand::Freq80M => Hertz::mhz(80),
        }
    }
}

/// HFRCODPLL configuration.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HfrcoDpllConfig {
    pub band: HfrcoBand,
}

/// LFXO load-capacitance trim source. Mirrors [`HfxoCtune`]'s resolution
/// chain — the LFXO MFG token lives at `0x0FE0009C` and is valid iff
/// `<= 0x7F` (the `LFXO_CAL_CAPTUNE` field is 7 bits wide).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LfxoCtune {
    /// Resolve at runtime: USERDATA → `default`. (LFXO has no DEVINFO
    /// module-info path equivalent to HFXO's `MODULEINFO.HFXOCALVAL`.)
    Auto { default: u8 },
    /// Uses this value, masked to `0..=0x7F`, and ignores the factory trim.
    Fixed(u8),
}

impl LfxoCtune {
    /// Resolve to a concrete CTUNE byte. Performs USERDATA reads for
    /// [`LfxoCtune::Auto`].
    pub fn resolve(self) -> u8 {
        match self {
            LfxoCtune::Fixed(v) => v & 0x7F,
            LfxoCtune::Auto { default } => resolve_lfxo_ctune_auto(default) & 0x7F,
        }
    }
}

fn resolve_lfxo_ctune_auto(default: u8) -> u8 {
    // Silicon Labs convention: 8-bit LFXO trim at `0x0FE0009C` in the
    // USERDATA flash page. Valid iff `<= 0x7F`. Unprogrammed flash reads
    // back `0xFF`, which fails the check.
    const USERDATA_LFXO_CTUNE: *const u8 = 0x0FE0009C as *const u8;
    let raw = unsafe { core::ptr::read_volatile(USERDATA_LFXO_CTUNE) };
    if raw <= 0x7F { raw } else { default }
}

/// LFXO oscillator configuration.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LfxoConfig {
    /// Load capacitance trim for the 32.768 kHz crystal.
    pub ctune: LfxoCtune,
}

impl LfxoConfig {
    /// Returns auto CTUNE with default `63`, the nominal LFXO trim of brd2713a.
    pub const fn new() -> Self {
        Self {
            ctune: LfxoCtune::Auto { default: 63 },
        }
    }
}

impl Default for LfxoConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Clock tree configuration for [`init_clocks`].
///
/// `Default::default()` describes the chip's POR / reset state — every
/// branch sources from its reset-default oscillator, no crystals
/// enabled. Build a `Config` by starting from `default()` and
/// overriding the fields that need to change.
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// HFXO crystal. `None` leaves HFXO untouched.
    pub hfxo: Option<HfxoConfig>,
    /// HFRCODPLL band calibration. `None` leaves HFRCO0 untouched — bus
    /// clock disabled, `HFRCO0.CAL` retains its POR calibration.
    pub hfrcodpll: Option<HfrcoDpllConfig>,
    /// LFXO crystal. `None` leaves LFXO untouched.
    pub lfxo: Option<LfxoConfig>,

    // High-frequency branches.
    /// SYSCLK source.
    pub sysclk: SysclkSource,
    /// EM01GRPACLK source select (TIMER0..9 live on this branch).
    pub em01grpaclk: Em01GrpAClkSource,
    /// EM01GRPCCLK source select (EUSART module B-instances live on this branch).
    /// Not present on MG22 (config 2).
    #[cfg(not(silabs_series_2_config = "2"))]
    pub em01grpcclk: Em01GrpCClkSource,
    /// HCLK = SYSCLK / `hclk_pre`.
    pub hclk_pre: HclkPrescaler,
    /// PCLK = HCLK / `pclk_pre`.
    pub pclk_pre: PclkPrescaler,

    // Low-frequency branches.
    /// EM23GRPACLK source select (peripherals active in EM2/EM3).
    pub em23grpaclk: Em23GrpAClkSource,
    /// EM4GRPACLK source select (peripherals active in EM4).
    pub em4grpaclk: Em4GrpAClkSource,
    /// SYSRTC0 clock source. Not present on MG22 (config 2).
    #[cfg(not(silabs_series_2_config = "2"))]
    pub sysrtc0: Sysrtc0ClkSource,
    /// WDOG0 clock source.
    pub wdog0: Wdog0ClkSource,
    /// WDOG1 clock source. Not present on MG22 (config 2).
    #[cfg(not(silabs_series_2_config = "2"))]
    pub wdog1: Wdog1ClkSource,

    // Per-peripheral branches.
    /// IADC clock source.
    pub iadc: IadcClkSource,
    /// EUSART0 clock source (EUSART0 is on the LF island so it gets a
    /// separate mux from EUSART1..3 which are on EM01GRPCCLK).
    /// Not present on MG22 (config 2 has EUART, not EUSART).
    #[cfg(not(silabs_series_2_config = "2"))]
    pub eusart0: Eusart0ClkSource,
    /// TRACE clock source. Only used by debugger ETM/ITM.
    /// Not a separate mux on MG22 (config 2).
    #[cfg(not(silabs_series_2_config = "2"))]
    pub trace: TraceClkSource,
}

impl Config {
    /// Configuration matching the chip's POR / reset state.
    pub const fn new() -> Self {
        Self {
            hfxo: None,
            hfrcodpll: None,
            lfxo: None,

            // POR defaults: all HF branches source from HFRCODPLL (theboot clock).
            // HCLK undivided, PCLK = HCLK/2.
            sysclk: SysclkSource::Hfrcodpll,
            em01grpaclk: Em01GrpAClkSource::Hfrcodpll,
            #[cfg(not(silabs_series_2_config = "2"))]
            em01grpcclk: Em01GrpCClkSource::Hfrcodpll,
            hclk_pre: HclkPrescaler::Div1,
            pclk_pre: PclkPrescaler::Div2,

            // POR defaults: LF branches source from LFRCO. Boards with
            // an LFXO crystal typically override to `Lfxo` for better
            // sleep-clock accuracy.
            em23grpaclk: Em23GrpAClkSource::Lfrco,
            em4grpaclk: Em4GrpAClkSource::Lfrco,
            #[cfg(not(silabs_series_2_config = "2"))]
            sysrtc0: Sysrtc0ClkSource::Lfrco,
            wdog0: Wdog0ClkSource::Lfrco,
            #[cfg(not(silabs_series_2_config = "2"))]
            wdog1: Wdog1ClkSource::Lfrco,

            // POR defaults: IADC on EM01GRPACLK, EUSART0 on EM01GRPCCLK,
            // TRACE on SYSCLK.
            iadc: IadcClkSource::Em01grpaclk,
            #[cfg(not(silabs_series_2_config = "2"))]
            eusart0: Eusart0ClkSource::Em01grpcclk,
            #[cfg(not(silabs_series_2_config = "2"))]
            trace: TraceClkSource::Sysclk,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Frozen clock-tree frequencies. Populated once by [`init_clocks`].
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Clocks {
    pub hfxo: Option<Hertz>,
    pub hfrcodpll: Hertz,
    pub hfrcoem23: Hertz,
    pub fsrco: Hertz,
    pub lfxo: Option<Hertz>,
    pub lfrco: Hertz,
    pub ulfrco: Hertz,

    pub sysclk: Hertz,
    pub hclk: Hertz,
    pub pclk: Hertz,
    pub em01grpaclk: Hertz,
    #[cfg(not(silabs_series_2_config = "2"))]
    pub em01grpcclk: Hertz,
}

/// Frozen clock frequencies. The existence of this value indicates that
/// the clock configuration can no longer be changed.
static mut CLOCK_FREQS: MaybeUninit<Clocks> = MaybeUninit::uninit();

/// Sets global clock frequencies.
///
/// # Safety
///
/// Safety: writes a mutable global. Caller must ensure no other code is
/// reading [`get_freqs`] concurrently.
unsafe fn set_freqs(freqs: Clocks) {
    debug!("cmu: {:?}", freqs);
    CLOCK_FREQS = MaybeUninit::new(freqs);
}

/// Read the frozen clock-tree table. Only call after [`init_clocks`] has run.
///
/// Safety: reads a mutable global. Caller must ensure [`set_freqs`] has
/// been called first.
pub(crate) unsafe fn get_freqs() -> &'static Clocks {
    (*&raw const CLOCK_FREQS).assume_init_ref()
}

fn hclk_divisor(p: HclkPrescaler) -> u32 {
    match p {
        HclkPrescaler::Div1 => 1,
        HclkPrescaler::Div2 => 2,
        HclkPrescaler::Div4 => 4,
        HclkPrescaler::Div8 => 8,
        HclkPrescaler::Div16 => 16,
        HclkPrescaler::_RESERVED_2
        | HclkPrescaler::_RESERVED_4
        | HclkPrescaler::_RESERVED_5
        | HclkPrescaler::_RESERVED_6
        | HclkPrescaler::_RESERVED_8
        | HclkPrescaler::_RESERVED_9
        | HclkPrescaler::_RESERVED_a
        | HclkPrescaler::_RESERVED_b
        | HclkPrescaler::_RESERVED_c
        | HclkPrescaler::_RESERVED_d
        | HclkPrescaler::_RESERVED_e => unreachable!(),
    }
}

fn pclk_divisor(p: PclkPrescaler) -> u32 {
    match p {
        PclkPrescaler::Div1 => 1,
        PclkPrescaler::Div2 => 2,
    }
}

fn sysclk_source_freq(src: SysclkSource, hfxo: Option<Hertz>, hfrcodpll: Hertz) -> Hertz {
    match src {
        SysclkSource::Fsrco => FSRCO_FREQ,
        SysclkSource::Hfrcodpll => hfrcodpll,
        SysclkSource::Hfxo => unwrap!(hfxo),
        SysclkSource::Clkin0 => panic!("Config.sysclk = Clkin0 not supported"),
        _ => unreachable!(),
    }
}

fn em01grpaclk_source_freq(src: Em01GrpAClkSource, hfxo: Option<Hertz>, hfrcodpll: Hertz, _hfrcoem23: Hertz) -> Hertz {
    match src {
        Em01GrpAClkSource::Fsrco => FSRCO_FREQ,
        Em01GrpAClkSource::Hfrcodpll => hfrcodpll,
        // The RT (radio) and HFRCOEM23 sources don't exist on MG22 (config 2).
        #[cfg(not(silabs_series_2_config = "2"))]
        Em01GrpAClkSource::Hfrcodpllrt => hfrcodpll,
        #[cfg(not(silabs_series_2_config = "2"))]
        Em01GrpAClkSource::Hfrcoem23 => _hfrcoem23,
        Em01GrpAClkSource::Hfxo => unwrap!(hfxo),
        #[cfg(not(silabs_series_2_config = "2"))]
        Em01GrpAClkSource::Hfxort => unwrap!(hfxo),
        _ => unreachable!(),
    }
}

// EM01GRPCCLK is a config-4/5/6 branch; MG22 (config 2) has no such mux.
#[cfg(not(silabs_series_2_config = "2"))]
fn em01grpcclk_source_freq(src: Em01GrpCClkSource, hfxo: Option<Hertz>, hfrcodpll: Hertz, hfrcoem23: Hertz) -> Hertz {
    match src {
        Em01GrpCClkSource::Fsrco => FSRCO_FREQ,
        Em01GrpCClkSource::Hfrcodpll | Em01GrpCClkSource::Hfrcodpllrt => hfrcodpll,
        Em01GrpCClkSource::Hfrcoem23 => hfrcoem23,
        Em01GrpCClkSource::Hfxo | Em01GrpCClkSource::Hfxort => unwrap!(hfxo),
        _ => unreachable!(),
    }
}

pub(crate) fn init_clocks(_cs: CriticalSection, config: Config) {
    // Bus-clock gates for the HAL's own peripherals + the SYSRTC0
    // mux that is configured below (its bus clock is independent of the
    // mux register and otherwise stays at reset-default off, which
    // BusFaults the first register access - including sleeptimer,
    // which the Silicon Labs SDK uses pervasively).
    CMU.clken0().modify(|w| {
        w.set_gpio(true);
        #[cfg(not(silabs_series_2_config = "2"))]
        w.set_sysrtc0(true);
        #[cfg(time_driver_timer0)]
        w.set_timer0(true);
        #[cfg(time_driver_timer1)]
        w.set_timer1(true);
    });

    // Start each oscillator before a branch selects it. SYSCLK on a stopped oscillator faults.
    if let Some(ref hfxo_cfg) = config.hfxo {
        init_hfxo(hfxo_cfg);
    }
    if let Some(ref lfxo_cfg) = config.lfxo {
        init_lfxo(lfxo_cfg);
        // No wait for the 4096-cycle (125 ms) start, because LF consumers (SYSRTC, WDOG) tolerate a
        // clock that is still starting.
    }
    if let Some(ref hfrcodpll_cfg) = config.hfrcodpll {
        init_hfrcodpll(hfrcodpll_cfg.band);
    }

    // Branch-select MUXes + prescalers. Authoritative write of every
    // clock-tree MUX — same-value writes are glitchless on Series 2 so
    // calling this repeatedly is safe. POR defaults survive on any
    // branch the Config doesn't override.
    CMU.sysclkctrl().modify(|w| {
        w.set_clksel(config.sysclk);
        w.set_hclkpresc(config.hclk_pre);
        w.set_pclkpresc(config.pclk_pre);
    });
    CMU.em01grpaclkctrl().modify(|w| w.set_clksel(config.em01grpaclk));
    CMU.em23grpaclkctrl().modify(|w| w.set_clksel(config.em23grpaclk));
    CMU.em4grpaclkctrl().modify(|w| w.set_clksel(config.em4grpaclk));
    CMU.wdog0clkctrl().modify(|w| w.set_clksel(config.wdog0));
    CMU.iadcclkctrl().modify(|w| w.set_clksel(config.iadc));
    // Branches that only exist on config 4/5/6 (absent on MG22 / config 2).
    #[cfg(not(silabs_series_2_config = "2"))]
    {
        CMU.em01grpcclkctrl().modify(|w| w.set_clksel(config.em01grpcclk));
        CMU.sysrtc0clkctrl().modify(|w| w.set_clksel(config.sysrtc0));
        CMU.wdog1clkctrl().modify(|w| w.set_clksel(config.wdog1));
        CMU.eusart0clkctrl().modify(|w| w.set_clksel(config.eusart0));
        CMU.traceclkctrl().modify(|w| w.set_clksel(config.trace));
    }

    let hfxo_hz = config.hfxo.map(|c| c.freq);
    let hfrcodpll_hz = match config.hfrcodpll {
        None => HFRCODPLL_RESET_FREQ,
        Some(c) => c.band.frequency(),
    };
    let hfrcoem23_hz = HFRCOEM23_RESET_FREQ;
    let lfxo_hz = config.lfxo.map(|_| LFXO_FREQ);

    let sysclk = sysclk_source_freq(config.sysclk, hfxo_hz, hfrcodpll_hz);
    let hclk = Hertz(sysclk.0 / hclk_divisor(config.hclk_pre));
    let pclk = Hertz(hclk.0 / pclk_divisor(config.pclk_pre));
    let em01grpaclk = em01grpaclk_source_freq(config.em01grpaclk, hfxo_hz, hfrcodpll_hz, hfrcoem23_hz);
    #[cfg(not(silabs_series_2_config = "2"))]
    let em01grpcclk = em01grpcclk_source_freq(config.em01grpcclk, hfxo_hz, hfrcodpll_hz, hfrcoem23_hz);

    cmu_assert!(sysclk.0 <= 80_000_000);
    cmu_assert!(hclk.0 <= 80_000_000);

    unsafe {
        set_freqs(Clocks {
            hfxo: hfxo_hz,
            hfrcodpll: hfrcodpll_hz,
            hfrcoem23: hfrcoem23_hz,
            fsrco: FSRCO_FREQ,
            lfxo: lfxo_hz,
            lfrco: LFRCO_FREQ,
            ulfrco: ULFRCO_FREQ,
            sysclk,
            hclk,
            pclk,
            em01grpaclk,
            #[cfg(not(silabs_series_2_config = "2"))]
            em01grpcclk,
        });
    }
}

/// Starts HFXO in crystal mode and leaves it on demand after the first lock.
///
/// HFXO stops for a short time, so no clock (SYSCLK, radio) may use it yet.
///
/// # Panics
///
/// Panics for a mode other than [`HfxoMode::Xtal`].
pub(crate) fn init_hfxo(config: &HfxoConfig) {
    use hfxo_mod::vals as hfxo_vals;

    use crate::pac::HFXO0;

    assert!(
        matches!(config.mode, HfxoMode::Xtal),
        "init_hfxo: only HfxoMode::Xtal supported"
    );

    let ctune = config.ctune.resolve();
    // On these configs the XI and XO capacitor banks differ, so XO gets a higher CTUNE.
    #[cfg(any(
        silabs_series_2_config = "3",
        silabs_series_2_config = "4",
        silabs_series_2_config = "5",
        silabs_series_2_config = "6",
        silabs_series_2_config = "8",
    ))]
    const CTUNE_XO_DELTA: i16 = 40;
    #[cfg(not(any(
        silabs_series_2_config = "3",
        silabs_series_2_config = "4",
        silabs_series_2_config = "5",
        silabs_series_2_config = "6",
        silabs_series_2_config = "8",
    )))]
    const CTUNE_XO_DELTA: i16 = 0;
    let ctune_xo = ((ctune as i16) + CTUNE_XO_DELTA).clamp(0, 0xFF) as u8;

    // 1. Bus clock on, registers unlocked.
    CMU.clken0().modify(|w| w.set_hfxo0(true));
    HFXO0.lock().write(|w| w.set_lockkey(hfxo_mod::vals::Lockkey::Unlock));

    // Stop HFXO. hfxo_s2v3 has no DISONDEMANDBUFOUT, so set only DISONDEMAND.
    HFXO0.ctrl_set().write(|w| w.set_disondemand(true));
    HFXO0.ctrl_clr().write(|w| w.set_forceen(true));
    while HFXO0.status().read().ens() {}

    // First-lock values.
    HFXO0.xtalcfg().write(|w| {
        w.set_timeoutcblsb(hfxo_vals::Timeoutcblsb::T416us);
        w.set_timeoutsteady(hfxo_vals::Timeoutsteady::T833us);
        w.set_ctunexostartup(0);
        w.set_ctunexistartup(0);
        w.set_corebiasstartup(32);
        w.set_corebiasstartupi(32);
    });

    // Keep SKIPCOREBIASOPT. A previous init sets it after the core-bias optimization.
    let skip = HFXO0.xtalctrl().read().skipcorebiasopt();
    HFXO0.xtalctrl().write(|w| {
        w.set_skipcorebiasopt(skip);
        w.set_coredgenana(hfxo_vals::Coredgenana::None);
        w.set_ctunefixana(CTUNEFIXANA_DEFAULT);
        w.set_ctunexoana(ctune_xo);
        w.set_ctunexiana(ctune);
        w.set_corebiasana(60);
    });

    // xG23 and xG28 need this HFXO workaround on an undocumented register at HFXO0 offset 0x38.
    #[cfg(any(silabs_series_2_config = "3", silabs_series_2_config = "8"))]
    {
        const HFXO0_ERRATA_PM2871: *mut u32 = 0x5A00_4038 as *mut u32;
        // SAFETY: 0x5A00_4038 is an aligned word inside the HFXO0_NS block 0x5A004000..=0x5A007FFF,
        // the address that the xG24 and xG25 reference manuals give (4.2.4.1, Peripheral Map). The
        // read-modify-write changes only bits 10 and 11.
        unsafe {
            let cur = core::ptr::read_volatile(HFXO0_ERRATA_PM2871);
            core::ptr::write_volatile(HFXO0_ERRATA_PM2871, (cur & !0xC00) | (2 << 10));
        }
    }

    HFXO0.cfg().modify(|w| {
        w.set_sqbufschtrgana(false);
        w.set_enxidcbiasana(false);
        w.set_mode(hfxo_vals::Mode::Xtal);
    });

    HFXO0.ctrl().modify(|w| {
        w.set_forcexo2gndana(false);
        w.set_forcexi2gndana(false);
        w.set_disondemand(false);
        // EM23ONDEMAND isn't present on MG22's HFXO (hfxo_s2v2).
        #[cfg(not(silabs_series_2_config = "2"))]
        w.set_em23ondemand(false);
        w.set_forceen(true);
    });

    // 8. Wait for HFXO lock and core-bias optimisation to complete.
    //    `STATUS.FSMLOCK` exists only on Series 2 configs 1/2/7/9
    //    (xG21/xG22/xG27/xG29); chips that lack it (MG24/MG26 etc.) skip
    //    that bit in the readiness check.
    while {
        let s = HFXO0.status().read();
        let ready = s.rdy() && s.corebiasoptrdy() && s.ens();
        #[cfg(any(
            silabs_series_2_config = "1",
            silabs_series_2_config = "2",
            silabs_series_2_config = "7",
            silabs_series_2_config = "9",
        ))]
        let ready = ready && s.fsmlock();
        !ready
    } {}

    // Set DISONDEMAND, so that the XTALCFG and XTALCTRL writes below apply to the next locks.
    HFXO0.ctrl_set().write(|w| w.set_disondemand(true));

    // `CMD.MANUALOVERRIDE` exists only on configs 2/7/9 (xG22/xG27/xG29) —
    // xG21 has FSMLOCK but no MANUALOVERRIDE.
    #[cfg(any(
        silabs_series_2_config = "2",
        silabs_series_2_config = "7",
        silabs_series_2_config = "9",
    ))]
    HFXO0.cmd().write(|w| w.set_manualoverride(true));

    // Spin for the FSMLOCK FSM to release — same set as step 8.
    #[cfg(any(
        silabs_series_2_config = "1",
        silabs_series_2_config = "2",
        silabs_series_2_config = "7",
        silabs_series_2_config = "9",
    ))]
    while HFXO0.status().read().fsmlock() {}

    // After the first lock, T83us is enough for TIMEOUTSTEADY.
    HFXO0.xtalcfg().modify(|w| {
        w.set_timeoutsteady(hfxo_vals::Timeoutsteady::T83us);
    });

    // The core-bias optimization runs only at the first lock.
    HFXO0.xtalctrl_set().write(|w| w.set_skipcorebiasopt(true));

    HFXO0.ctrl_clr().write(|w| w.set_disondemand(true));
    HFXO0.ctrl_clr().write(|w| w.set_forceen(true));
}

/// Fixed HFXO tuning capacitor setting (`HFXO_XTALCTRL.CTUNEFIXANA`) per chip family.
///
/// `Xo` on xG23, xG24, xG25, xG26 and xG28 (configs 3, 4, 5, 6 and 8), and `Both` on the others.
///
/// Silicon Labs applies this HFXO workaround on these families. The MG24 and FG25 errata sheets do
/// not list it. On xG24 and xG26 it is meant for the 20 dBm PA parts, but this code applies it to
/// every part of the family.
#[cfg(any(
    silabs_series_2_config = "3",
    silabs_series_2_config = "4",
    silabs_series_2_config = "5",
    silabs_series_2_config = "6",
    silabs_series_2_config = "8",
))]
const CTUNEFIXANA_DEFAULT: hfxo_mod::vals::Ctunefixana = hfxo_mod::vals::Ctunefixana::Xo;
#[cfg(not(any(
    silabs_series_2_config = "3",
    silabs_series_2_config = "4",
    silabs_series_2_config = "5",
    silabs_series_2_config = "6",
    silabs_series_2_config = "8",
)))]
const CTUNEFIXANA_DEFAULT: hfxo_mod::vals::Ctunefixana = hfxo_mod::vals::Ctunefixana::Both;

/// Sets up LFXO in crystal mode. It starts on demand.
///
/// LFXO stops for a short time, so call this before an LF branch (SYSRTC, WDOG, ...) uses it.
pub(crate) fn init_lfxo(config: &LfxoConfig) {
    use lfxo_mod::vals as lfxo_vals;

    use crate::pac::LFXO;

    let ctune = config.ctune.resolve();

    // 1. Bus clock on, registers unlocked.
    CMU.clken0().modify(|w| w.set_lfxo(true));
    LFXO.lock().write(|w| w.set_lockkey(lfxo_mod::vals::Lockkey::Unlock));

    // CAL and CFG are writable only while LFXO is stopped.
    LFXO.ctrl_set().write(|w| w.set_disondemand(true));
    LFXO.ctrl_clr().write(|w| w.set_forceen(true));
    while LFXO.status().read().ens() {}

    // Gain 1 is the value for a 32.768 kHz crystal.
    LFXO.cal().write(|w| {
        w.set_gain(1);
        w.set_captune(ctune);
    });

    LFXO.cfg().write(|w| {
        w.set_timeout(lfxo_vals::Timeout::Cycles4k);
        w.set_mode(lfxo_vals::Mode::Xtal);
        w.set_highampl(false);
        w.set_agc(true);
    });

    // With all flags off, LFXO starts on demand.
    LFXO.ctrl().write(|w| {
        w.set_faildetem4wuen(false);
        w.set_faildeten(false);
        w.set_disondemand(false);
        w.set_forceen(false);
    });
}

/// Sets the HFRCODPLL band without a DPLL lock.
///
/// Writes the band's factory calibration word to `HFRCO0.CAL`. The write glitches clocks on
/// HFRCODPLL, so call this before an HF branch selects it.
pub(crate) fn init_hfrcodpll(band: HfrcoBand) {
    use crate::pac::{DPLL0, HFRCO0};

    let cal_word = hfrcodpll_devinfo_get(band);
    assert!(
        cal_word != 0 && cal_word != u32::MAX,
        "init_hfrcodpll: DEVINFO calibration word missing for band {:?}",
        band as u8
    );

    CMU.clken0().modify(|w| {
        w.set_hfrco0(true);
        w.set_dpll0(true);
    });

    // Stop the DPLL before the band changes.
    if DPLL0.en().read().en() {
        DPLL0.en_clr().write(|w| w.set_en(true));
        // dpll_s2v0 (MG22) has no DISABLING status bit — disable is immediate.
        #[cfg(not(silabs_series_2_config = "2"))]
        while DPLL0.en().read().disabling() {}
    }

    // 1 MHz and 2 MHz use the 4 MHz word with a divider.
    let cal_word = (cal_word & !HFRCO_CAL_CLKDIV_MASK)
        | match band {
            HfrcoBand::Freq1M => HFRCO_CAL_CLKDIV_DIV4,
            HfrcoBand::Freq2M => HFRCO_CAL_CLKDIV_DIV2,
            _ => 0,
        };

    while {
        let s = HFRCO0.status().read();
        s.syncbusy() || s.freqbsy()
    } {}

    // When SYSCLK runs from HFRCODPLL, the band write changes the core frequency, so set the
    // slowest flash wait states and PCLK / 2 first. This code does not divide RHCLK: the radio does
    // not run during init, and `init_clocks` never resets `SYSCLKCTRL.RHCLKPRESC`.
    let sysclk_is_hfrcodpll = CMU.sysclkctrl().read().clksel() == SysclkSource::Hfrcodpll;
    if sysclk_is_hfrcodpll {
        CMU.clken1().modify(|w| w.set_msc(true));
        let msc = crate::pac::MSC;
        msc.lock().write(|w| w.set_lockkey(msc_mod::vals::Lockkey::Unlock));
        msc.readctrl().modify(|w| w.set_mode(msc_mod::vals::Mode::Ws3));

        CMU.sysclkctrl().modify(|w| w.set_pclkpresc(PclkPrescaler::Div2));
    }

    HFRCO0.cal().write_value(hfrco_mod::regs::Cal(cal_word));
}

/// Returns the `DEVINFO.HFRCODPLLCALn` word for `band`. `n` is a DEVINFO slot, not a frequency.
fn hfrcodpll_devinfo_get(band: HfrcoBand) -> u32 {
    use crate::pac::DEVINFO;
    let idx = match band {
        // 1, 2 and 4 MHz share one word.
        HfrcoBand::Freq1M | HfrcoBand::Freq2M | HfrcoBand::Freq4M => 0,
        HfrcoBand::Freq7M => 3,
        HfrcoBand::Freq13M => 6,
        HfrcoBand::Freq16M => 7,
        HfrcoBand::Freq19M => 8,
        HfrcoBand::Freq26M => 10,
        HfrcoBand::Freq32M => 11,
        HfrcoBand::Freq38M => 12,
        HfrcoBand::Freq48M => 13,
        HfrcoBand::Freq56M => 14,
        HfrcoBand::Freq64M => 15,
        HfrcoBand::Freq80M => 16,
    };
    // The metapac has one accessor per word, not an array.
    let reg = match idx {
        0 => DEVINFO.hfrcodpllcal0().read().0,
        3 => DEVINFO.hfrcodpllcal3().read().0,
        6 => DEVINFO.hfrcodpllcal6().read().0,
        7 => DEVINFO.hfrcodpllcal7().read().0,
        8 => DEVINFO.hfrcodpllcal8().read().0,
        10 => DEVINFO.hfrcodpllcal10().read().0,
        11 => DEVINFO.hfrcodpllcal11().read().0,
        12 => DEVINFO.hfrcodpllcal12().read().0,
        13 => DEVINFO.hfrcodpllcal13().read().0,
        14 => DEVINFO.hfrcodpllcal14().read().0,
        15 => DEVINFO.hfrcodpllcal15().read().0,
        16 => DEVINFO.hfrcodpllcal16().read().0,
        _ => unreachable!(),
    };
    reg
}

const HFRCO_CAL_CLKDIV_MASK: u32 = 0x0300_0000;
const HFRCO_CAL_CLKDIV_DIV2: u32 = 0x0100_0000;
const HFRCO_CAL_CLKDIV_DIV4: u32 = 0x0200_0000;
