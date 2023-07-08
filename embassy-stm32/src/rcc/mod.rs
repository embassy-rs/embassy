#![macro_use]

use core::mem::MaybeUninit;

use crate::time::Hertz;

#[cfg_attr(rcc_f0, path = "f0.rs")]
#[cfg_attr(any(rcc_f1, rcc_f100, rcc_f1cl), path = "f1.rs")]
#[cfg_attr(rcc_f2, path = "f2.rs")]
#[cfg_attr(rcc_f3, path = "f3.rs")]
#[cfg_attr(any(rcc_f4, rcc_f410), path = "f4.rs")]
#[cfg_attr(rcc_f7, path = "f7.rs")]
#[cfg_attr(rcc_c0, path = "c0.rs")]
#[cfg_attr(rcc_g0, path = "g0.rs")]
#[cfg_attr(rcc_g4, path = "g4.rs")]
#[cfg_attr(any(rcc_h7, rcc_h7ab), path = "h7.rs")]
#[cfg_attr(rcc_l0, path = "l0.rs")]
#[cfg_attr(rcc_l1, path = "l1.rs")]
#[cfg_attr(rcc_l4, path = "l4.rs")]
#[cfg_attr(rcc_l5, path = "l5.rs")]
#[cfg_attr(rcc_u5, path = "u5.rs")]
#[cfg_attr(rcc_wb, path = "wb.rs")]
#[cfg_attr(any(rcc_wl5, rcc_wle), path = "wl.rs")]
#[cfg_attr(any(rcc_h5, rcc_h50), path = "h5.rs")]
mod _version;
pub use _version::*;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Clocks {
    pub sys: Hertz,

    // APB
    pub apb1: Hertz,
    pub apb1_tim: Hertz,
    #[cfg(not(any(rcc_c0, rcc_g0)))]
    pub apb2: Hertz,
    #[cfg(not(any(rcc_c0, rcc_g0)))]
    pub apb2_tim: Hertz,
    #[cfg(any(rcc_wl5, rcc_wle, rcc_h5, rcc_h50, rcc_u5))]
    pub apb3: Hertz,
    #[cfg(any(rcc_h7, rcc_h7ab))]
    pub apb4: Hertz,

    // AHB
    pub ahb1: Hertz,
    #[cfg(any(
        rcc_l4, rcc_l5, rcc_f2, rcc_f4, rcc_f410, rcc_f7, rcc_h5, rcc_h50, rcc_h7, rcc_h7ab, rcc_g4, rcc_u5, rcc_wb,
        rcc_wl5, rcc_wle
    ))]
    pub ahb2: Hertz,
    #[cfg(any(
        rcc_l4, rcc_l5, rcc_f2, rcc_f4, rcc_f410, rcc_f7, rcc_h5, rcc_h50, rcc_h7, rcc_h7ab, rcc_u5, rcc_wb, rcc_wl5,
        rcc_wle
    ))]
    pub ahb3: Hertz,
    #[cfg(any(rcc_h5, rcc_h50, rcc_h7, rcc_h7ab))]
    pub ahb4: Hertz,

    #[cfg(any(rcc_f2, rcc_f4, rcc_f410, rcc_f7))]
    pub pll48: Option<Hertz>,

    #[cfg(all(rcc_f4, not(stm32f410)))]
    pub plli2s: Option<Hertz>,

    #[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f446, stm32f469, stm32f479))]
    pub pllsai: Option<Hertz>,

    #[cfg(stm32f1)]
    pub adc: Hertz,

    #[cfg(any(rcc_h5, rcc_h50, rcc_h7, rcc_h7ab))]
    pub adc: Option<Hertz>,
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
static mut CLOCK_FREQS: MaybeUninit<Clocks> = MaybeUninit::uninit();

/// Sets the clock frequencies
///
/// Safety: Sets a mutable global.
pub(crate) unsafe fn set_freqs(freqs: Clocks) {
    debug!("rcc: {:?}", freqs);
    CLOCK_FREQS = MaybeUninit::new(freqs);
}

/// Safety: Reads a mutable global.
pub(crate) unsafe fn get_freqs() -> &'static Clocks {
    CLOCK_FREQS.assume_init_ref()
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
