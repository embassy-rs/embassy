use super::DayOfWeek;
use crate::pac::rtc::regs::{IrqSetup0, IrqSetup1};

/// A filter used for [`RealTimeClock::schedule_alarm`].
///
/// [`RealTimeClock::schedule_alarm`]: struct.RealTimeClock.html#method.schedule_alarm
#[derive(Default)]
pub struct DateTimeFilter {
    /// The year that this alarm should trigger on, `None` if the RTC alarm should not trigger on a year value.
    pub year: Option<u16>,
    /// The month that this alarm should trigger on, `None` if the RTC alarm should not trigger on a month value.
    pub month: Option<u8>,
    /// The day that this alarm should trigger on, `None` if the RTC alarm should not trigger on a day value.
    pub day: Option<u8>,
    /// The day of week that this alarm should trigger on, `None` if the RTC alarm should not trigger on a day of week value.
    pub day_of_week: Option<DayOfWeek>,
    /// The hour that this alarm should trigger on, `None` if the RTC alarm should not trigger on a hour value.
    pub hour: Option<u8>,
    /// The minute that this alarm should trigger on, `None` if the RTC alarm should not trigger on a minute value.
    pub minute: Option<u8>,
    /// The second that this alarm should trigger on, `None` if the RTC alarm should not trigger on a second value.
    pub second: Option<u8>,
}

impl DateTimeFilter {
    /// Set a filter on the given year
    pub fn year(mut self, year: u16) -> Self {
        self.year = Some(year);
        self
    }
    /// Set a filter on the given month
    pub fn month(mut self, month: u8) -> Self {
        self.month = Some(month);
        self
    }
    /// Set a filter on the given day
    pub fn day(mut self, day: u8) -> Self {
        self.day = Some(day);
        self
    }
    /// Set a filter on the given day of the week
    pub fn day_of_week(mut self, day_of_week: DayOfWeek) -> Self {
        self.day_of_week = Some(day_of_week);
        self
    }
    /// Set a filter on the given hour
    pub fn hour(mut self, hour: u8) -> Self {
        self.hour = Some(hour);
        self
    }
    /// Set a filter on the given minute
    pub fn minute(mut self, minute: u8) -> Self {
        self.minute = Some(minute);
        self
    }
    /// Set a filter on the given second
    pub fn second(mut self, second: u8) -> Self {
        self.second = Some(second);
        self
    }
}

// register helper functions
impl DateTimeFilter {
    pub(super) fn write_setup_0(&self, w: &mut IrqSetup0) {
        if let Some(year) = self.year {
            w.set_year_ena(true);

            w.set_year(year);
        }
        if let Some(month) = self.month {
            w.set_month_ena(true);
            w.set_month(month);
        }
        if let Some(day) = self.day {
            w.set_day_ena(true);
            w.set_day(day);
        }
    }
    pub(super) fn write_setup_1(&self, w: &mut IrqSetup1) {
        if let Some(day_of_week) = self.day_of_week {
            w.set_dotw_ena(true);
            let bits = super::datetime::day_of_week_to_u8(day_of_week);

            w.set_dotw(bits);
        }
        if let Some(hour) = self.hour {
            w.set_hour_ena(true);
            w.set_hour(hour);
        }
        if let Some(minute) = self.minute {
            w.set_min_ena(true);
            w.set_min(minute);
        }
        if let Some(second) = self.second {
            w.set_sec_ena(true);
            w.set_sec(second);
        }
    }
}
