use crate::pac::rtc::regs::{Rtc0, Rtc1, Setup0, Setup1};

/// Errors regarding the [`DateTime`] and [`DateTimeFilter`] structs.
///
/// [`DateTimeFilter`]: struct.DateTimeFilter.html
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug)]
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

/// A day of the week
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
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

fn day_of_week_from_u8(v: u8) -> Result<DayOfWeek, Error> {
    Ok(match v {
        0 => DayOfWeek::Sunday,
        1 => DayOfWeek::Monday,
        2 => DayOfWeek::Tuesday,
        3 => DayOfWeek::Wednesday,
        4 => DayOfWeek::Thursday,
        5 => DayOfWeek::Friday,
        6 => DayOfWeek::Saturday,
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

pub(super) fn write_setup_0(dt: &DateTime, w: &mut Setup0) {
    w.set_year(dt.year);
    w.set_month(dt.month);
    w.set_day(dt.day);
}

pub(super) fn write_setup_1(dt: &DateTime, w: &mut Setup1) {
    w.set_dotw(dt.day_of_week as u8);
    w.set_hour(dt.hour);
    w.set_min(dt.minute);
    w.set_sec(dt.second);
}

pub(super) fn datetime_from_registers(rtc_0: Rtc0, rtc_1: Rtc1) -> Result<DateTime, Error> {
    let year = rtc_1.year();
    let month = rtc_1.month();
    let day = rtc_1.day();

    let day_of_week = rtc_0.dotw();
    let hour = rtc_0.hour();
    let minute = rtc_0.min();
    let second = rtc_0.sec();

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
