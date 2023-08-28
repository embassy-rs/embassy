#[cfg(feature = "chrono")]
use core::convert::From;

#[cfg(feature = "chrono")]
use chrono::{self, Datelike, NaiveDate, Timelike, Weekday};

use super::byte_to_bcd2;
use crate::pac::rtc::Rtc;

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
    pub year: u16,
    /// 1..12, 1 is January
    pub month: u8,
    /// 1..28,29,30,31 depending on month
    pub day: u8,
    ///
    pub day_of_week: DayOfWeek,
    /// 0..23
    pub hour: u8,
    /// 0..59
    pub minute: u8,
    /// 0..59
    pub second: u8,
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
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}

#[cfg(feature = "chrono")]
impl From<chrono::Weekday> for DayOfWeek {
    fn from(weekday: Weekday) -> Self {
        day_of_week_from_u8(weekday.num_days_from_monday() as u8).unwrap()
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

fn day_of_week_from_u8(v: u8) -> Result<DayOfWeek, Error> {
    Ok(match v {
        0 => DayOfWeek::Monday,
        1 => DayOfWeek::Tuesday,
        2 => DayOfWeek::Wednesday,
        3 => DayOfWeek::Thursday,
        4 => DayOfWeek::Friday,
        5 => DayOfWeek::Saturday,
        6 => DayOfWeek::Sunday,
        x => return Err(Error::InvalidDayOfWeek(x)),
    })
}

pub(super) fn day_of_week_to_u8(dotw: DayOfWeek) -> u8 {
    dotw as u8
}

pub(super) fn validate_datetime(dt: &DateTime) -> Result<(), Error> {
    if dt.year > 4095 {
        Err(Error::InvalidYear)
    } else if dt.month < 1 || dt.month > 12 {
        Err(Error::InvalidMonth)
    } else if dt.day < 1 || dt.day > 31 {
        Err(Error::InvalidDay)
    } else if dt.hour > 23 {
        Err(Error::InvalidHour)
    } else if dt.minute > 59 {
        Err(Error::InvalidMinute)
    } else if dt.second > 59 {
        Err(Error::InvalidSecond)
    } else {
        Ok(())
    }
}

pub(super) fn write_date_time(rtc: &Rtc, t: DateTime) {
    let (ht, hu) = byte_to_bcd2(t.hour as u8);
    let (mnt, mnu) = byte_to_bcd2(t.minute as u8);
    let (st, su) = byte_to_bcd2(t.second as u8);

    let (dt, du) = byte_to_bcd2(t.day as u8);
    let (mt, mu) = byte_to_bcd2(t.month as u8);
    let yr = t.year as u16;
    let yr_offset = (yr - 1970_u16) as u8;
    let (yt, yu) = byte_to_bcd2(yr_offset);

    use crate::pac::rtc::vals::Ampm;

    rtc.tr().write(|w| {
        w.set_ht(ht);
        w.set_hu(hu);
        w.set_mnt(mnt);
        w.set_mnu(mnu);
        w.set_st(st);
        w.set_su(su);
        w.set_pm(Ampm::AM);
    });

    rtc.dr().write(|w| {
        w.set_dt(dt);
        w.set_du(du);
        w.set_mt(mt > 0);
        w.set_mu(mu);
        w.set_yt(yt);
        w.set_yu(yu);
        w.set_wdu(day_of_week_to_u8(t.day_of_week));
    });
}

pub(super) fn datetime(
    year: u16,
    month: u8,
    day: u8,
    day_of_week: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> Result<DateTime, Error> {
    let day_of_week = day_of_week_from_u8(day_of_week)?;
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
