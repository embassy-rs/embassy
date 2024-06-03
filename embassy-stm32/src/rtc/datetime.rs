#[cfg(feature = "chrono")]
use chrono::{Datelike, NaiveDate, Timelike, Weekday};

/// Errors regarding the [`DateTime`] struct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The [DateTime] contains an invalid year value. Must be between `0..=4095`.
    InvalidYear,
    /// The [DateTime] contains an invalid month value. Must be between `1..=12`.
    InvalidMonth,
    /// The [DateTime] contains an invalid day value. Must be between `1..=31`.
    InvalidDay,
    /// The [DateTime] contains an invalid day of week. Must be between `0..=6` where 0 is Sunday.
    InvalidDayOfWeek(
        /// The value of the DayOfWeek that was given.
        u8,
    ),
    /// The [DateTime] contains an invalid hour value. Must be between `0..=23`.
    InvalidHour,
    /// The [DateTime] contains an invalid minute value. Must be between `0..=59`.
    InvalidMinute,
    /// The [DateTime] contains an invalid second value. Must be between `0..=59`.
    InvalidSecond,
}

/// Structure containing date and time information
pub struct DateTime {
    /// 0..4095
    year: u16,
    /// 1..12, 1 is January
    month: u8,
    /// 1..28,29,30,31 depending on month
    day: u8,
    ///
    day_of_week: DayOfWeek,
    /// 0..23
    hour: u8,
    /// 0..59
    minute: u8,
    /// 0..59
    second: u8,
}

impl DateTime {
    /// Get the year (0..=4095)
    pub const fn year(&self) -> u16 {
        self.year
    }

    /// Get the month (1..=12, 1 is January)
    pub const fn month(&self) -> u8 {
        self.month
    }

    /// Get the day (1..=31)
    pub const fn day(&self) -> u8 {
        self.day
    }

    /// Get the day of week
    pub const fn day_of_week(&self) -> DayOfWeek {
        self.day_of_week
    }

    /// Get the hour (0..=23)
    pub const fn hour(&self) -> u8 {
        self.hour
    }

    /// Get the minute (0..=59)
    pub const fn minute(&self) -> u8 {
        self.minute
    }

    /// Get the second (0..=59)
    pub const fn second(&self) -> u8 {
        self.second
    }

    /// Create a new DateTime with the given information.
    pub fn from(
        year: u16,
        month: u8,
        day: u8,
        day_of_week: DayOfWeek,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, Error> {
        if year > 4095 {
            Err(Error::InvalidYear)
        } else if !(1..=12).contains(&month) {
            Err(Error::InvalidMonth)
        } else if !(1..=31).contains(&day) {
            Err(Error::InvalidDay)
        } else if hour > 23 {
            Err(Error::InvalidHour)
        } else if minute > 59 {
            Err(Error::InvalidMinute)
        } else if second > 59 {
            Err(Error::InvalidSecond)
        } else {
            Ok(Self {
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
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDateTime> for DateTime {
    fn from(date_time: chrono::NaiveDateTime) -> Self {
        Self {
            year: date_time.year() as u16,
            month: date_time.month() as u8,
            day: date_time.day() as u8,
            day_of_week: date_time.weekday().into(),
            hour: date_time.hour() as u8,
            minute: date_time.minute() as u8,
            second: date_time.second() as u8,
        }
    }
}

#[cfg(feature = "chrono")]
impl From<DateTime> for chrono::NaiveDateTime {
    fn from(date_time: DateTime) -> Self {
        NaiveDate::from_ymd_opt(date_time.year as i32, date_time.month as u32, date_time.day as u32)
            .unwrap()
            .and_hms_opt(date_time.hour as u32, date_time.minute as u32, date_time.second as u32)
            .unwrap()
    }
}

/// A day of the week
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[allow(missing_docs)]
pub enum DayOfWeek {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

#[cfg(feature = "chrono")]
impl From<chrono::Weekday> for DayOfWeek {
    fn from(weekday: Weekday) -> Self {
        day_of_week_from_u8(weekday.number_from_monday() as u8).unwrap()
    }
}

#[cfg(feature = "chrono")]
impl From<DayOfWeek> for chrono::Weekday {
    fn from(weekday: DayOfWeek) -> Self {
        match weekday {
            DayOfWeek::Monday => Weekday::Mon,
            DayOfWeek::Tuesday => Weekday::Tue,
            DayOfWeek::Wednesday => Weekday::Wed,
            DayOfWeek::Thursday => Weekday::Thu,
            DayOfWeek::Friday => Weekday::Fri,
            DayOfWeek::Saturday => Weekday::Sat,
            DayOfWeek::Sunday => Weekday::Sun,
        }
    }
}

pub(super) const fn day_of_week_from_u8(v: u8) -> Result<DayOfWeek, Error> {
    Ok(match v {
        1 => DayOfWeek::Monday,
        2 => DayOfWeek::Tuesday,
        3 => DayOfWeek::Wednesday,
        4 => DayOfWeek::Thursday,
        5 => DayOfWeek::Friday,
        6 => DayOfWeek::Saturday,
        7 => DayOfWeek::Sunday,
        x => return Err(Error::InvalidDayOfWeek(x)),
    })
}

pub(super) const fn day_of_week_to_u8(dotw: DayOfWeek) -> u8 {
    dotw as u8
}
