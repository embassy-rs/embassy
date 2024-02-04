//! Reset and Clock Control (RCC)

#![macro_use]
#![allow(missing_docs)] // TODO

use core::mem::MaybeUninit;

mod bd;
pub use bd::*;

#[cfg(any(mco, mco1, mco2))]
mod mco;
#[cfg(any(mco, mco1, mco2))]
pub use mco::*;

#[cfg(crs)]
mod hsi48;
#[cfg(crs)]
pub use hsi48::*;

#[cfg_attr(rcc_f0, path = "f0.rs")]
#[cfg_attr(any(stm32f1), path = "f1.rs")]
#[cfg_attr(any(stm32f3), path = "f3.rs")]
#[cfg_attr(any(stm32f2, stm32f4, stm32f7), path = "f.rs")]
#[cfg_attr(rcc_c0, path = "c0.rs")]
#[cfg_attr(rcc_g0, path = "g0.rs")]
#[cfg_attr(rcc_g4, path = "g4.rs")]
#[cfg_attr(any(stm32h5, stm32h7), path = "h.rs")]
#[cfg_attr(any(stm32l0, stm32l1, stm32l4, stm32l5, stm32wb, stm32wl), path = "l.rs")]
#[cfg_attr(rcc_u5, path = "u5.rs")]
#[cfg_attr(rcc_wba, path = "wba.rs")]
mod _version;

pub use _version::*;

pub use crate::_generated::Clocks;

#[cfg(feature = "low-power")]
/// Must be written within a critical section
///
/// May be read without a critical section
pub(crate) static mut REFCOUNT_STOP1: u32 = 0;

#[cfg(feature = "low-power")]
/// Must be written within a critical section
///
/// May be read without a critical section
pub(crate) static mut REFCOUNT_STOP2: u32 = 0;

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
    use critical_section::CriticalSection;

    pub trait RccPeripheral {
        fn frequency() -> crate::time::Hertz;
        fn enable_and_reset_with_cs(cs: CriticalSection);
        fn disable_with_cs(cs: CriticalSection);

        fn enable_and_reset() {
            critical_section::with(|cs| Self::enable_and_reset_with_cs(cs))
        }
        fn disable() {
            critical_section::with(|cs| Self::disable_with_cs(cs))
        }
    }
}

pub trait RccPeripheral: sealed::RccPeripheral + 'static {}

#[allow(unused)]
mod util {
    use crate::time::Hertz;

    pub fn calc_pclk<D>(hclk: Hertz, ppre: D) -> (Hertz, Hertz)
    where
        Hertz: core::ops::Div<D, Output = Hertz>,
    {
        let pclk = hclk / ppre;
        let pclk_tim = if hclk == pclk { pclk } else { pclk * 2u32 };
        (pclk, pclk_tim)
    }

    pub fn all_equal<T: Eq>(mut iter: impl Iterator<Item = T>) -> bool {
        let Some(x) = iter.next() else { return true };
        if !iter.all(|y| y == x) {
            return false;
        }
        true
    }

    pub fn get_equal<T: Eq>(mut iter: impl Iterator<Item = T>) -> Result<Option<T>, ()> {
        let Some(x) = iter.next() else { return Ok(None) };
        if !iter.all(|y| y == x) {
            return Err(());
        }
        Ok(Some(x))
    }
}
