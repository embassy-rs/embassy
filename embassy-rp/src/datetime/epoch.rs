//! Unix epoch conversion (milliseconds since 1970-01-01 00:00:00 UTC)

#[cfg(any(not(feature = "chrono"), feature = "_rp235x"))]
pub(crate) const EPOCH_YEAR: u16 = 1970;

#[cfg(not(feature = "chrono"))]
mod no_deps {
    use super::super::{DateTime, DayOfWeek, Error};
    use super::EPOCH_YEAR;
    const DAYS_IN_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    const MS_PER_SECOND: u64 = 1000;
    const MS_PER_MINUTE: u64 = 60 * MS_PER_SECOND;
    const MS_PER_HOUR: u64 = 60 * MS_PER_MINUTE;
    const MS_PER_DAY: u64 = 24 * MS_PER_HOUR;
    const EPOCH_DAY_OF_WEEK: u8 = 4; // Thursday

    const fn is_leap_year(year: u16) -> bool {
        (year % 4 == 0) && ((year % 100 != 0) || (year % 400 == 0))
    }

    fn days_in_month(year: u16, month: u8) -> u8 {
        if month == 2 && is_leap_year(year) {
            29
        } else {
            DAYS_IN_MONTH[(month - 1) as usize]
        }
    }

    fn days_from_epoch_to_year(year: u16) -> u32 {
        let mut days = 0u32;
        for y in EPOCH_YEAR..year {
            days += if is_leap_year(y) { 366 } else { 365 };
        }
        days
    }

    fn day_of_week_from_days(days_since_epoch: u32) -> u8 {
        ((days_since_epoch + EPOCH_DAY_OF_WEEK as u32) % 7) as u8
    }

    pub(crate) fn datetime_to_millis(dt: &DateTime) -> Result<u64, Error> {
        if dt.year < EPOCH_YEAR {
            return Err(Error::OutOfRange);
        }

        let mut total_days = days_from_epoch_to_year(dt.year);
        for m in 1..dt.month {
            total_days += days_in_month(dt.year, m) as u32;
        }
        total_days += (dt.day - 1) as u32;

        // Convert to u64 only for final millisecond calculation
        let mut millis = total_days as u64 * MS_PER_DAY;
        millis += dt.hour as u64 * MS_PER_HOUR;
        millis += dt.minute as u64 * MS_PER_MINUTE;
        millis += dt.second as u64 * MS_PER_SECOND;

        Ok(millis)
    }

    #[cfg(not(feature = "chrono"))]
    pub(crate) fn millis_to_datetime(millis: u64) -> Result<DateTime, Error> {
        // Use u64 for initial division, then cast to u32 for subsequent calculations
        // Max total_days for year 4095 is ~776,000, fits in u32
        let total_days = (millis / MS_PER_DAY) as u32;
        // remaining_ms is at most MS_PER_DAY - 1 = 86,399,999, fits in u32
        let remaining_ms = (millis % MS_PER_DAY) as u32;

        let hour = (remaining_ms / MS_PER_HOUR as u32) as u8;
        let remaining_ms = remaining_ms % MS_PER_HOUR as u32;
        let minute = (remaining_ms / MS_PER_MINUTE as u32) as u8;
        let second = ((remaining_ms % MS_PER_MINUTE as u32) / MS_PER_SECOND as u32) as u8;

        let day_of_week = match day_of_week_from_days(total_days) {
            0 => DayOfWeek::Sunday,
            1 => DayOfWeek::Monday,
            2 => DayOfWeek::Tuesday,
            3 => DayOfWeek::Wednesday,
            4 => DayOfWeek::Thursday,
            5 => DayOfWeek::Friday,
            6 => DayOfWeek::Saturday,
            _ => unreachable!(),
        };

        let mut year = EPOCH_YEAR;
        let mut days_remaining = total_days;

        loop {
            let days_in_year: u32 = if is_leap_year(year) { 366 } else { 365 };
            if days_remaining < days_in_year {
                break;
            }
            days_remaining -= days_in_year;
            year += 1;

            if year > 4095 {
                return Err(Error::InvalidTimestamp);
            }
        }

        let mut month = 1u8;
        while month <= 12 {
            let days_in_this_month = days_in_month(year, month) as u32;
            if days_remaining < days_in_this_month {
                break;
            }
            days_remaining -= days_in_this_month;
            month += 1;
        }
        let day = (days_remaining + 1) as u8;

        Ok(DateTime {
            year,
            month,
            day,
            day_of_week,
            hour,
            minute,
            second,
        })
    }
}

#[cfg(not(feature = "chrono"))]
pub(crate) use no_deps::{datetime_to_millis, millis_to_datetime};

#[cfg(all(feature = "chrono", feature = "_rp235x"))]
mod chrono_rp235x {
    use super::super::{DateTime, Error};
    use super::EPOCH_YEAR;

    pub(crate) fn datetime_to_millis(dt: &DateTime) -> Result<u64, Error> {
        use chrono::Datelike;

        if dt.year() < EPOCH_YEAR as i32 {
            return Err(Error::OutOfRange);
        }

        let timestamp_millis = dt.and_utc().timestamp_millis();
        if timestamp_millis < 0 {
            return Err(Error::OutOfRange);
        }

        Ok(timestamp_millis as u64)
    }

    pub(crate) fn millis_to_datetime(millis: u64) -> Result<DateTime, Error> {
        use chrono::Datelike;

        let millis_i64 = millis.try_into().map_err(|_| Error::InvalidTimestamp)?;
        let dt = chrono::DateTime::from_timestamp_millis(millis_i64)
            .ok_or(Error::InvalidTimestamp)?
            .naive_utc();

        if dt.year() < 0 || dt.year() > 4095 {
            return Err(Error::InvalidYear);
        }

        Ok(dt)
    }
}

#[cfg(all(feature = "chrono", feature = "_rp235x"))]
pub(crate) use chrono_rp235x::{datetime_to_millis, millis_to_datetime};
