#![cfg_attr(not(any(feature = "std", feature = "wasm", test)), no_std)]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![allow(clippy::new_without_default)]
#![warn(missing_docs)]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod delay;
mod duration;
mod instant;
mod timer;

#[cfg(feature = "mock-driver")]
mod driver_mock;

#[cfg(feature = "mock-driver")]
pub use driver_mock::MockDriver;

#[cfg(feature = "std")]
mod driver_std;
#[cfg(feature = "wasm")]
mod driver_wasm;
#[cfg(feature = "generic-queue")]
mod queue_generic;

pub use delay::{block_for, Delay};
pub use duration::Duration;
pub use embassy_time_driver::TICK_HZ;
pub use instant::Instant;
pub use timer::{with_deadline, with_timeout, Ticker, TimeoutError, Timer};

const fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

pub(crate) const GCD_1K: u64 = gcd(TICK_HZ, 1_000);
pub(crate) const GCD_1M: u64 = gcd(TICK_HZ, 1_000_000);
pub(crate) const GCD_1G: u64 = gcd(TICK_HZ, 1_000_000_000);

#[cfg(feature = "defmt-timestamp-uptime-s")]
defmt::timestamp! {"{=u64}", Instant::now().as_secs() }

#[cfg(feature = "defmt-timestamp-uptime-ms")]
defmt::timestamp! {"{=u64:ms}", Instant::now().as_millis() }

#[cfg(any(feature = "defmt-timestamp-uptime", feature = "defmt-timestamp-uptime-us"))]
defmt::timestamp! {"{=u64:us}", Instant::now().as_micros() }

#[cfg(feature = "defmt-timestamp-uptime-ts")]
defmt::timestamp! {"{=u64:ts}", Instant::now().as_secs() }

#[cfg(feature = "defmt-timestamp-uptime-tms")]
defmt::timestamp! {"{=u64:tms}", Instant::now().as_millis() }

#[cfg(feature = "defmt-timestamp-uptime-tus")]
defmt::timestamp! {"{=u64:tus}", Instant::now().as_micros() }
