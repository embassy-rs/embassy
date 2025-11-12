//! Clock control helpers (no magic numbers, PAC field access only).
//! Provides reusable gate abstractions for peripherals used by the examples.
use mcxa_pac::scg0::{
    firccsr::{FircFclkPeriphEn, FircSclkPeriphEn, Fircsten},
    sirccsr::{SircClkPeriphEn, Sircsten},
};

use crate::pac;

/// Trait describing an AHB clock gate that can be toggled through MRCC.
pub trait Gate {
    /// Enable the clock gate.
    unsafe fn enable(mrcc: &pac::mrcc0::RegisterBlock);

    /// Return whether the clock gate is currently enabled.
    fn is_enabled(mrcc: &pac::mrcc0::RegisterBlock) -> bool;
}

/// Enable a clock gate for the given peripheral set.
#[inline]
pub unsafe fn enable<G: Gate>(peripherals: &pac::Peripherals) {
    let mrcc = &peripherals.mrcc0;
    G::enable(mrcc);
    while !G::is_enabled(mrcc) {}
    core::arch::asm!("dsb sy; isb sy", options(nomem, nostack, preserves_flags));
}

/// Check whether a gate is currently enabled.
#[inline]
pub fn is_enabled<G: Gate>(peripherals: &pac::Peripherals) -> bool {
    G::is_enabled(&peripherals.mrcc0)
}

macro_rules! impl_cc_gate {
    ($name:ident, $reg:ident, $field:ident) => {
        pub struct $name;

        impl Gate for $name {
            #[inline]
            unsafe fn enable(mrcc: &pac::mrcc0::RegisterBlock) {
                mrcc.$reg().modify(|_, w| w.$field().enabled());
            }

            #[inline]
            fn is_enabled(mrcc: &pac::mrcc0::RegisterBlock) -> bool {
                mrcc.$reg().read().$field().is_enabled()
            }
        }
    };
}

pub mod gate {
    use super::*;

    impl_cc_gate!(Port2, mrcc_glb_cc1, port2);
    impl_cc_gate!(Port3, mrcc_glb_cc1, port3);
    impl_cc_gate!(Ostimer0, mrcc_glb_cc1, ostimer0);
    impl_cc_gate!(Lpuart2, mrcc_glb_cc0, lpuart2);
    impl_cc_gate!(Gpio3, mrcc_glb_cc2, gpio3);
    impl_cc_gate!(Port1, mrcc_glb_cc1, port1);
    impl_cc_gate!(Adc1, mrcc_glb_cc1, adc1);
}

/// Convenience helper enabling the PORT2 and LPUART2 gates required for the debug UART.
pub unsafe fn enable_uart2_port2(peripherals: &pac::Peripherals) {
    enable::<gate::Port2>(peripherals);
    enable::<gate::Lpuart2>(peripherals);
}

/// Convenience helper enabling the PORT3 and GPIO3 gates used by the LED in the examples.
pub unsafe fn enable_led_port(peripherals: &pac::Peripherals) {
    enable::<gate::Port3>(peripherals);
    enable::<gate::Gpio3>(peripherals);
}

/// Convenience helper enabling the OSTIMER0 clock gate.
pub unsafe fn enable_ostimer0(peripherals: &pac::Peripherals) {
    enable::<gate::Ostimer0>(peripherals);
}

pub unsafe fn select_uart2_clock(peripherals: &pac::Peripherals) {
    // Use FRO_LF_DIV (already running) MUX=0 DIV=0
    let mrcc = &peripherals.mrcc0;
    mrcc.mrcc_lpuart2_clksel().write(|w| w.mux().clkroot_func_0());
    mrcc.mrcc_lpuart2_clkdiv().write(|w| unsafe { w.bits(0) });
}

pub unsafe fn ensure_frolf_running(peripherals: &pac::Peripherals) {
    // Ensure FRO_LF divider clock is running (reset default HALT=1 stops it)
    let sys = &peripherals.syscon;
    sys.frolfdiv().modify(|_, w| {
        // DIV defaults to 0; keep it explicit and clear HALT
        unsafe { w.div().bits(0) }.halt().run()
    });
}

/// Compute the FRO_LF_DIV output frequency currently selected for LPUART2.
/// Assumes select_uart2_clock() has chosen MUX=0 (FRO_LF_DIV) and DIV is set in SYSCON.FRO_LF_DIV.
pub unsafe fn uart2_src_hz(peripherals: &pac::Peripherals) -> u32 {
    // SYSCON.FRO_LF_DIV: DIV field is simple divider: freq_out = 12_000_000 / (DIV+1) for many NXP parts.
    // On MCXA276 FRO_LF base is 12 MHz; our init keeps DIV=0, so result=12_000_000.
    // Read it anyway for future generality.
    let div = peripherals.syscon.frolfdiv().read().div().bits() as u32;
    let base = 12_000_000u32;
    base / (div + 1)
}

/// Enable clock gate and release reset for OSTIMER0.
/// Select OSTIMER0 clock source = 1 MHz root (working bring-up configuration).
pub unsafe fn select_ostimer0_clock_1m(peripherals: &pac::Peripherals) {
    let mrcc = &peripherals.mrcc0;
    mrcc.mrcc_ostimer0_clksel().write(|w| w.mux().clkroot_1m());
}

pub unsafe fn init_fro16k(peripherals: &pac::Peripherals) {
    let vbat = &peripherals.vbat0;
    // Enable FRO16K oscillator
    vbat.froctla().modify(|_, w| w.fro_en().set_bit());

    // Lock the control register
    vbat.frolcka().modify(|_, w| w.lock().set_bit());

    // Enable clock outputs to both VSYS and VDD_CORE domains
    // Bit 0: clk_16k0 to VSYS domain
    // Bit 1: clk_16k1 to VDD_CORE domain
    vbat.froclke().modify(|_, w| unsafe { w.clke().bits(0x3) });
}

pub unsafe fn enable_adc(peripherals: &pac::Peripherals) {
    enable::<gate::Port1>(peripherals);
    enable::<gate::Adc1>(peripherals);
}

pub unsafe fn select_adc_clock(peripherals: &pac::Peripherals) {
    // Use FRO_LF_DIV (already running) MUX=0 DIV=0
    let mrcc = &peripherals.mrcc0;
    mrcc.mrcc_adc_clksel().write(|w| w.mux().clkroot_func_0());
    mrcc.mrcc_adc_clkdiv().write(|w| unsafe { w.bits(0) });
}

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
    HighPowerOnly,
    AlwaysEnabled,
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
    pub main_clk: Option<Clock>,
    pub pll1_clk: Option<Clock>,
}

static CLOCKS: critical_section::Mutex<Option<Clocks>> = critical_section::Mutex::new(None);

#[non_exhaustive]
pub enum ClockError {
    AlreadyInitialized,
    BadConfig { clock: &'static str, reason: &'static str },
}

struct ClockOperator<'a> {
    clocks: &'a mut Clocks,
    config: &'a ClocksConfig,

    _mrcc0: pac::Mrcc0,
    scg0: pac::Scg0,
    syscon: pac::Syscon,
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
            PoweredClock::HighPowerOnly => Fircsten::DisabledInStopModes,
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
            PoweredClock::HighPowerOnly => Sircsten::Disabled,
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

        todo!()
    }
}

pub fn init(settings: ClocksConfig) -> Result<(), ClockError> {
    critical_section::with(|cs| {
        if CLOCKS.borrow(cs).is_some() {
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
    };

    operator.configure_firc_clocks()?;
    operator.configure_sirc_clocks()?;
    // TODO, everything downstream

    Ok(())
}

/// Obtain the full clocks structure, calling the given closure in a critical section
///
/// NOTE: Clocks implements `Clone`,
pub fn with_clocks<R: 'static, F: FnOnce(&Clocks) -> R>(f: F) -> Option<R> {
    critical_section::with(|cs| {
        let c = CLOCKS.borrow(cs).as_ref()?;
        Some(f(c))
    })
}
