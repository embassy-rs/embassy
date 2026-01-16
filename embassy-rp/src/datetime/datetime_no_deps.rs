/// Errors regarding the [`DateTime`] struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The [DateTime] contains an invalid year value. Must be between `0..=4095`.
    InvalidYear,
    /// The [DateTime] contains an invalid month value. Must be between `1..=12`.
    InvalidMonth,
    /// The [DateTime] contains an invalid day value. Must be between `1..=31`.
    InvalidDay,
    /// The [DateTime] contains an invalid day of week. Must be between `0..=6` where 0 is Sunday.
    InvalidDayOfWeek,
    /// The [DateTime] contains an invalid hour value. Must be between `0..=23`.
    InvalidHour,
    /// The [DateTime] contains an invalid minute value. Must be between `0..=59`.
    InvalidMinute,
    /// The [DateTime] contains an invalid second value. Must be between `0..=59`.
    InvalidSecond,
    /// Outside valid range for Unix timestamp conversion
    OutOfRange,
    /// Invalid timestamp or cannot be converted to valid DateTime
    InvalidTimestamp,
}

/// Structure containing date and time information
#[derive(Clone, Debug)]
pub struct DateTime {
    /// 0..4095
    pub year: u16,
    /// 1..12, 1 is January
    pub month: u8,
    /// 1..28,29,30,31 depending on month
    pub day: u8,
    /// 0..6, 0 is Sunday
    pub day_of_week: DayOfWeek,
    /// 0..23
    pub hour: u8,
    /// 0..59
    pub minute: u8,
    /// 0..59
    pub second: u8,
}

/// A day of the week
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub enum DayOfWeek {
    Sunday = 0,
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
}

impl DateTime {
    /// Convert to Unix timestamp (milliseconds since 1970-01-01 00:00:00 UTC)
    ///
    /// # Errors
    /// Returns error if DateTime is before 1970-01-01.
    pub fn timestamp_millis(&self) -> Result<u64, Error> {
        crate::datetime::epoch::datetime_to_millis(self)
    }

    /// Create from Unix timestamp (milliseconds since 1970-01-01 00:00:00 UTC)
    ///
    /// # Errors
    /// Returns error if timestamp cannot be represented as valid DateTime.
    pub fn from_timestamp_millis(millis: u64) -> Result<Self, Error> {
        crate::datetime::epoch::millis_to_datetime(millis)
    }
}
