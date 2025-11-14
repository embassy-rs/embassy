//! Clock control helpers (no magic numbers, PAC field access only).
//! Provides reusable gate abstractions for peripherals used by the examples.
use core::cell::RefCell;

use mcxa_pac::scg0::{
    firccsr::{FircFclkPeriphEn, FircSclkPeriphEn, Fircsten},
    sirccsr::{SircClkPeriphEn, Sircsten},
};
use periph_helpers::SPConfHelper;

use crate::pac;
pub mod periph_helpers;

/// Trait describing an AHB clock gate that can be toggled through MRCC.
pub trait Gate {
    type MrccPeriphConfig: SPConfHelper;

    /// Enable the clock gate.
    unsafe fn enable_clock();

    /// Disable the clock gate.
    unsafe fn disable_clock();

    /// Drive the peripheral into reset.
    unsafe fn assert_reset();

    /// Drive the peripheral out of reset.
    unsafe fn release_reset();

    /// Return whether the clock gate is currently enabled.
    fn is_clock_enabled() -> bool;

    /// .
    fn is_reset_released() -> bool;
}

#[inline]
pub unsafe fn enable_and_reset<G: Gate>(cfg: &G::MrccPeriphConfig) -> Result<u32, ClockError> {
    let freq = enable::<G>(cfg)?;
    pulse_reset::<G>();
    Ok(freq)
}

/// Enable a clock gate for the given peripheral set.
#[inline]
pub unsafe fn enable<G: Gate>(cfg: &G::MrccPeriphConfig) -> Result<u32, ClockError> {
    G::enable_clock();
    while !G::is_clock_enabled() {}
    core::arch::asm!("dsb sy; isb sy", options(nomem, nostack, preserves_flags));

    let freq = critical_section::with(|cs| {
        let clocks = CLOCKS.borrow_ref(cs);
        let clocks = clocks.as_ref().ok_or(ClockError::NeverInitialized)?;
        cfg.post_enable_config(clocks)
    });

    freq.inspect_err(|_e| {
        G::disable_clock();
    })
}

pub unsafe fn disable<G: Gate>() {
    G::disable_clock();
}

/// Check whether a gate is currently enabled.
#[inline]
pub fn is_clock_enabled<G: Gate>() -> bool {
    G::is_clock_enabled()
}

/// Release a reset line for the given peripheral set.
#[inline]
pub unsafe fn release_reset<G: Gate>() {
    G::release_reset();
}

/// Assert a reset line for the given peripheral set.
#[inline]
pub unsafe fn assert_reset<G: Gate>() {
    G::assert_reset();
}

/// Pulse a reset line (assert then release) with a short delay.
#[inline]
pub unsafe fn pulse_reset<G: Gate>() {
    G::assert_reset();
    cortex_m::asm::nop();
    cortex_m::asm::nop();
    G::release_reset();
}

macro_rules! impl_cc_gate {
    ($name:ident, $reg:ident, $field:ident, $config:ty) => {
        impl Gate for crate::peripherals::$name {
            type MrccPeriphConfig = $config;

            #[inline]
            unsafe fn enable_clock() {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$reg().modify(|_, w| w.$field().enabled());
            }

            #[inline]
            unsafe fn disable_clock() {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$reg().modify(|_r, w| w.$field().disabled());
            }

            #[inline]
            fn is_clock_enabled() -> bool {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$reg().read().$field().is_enabled()
            }

            #[inline]
            unsafe fn release_reset() {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$reg().modify(|_, w| w.$field().enabled());
            }

            #[inline]
            unsafe fn assert_reset() {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$reg().modify(|_, w| w.$field().disabled());
            }

            #[inline]
            fn is_reset_released() -> bool {
                let mrcc = unsafe { pac::Mrcc0::steal() };
                mrcc.$reg().read().$field().is_enabled()
            }
        }
    };
}

pub struct UnimplementedConfig;
impl SPConfHelper for UnimplementedConfig {
    fn post_enable_config(&self, _clocks: &Clocks) -> Result<u32, ClockError> {
        Err(ClockError::UnimplementedConfig)
    }
}

pub mod gate {
    use super::{periph_helpers::LpuartConfig, *};

    impl_cc_gate!(PORT1, mrcc_glb_cc1, port1, UnimplementedConfig);
    impl_cc_gate!(PORT2, mrcc_glb_cc1, port2, UnimplementedConfig);
    impl_cc_gate!(PORT3, mrcc_glb_cc1, port3, UnimplementedConfig);
    impl_cc_gate!(OSTIMER0, mrcc_glb_cc1, ostimer0, UnimplementedConfig);
    impl_cc_gate!(LPUART2, mrcc_glb_cc0, lpuart2, LpuartConfig);
    impl_cc_gate!(GPIO3, mrcc_glb_cc2, gpio3, UnimplementedConfig);
    impl_cc_gate!(ADC1, mrcc_glb_cc1, adc1, UnimplementedConfig);
}

// /// Convenience helper enabling the PORT2 and LPUART2 gates required for the debug UART.
// pub unsafe fn enable_uart2_port2(peripherals: &pac::Peripherals) {
//     enable::<gate::Port2>(peripherals);
//     enable::<gate::Lpuart2>(peripherals);
// }

// /// Convenience helper enabling the PORT3 and GPIO3 gates used by the LED in the examples.
// pub unsafe fn enable_led_port(peripherals: &pac::Peripherals) {
//     enable::<gate::Port3>(peripherals);
//     enable::<gate::Gpio3>(peripherals);
// }

// /// Convenience helper enabling the OSTIMER0 clock gate.
// pub unsafe fn enable_ostimer0(peripherals: &pac::Peripherals) {
//     enable::<gate::Ostimer0>(peripherals);
// }

// pub unsafe fn select_uart2_clock(peripherals: &pac::Peripherals) {
//     // Use FRO_LF_DIV (already running) MUX=0 DIV=0
//     let mrcc = &peripherals.mrcc0;
//     mrcc.mrcc_lpuart2_clksel().write(|w| w.mux().clkroot_func_0());
//     mrcc.mrcc_lpuart2_clkdiv().write(|w| unsafe { w.bits(0) });
// }

// pub unsafe fn ensure_frolf_running(peripherals: &pac::Peripherals) {
//     // Ensure FRO_LF divider clock is running (reset default HALT=1 stops it)
//     let sys = &peripherals.syscon;
//     sys.frolfdiv().modify(|_, w| {
//         // DIV defaults to 0; keep it explicit and clear HALT
//         unsafe { w.div().bits(0) }.halt().run()
//     });
// }

// /// Compute the FRO_LF_DIV output frequency currently selected for LPUART2.
// /// Assumes select_uart2_clock() has chosen MUX=0 (FRO_LF_DIV) and DIV is set in SYSCON.FRO_LF_DIV.
// pub unsafe fn uart2_src_hz(peripherals: &pac::Peripherals) -> u32 {
//     // SYSCON.FRO_LF_DIV: DIV field is simple divider: freq_out = 12_000_000 / (DIV+1) for many NXP parts.
//     // On MCXA276 FRO_LF base is 12 MHz; our init keeps DIV=0, so result=12_000_000.
//     // Read it anyway for future generality.
//     let div = peripherals.syscon.frolfdiv().read().div().bits() as u32;
//     let base = 12_000_000u32;
//     base / (div + 1)
// }

// /// Enable clock gate and release reset for OSTIMER0.
// /// Select OSTIMER0 clock source = 1 MHz root (working bring-up configuration).
// pub unsafe fn select_ostimer0_clock_1m(peripherals: &pac::Peripherals) {
//     let mrcc = &peripherals.mrcc0;
//     mrcc.mrcc_ostimer0_clksel().write(|w| w.mux().clkroot_1m());
// }

// pub unsafe fn enable_adc(peripherals: &pac::Peripherals) {
//     enable::<gate::Port1>(peripherals);
//     enable::<gate::Adc1>(peripherals);
// }

// pub unsafe fn select_adc_clock(peripherals: &pac::Peripherals) {
//     // Use FRO_LF_DIV (already running) MUX=0 DIV=0
//     let mrcc = &peripherals.mrcc0;
//     mrcc.mrcc_adc_clksel().write(|w| w.mux().clkroot_func_0());
//     mrcc.mrcc_adc_clkdiv().write(|w| unsafe { w.bits(0) });
// }

// ==============================================

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

#[derive(Debug, Clone)]
pub struct Clock {
    pub frequency: u32,
    pub power: PoweredClock,
}

#[derive(Debug, Clone, Copy)]
pub enum PoweredClock {
    NormalEnabledDeepSleepDisabled,
    AlwaysEnabled,
}

impl PoweredClock {
    /// Does THIS clock meet the power requirements of the OTHER clock?
    pub fn meets_requirement_of(&self, other: &Self) -> bool {
        match (self, other) {
            (PoweredClock::NormalEnabledDeepSleepDisabled, PoweredClock::AlwaysEnabled) => false,
            (PoweredClock::NormalEnabledDeepSleepDisabled, PoweredClock::NormalEnabledDeepSleepDisabled) => true,
            (PoweredClock::AlwaysEnabled, PoweredClock::NormalEnabledDeepSleepDisabled) => true,
            (PoweredClock::AlwaysEnabled, PoweredClock::AlwaysEnabled) => true,
        }
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
    // FIRC, FRO180, 45/60/90/180M clock source
    pub firc: Option<FircConfig>,
    // NOTE: I don't think we *can* disable the SIRC?
    pub sirc: SircConfig,
    pub fro16k: Option<Fro16KConfig>,
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
    pub frequency: FircFreqSel,
    pub power: PoweredClock,
    /// Is the "fro_hf" gated clock enabled?
    pub fro_hf_enabled: bool,
    /// Is the "clk_45m" gated clock enabled?
    pub clk_45m_enabled: bool,
    /// Is the "fro_hf_div" clock enabled? Requires `fro_hf`!
    pub fro_hf_div: Option<Div8>,
}

pub enum FircFreqSel {
    Mhz45,
    Mhz60,
    Mhz90,
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

#[derive(Default, Debug, Clone)]
#[non_exhaustive]
pub struct Clocks {
    pub clk_in: Option<Clock>,

    // FRO180M stuff
    //
    pub fro_hf_root: Option<Clock>,
    pub fro_hf: Option<Clock>,
    pub clk_45m: Option<Clock>,
    pub fro_hf_div: Option<Clock>,
    //
    // End FRO180M

    // FRO12M stuff
    pub fro_12m_root: Option<Clock>,
    pub fro_12m: Option<Clock>,
    pub clk_1m: Option<Clock>,
    pub fro_lf_div: Option<Clock>,
    //
    // End FRO12M stuff

    pub clk_16k_vsys: Option<Clock>,
    pub clk_16k_vdd_core: Option<Clock>,
    pub main_clk: Option<Clock>,
    pub pll1_clk: Option<Clock>,
}

#[non_exhaustive]
pub struct Fro16KConfig {
    pub vsys_domain_active: bool,
    pub vdd_core_domain_active: bool,
}

static CLOCKS: critical_section::Mutex<RefCell<Option<Clocks>>> = critical_section::Mutex::new(RefCell::new(None));

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ClockError {
    NeverInitialized,
    AlreadyInitialized,
    BadConfig { clock: &'static str, reason: &'static str },
    NotImplemented { clock: &'static str },
    UnimplementedConfig,
}

struct ClockOperator<'a> {
    clocks: &'a mut Clocks,
    config: &'a ClocksConfig,

    _mrcc0: pac::Mrcc0,
    scg0: pac::Scg0,
    syscon: pac::Syscon,
    vbat0: pac::Vbat0,
}

impl ClockOperator<'_> {
    fn configure_firc_clocks(&mut self) -> Result<(), ClockError> {
        const HARDCODED_ERR: Result<(), ClockError> = Err(ClockError::BadConfig {
            clock: "firc",
            reason: "For now, FIRC must be enabled and in default state!",
        });

        // Did the user give us a FIRC config?
        let Some(firc) = self.config.firc.as_ref() else {
            return HARDCODED_ERR;
        };
        // Is the FIRC set to 45MHz (should be reset default)
        if !matches!(firc.frequency, FircFreqSel::Mhz45) {
            return HARDCODED_ERR;
        }
        let base_freq = 45_000_000;

        // Is the FIRC as expected?
        let mut firc_ok = true;

        // Is the hardware currently set to the default 45MHz?
        //
        // NOTE: the SVD currently has the wrong(?) values for these:
        // 45 -> 48
        // 60 -> 64
        // 90 -> 96
        // 180 -> 192
        // Probably correct-ish, but for a different trim value?
        firc_ok &= self.scg0.firccfg().read().freq_sel().is_firc_48mhz_192s();

        // Check some values in the CSR
        let csr = self.scg0.firccsr().read();
        // Is it enabled?
        firc_ok &= csr.fircen().is_enabled();
        // Is it accurate?
        firc_ok &= csr.fircacc().is_enabled_and_valid();
        // Is there no error?
        firc_ok &= csr.fircerr().is_error_not_detected();
        // Is the FIRC the system clock?
        firc_ok &= csr.fircsel().is_firc();
        // Is it valid?
        firc_ok &= csr.fircvld().is_enabled_and_valid();

        // Are we happy with the current (hardcoded) state?
        if !firc_ok {
            return HARDCODED_ERR;
        }

        // Note that the fro_hf_root is active
        self.clocks.fro_hf_root = Some(Clock {
            frequency: base_freq,
            power: firc.power,
        });

        // Okay! Now we're past that, let's enable all the downstream clocks.
        let FircConfig {
            frequency: _,
            power,
            fro_hf_enabled,
            clk_45m_enabled,
            fro_hf_div,
        } = firc;

        // When is the FRO enabled?
        let pow_set = match power {
            PoweredClock::NormalEnabledDeepSleepDisabled => Fircsten::DisabledInStopModes,
            PoweredClock::AlwaysEnabled => Fircsten::EnabledInStopModes,
        };

        // Do we enable the `fro_hf` output?
        let fro_hf_set = if *fro_hf_enabled {
            self.clocks.fro_hf = Some(Clock {
                frequency: base_freq,
                power: *power,
            });
            FircFclkPeriphEn::Enabled
        } else {
            FircFclkPeriphEn::Disabled
        };

        // Do we enable the `clk_45m` output?
        let clk_45m_set = if *clk_45m_enabled {
            self.clocks.clk_45m = Some(Clock {
                frequency: 45_000_000,
                power: *power,
            });
            FircSclkPeriphEn::Enabled
        } else {
            FircSclkPeriphEn::Disabled
        };

        self.scg0.firccsr().modify(|_r, w| {
            w.fircsten().variant(pow_set);
            w.firc_fclk_periph_en().variant(fro_hf_set);
            w.firc_sclk_periph_en().variant(clk_45m_set);
            w
        });

        // Do we enable the `fro_hf_div` output?
        if let Some(d) = fro_hf_div.as_ref() {
            // We need `fro_hf` to be enabled
            if !*fro_hf_enabled {
                return Err(ClockError::BadConfig {
                    clock: "fro_hf_div",
                    reason: "fro_hf not enabled",
                });
            }

            // Halt and reset the div
            self.syscon.frohfdiv().write(|w| {
                w.halt().halt();
                w.reset().asserted();
                w
            });
            // Then change the div, unhalt it, and reset it
            self.syscon.frohfdiv().write(|w| {
                unsafe {
                    w.div().bits(d.into_bits());
                }
                w.halt().run();
                w.reset().released();
                w
            });

            // Wait for clock to stabilize
            while self.syscon.frohfdiv().read().unstab().is_ongoing() {}

            // Store off the clock info
            self.clocks.fro_hf_div = Some(Clock {
                frequency: base_freq / d.into_divisor(),
                power: *power,
            });
        }

        Ok(())
    }

    fn configure_sirc_clocks(&mut self) -> Result<(), ClockError> {
        let SircConfig {
            power,
            fro_12m_enabled,
            fro_lf_div,
        } = &self.config.sirc;
        let base_freq = 12_000_000;

        // Allow writes
        self.scg0.sirccsr().modify(|_r, w| w.lk().write_enabled());
        self.clocks.fro_12m_root = Some(Clock {
            frequency: base_freq,
            power: *power,
        });

        let deep = match power {
            PoweredClock::NormalEnabledDeepSleepDisabled => Sircsten::Disabled,
            PoweredClock::AlwaysEnabled => Sircsten::Enabled,
        };
        let pclk = if *fro_12m_enabled {
            self.clocks.fro_12m = Some(Clock {
                frequency: base_freq,
                power: *power,
            });
            self.clocks.clk_1m = Some(Clock {
                frequency: base_freq / 12,
                power: *power,
            });
            SircClkPeriphEn::Enabled
        } else {
            SircClkPeriphEn::Disabled
        };

        // Set sleep/peripheral usage
        self.scg0.sirccsr().modify(|_r, w| {
            w.sircsten().variant(deep);
            w.sirc_clk_periph_en().variant(pclk);
            w
        });

        while self.scg0.sirccsr().read().sircvld().is_disabled_or_not_valid() {}
        if self.scg0.sirccsr().read().sircerr().is_error_detected() {
            return Err(ClockError::BadConfig {
                clock: "sirc",
                reason: "error set",
            });
        }

        // reset lock
        self.scg0.sirccsr().modify(|_r, w| w.lk().write_disabled());

        // Do we enable the `fro_lf_div` output?
        if let Some(d) = fro_lf_div.as_ref() {
            // We need `fro_lf` to be enabled
            if !*fro_12m_enabled {
                return Err(ClockError::BadConfig {
                    clock: "fro_lf_div",
                    reason: "fro_12m not enabled",
                });
            }

            // Halt and reset the div
            self.syscon.frolfdiv().write(|w| {
                w.halt().halt();
                w.reset().asserted();
                w
            });
            // Then change the div, unhalt it, and reset it
            self.syscon.frolfdiv().write(|w| {
                unsafe {
                    w.div().bits(d.into_bits());
                }
                w.halt().run();
                w.reset().released();
                w
            });

            // Wait for clock to stabilize
            while self.syscon.frolfdiv().read().unstab().is_ongoing() {}

            // Store off the clock info
            self.clocks.fro_lf_div = Some(Clock {
                frequency: base_freq / d.into_divisor(),
                power: *power,
            });
        }

        Ok(())
    }

    fn configure_fro16k_clocks(&mut self) -> Result<(), ClockError> {
        let Some(fro16k) = self.config.fro16k.as_ref() else {
            return Ok(());
        };
        // Enable FRO16K oscillator
        self.vbat0.froctla().modify(|_, w| w.fro_en().set_bit());

        // Lock the control register
        self.vbat0.frolcka().modify(|_, w| w.lock().set_bit());

        let Fro16KConfig { vsys_domain_active, vdd_core_domain_active } = fro16k;

        // Enable clock outputs to both VSYS and VDD_CORE domains
        // Bit 0: clk_16k0 to VSYS domain
        // Bit 1: clk_16k1 to VDD_CORE domain
        //
        // TODO: Define sub-fields for this register with a PAC patch?
        let mut bits = 0;
        if *vsys_domain_active {
            bits |= 0b01;
            self.clocks.clk_16k_vsys = Some(Clock {
                frequency: 16_384,
                power: PoweredClock::AlwaysEnabled,
            });
        }
        if *vdd_core_domain_active {
            bits |= 0b10;
            self.clocks.clk_16k_vdd_core = Some(Clock {
                frequency: 16_384,
                power: PoweredClock::AlwaysEnabled,
            });
        }
        self.vbat0.froclke().modify(|_r, w| {
            unsafe { w.clke().bits(bits) }
        });

        Ok(())
    }
}

pub fn init(settings: ClocksConfig) -> Result<(), ClockError> {
    critical_section::with(|cs| {
        if CLOCKS.borrow_ref(cs).is_some() {
            Err(ClockError::AlreadyInitialized)
        } else {
            Ok(())
        }
    })?;

    let mut clocks = Clocks::default();
    let mut operator = ClockOperator {
        clocks: &mut clocks,
        config: &settings,

        _mrcc0: unsafe { pac::Mrcc0::steal() },
        scg0: unsafe { pac::Scg0::steal() },
        syscon: unsafe { pac::Syscon::steal() },
        vbat0: unsafe { pac::Vbat0::steal() },
    };

    operator.configure_firc_clocks()?;
    operator.configure_sirc_clocks()?;
    operator.configure_fro16k_clocks()?;
    // TODO, everything downstream

    critical_section::with(|cs| {
        let mut clks = CLOCKS.borrow_ref_mut(cs);
        assert!(clks.is_none(), "Clock setup race!");
        *clks = Some(clocks);
    });

    Ok(())
}

/// Obtain the full clocks structure, calling the given closure in a critical section
///
/// NOTE: Clocks implements `Clone`,
pub fn with_clocks<R: 'static, F: FnOnce(&Clocks) -> R>(f: F) -> Option<R> {
    critical_section::with(|cs| {
        let c = CLOCKS.borrow_ref(cs);
        let c = c.as_ref()?;
        Some(f(c))
    })
}

impl Clocks {
    pub fn ensure_fro_lf_div_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        let Some(clk) = self.fro_lf_div.as_ref() else {
            return Err(ClockError::BadConfig {
                clock: "fro_lf_div",
                reason: "required but not active",
            });
        };
        if !clk.power.meets_requirement_of(at_level) {
            return Err(ClockError::BadConfig {
                clock: "fro_lf_div",
                reason: "not low power active",
            });
        }
        Ok(clk.frequency)
    }

    pub fn ensure_fro_hf_div_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        let Some(clk) = self.fro_hf_div.as_ref() else {
            return Err(ClockError::BadConfig {
                clock: "fro_hf_div",
                reason: "required but not active",
            });
        };
        if !clk.power.meets_requirement_of(at_level) {
            return Err(ClockError::BadConfig {
                clock: "fro_hf_div",
                reason: "not low power active",
            });
        }
        Ok(clk.frequency)
    }

    pub fn ensure_clk_in_active(&self, _at_level: &PoweredClock) -> Result<u32, ClockError> {
        Err(ClockError::NotImplemented { clock: "clk_in" })
    }

    pub fn ensure_clk_16k_active(&self, _at_level: &PoweredClock) -> Result<u32, ClockError> {
        Err(ClockError::NotImplemented { clock: "clk_16k" })
    }

    pub fn ensure_clk_1m_active(&self, at_level: &PoweredClock) -> Result<u32, ClockError> {
        let Some(clk) = self.clk_1m.as_ref() else {
            return Err(ClockError::BadConfig {
                clock: "clk_1m",
                reason: "required but not active",
            });
        };
        if !clk.power.meets_requirement_of(at_level) {
            return Err(ClockError::BadConfig {
                clock: "clk_1m",
                reason: "not low power active",
            });
        }
        Ok(clk.frequency)
    }

    pub fn ensure_pll1_clk_div_active(&self, _at_level: &PoweredClock) -> Result<u32, ClockError> {
        Err(ClockError::NotImplemented { clock: "pll1_clk_div" })
    }
}
