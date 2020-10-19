mod duration;
mod instant;
mod timer;
mod traits;

pub use duration::Duration;
pub use instant::Instant;
pub use timer::Timer;
pub use traits::*;

use crate::util::Dewrap;

// TODO allow customizing, probably via Cargo features `tick-hz-32768` or something.
pub const TICKS_PER_SECOND: u32 = 32768;

static mut CLOCK: Option<&'static dyn Clock> = None;

pub unsafe fn set_clock(clock: &'static dyn Clock) {
    CLOCK = Some(clock);
}

pub(crate) fn now() -> u64 {
    unsafe { CLOCK.dexpect(defmt::intern!("No clock set")).now() }
}
