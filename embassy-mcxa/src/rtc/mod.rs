//! RTC DateTime driver.

#[cfg(feature = "mcxa2xx")]
mod mcxa2xx;

#[cfg(feature = "mcxa2xx")]
pub use mcxa2xx::{Config, DateTime, InterruptHandler, Rtc};

#[cfg(feature = "mcxa5xx")]
mod mcxa5xx;

#[cfg(feature = "mcxa5xx")]
pub use mcxa5xx::{Compensation, Config, DateTime, InterruptHandler, Month, Rtc, Weekday};
