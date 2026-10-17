//! Minimal MCXA1xx clock support for the development board bring-up.
//!
//! MCXA156 has distinct MRCC/SCG register layouts and clock selectors, so it
//! does not use the MCXA2xx clock operator or its voltage/flash timing sequence. This
//! module keeps the reset 12 MHz system clock and only configures the exact
//! peripheral selectors needed by OSTIMER0 and LPUART0.

use core::cell::RefCell;
use core::sync::atomic::AtomicUsize;

use critical_section::CriticalSection;

#[path = "gate.rs"]
mod gate;
#[path = "types.rs"]
mod types;

pub(crate) use gate::default_register_value;
pub use gate::{Gate, assert_reset, disable, enable, enable_and_reset, is_reset_released, pulse_reset, release_reset};
pub use types::{Clock, ClockError, Clocks, PoweredClock, WakeGuard};

pub mod config {
    /// MCXA1xx bootstrap clock configuration.
    ///
    /// The development slice intentionally keeps the documented 12 MHz reset
    /// clock. Higher-frequency operating points are added only after their
    /// voltage and flash timing sequences are reconciled.
    #[derive(Default)]
    pub struct ClocksConfig;

    #[derive(Copy, Clone, PartialEq, Debug, Default)]
    #[non_exhaustive]
    pub enum VddLevel {
        #[default]
        MidDriveMode,
        OverDriveMode,
    }

    #[derive(Copy, Clone, Default, Debug)]
    #[non_exhaustive]
    pub enum CoreSleep {
        #[default]
        WfeUngated,
        WfeGated,
        DeepSleep,
    }
}

pub mod periph_helpers {
    use super::{ClockError, Clocks, PoweredClock, WakeGuard};
    use crate::pac::mrcc::{
        MrccLpuart0ClkdivHalt as ClkdivHalt, MrccLpuart0ClkdivReset as ClkdivReset,
        MrccLpuart0ClkdivUnstab as ClkdivUnstab, MrccLpuart0ClkselMux as LpuartClkselMux,
        MrccOstimer0ClkselMux as OstimerClkselMux,
    };

    #[must_use]
    pub struct PreEnableParts {
        pub freq: u32,
        pub wake_guard: Option<WakeGuard>,
    }

    impl PreEnableParts {
        pub fn empty() -> Self {
            Self {
                freq: 0,
                wake_guard: None,
            }
        }
    }

    pub trait SPConfHelper {
        fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError>;
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Div4(u8);

    impl Div4 {
        pub const fn no_div() -> Self {
            Self(0)
        }

        pub const fn from_raw(n: u8) -> Option<Self> {
            if n > 0b1111 { None } else { Some(Self(n)) }
        }

        pub const fn from_divisor(n: u8) -> Option<Self> {
            let Some(n) = n.checked_sub(1) else {
                return None;
            };
            Self::from_raw(n)
        }

        pub const fn into_bits(self) -> u8 {
            self.0
        }

        pub const fn into_divisor(self) -> u32 {
            self.0 as u32 + 1
        }
    }

    pub struct NoConfig;

    impl SPConfHelper for NoConfig {
        fn pre_enable_config(&self, _clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
            Ok(PreEnableParts::empty())
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum OstimerClockSel {
        Clk1M,
    }

    pub struct OsTimerConfig {
        pub power: PoweredClock,
        pub source: OstimerClockSel,
    }

    impl SPConfHelper for OsTimerConfig {
        fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
            let freq = match self.source {
                OstimerClockSel::Clk1M => clocks.ensure_clk_1m_active(&self.power)?,
            };
            crate::pac::MRCC0
                .mrcc_ostimer0_clksel()
                .write(|w| w.set_mux(OstimerClkselMux::Clkroot1m));
            Ok(PreEnableParts {
                freq,
                wake_guard: WakeGuard::for_power(&self.power),
            })
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum LpuartClockSel {
        FroLfDiv,
        FroHfDiv,
        #[cfg(not(feature = "sosc-as-gpio"))]
        ClkIn,
        Clk16K,
        Clk1M,
        Pll1ClkDiv,
        None,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum LpuartInstance {
        Lpuart0,
    }

    pub struct LpuartConfig {
        pub power: PoweredClock,
        pub source: LpuartClockSel,
        pub div: Div4,
        pub(crate) instance: LpuartInstance,
    }

    impl SPConfHelper for LpuartConfig {
        fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
            match self.instance {
                LpuartInstance::Lpuart0 => {}
            }

            let mrcc0 = crate::pac::MRCC0;
            let clksel = mrcc0.mrcc_lpuart0_clksel();
            let clkdiv = mrcc0.mrcc_lpuart0_clkdiv();

            let (freq, mux) = match self.source {
                LpuartClockSel::FroLfDiv => (
                    clocks.ensure_fro_lf_div_active(&self.power)?,
                    LpuartClkselMux::ClkrootFunc0,
                ),
                LpuartClockSel::Clk16K => (
                    clocks.ensure_clk_16k_vdd_core_active(&self.power)?,
                    LpuartClkselMux::ClkrootFunc4,
                ),
                LpuartClockSel::Clk1M => (clocks.ensure_clk_1m_active(&self.power)?, LpuartClkselMux::ClkrootFunc5),
                LpuartClockSel::None => {
                    clksel.write(|w| w.set_mux(LpuartClkselMux::_RESERVED_7));
                    clkdiv.modify(|w| {
                        w.set_reset(ClkdivReset::Off);
                        w.set_halt(ClkdivHalt::Off);
                    });
                    return Ok(PreEnableParts::empty());
                }
                LpuartClockSel::FroHfDiv => {
                    return Err(ClockError::NotImplemented { clock: "fro_hf_div" });
                }
                #[cfg(not(feature = "sosc-as-gpio"))]
                LpuartClockSel::ClkIn => return Err(ClockError::NotImplemented { clock: "clk_in" }),
                LpuartClockSel::Pll1ClkDiv => {
                    return Err(ClockError::NotImplemented { clock: "pll1_clk_div" });
                }
            };

            clksel.write(|w| w.set_mux(mux));
            clkdiv.modify(|w| {
                w.set_div(self.div.into_bits());
                w.set_halt(ClkdivHalt::Off);
                w.set_reset(ClkdivReset::Off);
            });
            clkdiv.modify(|w| {
                w.set_halt(ClkdivHalt::On);
                w.set_reset(ClkdivReset::On);
            });
            while clkdiv.read().unstab() == ClkdivUnstab::Off {}

            Ok(PreEnableParts {
                freq: freq / self.div.into_divisor(),
                wake_guard: WakeGuard::for_power(&self.power),
            })
        }
    }
}

pub(super) static CLOCKS: critical_section::Mutex<RefCell<Option<Clocks>>> =
    critical_section::Mutex::new(RefCell::new(None));
pub(super) static LIVE_HP_TOKENS: AtomicUsize = AtomicUsize::new(0);

/// Initialize the clocks used by the MCXA156 development slice.
pub fn init(_settings: config::ClocksConfig) -> Result<(), ClockError> {
    critical_section::with(|cs| {
        if CLOCKS.borrow_ref(cs).is_some() {
            Err(ClockError::AlreadyInitialized)
        } else {
            Ok(())
        }
    })?;

    use crate::pac::scg::{SirccsrLk, Sircvld};

    let scg0 = crate::pac::SCG0;
    scg0.sirccsr().modify(|w| w.set_lk(SirccsrLk::WriteEnabled));
    scg0.sirccsr().modify(|w| w.set_sirc_clk_periph_en(true));
    scg0.sirccsr().modify(|w| w.set_lk(SirccsrLk::WriteDisabled));
    while scg0.sirccsr().read().sircvld() != Sircvld::EnabledAndValid {}

    let active = PoweredClock::NormalEnabledDeepSleepDisabled;
    let always = PoweredClock::AlwaysEnabled;
    let fro12m = Clock {
        frequency: 12_000_000,
        power: active,
    };
    let mut clocks = Clocks::default();
    clocks.fro_12m_root = Some(fro12m.clone());
    clocks.fro_12m = Some(fro12m.clone());
    clocks.fro_lf_div = Some(fro12m.clone());
    clocks.clk_1m = Some(Clock {
        frequency: 1_000_000,
        power: always,
    });
    clocks.main_clk = Some(fro12m.clone());
    clocks.cpu_system_clk = Some(fro12m);

    critical_section::with(|cs| {
        *CLOCKS.borrow_ref_mut(cs) = Some(clocks);
    });
    Ok(())
}

pub fn with_clocks<R: 'static, F: FnOnce(&Clocks) -> R>(f: F) -> Option<R> {
    critical_section::with(|cs| {
        let clocks = CLOCKS.borrow_ref(cs);
        Some(f(clocks.as_ref()?))
    })
}

pub fn active_wake_guards(_cs: &CriticalSection) -> bool {
    use core::sync::atomic::Ordering;
    LIVE_HP_TOKENS.load(Ordering::Relaxed) != 0
}
