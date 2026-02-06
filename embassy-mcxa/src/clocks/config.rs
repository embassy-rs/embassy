//! Clock Configuration
//!
//! This module holds configuration types used for the system clocks. For
//! configuration of individual peripherals, see [`super::periph_helpers`].

use super::PoweredClock;

/// This type represents a divider in the range 1..=256.
///
/// At a hardware level, this is an 8-bit register from 0..=255,
/// which adds one.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Div8(pub(super) u8);

impl Div8 {
    /// Store a "raw" divisor value that will divide the source by
    /// `(n + 1)`, e.g. `Div8::from_raw(0)` will divide the source
    /// by 1, and `Div8::from_raw(255)` will divide the source by
    /// 256.
    pub const fn from_raw(n: u8) -> Self {
        Self(n)
    }

    /// Divide by one, or no division
    pub const fn no_div() -> Self {
        Self(0)
    }

    /// Store a specific divisor value that will divide the source
    /// by `n`. e.g. `Div8::from_divisor(1)` will divide the source
    /// by 1, and `Div8::from_divisor(256)` will divide the source
    /// by 256.
    ///
    /// Will return `None` if `n` is not in the range `1..=256`.
    /// Consider [`Self::from_raw`] for an infallible version.
    pub const fn from_divisor(n: u16) -> Option<Self> {
        let Some(n) = n.checked_sub(1) else {
            return None;
        };
        if n > (u8::MAX as u16) {
            return None;
        }
        Some(Self(n as u8))
    }

    /// Convert into "raw" bits form
    #[inline(always)]
    pub const fn into_bits(self) -> u8 {
        self.0
    }

    /// Convert into "divisor" form, as a u32 for convenient frequency math
    #[inline(always)]
    pub const fn into_divisor(self) -> u32 {
        self.0 as u32 + 1
    }
}

/// ```text
///               ┌─────────────────────────────────────────────────────────┐
///               │                                                         │
///               │   ┌───────────┐  clk_out   ┌─────────┐                  │
///    XTAL ──────┼──▷│ System    │───────────▷│         │       clk_in     │
///               │   │  OSC      │ clkout_byp │   MUX   │──────────────────┼──────▷
///   EXTAL ──────┼──▷│           │───────────▷│         │                  │
///               │   └───────────┘            └─────────┘                  │
///               │                                                         │
///               │   ┌───────────┐ fro_hf_root  ┌────┐          fro_hf     │
///               │   │ FRO180    ├───────┬─────▷│ CG │─────────────────────┼──────▷
///               │   │           │       │      ├────┤         clk_45m     │
///               │   │           │       └─────▷│ CG │─────────────────────┼──────▷
///               │   └───────────┘              └────┘                     │
///               │   ┌───────────┐ fro_12m_root  ┌────┐         fro_12m    │
///               │   │ FRO12M    │────────┬─────▷│ CG │────────────────────┼──────▷
///               │   │           │        │      ├────┤          clk_1m    │
///               │   │           │        └─────▷│1/12│────────────────────┼──────▷
///               │   └───────────┘               └────┘                    │
///               │                                                         │
///               │                  ┌──────────┐                           │
///               │                  │000       │                           │
///               │      clk_in      │          │                           │
///               │  ───────────────▷│001       │                           │
///               │      fro_12m     │          │                           │
///               │  ───────────────▷│010       │                           │
///               │    fro_hf_root   │          │                           │
///               │  ───────────────▷│011       │              main_clk     │
///               │                  │          │───────────────────────────┼──────▷
/// clk_16k ──────┼─────────────────▷│100       │                           │
///               │       none       │          │                           │
///               │  ───────────────▷│101       │                           │
///               │     pll1_clk     │          │                           │
///               │  ───────────────▷│110       │                           │
///               │       none       │          │                           │
///               │  ───────────────▷│111       │                           │
///               │                  └──────────┘                           │
///               │                        ▲                                │
///               │                        │                                │
///               │                     SCG SCS                             │
///               │ SCG-Lite                                                │
///               └─────────────────────────────────────────────────────────┘
///
///
///                      clk_in      ┌─────┐
///                  ───────────────▷│00   │
///                      clk_45m     │     │
///                  ───────────────▷│01   │      ┌───────────┐   pll1_clk
///                       none       │     │─────▷│   SPLL    │───────────────▷
///                  ───────────────▷│10   │      └───────────┘
///                      fro_12m     │     │
///                  ───────────────▷│11   │
///                                  └─────┘
/// ```
#[non_exhaustive]
pub struct ClocksConfig {
    /// Power states of VDD Core
    pub vdd_power: VddPowerConfig,
    /// Clocks that are used to drive the main clock, including the AHB and CPU core
    pub main_clock: MainClockConfig,
    /// FIRC, FRO180, 45/60/90/180M clock source
    pub firc: Option<FircConfig>,
    /// SIRC, FRO12M, clk_12m clock source
    // NOTE: I don't think we *can* disable the SIRC?
    pub sirc: SircConfig,
    /// FRO16K clock source
    pub fro16k: Option<Fro16KConfig>,
    /// SOSC, clk_in clock source
    ///
    /// NOTE: Requires `sosc-as-gpio` feature disabled, which also disables GPIO access to P1_30 and P1_31
    #[cfg(not(feature = "sosc-as-gpio"))]
    pub sosc: Option<SoscConfig>,
    /// SPLL
    pub spll: Option<SpllConfig>,
}

// Power (which is not a clock)

/// Selected VDD Power Mode
#[derive(Copy, Clone, PartialEq, Debug, Default)]
#[non_exhaustive]
pub enum VddLevel {
    /// Standard "mid drive" "MD" power, 1.0v VDD Core
    #[default]
    MidDriveMode,

    /// Overdrive "OD" power, 1.2v VDD Core
    OverDriveMode,
}

#[derive(Copy, Clone, PartialEq)]
#[non_exhaustive]
pub enum VddDriveStrength {
    /// Low drive
    Low { enable_bandgap: bool },

    /// Normal drive
    Normal,
}

#[derive(Copy, Clone)]
#[non_exhaustive]
pub struct VddModeConfig {
    /// VDD_CORE/LDO_CORE voltage level
    pub level: VddLevel,
    /// VDD_CORE/LDO_CORE drive strength
    pub drive: VddDriveStrength,
}

/// Settings for gating power to on-chip flash
///
/// Applies to both "light" WFE sleep, as well as Deep Sleep. Requires that
///
/// ## FlashDoze
///
/// Disables flash memory accesses and places flash memory in Low-Power state whenever the core clock
/// is gated (CKMODE > 0) because of execution of WFI, WFE, or SLEEPONEXIT. Other bus masters that
/// attempt to access the flash memory stalls until the core is no longer sleeping.
///
/// # FlashWake
///
/// Specifies that when this field becomes 1, an attempt to read the flash memory when it is in Low-Power
/// state because of FLASHCR[FLASHDIS] or FLASHCR[FLASHDOZE], causes the flash memory to exit
/// Low-Power state for the duration of the flash memory access.
#[derive(Copy, Clone, Default)]
#[non_exhaustive]
pub enum FlashSleep {
    /// Don't ever set the flash to sleep
    #[default]
    Never,
    /// Set FlashDoze
    ///
    /// This setting is only effective if [CoreSleep] has been configured with at least
    /// the `WfeGated` option or deeper.
    FlashDoze,
    /// Set FlashDoze + FlashWake
    ///
    /// This setting is only effective if [CoreSleep] has been configured with at least
    /// the `WfeGated` option or deeper.
    //
    // TODO: This *might* be required for DMA out of flash to actually work when
    // the core is sleeping, otherwise DMA will stall? Needs to be confirmed.
    FlashDozeWithFlashWake,
}

/// Maximum sleep depth for the CPU core
#[derive(Copy, Clone, Default, Debug)]
#[non_exhaustive]
pub enum CoreSleep {
    /// System will sleep using WFE when idle, but the CPU clock domain will not ever
    /// be gated. This mode uses the most power, but allows for debugging to
    /// continue uninterrupted.
    ///
    /// With this setting, the system never leaves the "Active" configuration mode.
    #[default]
    WfeUngated,
    /// The system will sleep using WFE when idle, and the CPU clock domain will be
    /// be gated. If configured with [FlashSleep], the internal flash may be gated
    /// as well.
    ///
    /// ## WARNING
    ///
    /// Enabling this mode has potential danger to soft-lock the system!
    ///
    /// * This mode WILL detach the debugging/RTT/defmt session if active upon first sleep.
    /// * This mode WILL also require ISP mode recovery in order to re-flash if the core becomes
    ///   "stuck" in sleep.
    WfeGated,
    /// The system will go to deep sleep when idle, and the CPU clock domain will be
    /// be gated. If configured with [FlashSleep], the internal flash may be gated
    /// as well.
    ///
    /// This will also move the system into the "low power" state, which will disable any
    /// clocks not configured as `PoweredClock::AlwaysActive".
    ///
    /// ## TODO
    ///
    /// For now, this REQUIRES calling unsafe `okay_but_actually_enable_deep_sleep()`
    /// otherwise we'd ALWAYS go to deep sleep on every WFE. We need to implement a
    /// custom executor that does proper go-to-deepsleep and come-back-from-deepsleep
    /// before un-chickening this. If the method isn't called, we just set to `WfeGated`
    /// instead.
    ///
    /// ## WARNING
    ///
    /// Enabling this mode has potential danger to soft-lock the system!
    ///
    /// * This mode WILL detach the debugging/RTT/defmt session if active upon first sleep.
    /// * This mode WILL also require ISP mode recovery in order to re-flash if the core becomes
    ///   "stuck" in sleep.
    DeepSleep,
}

/// Power control options for the VDD domain, including the CPU and flash memory
#[derive(Copy, Clone)]
#[non_exhaustive]
pub struct VddPowerConfig {
    /// Active power mode, used when not in Deep Sleep
    pub active_mode: VddModeConfig,
    /// Low power mode, used when in Deep Sleep
    pub low_power_mode: VddModeConfig,
    /// CPU core clock gating settings
    pub core_sleep: CoreSleep,
    /// Internal flash clock gating settings
    pub flash_sleep: FlashSleep,
}

// Main Clock

/// Main clock source
#[derive(Copy, Clone)]
pub enum MainClockSource {
    /// Clock derived from `clk_in`, via the external oscillator (8-50MHz)
    ///
    /// NOTE: Requires `sosc-as-gpio` feature disabled, which also disables GPIO access to P1_30 and P1_31
    #[cfg(not(feature = "sosc-as-gpio"))]
    SoscClkIn,
    /// Clock derived from `fro_12m`, via the internal 12MHz oscillator (12MHz)
    SircFro12M,
    /// Clock derived from `fro_hf_root`, via the internal 45/60/90/180M clock source (45-180MHz)
    FircHfRoot,
    /// Clock derived from `clk_16k` (vdd core)
    RoscFro16K,
    /// Clock derived from `pll1_clk`, via the internal PLL
    SPll1,
}

#[derive(Copy, Clone)]
pub struct MainClockConfig {
    /// Selected clock source
    pub source: MainClockSource,
    /// Power state of the main clock
    pub power: PoweredClock,
    /// AHB Clock Divider
    pub ahb_clk_div: Div8,
}

// SOSC

/// The mode of the external reference clock
#[derive(Copy, Clone)]
pub enum SoscMode {
    /// Passive crystal oscillators
    CrystalOscillator,
    /// Active external reference clock
    ActiveClock,
}

/// SOSC/clk_in configuration
#[derive(Copy, Clone)]
pub struct SoscConfig {
    /// Mode of the external reference clock
    pub mode: SoscMode,
    /// Specific frequency of the external reference clock
    pub frequency: u32,
    /// Power state of the external reference clock
    pub power: PoweredClock,
}

// SPLL

/// PLL1/SPLL configuration
pub struct SpllConfig {
    /// Input clock source for the PLL1/SPLL
    pub source: SpllSource,
    /// Mode of operation for the PLL1/SPLL
    pub mode: SpllMode,
    /// Power state of the SPLL
    pub power: PoweredClock,
    /// Is the "pll1_clk_div" clock enabled?
    pub pll1_clk_div: Option<Div8>,
}

/// Input clock source for the PLL1/SPLL
pub enum SpllSource {
    /// External Oscillator (8-50MHz)
    #[cfg(not(feature = "sosc-as-gpio"))]
    Sosc,
    /// Fast Internal Oscillator (45MHz)
    // NOTE: Figure 69 says "firc_45mhz"/"clk_45m", not "fro_hf_gated",
    // so this is is always 45MHz.
    Firc,
    /// S Internal Oscillator (12M)
    Sirc,
    // TODO: the reference manual hints that ROSC is possible,
    // however the minimum input frequency is 32K, but ROSC is 16K.
    // Some diagrams show this option, and some diagrams omit it.
    // SVD shows it as "reserved".
    //
    // /// Realtime Internal Oscillator (16K Osc)
    // Rosc,
}

/// Mode of operation for the SPLL/PLL1
///
/// NOTE: Currently, only "Mode 1" normal operational modes are implemented,
/// as described in the Reference Manual.
#[non_exhaustive]
pub enum SpllMode {
    /// Mode 1a does not use the Pre/Post dividers.
    ///
    /// `Fout = m_mult x SpllSource`
    ///
    /// Both of the following constraints must be met:
    ///
    /// * Fout: 275MHz to 550MHz
    /// * Fout: 4.3MHz to 2x Max CPU Frequency
    Mode1a {
        /// PLL Multiplier. Must be in the range 1..=65535.
        m_mult: u16,
    },

    /// Mode 1b does not use the Pre-divider.
    ///
    /// * `if !bypass_p2_div: Fout = (M / (2 x P)) x Fin`
    /// * `if  bypass_p2_div: Fout = (M /    P   ) x Fin`
    ///
    /// Both of the following constraints must be met:
    ///
    /// * Fcco: 275MHz to 550MHz
    ///   * `Fcco = m_mult x SpllSource`
    /// * Fout: 4.3MHz to 2x Max CPU Frequency
    Mode1b {
        /// PLL Multiplier. `m_mult` must be in the range 1..=65535.
        m_mult: u16,
        /// Post Divider. `p_div` must be in the range 1..=31.
        p_div: u8,
        /// Bonus post divider
        bypass_p2_div: bool,
    },

    /// Mode 1c does use the Pre-divider, but does not use the Post-divider
    ///
    /// `Fout = (M / N) x Fin`
    ///
    /// Both of the following constraints must be met:
    ///
    /// * Fout: 275MHz to 550MHz
    /// * Fout: 4.3MHz to 2x Max CPU Frequency
    Mode1c {
        /// PLL Multiplier. `m_mult` must be in the range 1..=65535.
        m_mult: u16,
        /// Pre Divider. `n_div` must be in the range 1..=255.
        n_div: u8,
    },

    /// Mode 1b uses both the Pre and Post dividers.
    ///
    /// * `if !bypass_p2_div: Fout = (M / (N x 2 x P)) x Fin`
    /// * `if  bypass_p2_div: Fout = (M / (  N x P  )) x Fin`
    ///
    /// Both of the following constraints must be met:
    ///
    /// * Fcco: 275MHz to 550MHz
    ///   * `Fcco = (m_mult x SpllSource) / (n_div x p_div (x 2))`
    /// * Fout: 4.3MHz to 2x Max CPU Frequency
    Mode1d {
        /// PLL Multiplier. `m_mult` must be in the range 1..=65535.
        m_mult: u16,
        /// Pre Divider. `n_div` must be in the range 1..=255.
        n_div: u8,
        /// Post Divider. `p_div` must be in the range 1..=31.
        p_div: u8,
        /// Bonus post divider
        bypass_p2_div: bool,
    },
}

// FIRC/FRO180M

/// ```text
/// ┌───────────┐ fro_hf_root  ┌────┐   fro_hf
/// │ FRO180M   ├───────┬─────▷│GATE│──────────▷
/// │           │       │      ├────┤  clk_45m
/// │           │       └─────▷│GATE│──────────▷
/// └───────────┘              └────┘
/// ```
#[non_exhaustive]
pub struct FircConfig {
    /// Selected clock frequency
    pub frequency: FircFreqSel,
    /// Selected power state of the clock
    pub power: PoweredClock,
    /// Is the "fro_hf" gated clock enabled?
    pub fro_hf_enabled: bool,
    /// Is the "clk_45m" gated clock enabled?
    pub clk_45m_enabled: bool,
    /// Is the "fro_hf_div" clock enabled? Requires `fro_hf`!
    pub fro_hf_div: Option<Div8>,
}

/// Selected FIRC frequency
pub enum FircFreqSel {
    /// 45MHz Output
    Mhz45,
    /// 60MHz Output
    Mhz60,
    /// 90MHz Output
    Mhz90,
    /// 180MHz Output
    Mhz180,
}

// SIRC/FRO12M

/// ```text
/// ┌───────────┐ fro_12m_root  ┌────┐ fro_12m
/// │ FRO12M    │────────┬─────▷│ CG │──────────▷
/// │           │        │      ├────┤  clk_1m
/// │           │        └─────▷│1/12│──────────▷
/// └───────────┘               └────┘
/// ```
#[non_exhaustive]
pub struct SircConfig {
    pub power: PoweredClock,
    // peripheral output, aka sirc_12mhz
    pub fro_12m_enabled: bool,
    /// Is the "fro_lf_div" clock enabled? Requires `fro_12m`!
    pub fro_lf_div: Option<Div8>,
}

#[non_exhaustive]
pub struct Fro16KConfig {
    pub vsys_domain_active: bool,
    pub vdd_core_domain_active: bool,
}

impl Default for Fro16KConfig {
    fn default() -> Self {
        Self {
            vsys_domain_active: true,
            vdd_core_domain_active: true,
        }
    }
}

impl Default for ClocksConfig {
    fn default() -> Self {
        Self {
            vdd_power: VddPowerConfig {
                active_mode: VddModeConfig {
                    level: VddLevel::MidDriveMode,
                    drive: VddDriveStrength::Normal,
                },
                low_power_mode: VddModeConfig {
                    level: VddLevel::MidDriveMode,
                    drive: VddDriveStrength::Normal,
                },
                core_sleep: CoreSleep::WfeUngated,
                flash_sleep: FlashSleep::Never,
            },
            main_clock: MainClockConfig {
                source: MainClockSource::FircHfRoot,
                power: PoweredClock::NormalEnabledDeepSleepDisabled,
                ahb_clk_div: Div8::no_div(),
            },
            firc: Some(FircConfig {
                frequency: FircFreqSel::Mhz45,
                power: PoweredClock::NormalEnabledDeepSleepDisabled,
                fro_hf_enabled: true,
                clk_45m_enabled: true,
                fro_hf_div: None,
            }),
            sirc: SircConfig {
                power: PoweredClock::AlwaysEnabled,
                fro_12m_enabled: true,
                fro_lf_div: None,
            },
            fro16k: Some(Fro16KConfig {
                vsys_domain_active: true,
                vdd_core_domain_active: true,
            }),
            #[cfg(not(feature = "sosc-as-gpio"))]
            sosc: None,
            spll: None,
        }
    }
}
