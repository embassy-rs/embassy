//! Time abstractions

mod delay;
pub mod driver;
mod duration;
mod instant;
mod timer;

pub use delay::{block_for, Delay};
pub use duration::Duration;
pub use instant::Instant;
pub use timer::{with_timeout, Ticker, TimeoutError, Timer};

#[cfg(feature = "time-tick-1000hz")]
pub const TICKS_PER_SECOND: u64 = 1_000;

#[cfg(feature = "time-tick-32768hz")]
pub const TICKS_PER_SECOND: u64 = 32_768;

#[cfg(feature = "time-tick-1mhz")]
pub const TICKS_PER_SECOND: u64 = 1_000_000;
