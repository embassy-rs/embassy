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
    /// Returns the kernel clock frequency of the peripheral, after its prescaler.
    ///
    /// # Panics
    ///
    /// Panics if the peripheral has no kernel clock or its source is not running.
    fn frequency() -> Hertz;
    /// Sets the kernel clock prescaler (metapac `cmu.prescaler`) to divide by `div`.
    ///
    /// # Panics
    ///
    /// Panics if the peripheral has no prescaler, or if the prescaler has no such divider.
    #[allow(unused)]
    fn set_divider(div: u32) {
        let _ = div;
        panic!("this peripheral has no kernel-clock prescaler");
    }
}

/// A peripheral with a CMU bus clock gate. `build.rs` implements it from metapac metadata.
#[allow(private_bounds)]
pub trait CmuPeripheral: SealedCmuPeripheral + 'static {}

/// Turns on bus clock for peripheral `T`.
///
/// A shared gate is refcounted and stays on until each user calls [`disable`].
///
/// # Panics
///
/// Panics on Series 0 if the LF branch of `T` has no clock: its `CMU.SYNCBUSY` bit never clears.
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

/// Returns the kernel clock frequency of `T`, after its clock select and prescaler.
///
/// # Panics
///
/// Panics if `T` has no kernel clock or its source is not running.
pub fn frequency<T: CmuPeripheral>() -> Hertz {
    T::frequency()
}

/// Sets the kernel clock prescaler of `T` to divide by `div`.
///
/// On Series 0, the prescaler is `LFAPRESC0.<T>` or `LFBPRESC0.<T>`.
///
/// # Panics
///
/// Panics if `T` has no prescaler or no divider `div`, or if the LF branch of `T` has no clock.
#[allow(unused)]
pub(crate) fn set_divider<T: CmuPeripheral>(div: u32) {
    T::set_divider(div);
}

/// Frozen clock frequencies, set once in [`crate::init`].
static mut CLOCK_FREQS: MaybeUninit<Clocks> = MaybeUninit::uninit();

/// Whether `CLOCK_FREQS` holds a value.
static CLOCK_FREQS_SET: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// Sets global clock frequencies.
///
/// # Safety
///
/// No other code may read [`get_freqs`] at the same time.
unsafe fn set_freqs(freqs: Clocks) {
    debug!("cmu: {:?}", freqs);
    CLOCK_FREQS = MaybeUninit::new(freqs);
    CLOCK_FREQS_SET.store(true, core::sync::atomic::Ordering::Release);
}

/// Returns the clock frequencies that [`crate::init`] set up.
///
/// A field is `None` when its clock is off.
///
/// # Panics
///
/// Panics before [`crate::init`].
pub fn clocks() -> Clocks {
    assert!(
        CLOCK_FREQS_SET.load(core::sync::atomic::Ordering::Acquire),
        "cmu::clocks() before embassy_silabs::init()"
    );
    // SAFETY: `set_freqs` wrote the value before it set the flag, and it runs once, in `init`.
    *unsafe { get_freqs() }
}

/// Returns the frozen clock table.
///
/// # Safety
///
/// [`set_freqs`] must have run first.
pub(crate) unsafe fn get_freqs() -> &'static Clocks {
    (*&raw const CLOCK_FREQS).assume_init_ref()
}
