//! Time abstractions
//! To use these abstractions, first call `set_clock` with an instance of an [Clock](trait.Clock.html).
//!
mod duration;
mod instant;
mod traits;

pub use crate::executor::timer::{with_timeout, Delay, Ticker, TimeoutError, Timer};
pub use duration::Duration;
pub use instant::Instant;
pub use traits::*;

#[cfg(any(
    all(feature = "tick-hz-32768", feature = "tick-hz-1000"),
    all(feature = "tick-hz-32768", feature = "tick-mhz-1"),
))]
compile_error!(
    "Disable default-features to be able to use a tick rate other than the default (32768 Hz)"
);

#[cfg(feature = "tick-hz-1000")]
pub const TICKS_PER_SECOND: u64 = 1_000;

#[cfg(feature = "tick-hz-32768")]
pub const TICKS_PER_SECOND: u64 = 32_768;

#[cfg(feature = "tick-mhz-1")]
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

pub struct BlockingTimer;

impl embedded_hal::blocking::delay::DelayMs<u8> for BlockingTimer {
    fn delay_ms(&mut self, ms: u8) {
        block_for(Duration::from_millis(u64::from(ms)))
    }
}

impl embedded_hal::blocking::delay::DelayMs<u16> for BlockingTimer {
    fn delay_ms(&mut self, ms: u16) {
        block_for(Duration::from_millis(u64::from(ms)))
    }
}

impl embedded_hal::blocking::delay::DelayMs<u32> for BlockingTimer {
    fn delay_ms(&mut self, ms: u32) {
        block_for(Duration::from_millis(u64::from(ms)))
    }
}

#[cfg(feature = "tick-mhz-1")]
impl embedded_hal::blocking::delay::DelayUs<u8> for BlockingTimer {
    fn delay_us(&mut self, us: u8) {
        block_for(Duration::from_micros(u64::from(us)))
    }
}

#[cfg(feature = "tick-mhz-1")]
impl embedded_hal::blocking::delay::DelayUs<u16> for BlockingTimer {
    fn delay_us(&mut self, us: u16) {
        block_for(Duration::from_micros(u64::from(us)))
    }
}

#[cfg(feature = "tick-mhz-1")]
impl embedded_hal::blocking::delay::DelayUs<u32> for BlockingTimer {
    fn delay_us(&mut self, us: u32) {
        block_for(Duration::from_micros(u64::from(us)))
    }
}

pub fn block_for(duration: Duration) {
    let expires_at = Instant::now() + duration;
    while Instant::now() < expires_at {}
}
