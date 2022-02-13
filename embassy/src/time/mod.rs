//! Timekeeping, delays and timeouts.
//!
//! Timekeeping is done with elapsed time since system boot. Time is represented in
//! ticks, where the tick rate is defined by the current driver, usually to match
//! the tick rate of the hardware.
//!
//! Tick counts are 64 bits. At the highest supported tick rate of 1Mhz this supports
//! representing time spans of up to ~584558 years, which is big enough for all practical
//! purposes and allows not having to worry about overflows.
//!
//! [`Instant`] represents a given instant of time (relative to system boot), and [`Duration`]
//! represents the duration of a span of time. They implement the math operations you'd expect,
//! like addition and substraction.
//!
//! # Delays and timeouts
//!
//! [`Timer`] allows performing async delays. [`Ticker`] allows periodic delays without drifting over time.
//!
//! An implementation of the `embedded-hal` delay traits is provided by [`Delay`], for compatibility
//! with libraries from the ecosystem.
//!
//! # Wall-clock time
//!
//! The `time` module deals exclusively with a monotonically increasing tick count.
//! Therefore it has no direct support for wall-clock time ("real life" datetimes
//! like `2021-08-24 13:33:21`).
//!
//! If persistence across reboots is not needed, support can be built on top of
//! `embassy::time` by storing the offset between "seconds elapsed since boot"
//! and "seconds since unix epoch".
//!
//! # Time driver
//!
//! The `time` module is backed by a global "time driver" specified at build time.
//! Only one driver can be active in a program.
//!
//! All methods and structs transparently call into the active driver. This makes it
//! possible for libraries to use `embassy::time` in a driver-agnostic way without
//! requiring generic parameters.
//!
//! For more details, check the [`driver`] module.

#![deny(missing_docs)]

mod delay;
pub mod driver;
mod duration;
mod instant;
mod timer;

#[cfg(feature = "std")]
mod driver_std;

#[cfg(feature = "wasm")]
mod driver_wasm;

pub use delay::{block_for, Delay};
pub use duration::Duration;
pub use instant::Instant;
pub use timer::{with_timeout, Ticker, TimeoutError, Timer};

#[cfg(feature = "time-tick-1000hz")]
const TPS: u64 = 1_000;

#[cfg(feature = "time-tick-32768hz")]
const TPS: u64 = 32_768;

#[cfg(feature = "time-tick-1mhz")]
const TPS: u64 = 1_000_000;

/// Ticks per second of the global timebase.
///
/// This value is specified by the `time-tick-*` Cargo features, which
/// should be set by the time driver. Some drivers support a fixed tick rate, others
/// allow you to choose a tick rate with Cargo features of their own. You should not
/// set the `time-tick-*` features for embassy yourself as an end user.
pub const TICKS_PER_SECOND: u64 = TPS;

const fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

pub(crate) const GCD_1K: u64 = gcd(TICKS_PER_SECOND, 1_000);
pub(crate) const GCD_1M: u64 = gcd(TICKS_PER_SECOND, 1_000_000);
