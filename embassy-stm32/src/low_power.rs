use crate::rtc::{Rtc, RtcInstant};

static mut RTC: Option<&'static Rtc> = None;

pub fn stop_with_rtc(rtc: &'static Rtc) {
    unsafe { RTC = Some(rtc) };
}

pub fn start_wakeup_alarm(requested_duration: embassy_time::Duration) -> RtcInstant {
    unsafe { RTC }.unwrap().start_wakeup_alarm(requested_duration)
}

pub fn stop_wakeup_alarm() -> RtcInstant {
    unsafe { RTC }.unwrap().stop_wakeup_alarm()
}
