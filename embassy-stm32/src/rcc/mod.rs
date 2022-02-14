#![macro_use]

use crate::time::Hertz;
use core::mem::MaybeUninit;

#[cfg_attr(rcc_f0, path = "f0.rs")]
#[cfg_attr(rcc_f1, path = "f1.rs")]
#[cfg_attr(rcc_f3, path = "f3.rs")]
#[cfg_attr(any(rcc_f4, rcc_f410), path = "f4.rs")]
#[cfg_attr(rcc_f7, path = "f7.rs")]
#[cfg_attr(rcc_g0, path = "g0.rs")]
#[cfg_attr(rcc_g4, path = "g4.rs")]
#[cfg_attr(any(rcc_h7, rcc_h7ab), path = "h7.rs")]
#[cfg_attr(rcc_l0, path = "l0.rs")]
#[cfg_attr(rcc_l1, path = "l1.rs")]
#[cfg_attr(rcc_l4, path = "l4.rs")]
#[cfg_attr(rcc_u5, path = "u5.rs")]
#[cfg_attr(rcc_wb, path = "wb.rs")]
#[cfg_attr(rcc_wl5, path = "wl5.rs")]
mod _version;
pub use _version::*;

#[derive(Clone, Copy)]
pub struct Clocks {
    pub sys: Hertz,

    // APB
    pub apb1: Hertz,
    pub apb1_tim: Hertz,
    #[cfg(not(rcc_g0))]
    pub apb2: Hertz,
    #[cfg(not(rcc_g0))]
    pub apb2_tim: Hertz,
    #[cfg(any(rcc_wl5, rcc_u5))]
    pub apb3: Hertz,
    #[cfg(any(rcc_h7))]
    pub apb4: Hertz,

    // AHB
    pub ahb1: Hertz,
    #[cfg(any(
        rcc_l4, rcc_f4, rcc_f410, rcc_f7, rcc_h7, rcc_g4, rcc_u5, rcc_wb, rcc_wl5
    ))]
    pub ahb2: Hertz,
    #[cfg(any(rcc_l4, rcc_f4, rcc_f410, rcc_f7, rcc_h7, rcc_u5, rcc_wb, rcc_wl5))]
    pub ahb3: Hertz,
    #[cfg(any(rcc_h7))]
    pub ahb4: Hertz,

    #[cfg(any(rcc_f4, rcc_f410, rcc_f7))]
    pub pll48: Option<Hertz>,

    #[cfg(rcc_f1)]
    pub adc: Hertz,
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
static mut CLOCK_FREQS: MaybeUninit<Clocks> = MaybeUninit::uninit();

/// Sets the clock frequencies
///
/// Safety: Sets a mutable global.
pub(crate) unsafe fn set_freqs(freqs: Clocks) {
    CLOCK_FREQS.as_mut_ptr().write(freqs);
}

/// Safety: Reads a mutable global.
pub(crate) unsafe fn get_freqs() -> &'static Clocks {
    &*CLOCK_FREQS.as_ptr()
}

#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

pub(crate) mod sealed {
    pub trait RccPeripheral {
        fn frequency() -> crate::time::Hertz;
        fn reset();
        fn enable();
        fn disable();
    }
}

pub trait RccPeripheral: sealed::RccPeripheral + 'static {}
