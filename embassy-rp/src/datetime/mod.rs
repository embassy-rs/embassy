//! Shared datetime types for RTC and other time-keeping peripherals.
//!
//! This module provides DateTime and DayOfWeek types that can be used across
//! multiple peripherals in embassy-rp. The implementation can be backed by
//! the `chrono` crate (when the `chrono` feature is enabled) or use a
//! standalone no-dependencies implementation.

#[cfg_attr(feature = "chrono", path = "datetime_chrono.rs")]
#[cfg_attr(not(feature = "chrono"), path = "datetime_no_deps.rs")]
mod datetime;

pub(crate) mod epoch;

pub use self::datetime::{DateTime, DayOfWeek, Error};
#[cfg(feature = "chrono")]
pub use self::datetime::{from_timestamp_millis, timestamp_millis};
