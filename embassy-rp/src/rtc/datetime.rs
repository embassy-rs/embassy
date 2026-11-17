/// Raw contents of the RTC calendar registers.
///
/// This is a dumb container for the values in the RTC registers. It does **no validation
/// whatsoever**: it can hold dates that do not exist (February 30, month 0, hour 61...), and
/// [`Rtc::set_raw`](super::Rtc::set_raw) writes whatever it holds to the hardware. Out-of-range
/// values are truncated to the width of the register fields.
///
/// It is meant for code that cares about the last few hundred bytes of flash, or that needs the
/// hardware day of week. For anything else, enable the `jiff` feature and use `Rtc::read` and
/// `Rtc::set`, which work with validated `jiff::civil::DateTime` values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawDateTime {
    /// Year, `0..=4095`.
    pub year: u16,
    /// Month, `1..=12`, 1 is January.
    pub month: u8,
    /// Day of the month, `1..=31`.
    pub day: u8,
    /// Day of the week, `0..=6`, 0 is Sunday.
    ///
    /// The RTC does not check it against the date, it only increments it every day.
    pub day_of_week: u8,
    /// Hour, `0..=23`.
    pub hour: u8,
    /// Minute, `0..=59`.
    pub minute: u8,
    /// Second, `0..=59`.
    pub second: u8,
}

/// The RTC holds a date and time that does not exist.
#[cfg(feature = "jiff")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidDateTimeError;

/// The date and time is outside the range the RTC can store, the years 0 to 4095.
#[cfg(feature = "jiff")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OutOfRangeError;

/// Converts `t` to a jiff `DateTime`, ignoring `day_of_week`.
#[cfg(feature = "jiff")]
pub(super) fn to_jiff(t: RawDateTime) -> Result<jiff::civil::DateTime, InvalidDateTimeError> {
    // jiff checks the other fields: a `u8` above `i8::MAX` turns negative, which it rejects.
    // A year above 4095 would still be valid for jiff, and could wrap when cast to `i16`.
    if t.year > 4095 {
        return Err(InvalidDateTimeError);
    }
    jiff::civil::DateTime::new(
        t.year as i16,
        t.month as i8,
        t.day as i8,
        t.hour as i8,
        t.minute as i8,
        t.second as i8,
        0,
    )
    .map_err(|_| InvalidDateTimeError)
}

/// Converts `t` to a [`RawDateTime`], truncated to whole seconds. Sets `day_of_week` to 0 instead
/// of spending the code to compute it.
#[cfg(feature = "jiff")]
pub(super) fn from_jiff(t: jiff::civil::DateTime) -> Result<RawDateTime, OutOfRangeError> {
    if !(0..=4095).contains(&t.year()) {
        return Err(OutOfRangeError);
    }
    Ok(RawDateTime {
        year: t.year() as u16,
        month: t.month() as u8,
        day: t.day() as u8,
        day_of_week: 0,
        hour: t.hour() as u8,
        minute: t.minute() as u8,
        second: t.second() as u8,
    })
}

#[cfg(all(test, feature = "jiff"))]
mod tests {
    use jiff::civil::date;

    use super::*;

    fn raw(year: u16, month: u8, day: u8) -> RawDateTime {
        RawDateTime {
            year,
            month,
            day,
            day_of_week: 0,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }

    #[test]
    fn to_jiff_checks_the_date() {
        assert_eq!(to_jiff(raw(2024, 2, 29)), Ok(date(2024, 2, 29).at(0, 0, 0, 0)));
        // The RTC takes every year divisible by 4 as a leap year, unless told otherwise.
        assert_eq!(to_jiff(raw(2100, 2, 29)), Err(InvalidDateTimeError));
        assert_eq!(to_jiff(raw(2024, 4, 31)), Err(InvalidDateTimeError));
        assert_eq!(to_jiff(raw(2024, 0, 1)), Err(InvalidDateTimeError));
        assert_eq!(to_jiff(raw(2024, 1, 0)), Err(InvalidDateTimeError));
        assert_eq!(to_jiff(raw(2024, 200, 1)), Err(InvalidDateTimeError));
        assert_eq!(to_jiff(raw(4095, 12, 31)), Ok(date(4095, 12, 31).at(0, 0, 0, 0)));
        assert_eq!(to_jiff(raw(4096, 1, 1)), Err(InvalidDateTimeError));
        assert_eq!(to_jiff(raw(u16::MAX, 1, 1)), Err(InvalidDateTimeError));
        assert_eq!(to_jiff(raw(0, 1, 1)), Ok(date(0, 1, 1).at(0, 0, 0, 0)));
    }

    #[test]
    fn to_jiff_checks_time_fields() {
        let ok = RawDateTime {
            hour: 23,
            minute: 59,
            second: 59,
            ..raw(2024, 1, 1)
        };
        assert_eq!(to_jiff(ok), Ok(date(2024, 1, 1).at(23, 59, 59, 0)));
        assert!(to_jiff(RawDateTime { hour: 24, ..ok }).is_err());
        assert!(to_jiff(RawDateTime { minute: 60, ..ok }).is_err());
        assert!(to_jiff(RawDateTime { second: 60, ..ok }).is_err());
    }

    #[test]
    fn from_jiff_checks_the_year() {
        assert_eq!(
            from_jiff(date(2024, 5, 15).at(10, 30, 15, 123_456_789)),
            Ok(RawDateTime {
                year: 2024,
                month: 5,
                day: 15,
                day_of_week: 0,
                hour: 10,
                minute: 30,
                second: 15,
            })
        );
        assert!(from_jiff(date(0, 1, 1).at(0, 0, 0, 0)).is_ok());
        assert!(from_jiff(date(4095, 12, 31).at(23, 59, 59, 0)).is_ok());
        assert_eq!(from_jiff(date(-1, 12, 31).at(0, 0, 0, 0)), Err(OutOfRangeError));
        assert_eq!(from_jiff(date(4096, 1, 1).at(0, 0, 0, 0)), Err(OutOfRangeError));
    }
}
