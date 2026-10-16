//! Preserve the concrete PCR register when a pin is type-erased.
//!
//! MCXA156 PCRs differ in optional fields and mux width. Dispatch through the
//! PAC's actual type so updates preserve those fields and adjacent reserved bits.

use crate::pac::common::{RW, Reg};
#[cfg(feature = "_mcxa1xx")]
use crate::pac::port;
use crate::pac::port::{Dse, Ibe, Inv, Mux, Ode, Pcr, Pe, Ps, Sre};

#[derive(Clone, Copy)]
pub(crate) enum PcrRegister {
    Common(Reg<Pcr, RW>),
    #[cfg(feature = "_mcxa1xx")]
    Dse1Mux3(Reg<port::PcrDse1Mux3, RW>),
    #[cfg(feature = "_mcxa1xx")]
    Mux3(Reg<port::PcrMux3, RW>),
    #[cfg(feature = "_mcxa1xx")]
    Pfe(Reg<port::PcrPfe, RW>),
    #[cfg(feature = "_mcxa1xx")]
    PfeDse1(Reg<port::PcrPfeDse1, RW>),
    #[cfg(feature = "_mcxa1xx")]
    Port4(Reg<port::PcrPort4, RW>),
    #[cfg(feature = "_mcxa1xx")]
    PvPfeDse1(Reg<port::PcrPvPfeDse1, RW>),
    #[cfg(feature = "_mcxa1xx")]
    PvPfeMux2(Reg<port::PcrPvPfeMux2, RW>),
}

impl From<Reg<Pcr, RW>> for PcrRegister {
    fn from(reg: Reg<Pcr, RW>) -> Self {
        Self::Common(reg)
    }
}

#[cfg(feature = "_mcxa1xx")]
macro_rules! impl_from_pcr {
    ($($variant:ident => $ty:ident),+ $(,)?) => {
        $(impl From<Reg<port::$ty, RW>> for PcrRegister {
            fn from(reg: Reg<port::$ty, RW>) -> Self {
                Self::$variant(reg)
            }
        })+
    };
}

#[cfg(feature = "_mcxa1xx")]
impl_from_pcr! {
    Dse1Mux3 => PcrDse1Mux3,
    Mux3 => PcrMux3,
    Pfe => PcrPfe,
    PfeDse1 => PcrPfeDse1,
    Port4 => PcrPort4,
    PvPfeDse1 => PcrPvPfeDse1,
    PvPfeMux2 => PcrPvPfeMux2,
}

// Each arm is checked against its concrete PAC type; no register cast or raw
// register layout is needed for the fields shared by all supported PCRs.
macro_rules! with_pcr {
    ($this:expr, |$reg:ident| $body:expr) => {
        match $this {
            PcrRegister::Common($reg) => $body,
            #[cfg(feature = "_mcxa1xx")]
            PcrRegister::Dse1Mux3($reg) => $body,
            #[cfg(feature = "_mcxa1xx")]
            PcrRegister::Mux3($reg) => $body,
            #[cfg(feature = "_mcxa1xx")]
            PcrRegister::Pfe($reg) => $body,
            #[cfg(feature = "_mcxa1xx")]
            PcrRegister::PfeDse1($reg) => $body,
            #[cfg(feature = "_mcxa1xx")]
            PcrRegister::Port4($reg) => $body,
            #[cfg(feature = "_mcxa1xx")]
            PcrRegister::PvPfeDse1($reg) => $body,
            #[cfg(feature = "_mcxa1xx")]
            PcrRegister::PvPfeMux2($reg) => $body,
        }
    };
}

impl PcrRegister {
    #[inline(always)]
    pub(crate) fn set_function(self, function: Mux) {
        // The PAC's from_bits() masks its input. Check first so a bad route
        // cannot silently select a different function on a narrower PCR.
        #[cfg(feature = "_mcxa1xx")]
        fn checked_mux(function: Mux, max: u8) -> u8 {
            let bits = function.to_bits();
            assert!(bits <= max, "mux function unavailable on this pin");
            bits
        }

        match self {
            Self::Common(reg) => reg.modify(|w| w.set_mux(function)),
            #[cfg(feature = "_mcxa1xx")]
            Self::Dse1Mux3(reg) => {
                let mux = port::Mux3::from_bits(checked_mux(function, 7));
                reg.modify(|w| w.set_mux(mux));
            }
            #[cfg(feature = "_mcxa1xx")]
            Self::Mux3(reg) => {
                let mux = port::Mux3::from_bits(checked_mux(function, 7));
                reg.modify(|w| w.set_mux(mux));
            }
            #[cfg(feature = "_mcxa1xx")]
            Self::Pfe(reg) => reg.modify(|w| w.set_mux(function)),
            #[cfg(feature = "_mcxa1xx")]
            Self::PfeDse1(reg) => reg.modify(|w| w.set_mux(function)),
            #[cfg(feature = "_mcxa1xx")]
            Self::Port4(reg) => {
                // The adopted PAC reserves PORT4 mux 7; the RM/SVD conflict
                // remains documented in the MCXA156 PAC source notes.
                let mux = port::MuxPort4::from_bits(checked_mux(function, 6));
                reg.modify(|w| w.set_mux(mux));
            }
            #[cfg(feature = "_mcxa1xx")]
            Self::PvPfeDse1(reg) => reg.modify(|w| w.set_mux(function)),
            #[cfg(feature = "_mcxa1xx")]
            Self::PvPfeMux2(reg) => {
                let mux = port::Mux2::from_bits(checked_mux(function, 3));
                reg.modify(|w| w.set_mux(mux));
            }
        }
    }

    #[inline(always)]
    pub(crate) fn set_pull(self, enable: Pe, select: Ps) {
        with_pcr!(self, |reg| reg.modify(|w| {
            w.set_pe(enable);
            w.set_ps(select);
        }));
    }

    #[inline(always)]
    pub(crate) fn set_drive_strength(self, strength: Dse) {
        with_pcr!(self, |reg| reg.modify(|w| w.set_dse(strength)));
    }

    #[inline(always)]
    pub(crate) fn set_slew_rate(self, slew: Sre) {
        with_pcr!(self, |reg| reg.modify(|w| w.set_sre(slew)));
    }

    #[inline(always)]
    pub(crate) fn set_enable_input_buffer(self, enable: Ibe) {
        with_pcr!(self, |reg| reg.modify(|w| w.set_ibe(enable)));
    }

    #[inline(always)]
    pub(crate) fn set_open_drain(self, enable: Ode) {
        with_pcr!(self, |reg| reg.modify(|w| w.set_ode(enable)));
    }

    #[inline(always)]
    pub(crate) fn set_input_inversion(self, invert: Inv) {
        with_pcr!(self, |reg| reg.modify(|w| w.set_inv(invert)));
    }
}

#[cfg(all(test, feature = "_mcxa1xx"))]
mod tests {
    extern crate std;

    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;

    // Back the actual PAC register accessors with aligned RAM, never MCU MMIO.
    fn register<T: Copy>(word: &mut u32) -> Reg<T, RW> {
        // SAFETY: word is aligned, points to a live u32 for the entire test,
        // and every tested PCR is a transparent u32. No concurrent access occurs.
        unsafe { Reg::from_ptr(core::ptr::from_mut(word).cast()) }
    }

    macro_rules! for_each_pcr {
        ($test:ident) => {
            $test!(Pcr, 0x0f00, Mux::Mux13);
            $test!(PcrDse1Mux3, 0x0700, Mux::Mux7);
            $test!(PcrMux3, 0x0700, Mux::Mux7);
            $test!(PcrPfe, 0x0f00, Mux::Mux13);
            $test!(PcrPfeDse1, 0x0f00, Mux::Mux13);
            $test!(PcrPort4, 0x0700, Mux::Mux6);
            $test!(PcrPvPfeDse1, 0x0f00, Mux::Mux13);
            $test!(PcrPvPfeMux2, 0x0300, Mux::Mux3);
        };
    }

    #[test]
    fn mux_updates_preserve_bits_outside_each_pins_selector() {
        macro_rules! check {
            ($ty:ident, $mask:expr, $mux:expr) => {
                for initial in [0, u32::MAX, 0xa5a5_5a5a] {
                    let mut word = initial;
                    let pcr = PcrRegister::from(register::<port::$ty>(&mut word));
                    pcr.set_function($mux);
                    assert_eq!(word, (initial & !$mask) | (($mux.to_bits() as u32) << 8));
                    pcr.set_function(Mux::Mux0);
                    assert_eq!(word, initial & !$mask);
                }
            };
        }
        for_each_pcr!(check);
    }

    #[test]
    fn common_controls_preserve_optional_and_reserved_fields() {
        macro_rules! check {
            ($ty:ident, $mask:expr, $mux:expr) => {
                for initial in [0, u32::MAX, 0xa5a5_5a5a] {
                    let mut word = initial;
                    let pcr = PcrRegister::from(register::<port::$ty>(&mut word));
                    // PS, PE, SRE, ODE, DSE, IBE and INV only. Optional PV,
                    // PFE and DSE1, LK, mux and reserved bits must survive.
                    const COMMON_MASK: u32 = 0x306b;
                    pcr.set_pull(Pe::Pe1, Ps::Ps1);
                    pcr.set_slew_rate(Sre::Sre1);
                    pcr.set_open_drain(Ode::Ode1);
                    pcr.set_drive_strength(Dse::Dse1);
                    pcr.set_enable_input_buffer(Ibe::Ibe1);
                    pcr.set_input_inversion(Inv::Inv1);
                    assert_eq!(word, initial | COMMON_MASK);
                    pcr.set_pull(Pe::Pe0, Ps::Ps0);
                    pcr.set_slew_rate(Sre::Sre0);
                    pcr.set_open_drain(Ode::Ode0);
                    pcr.set_drive_strength(Dse::Dse0);
                    pcr.set_enable_input_buffer(Ibe::Ibe0);
                    pcr.set_input_inversion(Inv::Inv0);
                    assert_eq!(word, initial & !COMMON_MASK);
                }
            };
        }
        for_each_pcr!(check);
    }

    #[test]
    fn invalid_mux_is_rejected_without_a_register_write() {
        macro_rules! check {
            ($ty:ident, $mux:expr) => {
                let mut word = 0xa5a5_5a5a;
                let pcr = PcrRegister::from(register::<port::$ty>(&mut word));
                assert!(catch_unwind(AssertUnwindSafe(|| pcr.set_function($mux))).is_err());
                assert_eq!(word, 0xa5a5_5a5a);
            };
        }
        check!(PcrDse1Mux3, Mux::Mux8);
        check!(PcrMux3, Mux::Mux8);
        check!(PcrPvPfeMux2, Mux::Mux4);
        check!(PcrPort4, Mux::Mux7);
        check!(PcrPort4, Mux::Mux8);
    }
}
