// required-features: jiff

#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::{assert, assert_eq};
use embassy_executor::Spawner;
use embassy_stm32::rcc::LsConfig;
use embassy_stm32::rtc::{RawDateTime, Rtc, RtcConfig};
use embassy_time::Timer;
use jiff::civil::date;

#[embassy_executor::main(executor = "common::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut config = config();
    config.rcc.ls = LsConfig::default_lse();

    let p = init_with_config(config);
    info!("Hello World!");

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());

    // The raw API keeps the day of week, even when it doesn't match the date.
    let raw = RawDateTime {
        year: 20,
        month: 5,
        day: 15,
        day_of_week: 2,
        hour: 10,
        minute: 30,
        second: 15,
        subsecond: 0,
    };
    rtc.set_raw(raw);
    let read = rtc.read_raw();
    info!("raw: {}", read);
    assert_eq!(RawDateTime { subsecond: 0, ..read }, raw);

    // The day of week is incremented with the date, wrapping from 7 to 1. Setting the calendar
    // restarts the second, so the tick is 1 s later.
    rtc.set_raw(RawDateTime {
        year: 20,
        month: 12,
        day: 31,
        day_of_week: 7,
        hour: 23,
        minute: 59,
        second: 59,
        subsecond: 0,
    });
    Timer::after_millis(1100).await;
    let read = rtc.read_raw();
    info!("rollover: {}", read);
    assert_eq!(
        RawDateTime { subsecond: 0, ..read },
        RawDateTime {
            year: 21,
            month: 1,
            day: 1,
            day_of_week: 1,
            hour: 0,
            minute: 0,
            second: 0,
            subsecond: 0,
        }
    );

    // `read` rejects a date that doesn't exist.
    rtc.set_raw(RawDateTime {
        day: 30,
        month: 2,
        ..raw
    });
    assert!(rtc.read().is_err());

    let now = date(2020, 5, 15).at(10, 30, 15, 0);
    rtc.set(now).unwrap();
    assert!(rtc.set(date(2100, 1, 1).at(0, 0, 0, 0)).is_err());

    info!("Waiting 5 seconds");
    Timer::after_millis(5000).await;

    // An `RtcReader` reads the same calendar.
    let then = rtc.reader().read().unwrap();
    let seconds = then.duration_since(now).as_secs();

    info!("measured = {}", seconds);

    assert!(seconds > 3 && seconds < 7);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
