//! Peripheral Helpers
//!
//! The purpose of this module is to define the per-peripheral special handling
//! required from a clocking perspective. Different peripherals have different
//! selectable source clocks, and some peripherals have additional pre-dividers
//! that can be used.
//!
//! See the docs of [`SPConfHelper`] for more details.

use super::{ClockError, Clocks, PoweredClock, WakeGuard};

#[cfg(feature = "mcxa2xx")]
mod mcxa2xx;

#[cfg(feature = "mcxa2xx")]
pub use mcxa2xx::*;

#[cfg(feature = "mcxa5xx")]
mod mcxa5xx;

#[allow(unused_imports)]
#[cfg(feature = "mcxa5xx")]
pub use mcxa5xx::*;
use nxp_pac::mrcc::vals::OstimerClkselMux;

#[must_use]
pub struct PreEnableParts {
    /// The frequency fed into the peripheral, taking into account the selected
    /// source clock, as well as any pre-divisors.
    pub freq: u32,
    /// The wake guard, if necessary for the selected clock source
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

/// Sealed Peripheral Configuration Helper
///
/// NOTE: the name "sealed" doesn't *totally* make sense because its not sealed yet in the
/// embassy-mcxa project, but it derives from embassy-imxrt where it is. We should
/// fix the name, or actually do the sealing of peripherals.
///
/// This trait serves to act as a per-peripheral customization for clocking behavior.
///
/// This trait should be implemented on a configuration type for a given peripheral, and
/// provide the methods that will be called by the higher level operations like
/// `embassy_mcxa::clocks::enable_and_reset()`.
pub trait SPConfHelper {
    /// This method is called AFTER a given MRCC peripheral has been disabled, and BEFORE
    /// the peripheral is to be enabled.
    ///
    /// This function SHOULD NOT make any changes to the system clock configuration, even
    /// unsafely, as this should remain static for the duration of the program.
    ///
    /// This function should check that any relevant upstream clocks are enabled, are in a
    /// reasonable power state, and that the requested configuration can be made. If any of
    /// these checks fail, an `Err(ClockError)` should be returned, likely `ClockError::BadConfig`.
    ///
    /// This function WILL be called in a critical section, care should be taken not to delay
    /// for an unreasonable amount of time.
    ///
    /// On success, this function MUST return an `Ok(parts)`.
    fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError>;
}

/// Copy and paste macro that:
///
/// * Sets the clocksel mux to `$selvar`
/// * Resets and halts the div, and applies the calculated div4 bits
/// * Releases reset + halt
/// * Waits for the div to stabilize
/// * Returns `Ok($freq / $conf.div.into_divisor())`
///
/// Assumes:
///
/// * self is a configuration struct that has fields called:
///   * `div`, which is a `Div4`
///   * `power`, which is a `PoweredClock`
///
/// usage:
///
/// ```rust
/// apply_div4!(self, clksel, clkdiv, variant, freq)
/// ```
///
/// In the future if we make all the clksel+clkdiv pairs into commonly derivedFrom
/// registers, or if we put some kind of simple trait around those regs, we could
/// do this with something other than a macro, but for now, this is harm-reduction
/// to avoid incorrect copy + paste
#[doc(hidden)]
#[macro_export]
macro_rules! apply_div4 {
    ($conf:ident, $selreg:ident, $divreg:ident, $selvar:ident, $freq:ident) => {{
        // set clksel
        $selreg.modify(|w| w.set_mux($selvar));

        // Set up clkdiv
        $divreg.modify(|w| {
            w.set_div($conf.div.into_bits());
            w.set_halt(ClkdivHalt::OFF);
            w.set_reset(ClkdivReset::OFF);
        });
        $divreg.modify(|w| {
            w.set_halt(ClkdivHalt::ON);
            w.set_reset(ClkdivReset::ON);
        });

        while $divreg.read().unstab() == ClkdivUnstab::OFF {}

        Ok(PreEnableParts {
            freq: $freq / $conf.div.into_divisor(),
            wake_guard: WakeGuard::for_power(&$conf.power),
        })
    }};
}

// config types

/// This type represents a divider in the range 1..=16.
///
/// At a hardware level, this is an 8-bit register from 0..=15,
/// which adds one.
///
/// While the *clock* domain seems to use 8-bit dividers, the *peripheral* domain
/// seems to use 4 bit dividers!
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Div4(pub(super) u8);

impl Div4 {
    /// Divide by one, or no division
    pub const fn no_div() -> Self {
        Self(0)
    }

    /// Store a "raw" divisor value that will divide the source by
    /// `(n + 1)`, e.g. `Div4::from_raw(0)` will divide the source
    /// by 1, and `Div4::from_raw(15)` will divide the source by
    /// 16.
    pub const fn from_raw(n: u8) -> Option<Self> {
        if n > 0b1111 { None } else { Some(Self(n)) }
    }

    /// Store a specific divisor value that will divide the source
    /// by `n`. e.g. `Div4::from_divisor(1)` will divide the source
    /// by 1, and `Div4::from_divisor(16)` will divide the source
    /// by 16.
    ///
    /// Will return `None` if `n` is not in the range `1..=16`.
    /// Consider [`Self::from_raw`] for an infallible version.
    pub const fn from_divisor(n: u8) -> Option<Self> {
        let Some(n) = n.checked_sub(1) else {
            return None;
        };
        if n > 0b1111 {
            return None;
        }
        Some(Self(n))
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

/// A basic type that always returns an error when `post_enable_config` is called.
///
/// Should only be used as a placeholder.
pub struct UnimplementedConfig;

impl SPConfHelper for UnimplementedConfig {
    fn pre_enable_config(&self, _clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        Err(ClockError::UnimplementedConfig)
    }
}

/// A basic type that always returns `Ok` when `PreEnableParts` is called.
///
/// This should only be used for peripherals that are "ambiently" clocked, like `PORTn`
/// peripherals, which have no selectable/configurable source clock.
pub struct NoConfig;
impl SPConfHelper for NoConfig {
    fn pre_enable_config(&self, _clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        Ok(PreEnableParts::empty())
    }
}

//
// OSTimer
//

/// Selectable clocks for the OSTimer peripheral
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OstimerClockSel {
    /// 16k clock, sourced from FRO16K (Vdd Core)
    #[cfg(feature = "mcxa2xx")]
    Clk16kVddCore,
    /// 1 MHz Clock sourced from FRO12M
    Clk1M,
    /// Disabled
    None,
}

/// Top level configuration for the `OSTimer` peripheral
pub struct OsTimerConfig {
    /// Power state required for this peripheral
    pub power: PoweredClock,
    /// Selected clock source for this peripheral
    pub source: OstimerClockSel,
}

impl SPConfHelper for OsTimerConfig {
    fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        let mrcc0 = crate::pac::MRCC0;
        // NOTE: complies with 22.3.2 peripheral clock max functional clock limits
        // which is 1MHz, and we can only select 1mhz/16khz.
        Ok(match self.source {
            #[cfg(feature = "mcxa2xx")]
            OstimerClockSel::Clk16kVddCore => {
                let freq = clocks.ensure_clk_16k_vdd_core_active(&self.power)?;
                mrcc0
                    .mrcc_ostimer0_clksel()
                    .write(|w| w.set_mux(OstimerClkselMux::CLKROOT_16K));
                PreEnableParts {
                    freq,
                    wake_guard: WakeGuard::for_power(&self.power),
                }
            }
            OstimerClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = OstimerClkselMux::CLKROOT_1M;
                #[cfg(feature = "mcxa5xx")]
                let mux = OstimerClkselMux::I2_CLKROOT_1M;

                mrcc0.mrcc_ostimer0_clksel().write(|w| w.set_mux(mux));
                PreEnableParts {
                    freq,
                    wake_guard: WakeGuard::for_power(&self.power),
                }
            }
            OstimerClockSel::None => {
                mrcc0
                    .mrcc_ostimer0_clksel()
                    .write(|w| w.set_mux(OstimerClkselMux::_RESERVED_3));
                PreEnableParts::empty()
            }
        })
    }
}
