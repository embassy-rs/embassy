//! Reset and Clock Control (RCC)

#![macro_use]
#![allow(missing_docs)] // TODO

use core::mem::MaybeUninit;

mod bd;
pub use bd::*;

#[cfg(any(mco, mco1, mco2))]
mod mco;
use critical_section::CriticalSection;
#[cfg(any(mco, mco1, mco2))]
pub use mco::*;

#[cfg(crs)]
mod hsi48;
#[cfg(crs)]
pub use hsi48::*;

#[cfg_attr(any(stm32f0, stm32f1, stm32f3), path = "f013.rs")]
#[cfg_attr(any(stm32f2, stm32f4, stm32f7), path = "f247.rs")]
#[cfg_attr(stm32c0, path = "c0.rs")]
#[cfg_attr(stm32g0, path = "g0.rs")]
#[cfg_attr(stm32g4, path = "g4.rs")]
#[cfg_attr(any(stm32h5, stm32h7, stm32h7rs), path = "h.rs")]
#[cfg_attr(any(stm32l0, stm32l1, stm32l4, stm32l5, stm32wb, stm32wl, stm32u0), path = "l.rs")]
#[cfg_attr(stm32u5, path = "u5.rs")]
#[cfg_attr(stm32wba, path = "wba.rs")]
mod _version;

pub use _version::*;
use stm32_metapac::RCC;

pub use crate::_generated::{mux, Clocks};
use crate::time::Hertz;

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

pub(crate) trait SealedRccPeripheral {
    const ENABLE_BIT: ClockEnableBit;

    fn frequency() -> Hertz;
    fn enable_and_reset_with_cs(cs: CriticalSection);
    fn disable_with_cs(cs: CriticalSection);

    fn enable_and_reset() {
        critical_section::with(|cs| Self::enable_and_reset_with_cs(cs))
    }
    fn disable() {
        critical_section::with(|cs| Self::disable_with_cs(cs))
    }
}

#[allow(private_bounds)]
pub trait RccPeripheral: SealedRccPeripheral + 'static {}

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

/// Get the kernel clock frequency of the peripheral `T`.
///
/// # Panics
///
/// Panics if the clock is not active.
pub fn frequency<T: RccPeripheral>() -> Hertz {
    T::frequency()
}

/// Enables and resets peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
pub unsafe fn enable_and_reset<T: RccPeripheral>() {
    T::enable_and_reset();
}

/// Disables peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
pub unsafe fn disable<T: RccPeripheral>() {
    T::disable();
}

/// Struct representing some clock enable bit (xxxENR.xxEN), only known at runtime.
#[derive(Clone, Copy)]
pub(crate) struct ClockEnableBit {
    /// offset in 32bit words of the xxxENR register into the RCC register block.
    offset: u8,
    /// bit within the register (0..=31)
    bit: u8,
}

impl ClockEnableBit {
    /// Safety: offset+bit must correspond to a valid xxxEN bit.
    pub(crate) const unsafe fn new(offset: u8, bit: u8) -> Self {
        Self { offset, bit }
    }

    fn ptr(self) -> *mut u32 {
        unsafe { (RCC.as_ptr() as *mut u32).add(self.offset as _) }
    }

    #[allow(unused)]
    pub(crate) fn enable_with_cs(self, _cs: CriticalSection) {
        let p = self.ptr();
        unsafe {
            let val = p.read_volatile();
            p.write_volatile(val | 1u32 << self.bit);
        }
    }

    pub(crate) fn disable_with_cs(self, _cs: CriticalSection) {
        let p = self.ptr();
        unsafe {
            let val = p.read_volatile();
            p.write_volatile(val & !(1u32 << self.bit));
        }
    }

    #[allow(unused)]
    pub(crate) fn enable(self) {
        critical_section::with(|cs| self.enable_with_cs(cs))
    }

    pub(crate) fn disable(self) {
        critical_section::with(|cs| self.disable_with_cs(cs))
    }
}
