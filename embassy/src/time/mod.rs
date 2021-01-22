mod duration;
mod instant;
mod traits;

pub use crate::executor::timer::{Ticker, Timer};
pub use duration::Duration;
pub use instant::Instant;
pub use traits::*;

use crate::fmt::*;

// TODO allow customizing, probably via Cargo features `tick-hz-32768` or something.
pub const TICKS_PER_SECOND: u64 = 32768;

static mut CLOCK: Option<&'static dyn Clock> = None;

pub unsafe fn set_clock(clock: &'static dyn Clock) {
    CLOCK = Some(clock);
}

pub(crate) fn is_clock_none() -> bool {
    unsafe { CLOCK.is_none() }
}

pub(crate) fn now() -> u64 {
    unsafe { unwrap!(CLOCK, "No clock set").now() }
}
