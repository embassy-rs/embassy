//! System controller (SYSCTL) driver.

#![macro_use]

use crate::gpio::{AnyPin, PfType, Pin, Pull, SealedPin};
use crate::pac::sysctl::vals;
use crate::peripherals::CLK_OUT;
use crate::{Peri, pac};

// TODO: Use sysctl version instead
#[cfg_attr(mspm0c110x, path = "c1103_1104.rs")]
#[cfg_attr(mspm0c1105_c1106, path = "c1105_1106.rs")]
#[cfg_attr(
    any(mspm0g110x, mspm0g150x, mspm0g310x, mspm0g350x),
    path = "g110x_150x_310x_350x.rs"
)]
#[cfg_attr(any(mspm0g151x, mspm0g351x), path = "g151x_351x.rs")]
#[cfg_attr(mspm0g518x, path = "g511x_518x.rs")]
#[cfg_attr(mspm0h321x, path = "h321x.rs")]
#[cfg_attr(any(mspm0l110x, mspm0l130x, mspm0l134x), path = "l_typea.rs")]
#[cfg_attr(any(mspm0l122x, mspm0l222x), path = "l_typeb.rs")]
mod inner;

pub use inner::ClkOutSource;

/// Deep-sleep modes, shallowest (most capable) to deepest (least capable).
///
/// The argument to [`WakeGuard::new`] and the vocabulary the low-power executor idles into. Not every
/// family implements every mode (only G/L have STOP1); a mode a family lacks is rounded to the next
/// shallower one it does have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SleepLevel {
    /// STOP0.
    Stop0,
    /// STOP1; rounded to STOP0 on families without it.
    Stop1,
    /// STOP2.
    Stop2,
    /// STANDBY0.
    Standby0,
    /// STANDBY1.
    Standby1,
}

impl SleepLevel {
    /// Shallowest level to forbid so a PD0 peripheral clocked at `clock_hz` keeps running, or `None`
    /// to forbid nothing (any sleep depth is fine, or the peripheral needs no functional clock).
    ///
    /// `clock_hz` is the frequency of the functional clock that must stay live for the peripheral —
    /// ULPCLK for bus-clocked peripherals, or the LFCLK/MFCLK source rate for those clocked directly.
    /// Feed the result to [`WakeGuard::new`] (via [`Option::map`]) for as long as the peripheral must
    /// stay clocked. The per-mode ceiling is architectural across every MSPM0 family: STOP0/STOP1 cap
    /// at 4 MHz, STOP2 and STANDBY0 at 32 kHz (LFCLK), and only STANDBY1 unclocks PD0 (there just
    /// TIMG0/1 stay clocked). Assumes the RUN0 run mode, the only one the HAL configures today (STOP0
    /// reaches 4 MHz only when entered from RUN0).
    pub const fn floor_for_clock_hz(clock_hz: u32) -> Option<Self> {
        // Per-mode clock ceilings, from the family TRMs' "DMA Operating Mode Support" and "Operating
        // Modes" sections. STANDBY0 clocks all PD0 peripherals from LFCLK; STANDBY1 does not.
        const STOP_HZ: u32 = 4_000_000; // STOP0 and STOP1
        const LFCLK_HZ: u32 = 32_768; // STOP2 and STANDBY0, both sourced from LFCLK

        if clock_hz > STOP_HZ {
            Some(Self::Stop0) // more than any STOP mode offers: only RUN/SLEEP will do
        } else if clock_hz > LFCLK_HZ {
            Some(Self::Stop2) // STOP0/STOP1 suffice; STOP2 and STANDBY0 are too slow
        } else if clock_hz > 0 {
            Some(Self::Standby1) // STOP2/STANDBY0 (LFCLK) suffice; only STANDBY1 unclocks PD0
        } else {
            None // clock-agnostic
        }
    }
}

// Boundary checks for `floor_for_clock_hz`, verified at compile time.
// `core::assert!` bypasses the crate's defmt-routing `assert!` shim, which is not const-evaluable.
const _: () = {
    core::assert!(matches!(
        SleepLevel::floor_for_clock_hz(4_000_001),
        Some(SleepLevel::Stop0)
    ));
    core::assert!(matches!(
        SleepLevel::floor_for_clock_hz(4_000_000), // MFCLK
        Some(SleepLevel::Stop2)
    ));
    core::assert!(matches!(
        SleepLevel::floor_for_clock_hz(32_769),
        Some(SleepLevel::Stop2)
    ));
    core::assert!(matches!(
        SleepLevel::floor_for_clock_hz(32_768), // LFCLK: STANDBY0 still clocks PD0, only STANDBY1 does not
        Some(SleepLevel::Standby1)
    ));
    core::assert!(matches!(SleepLevel::floor_for_clock_hz(1), Some(SleepLevel::Standby1)));
    core::assert!(matches!(SleepLevel::floor_for_clock_hz(0), None));
};

/// RAII token forbidding a deep-sleep mode (and anything deeper) while held.
///
/// A guard at `level` blocks that [`SleepLevel`] and every deeper mode; the low-power executor then
/// idles into the deepest mode still permitted, or a plain `WFI` if even [`SleepLevel::Stop0`] is
/// blocked. Guards are refcounted per level and compose: the shallowest blocked mode wins. Drop
/// releases it.
///
/// Always available so drivers can hold one unconditionally; without the `low-power` feature it is a
/// no-op (the `level` is ignored and nothing is tracked).
#[must_use]
pub struct WakeGuard {
    #[cfg(feature = "low-power")]
    level: SleepLevel,
}

impl WakeGuard {
    /// Forbid entering `level` or any deeper mode until dropped.
    ///
    /// [`SleepLevel::Stop0`] blocks all deep sleep, leaving only `WFI`. Without the `low-power`
    /// feature `level` is ignored and this does nothing.
    #[inline]
    pub fn new(level: SleepLevel) -> Self {
        #[cfg(feature = "low-power")]
        crate::low_power::block(level);
        #[cfg(not(feature = "low-power"))]
        let _ = level;

        Self {
            #[cfg(feature = "low-power")]
            level,
        }
    }
}

impl Drop for WakeGuard {
    #[inline]
    fn drop(&mut self) {
        #[cfg(feature = "low-power")]
        crate::low_power::unblock(self.level);
    }
}

/// Divider applied to the clock source of the CLK_OUT pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClkOutDiv {
    /// Divide by 2.
    Div2,

    /// Divide by 4.
    Div4,

    /// Divide by 6.
    Div6,

    /// Divide by 8.
    Div8,

    /// Divide by 10.
    Div10,

    /// Divide by 12.
    Div12,

    /// Divide by 14.
    Div14,

    /// Divide by 16.
    Div16,
}

/// CLK_OUT pin driver.
pub struct ClkOut<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> ClkOut<'d> {
    /// Create a bew CLK_OUT instance.
    pub fn new(_peri: Peri<'d, CLK_OUT>, pin: Peri<'d, impl ClkOutPin>, source: ClkOutSource) -> Self {
        // FIXME: Config (pull, invert, etc?)
        let pf = PfType::output(Pull::None, false);
        // FIXME: Infallible operation
        let pin = unwrap!(new_pin!(pin, pf));

        let (en_div, div) = source.convert_div();
        let src = source.convert_src();
        pac::SYSCTL.genclkcfg().modify(|w| {
            w.set_exclksrc(src);
            w.set_exclkdivval(div);
            w.set_exclkdiven(en_div);
        });

        pac::SYSCTL.genclken().modify(|w| {
            w.set_exclken(true);
        });

        Self { pin }
    }
}

impl<'d> Drop for ClkOut<'d> {
    fn drop(&mut self) {
        pac::SYSCTL.genclken().modify(|w| {
            w.set_exclken(false);
        });

        self.pin.set_as_disconnected();
    }
}

/// ClkOut pin trait.
pub trait ClkOutPin: Pin {
    /// Get the PF number needed to use this pin aas ClkOut pin.
    fn pf_num(&self) -> u8;
}

macro_rules! impl_clk_out_pin {
    ($pin: ident, $pf: expr) => {
        impl crate::sysctl::ClkOutPin for $crate::peripherals::$pin {
            fn pf_num(&self) -> u8 {
                $pf
            }
        }
    };
}

/// (DIVEN, DIVVAL)
fn div_to_pac(div: Option<ClkOutDiv>) -> (bool, vals::Exclkdivval) {
    match div {
        Some(ClkOutDiv::Div2) => (true, vals::Exclkdivval::DIV2),
        Some(ClkOutDiv::Div4) => (true, vals::Exclkdivval::DIV4),
        Some(ClkOutDiv::Div6) => (true, vals::Exclkdivval::DIV6),
        Some(ClkOutDiv::Div8) => (true, vals::Exclkdivval::DIV8),
        Some(ClkOutDiv::Div10) => (true, vals::Exclkdivval::DIV10),
        Some(ClkOutDiv::Div12) => (true, vals::Exclkdivval::DIV12),
        Some(ClkOutDiv::Div14) => (true, vals::Exclkdivval::DIV14),
        Some(ClkOutDiv::Div16) => (true, vals::Exclkdivval::DIV16),
        // divider is ignored. set to default value
        None => (false, vals::Exclkdivval::DIV2),
    }
}
