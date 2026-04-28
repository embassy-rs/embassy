//! CLKOUT pseudo-peripheral
//!
//! CLKOUT is a part of the clock generation subsystem, and can be used
//! either to generate arbitrary waveforms, or to debug the state of
//! internal oscillators.

use core::marker::PhantomData;

use embassy_hal_internal::Peri;

use crate::clocks::config::VddLevel;
pub use crate::clocks::periph_helpers::Div4;
use crate::clocks::{ClockError, PoweredClock, WakeGuard, with_clocks};
use crate::gpio::{AnyPin, SealedPin};
use crate::pac::mrcc::{ClkdivHalt, ClkdivReset, ClkdivUnstab, ClkoutClkselMux as Mux};
use crate::peripherals::CLKOUT;

/// A peripheral representing the CLKOUT pseudo-peripheral
pub struct ClockOut<'a> {
    _p: PhantomData<&'a mut CLKOUT>,
    pin: Peri<'a, AnyPin>,
    freq: u32,
    _wg: Option<WakeGuard>,
}

/// Selected clock source to output
#[derive(Copy, Clone)]
pub enum ClockOutSel {
    /// 12MHz Internal Oscillator
    Fro12M,
    /// FRO180M/FRO192M Internal Oscillator, via divisor
    FroHfDiv,
    /// External Oscillator
    #[cfg(not(feature = "sosc-as-gpio"))]
    ClkIn,
    /// 16KHz oscillator
    #[cfg(feature = "mcxa2xx")]
    Clk16K,
    /// Either the 16K or 32K oscillator, depending on settings
    #[cfg(feature = "mcxa5xx")]
    LpOsc,
    /// Output of PLL1
    #[cfg(feature = "mcxa2xx")]
    Pll1Clk,
    /// Output of divided PLL1
    #[cfg(feature = "mcxa5xx")]
    Pll1ClkDiv,
    /// Main System CPU clock, divided by 6
    SlowClk,
}

/// Configuration for the ClockOut
#[derive(Copy, Clone)]
pub struct Config {
    /// Selected Source Clock
    pub sel: ClockOutSel,
    /// Selected division level
    pub div: Div4,
    /// Selected power level
    pub level: PoweredClock,
}

impl<'a> ClockOut<'a> {
    /// Create a new ClockOut pin. On success, the clock signal will begin immediately
    /// on the given pin.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new(
        _peri: Peri<'a, CLKOUT>,
        pin: Peri<'a, impl sealed::ClockOutPin>,
        cfg: Config,
    ) -> Result<Self, ClockError> {
        // There's no MRCC enable bit, so we check the validity of the clocks here
        let (freq, mux, _wg) = check_sel(cfg.sel, cfg.level, cfg.div.into_divisor())?;

        // All good! Apply requested config, starting with the pin.
        pin.mux();

        setup_clkout(mux, cfg.div);

        Ok(Self {
            _p: PhantomData,
            pin: pin.into(),
            freq: freq / cfg.div.into_divisor(),
            _wg,
        })
    }

    /// Unsafe constructor that ignores PoweredClock checks and discards WakeGuards
    ///
    /// Only intended for debugging low power clock gating, to ensure that clocks start/stop
    /// appropriately.
    ///
    /// ## SAFETY
    ///
    /// The caller must not rely on the clock running for correctness if the provided
    /// clock will be gated in deep sleep mode.
    pub unsafe fn new_unchecked(
        _peri: Peri<'a, CLKOUT>,
        pin: Peri<'a, impl sealed::ClockOutPin>,
        mut cfg: Config,
    ) -> Result<Self, ClockError> {
        // Ignore the users clock selection so it Just Works
        cfg.level = PoweredClock::NormalEnabledDeepSleepDisabled;

        // There's no MRCC enable bit, so we check the validity of the clocks here
        let (freq, mux, _wg) = check_sel(cfg.sel, cfg.level, cfg.div.into_divisor())?;

        // All good! Apply requested config, starting with the pin.
        pin.mux();

        setup_clkout(mux, cfg.div);

        Ok(Self {
            _p: PhantomData,
            pin: pin.into(),
            freq: freq / cfg.div.into_divisor(),
            // No wake guards here!
            _wg: None,
        })
    }

    /// Frequency of the clkout pin
    #[inline]
    pub fn frequency(&self) -> u32 {
        self.freq
    }
}

impl Drop for ClockOut<'_> {
    fn drop(&mut self) {
        disable_clkout();
        self.pin.set_as_disabled();
    }
}

/// Check whether the given clock selection is valid
fn check_sel(sel: ClockOutSel, level: PoweredClock, divisor: u32) -> Result<(u32, Mux, Option<WakeGuard>), ClockError> {
    let res = with_clocks(|c| {
        #[cfg(feature = "mcxa2xx")]
        let (freq, mux, fmax, expected) = {
            let (freq, mux) = match sel {
                ClockOutSel::Fro12M => (c.ensure_fro_hf_active(&level)?, Mux::Clkroot12m),
                ClockOutSel::FroHfDiv => (c.ensure_fro_hf_div_active(&level)?, Mux::ClkrootFircDiv),
                #[cfg(not(feature = "sosc-as-gpio"))]
                ClockOutSel::ClkIn => (c.ensure_clk_in_active(&level)?, Mux::ClkrootSosc),
                ClockOutSel::Clk16K => (c.ensure_clk_16k_vdd_core_active(&level)?, Mux::Clkroot16k),
                ClockOutSel::Pll1Clk => (c.ensure_pll1_clk_active(&level)?, Mux::ClkrootSpll),
                ClockOutSel::SlowClk => (c.ensure_slow_clk_active(&level)?, Mux::ClkrootSlow),
            };
            let expected = freq / divisor;
            let fmax = match c.active_power {
                VddLevel::MidDriveMode => 45_000_000,
                VddLevel::OverDriveMode => 90_000_000,
            };
            (freq, mux, fmax, expected)
        };
        #[cfg(feature = "mcxa5xx")]
        let (freq, mux, fmax, expected) = {
            let (freq, mux) = match sel {
                ClockOutSel::Fro12M => (c.ensure_fro_hf_active(&level)?, Mux::I0Clkroot12m),
                ClockOutSel::FroHfDiv => (c.ensure_fro_hf_div_active(&level)?, Mux::I1ClkrootFircDiv),
                #[cfg(not(feature = "sosc-as-gpio"))]
                ClockOutSel::ClkIn => (c.ensure_clk_in_active(&level)?, Mux::I2ClkrootSosc),
                // TODO: we need this to be an lp_osc clock
                ClockOutSel::LpOsc => (c.ensure_clk_32k_vdd_core_active(&level)?, Mux::I3ClkrootLposc),
                ClockOutSel::Pll1ClkDiv => (c.ensure_pll1_clk_div_active(&level)?, Mux::I5ClkrootSpllDiv),
                ClockOutSel::SlowClk => (c.ensure_slow_clk_active(&level)?, Mux::I6ClkrootSlow),
            };
            let expected = freq / divisor;
            let fmax = match c.active_power {
                VddLevel::MidDriveMode => 48_000_000,
                VddLevel::NormalMode => 100_000_000,
                VddLevel::OverDriveMode => 100_000_000,
            };
            (freq, mux, fmax, expected)
        };

        if expected > fmax {
            Err(ClockError::BadConfig {
                clock: "clkout fclk",
                reason: "exceeds fclk max",
            })
        } else {
            let wg = WakeGuard::for_power(&level);
            Ok((freq, mux, wg))
        }
    });
    let Some(res) = res else {
        return Err(ClockError::NeverInitialized);
    };
    res
}

/// Set up the clkout pin using the given mux and div settings
fn setup_clkout(mux: Mux, div: Div4) {
    let mrcc = crate::pac::MRCC0;

    mrcc.mrcc_clkout_clksel().write(|w| w.set_mux(mux));

    // Set up clkdiv
    mrcc.mrcc_clkout_clkdiv().write(|w| {
        w.set_halt(ClkdivHalt::Off);
        w.set_reset(ClkdivReset::Off);
        w.set_div(div.into_bits());
    });
    mrcc.mrcc_clkout_clkdiv().write(|w| {
        w.set_halt(ClkdivHalt::On);
        w.set_reset(ClkdivReset::On);
        w.set_div(div.into_bits());
    });

    while mrcc.mrcc_clkout_clkdiv().read().unstab() == ClkdivUnstab::On {}
}

/// Stop the
fn disable_clkout() {
    // Stop the output by selecting the "none" clock
    //
    // TODO: restore the pin to hi-z or something?
    let mrcc = crate::pac::MRCC0;
    mrcc.mrcc_clkout_clkdiv().write(|w| {
        w.set_reset(ClkdivReset::Off);
        w.set_halt(ClkdivHalt::Off);
        w.set_div(0);
    });
    mrcc.mrcc_clkout_clksel().write(|w| w.0 = 0b111);
}

pub(crate) mod sealed {
    use embassy_hal_internal::PeripheralType;

    use crate::gpio::GpioPin;

    /// Sealed marker trait for clockout pins
    pub trait ClockOutPin: GpioPin + PeripheralType {
        /// Set the given pin to the correct muxing state
        fn mux(&self);
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! impl_clkout_pin {
        ($pin:ident, $func:ident) => {
            impl crate::clkout::sealed::ClockOutPin for crate::peripherals::$pin {
                fn mux(&self) {
                    use crate::gpio::SealedPin;

                    self.set_function(crate::pac::port::Mux::$func);
                    self.set_pull(crate::gpio::Pull::Disabled);

                    // TODO: we may want to expose these as options to allow the slew rate
                    // and drive strength for clocks if they are particularly high speed.
                    //
                    // self.set_drive_strength(crate::pac::port::pcr::Dse::Dse1);
                    // self.set_slew_rate(crate::pac::port::pcr::Sre::Sre0);
                }
            }
        };
    }
}
