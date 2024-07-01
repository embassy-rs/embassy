use chrono::{Datelike, Timelike};

use crate::pac::rtc::regs::{Rtc0, Rtc1, Setup0, Setup1};

/// Alias for [`chrono::NaiveDateTime`]
pub type DateTime = chrono::NaiveDateTime;
/// Alias for [`chrono::Weekday`]
pub type DayOfWeek = chrono::Weekday;

/// Errors regarding the [`DateTime`] and [`DateTimeFilter`] structs.
///
/// [`DateTimeFilter`]: struct.DateTimeFilter.html
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The [DateTime] has an invalid year. The year must be between 0 and 4095.
    InvalidYear,
    /// The [DateTime] contains an invalid date.
    InvalidDate,
    /// The [DateTime] contains an invalid time.
    InvalidTime,
}

pub(super) fn day_of_week_to_u8(dotw: DayOfWeek) -> u8 {
    dotw.num_days_from_sunday() as u8
}

pub(crate) fn validate_datetime(dt: &DateTime) -> Result<(), Error> {
    if dt.year() < 0 || dt.year() > 4095 {
        // rp2040 can't hold these years
        Err(Error::InvalidYear)
    } else {
        // The rest of the chrono date is assumed to be valid
        Ok(())
    }
}

pub(super) fn write_setup_0(dt: &DateTime, w: &mut Setup0) {
    w.set_year(dt.year() as u16);
    w.set_month(dt.month() as u8);
    w.set_day(dt.day() as u8);
}

pub(super) fn write_setup_1(dt: &DateTime, w: &mut Setup1) {
    w.set_dotw(dt.weekday().num_days_from_sunday() as u8);
    w.set_hour(dt.hour() as u8);
    w.set_min(dt.minute() as u8);
    w.set_sec(dt.second() as u8);
}

pub(super) fn datetime_from_registers(rtc_0: Rtc0, rtc_1: Rtc1) -> Result<DateTime, Error> {
    let year = rtc_1.year() as i32;
    let month = rtc_1.month() as u32;
    let day = rtc_1.day() as u32;

    let hour = rtc_0.hour() as u32;
    let minute = rtc_0.min() as u32;
    let second = rtc_0.sec() as u32;

    let date = chrono::NaiveDate::from_ymd_opt(year, month, day).ok_or(Error::InvalidDate)?;
    let time = chrono::NaiveTime::from_hms_opt(hour, minute, second).ok_or(Error::InvalidTime)?;
    Ok(DateTime::new(date, time))
}
