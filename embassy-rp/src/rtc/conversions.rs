use crate::datetime::{DateTime, DayOfWeek, Error};
use crate::pac::rtc::regs::{Rtc0, Rtc1, Setup0, Setup1};

#[cfg(not(feature = "chrono"))]
fn day_of_week_from_u8(v: u8) -> Result<DayOfWeek, Error> {
    Ok(match v {
        0 => DayOfWeek::Sunday,
        1 => DayOfWeek::Monday,
        2 => DayOfWeek::Tuesday,
        3 => DayOfWeek::Wednesday,
        4 => DayOfWeek::Thursday,
        5 => DayOfWeek::Friday,
        6 => DayOfWeek::Saturday,
        _ => return Err(Error::InvalidDayOfWeek),
    })
}

pub(super) fn day_of_week_to_u8(dotw: DayOfWeek) -> u8 {
    #[cfg(feature = "chrono")]
    {
        dotw.num_days_from_sunday() as u8
    }
    #[cfg(not(feature = "chrono"))]
    {
        dotw as u8
    }
}

pub(super) fn validate_datetime(dt: &DateTime) -> Result<(), Error> {
    #[cfg(feature = "chrono")]
    {
        use chrono::Datelike;
        if dt.year() < 0 || dt.year() > 4095 {
            Err(Error::InvalidYear)
        } else {
            Ok(())
        }
    }
    #[cfg(not(feature = "chrono"))]
    {
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
}

pub(super) fn write_setup_0(dt: &DateTime, w: &mut Setup0) {
    #[cfg(feature = "chrono")]
    {
        use chrono::Datelike;
        w.set_year(dt.year() as u16);
        w.set_month(dt.month() as u8);
        w.set_day(dt.day() as u8);
    }
    #[cfg(not(feature = "chrono"))]
    {
        w.set_year(dt.year);
        w.set_month(dt.month);
        w.set_day(dt.day);
    }
}

pub(super) fn write_setup_1(dt: &DateTime, w: &mut Setup1) {
    #[cfg(feature = "chrono")]
    {
        use chrono::Timelike;
        w.set_dotw(dt.weekday().num_days_from_sunday() as u8);
        w.set_hour(dt.hour() as u8);
        w.set_min(dt.minute() as u8);
        w.set_sec(dt.second() as u8);
    }
    #[cfg(not(feature = "chrono"))]
    {
        w.set_dotw(dt.day_of_week as u8);
        w.set_hour(dt.hour);
        w.set_min(dt.minute);
        w.set_sec(dt.second);
    }
}

pub(super) fn datetime_from_registers(rtc_0: Rtc0, rtc_1: Rtc1) -> Result<DateTime, Error> {
    #[cfg(feature = "chrono")]
    {
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
    #[cfg(not(feature = "chrono"))]
    {
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
}
