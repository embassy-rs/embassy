//! Clock Management Unit (CMU).
//!
//! Clock gates, [`frequency`] and the frozen [`Clocks`] table are shared by both series. The clock
//! tree for each series is in `s0.rs` (EFM32GG) and `s2.rs` (EFR32).

use core::mem::MaybeUninit;

pub use crate::time::Hertz;

#[cfg(silabs_series = "0")]
mod s0;
#[cfg(silabs_series = "0")]
pub use s0::*;
#[cfg(silabs_series = "2")]
mod s2;
#[cfg(silabs_series = "2")]
pub use s2::*;

// `Clocks`, group-clock frequency functions and `mux::ClockMux`, generated from metapac data.
include!(concat!(env!("OUT_DIR"), "/_cmu_mux.rs"));

pub(crate) trait SealedCmuPeripheral {
    fn enable_bus_clock();
    fn disable_bus_clock();
    /// Frequency of the peripheral's kernel clock. Panics when the
    /// peripheral has no kernel clock or its source is not running.
    fn frequency() -> Hertz;
}

/// A peripheral with a CMU bus clock gate. `build.rs` implements it from metapac metadata.
#[allow(private_bounds)]
pub trait CmuPeripheral: SealedCmuPeripheral + 'static {}

/// Turns on bus clock for peripheral `T`.
///
/// Most gates belong to one peripheral. On Series 2, this is then one write
/// to the gate register's SET alias, so it needs no critical section.
/// Series 0 has no SET alias, so the write is a read-modify-write in a
/// critical section. A Series 0 LF gate (`LFACLKEN0`) also waits for its
/// `SYNCBUSY` bit. A gate that several peripherals share (DMEM0 and DMEM1
/// on MG26) is refcounted in a critical section: it stays on until each
/// user calls [`disable`].
pub fn enable<T: CmuPeripheral>() {
    T::enable_bus_clock();
}

/// Turns off bus clock for peripheral `T`.
///
/// A shared gate turns off when its last user calls [`disable`].
///
/// # Panics
///
/// Panics if `disable` is called more times than `enable` for a shared gate.
pub fn disable<T: CmuPeripheral>() {
    T::disable_bus_clock();
}

/// Frequency of the kernel clock of peripheral `T`, like embassy-stm32's
/// `rcc::frequency`. Reads the peripheral's clock select, if it has one.
///
/// Panics if `T` has no kernel clock or its source is not running.
pub fn frequency<T: CmuPeripheral>() -> Hertz {
    T::frequency()
}

/// Frozen clock frequencies, set once in [`crate::init`].
static mut CLOCK_FREQS: MaybeUninit<Clocks> = MaybeUninit::uninit();

/// Sets global clock frequencies.
///
/// # Safety
///
/// No other code may read [`get_freqs`] at the same time.
unsafe fn set_freqs(freqs: Clocks) {
    debug!("cmu: {:?}", freqs);
    CLOCK_FREQS = MaybeUninit::new(freqs);
}

/// Returns the frozen clock table.
///
/// # Safety
///
/// [`set_freqs`] must have run first.
pub(crate) unsafe fn get_freqs() -> &'static Clocks {
    (*&raw const CLOCK_FREQS).assume_init_ref()
}
