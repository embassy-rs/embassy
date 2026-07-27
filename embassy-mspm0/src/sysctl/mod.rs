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
