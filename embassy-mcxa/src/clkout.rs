//! CLKOUT pseudo-peripheral
//!
//! CLKOUT is a part of the clock generation subsystem, and can be used
//! either to generate arbitrary waveforms, or to debug the state of
//! internal oscillators.

use core::marker::PhantomData;

use embassy_hal_internal::Peri;

pub use crate::clocks::periph_helpers::Div4;
use crate::clocks::{ClockError, PoweredClock, with_clocks};
use crate::pac::mrcc0::mrcc_clkout_clksel::Mux;
use crate::peripherals::CLKOUT;

/// A peripheral representing the CLKOUT pseudo-peripheral
pub struct ClockOut<'a> {
    _p: PhantomData<&'a mut CLKOUT>,
    freq: u32,
}

/// Selected clock source to output
#[derive(Copy, Clone)]
pub enum ClockOutSel {
    /// 12MHz Internal Oscillator
    Fro12M,
    /// FRO180M Internal Oscillator, via divisor
    FroHfDiv,
    /// External Oscillator
    ClkIn,
    /// 16KHz oscillator
    Clk16K,
    /// Output of PLL1
    Pll1Clk,
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
    pub fn new(
        _peri: Peri<'a, CLKOUT>,
        pin: Peri<'a, impl sealed::ClockOutPin>,
        cfg: Config,
    ) -> Result<Self, ClockError> {
        // There's no MRCC enable bit, so we check the validity of the clocks here
        //
        // TODO: Should we check that the frequency is suitably low?
        let (freq, mux) = check_sel(cfg.sel, cfg.level)?;

        // All good! Apply requested config, starting with the pin.
        pin.mux();

        setup_clkout(mux, cfg.div);

        Ok(Self {
            _p: PhantomData,
            freq: freq / cfg.div.into_divisor(),
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
    }
}

/// Check whether the given clock selection is valid
fn check_sel(sel: ClockOutSel, level: PoweredClock) -> Result<(u32, Mux), ClockError> {
    let res = with_clocks(|c| {
        Ok(match sel {
            ClockOutSel::Fro12M => (c.ensure_fro_hf_active(&level)?, Mux::Clkroot12m),
            ClockOutSel::FroHfDiv => (c.ensure_fro_hf_div_active(&level)?, Mux::ClkrootFircDiv),
            ClockOutSel::ClkIn => (c.ensure_clk_in_active(&level)?, Mux::ClkrootSosc),
            ClockOutSel::Clk16K => (c.ensure_clk_16k_vdd_core_active(&level)?, Mux::Clkroot16k),
            ClockOutSel::Pll1Clk => (c.ensure_pll1_clk_active(&level)?, Mux::ClkrootSpll),
            ClockOutSel::SlowClk => (c.ensure_slow_clk_active(&level)?, Mux::ClkrootSlow),
        })
    });
    let Some(res) = res else {
        return Err(ClockError::NeverInitialized);
    };
    res
}

/// Set up the clkout pin using the given mux and div settings
fn setup_clkout(mux: Mux, div: Div4) {
    let mrcc = unsafe { crate::pac::Mrcc0::steal() };

    mrcc.mrcc_clkout_clksel().write(|w| w.mux().variant(mux));

    // Set up clkdiv
    mrcc.mrcc_clkout_clkdiv().write(|w| {
        w.halt().set_bit();
        w.reset().set_bit();
        unsafe { w.div().bits(div.into_bits()) };
        w
    });
    mrcc.mrcc_clkout_clkdiv().write(|w| {
        w.halt().clear_bit();
        w.reset().clear_bit();
        unsafe { w.div().bits(div.into_bits()) };
        w
    });

    while mrcc.mrcc_clkout_clkdiv().read().unstab().bit_is_set() {}
}

/// Stop the clkout
fn disable_clkout() {
    // Stop the output by selecting the "none" clock
    //
    // TODO: restore the pin to hi-z or something?
    let mrcc = unsafe { crate::pac::Mrcc0::steal() };
    mrcc.mrcc_clkout_clkdiv().write(|w| {
        w.reset().set_bit();
        w.halt().set_bit();
        unsafe {
            w.div().bits(0);
        }
        w
    });
    mrcc.mrcc_clkout_clksel().write(|w| unsafe { w.bits(0b111) });
}

mod sealed {
    use embassy_hal_internal::PeripheralType;

    use crate::gpio::{Pull, SealedPin};

    /// Sealed marker trait for clockout pins
    pub trait ClockOutPin: PeripheralType {
        /// Set the given pin to the correct muxing state
        fn mux(&self);
    }

    macro_rules! impl_pin {
        ($pin:ident, $func:ident) => {
            impl ClockOutPin for crate::peripherals::$pin {
                fn mux(&self) {
                    self.set_function(crate::pac::port0::pcr0::Mux::$func);
                    self.set_pull(Pull::Disabled);

                    // TODO: we may want to expose these as options to allow the slew rate
                    // and drive strength for clocks if they are particularly high speed.
                    //
                    // self.set_drive_strength(crate::pac::port0::pcr0::Dse::Dse1);
                    // self.set_slew_rate(crate::pac::port0::pcr0::Sre::Sre0);
                }
            }
        };
    }

    impl_pin!(P0_6, Mux12);
    impl_pin!(P3_6, Mux1);
    impl_pin!(P3_8, Mux12);
    impl_pin!(P4_2, Mux1);
}
