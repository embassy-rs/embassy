//! Time abstractions
//! To use these abstractions, first call `set_clock` with an instance of an [Clock](trait.Clock.html).
//!
mod delay;
mod duration;
mod instant;
mod timer;
mod traits;

pub use delay::{block_for, Delay};
pub use duration::Duration;
pub use instant::Instant;
pub use timer::{with_timeout, Ticker, TimeoutError, Timer};
pub use traits::*;

#[cfg(any(
    all(feature = "tick-32768hz", feature = "tick-1000hz"),
    all(feature = "tick-32768hz", feature = "tick-1mhz"),
))]
compile_error!(
    "Disable default-features to be able to use a tick rate other than the default (32768 Hz)"
);

#[cfg(feature = "tick-1000hz")]
pub const TICKS_PER_SECOND: u64 = 1_000;

#[cfg(feature = "tick-32768hz")]
pub const TICKS_PER_SECOND: u64 = 32_768;

#[cfg(feature = "tick-1mhz")]
pub const TICKS_PER_SECOND: u64 = 1_000_000;

static mut CLOCK: Option<&'static dyn Clock> = None;

/// Sets the clock used for the timing abstractions
///
/// Safety: Sets a mutable global.
pub unsafe fn set_clock(clock: &'static dyn Clock) {
    CLOCK = Some(clock);
}

/// Return the current timestamp in ticks.
/// This is guaranteed to be monotonic, i.e. a call to now() will always return
/// a greater or equal value than earler calls.
pub(crate) fn now() -> u64 {
    unsafe { unwrap!(CLOCK, "No clock set").now() }
}
