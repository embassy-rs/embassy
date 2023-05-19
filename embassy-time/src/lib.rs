#![cfg_attr(not(any(feature = "std", feature = "wasm", test)), no_std)]
#![cfg_attr(feature = "nightly", feature(async_fn_in_trait))]
#![doc = include_str!("../README.md")]
#![allow(clippy::new_without_default)]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod delay;
pub mod driver;
mod duration;
mod instant;
pub mod queue;
mod tick;
mod timer;

#[cfg(feature = "std")]
mod driver_std;
#[cfg(feature = "wasm")]
mod driver_wasm;
#[cfg(feature = "generic-queue")]
mod queue_generic;

pub use delay::{block_for, Delay};
pub use duration::Duration;
pub use instant::Instant;
pub use timer::{with_timeout, Ticker, TimeoutError, Timer};

/// Ticks per second of the global timebase.
///
/// This value is specified by the `tick-*` Cargo features, which
/// should be set by the time driver. Some drivers support a fixed tick rate, others
/// allow you to choose a tick rate with Cargo features of their own. You should not
/// set the `tick-*` features for embassy yourself as an end user.
pub const TICK_HZ: u64 = tick::TICK_HZ;

const fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

pub(crate) const GCD_1K: u64 = gcd(TICK_HZ, 1_000);
pub(crate) const GCD_1M: u64 = gcd(TICK_HZ, 1_000_000);

#[cfg(feature = "defmt-timestamp-uptime")]
defmt::timestamp! {"{=u64:us}", Instant::now().as_micros() }
