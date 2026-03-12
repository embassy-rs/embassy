//! RTC DateTime driver.

#[cfg(feature = "mcxa2xx")]
mod mcxa2xx;

#[cfg(feature = "mcxa2xx")]
pub use mcxa2xx::{InterruptHandler, Rtc, Rtc0, RtcConfig};

pub(crate) mod consts {
    /// Number of days in a standard year
    pub(crate) const DAYS_IN_A_YEAR: u32 = 365;
    /// Number of seconds in a day
    pub(crate) const SECONDS_IN_A_DAY: u32 = 86400;
    /// Number of seconds in an hour
    pub(crate) const SECONDS_IN_A_HOUR: u32 = 3600;
    /// Number of seconds in a minute
    pub(crate) const SECONDS_IN_A_MINUTE: u32 = 60;
    /// Unix epoch start year
    pub(crate) const EPOCH_YEAR_RANGE_START: u16 = 1970;
}

/// Date and time structure for RTC operations
#[derive(Debug, Clone, Copy)]
pub struct RtcDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
