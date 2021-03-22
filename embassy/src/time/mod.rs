//! Time abstractions
//! To use these abstractions, first call `set_clock` with an instance of an [Clock](trait.Clock.html).
//!
mod duration;
mod instant;
mod traits;

pub use crate::executor::timer::{Delay, Ticker, Timer};
pub use duration::Duration;
pub use instant::Instant;
pub use traits::*;

use crate::fmt::*;

// TODO allow customizing, probably via Cargo features `tick-hz-32768` or something.
pub const TICKS_PER_SECOND: u64 = 32768;

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
