use chrono::{Datelike, Timelike};

use super::byte_to_bcd2;
use crate::pac::rtc::Rtc;

/// Alias for [`chrono::NaiveDateTime`]
pub type DateTime = chrono::NaiveDateTime;
/// Alias for [`chrono::Weekday`]
pub type DayOfWeek = chrono::Weekday;

/// Errors regarding the [`DateTime`] and [`DateTimeFilter`] structs.
///
/// [`DateTimeFilter`]: struct.DateTimeFilter.html
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The [DateTime] has an invalid year. The year must be between 0 and 4095.
    InvalidYear,
    /// The [DateTime] contains an invalid date.
    InvalidDate,
    /// The [DateTime] contains an invalid time.
    InvalidTime,
}

pub(super) fn day_of_week_to_u8(dotw: DayOfWeek) -> u8 {
    dotw.num_days_from_monday() as u8
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

pub(super) fn write_date_time(rtc: &Rtc, t: DateTime) {
    let (ht, hu) = byte_to_bcd2(t.hour() as u8);
    let (mnt, mnu) = byte_to_bcd2(t.minute() as u8);
    let (st, su) = byte_to_bcd2(t.second() as u8);

    let (dt, du) = byte_to_bcd2(t.day() as u8);
    let (mt, mu) = byte_to_bcd2(t.month() as u8);
    let yr = t.year() as u16;
    let yr_offset = (yr - 1970_u16) as u8;
    let (yt, yu) = byte_to_bcd2(yr_offset);

    unsafe {
        rtc.tr().write(|w| {
            w.set_ht(ht);
            w.set_hu(hu);
            w.set_mnt(mnt);
            w.set_mnu(mnu);
            w.set_st(st);
            w.set_su(su);
            w.set_pm(stm32_metapac::rtc::vals::Ampm::AM);
        });

        rtc.dr().write(|w| {
            w.set_dt(dt);
            w.set_du(du);
            w.set_mt(mt > 0);
            w.set_mu(mu);
            w.set_yt(yt);
            w.set_yu(yu);
            w.set_wdu(day_of_week_to_u8(t.weekday()));
        });
    }
}

pub(super) fn datetime(
    year: u16,
    month: u8,
    day: u8,
    _day_of_week: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> Result<DateTime, Error> {
    let date = chrono::NaiveDate::from_ymd_opt(year.into(), month.try_into().unwrap(), day.into())
        .ok_or(Error::InvalidDate)?;
    let time = chrono::NaiveTime::from_hms_opt(hour.into(), minute.into(), second.into()).ok_or(Error::InvalidTime)?;
    Ok(DateTime::new(date, time))
}
