use chrono::{Datelike, Timelike};

use super::DayOfWeek;

pub enum AlarmDate {
    /// 1..28,29,30,31 depending on month
    Date(u8),
    ///
    WeekDay(DayOfWeek),
}

pub struct Alarm {
    pub date: Option<AlarmDate>,
    /// None, or 0..23
    pub hour: Option<u8>,
    /// None, or 0..59
    pub minute: Option<u8>,
    /// 0..59
    pub second: Option<u8>,
    // todo: add subseconds
}

impl From<chrono::NaiveDateTime> for Alarm {
    fn from(value: chrono::NaiveDateTime) -> Self {
        let date = value.day() as u8;
        let hour = value.hour() as u8;
        let minute = value.minute() as u8;
        let second = value.second() as u8;

        Self {
            date: Some(AlarmDate::Date(date)),
            hour: Some(hour),
            minute: Some(minute),
            second: Some(second),
        }
    }
}
