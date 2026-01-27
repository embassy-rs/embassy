/// Alias for [`chrono::NaiveDateTime`]
pub type DateTime = chrono::NaiveDateTime;
/// Alias for [`chrono::Weekday`]
pub type DayOfWeek = chrono::Weekday;

/// Errors regarding the [`DateTime`] struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The [DateTime] has an invalid year. The year must be between 0 and 4095.
    InvalidYear,
    /// The [DateTime] contains an invalid date.
    InvalidDate,
    /// The [DateTime] contains an invalid time.
    InvalidTime,
    /// Outside valid range for Unix timestamp conversion
    OutOfRange,
    /// Invalid timestamp or cannot be converted to valid DateTime
    InvalidTimestamp,
}

/// Convert to Unix timestamp (milliseconds since 1970-01-01 00:00:00 UTC)
///
/// # Errors
/// Returns error if DateTime is before 1970-01-01.
pub fn timestamp_millis(dt: &DateTime) -> Result<u64, Error> {
    crate::datetime::epoch::datetime_to_millis(dt)
}

/// Create from Unix timestamp (milliseconds since 1970-01-01 00:00:00 UTC)
///
/// # Errors
/// Returns error if timestamp cannot be represented as valid DateTime.
pub fn from_timestamp_millis(millis: u64) -> Result<DateTime, Error> {
    crate::datetime::epoch::millis_to_datetime(millis)
}
