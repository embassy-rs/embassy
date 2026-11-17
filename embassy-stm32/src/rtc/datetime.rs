/// Raw contents of the RTC calendar registers.
///
/// This is a dumb container for the values in the RTC time and date registers, decoded from BCD.
/// It does **no validation whatsoever**: it can hold dates that do not exist (February 30, month
/// 0, hour 61...), and [`Rtc::set_raw`](super::Rtc::set_raw) writes whatever it holds to the
/// hardware. Out-of-range values are truncated to the width of the register fields.
///
/// It is meant for code that cares about the last few hundred bytes of flash, or that needs the
/// hardware day of week. For anything else, enable the `jiff` feature and use
/// `RtcTimeProvider::read` and `Rtc::set`, which work with validated `jiff::civil::DateTime`
/// values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawDateTime {
    /// Year within the century, as stored in the RTC, `0..=99`. The `jiff` API takes it as the
    /// years 2000 to 2099.
    pub year: u8,
    /// Month, `1..=12`, 1 is January.
    pub month: u8,
    /// Day of the month, `1..=31`.
    pub day: u8,
    /// Day of the week, `1..=7`, 1 is Monday. The hardware does not allow 0.
    ///
    /// The RTC does not check it against the date, it only increments it every day, wrapping
    /// from 7 to 1.
    pub day_of_week: u8,
    /// Hour, `0..=23`.
    pub hour: u8,
    /// Minute, `0..=59`.
    pub minute: u8,
    /// Second, `0..=59`.
    pub second: u8,
    /// Subsecond counter, the `SS` field of `RTC_SSR`.
    ///
    /// It counts down from `PREDIV_S` (`RTC_PRER`) to 0 once per second, so the fraction of the
    /// current second that has elapsed is `(PREDIV_S - subsecond) / (PREDIV_S + 1)`. Right after
    /// `Rtc::synchronize` it can be above `PREDIV_S`, meaning the time is still before `second`.
    ///
    /// [`Rtc::set_raw`](super::Rtc::set_raw) ignores it: setting the calendar resets the counter.
    pub subsecond: u16,
}

/// The RTC holds a date and time that does not exist.
#[cfg(feature = "jiff")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidDateTimeError;

/// The date and time is outside the range the RTC can store, the years 2000 to 2099.
#[cfg(feature = "jiff")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OutOfRangeError;

/// Converts `t` to a jiff `DateTime`, ignoring `day_of_week`. `prediv_s` is the `PREDIV_S` the
/// subsecond counter runs with.
#[cfg(feature = "jiff")]
pub(super) fn to_jiff(t: RawDateTime, prediv_s: u16) -> Result<jiff::civil::DateTime, InvalidDateTimeError> {
    // jiff checks the other fields: a `u8` above `i8::MAX` turns negative, which it rejects.
    // The year would still be valid for jiff.
    if t.year > 99 {
        return Err(InvalidDateTimeError);
    }
    // A subsecond counter above `prediv_s` (right after a shift) is taken as the start of
    // the second. Below 1e9, as `elapsed <= prediv_s`.
    let elapsed = prediv_s.saturating_sub(t.subsecond) as u64;
    let nanosecond = (elapsed * 1_000_000_000 / (prediv_s as u64 + 1)) as i32;
    jiff::civil::DateTime::new(
        2000 + t.year as i16,
        t.month as i8,
        t.day as i8,
        t.hour as i8,
        t.minute as i8,
        t.second as i8,
        nanosecond,
    )
    .map_err(|_| InvalidDateTimeError)
}

/// Converts `t` to a [`RawDateTime`], truncated to whole seconds. Sets `day_of_week` to 1 instead
/// of spending the code to compute it.
#[cfg(feature = "jiff")]
pub(super) fn from_jiff(t: jiff::civil::DateTime) -> Result<RawDateTime, OutOfRangeError> {
    if !(2000..=2099).contains(&t.year()) {
        return Err(OutOfRangeError);
    }
    Ok(RawDateTime {
        year: (t.year() - 2000) as u8,
        month: t.month() as u8,
        day: t.day() as u8,
        day_of_week: 1,
        hour: t.hour() as u8,
        minute: t.minute() as u8,
        second: t.second() as u8,
        subsecond: 0,
    })
}

#[cfg(all(test, feature = "jiff"))]
mod tests {
    use jiff::civil::date;

    use super::*;

    fn raw(year: u8, month: u8, day: u8) -> RawDateTime {
        RawDateTime {
            year,
            month,
            day,
            day_of_week: 1,
            hour: 0,
            minute: 0,
            second: 0,
            subsecond: 255,
        }
    }

    fn jiff(t: RawDateTime) -> Result<jiff::civil::DateTime, InvalidDateTimeError> {
        to_jiff(t, 255)
    }

    #[test]
    fn to_jiff_checks_the_day_exists() {
        assert_eq!(jiff(raw(24, 2, 29)), Ok(date(2024, 2, 29).at(0, 0, 0, 0)));
        assert_eq!(jiff(raw(23, 2, 29)), Err(InvalidDateTimeError));
        assert_eq!(jiff(raw(24, 4, 31)), Err(InvalidDateTimeError));
        assert_eq!(jiff(raw(24, 0, 1)), Err(InvalidDateTimeError));
        assert_eq!(jiff(raw(24, 1, 0)), Err(InvalidDateTimeError));
        assert_eq!(jiff(raw(100, 1, 1)), Err(InvalidDateTimeError));
        assert_eq!(jiff(raw(24, 200, 1)), Err(InvalidDateTimeError));
    }

    #[test]
    fn to_jiff_checks_time_fields() {
        let ok = RawDateTime {
            hour: 23,
            minute: 59,
            second: 59,
            ..raw(99, 12, 31)
        };
        assert_eq!(jiff(ok), Ok(date(2099, 12, 31).at(23, 59, 59, 0)));
        assert!(jiff(RawDateTime { hour: 24, ..ok }).is_err());
        assert!(jiff(RawDateTime { minute: 60, ..ok }).is_err());
        assert!(jiff(RawDateTime { second: 60, ..ok }).is_err());
    }

    #[test]
    fn to_jiff_subseconds() {
        let at = |subsecond, prediv_s| {
            to_jiff(
                RawDateTime {
                    subsecond,
                    ..raw(24, 1, 1)
                },
                prediv_s,
            )
            .unwrap()
            .subsec_nanosecond()
        };
        // The counter starts the second at `prediv_s` and counts down.
        assert_eq!(at(255, 255), 0);
        assert_eq!(at(127, 255), 500_000_000);
        assert_eq!(at(0, 255), 996_093_750);
        assert_eq!(at(0, u16::MAX), 999_984_741);
        // Right after a shift, the counter is above `prediv_s`.
        assert_eq!(at(300, 255), 0);
        // F2 has no subsecond counter, both are 0.
        assert_eq!(at(0, 0), 0);
    }

    #[test]
    fn from_jiff_checks_the_year() {
        let t = date(2024, 5, 15).at(10, 30, 15, 123_456_789);
        assert_eq!(
            from_jiff(t),
            Ok(RawDateTime {
                year: 24,
                month: 5,
                day: 15,
                day_of_week: 1,
                hour: 10,
                minute: 30,
                second: 15,
                subsecond: 0,
            })
        );
        assert!(from_jiff(date(2000, 1, 1).at(0, 0, 0, 0)).is_ok());
        assert!(from_jiff(date(2099, 12, 31).at(23, 59, 59, 0)).is_ok());
        assert_eq!(from_jiff(date(1999, 12, 31).at(23, 59, 59, 0)), Err(OutOfRangeError));
        assert_eq!(from_jiff(date(2100, 1, 1).at(0, 0, 0, 0)), Err(OutOfRangeError));
    }
}
