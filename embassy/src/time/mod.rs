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

/// Type used for blocking delays through embedded-hal traits.
///
/// For this interface to work, the Executor's clock must be correctly initialized before using it.
/// The delays are implemented in a "best-effort" way, meaning that the cpu will block for at least
/// the amount provided, but accuracy can be affected by many factors, including interrupt usage.
/// Make sure to use a suitable tick rate for your use case. The tick rate can be chosen through
/// features flags of this crate.
pub struct BlockingTimer;

impl embedded_hal::blocking::delay::DelayMs<u8> for BlockingTimer {
    fn delay_ms(&mut self, ms: u8) {
        block_for(Duration::from_millis(ms as u64))
    }
}

impl embedded_hal::blocking::delay::DelayMs<u16> for BlockingTimer {
    fn delay_ms(&mut self, ms: u16) {
        block_for(Duration::from_millis(ms as u64))
    }
}

impl embedded_hal::blocking::delay::DelayMs<u32> for BlockingTimer {
    fn delay_ms(&mut self, ms: u32) {
        block_for(Duration::from_millis(ms as u64))
    }
}

impl embedded_hal::blocking::delay::DelayUs<u8> for BlockingTimer {
    fn delay_us(&mut self, us: u8) {
        block_for(Duration::from_micros(us as u64))
    }
}

impl embedded_hal::blocking::delay::DelayUs<u16> for BlockingTimer {
    fn delay_us(&mut self, us: u16) {
        block_for(Duration::from_micros(us as u64))
    }
}

impl embedded_hal::blocking::delay::DelayUs<u32> for BlockingTimer {
    fn delay_us(&mut self, us: u32) {
        block_for(Duration::from_micros(us as u64))
    }
}

/// Blocks the cpu for at least `duration`.
///
/// For this interface to work, the Executor's clock must be correctly initialized before using it.
pub fn block_for(duration: Duration) {
    let expires_at = Instant::now() + duration;
    while Instant::now() < expires_at {}
}
