// required-features: jiff

#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::{assert, assert_eq};
use embassy_executor::Spawner;
use embassy_stm32::rcc::LsConfig;
#[cfg(feature = "stop")]
use embassy_stm32::rtc::AnyRtc;
#[cfg(not(feature = "stop"))]
use embassy_stm32::rtc::RtcConfig;
use embassy_stm32::rtc::{RawDateTime, Rtc};
use embassy_time::Timer;
use jiff::civil::date;

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let mut config = config();
    config.rcc.ls = LsConfig::default_lse();

    let p = init_with_config(config);
    info!("Hello World!");

    #[cfg(not(feature = "stop"))]
    let (mut rtc, time_provider) = Rtc::new(p.RTC, RtcConfig::default());

    #[cfg(feature = "stop")]
    let (mut rtc, time_provider) = Rtc::new(p.RTC);

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
    let read = time_provider.read_raw();
    info!("raw: {}", read);
    assert_eq!(RawDateTime { subsecond: 0, ..read }, raw);

    let now = date(2020, 5, 15).at(10, 30, 15, 0);
    rtc.set(now).unwrap();
    assert!(rtc.set(date(2100, 1, 1).at(0, 0, 0, 0)).is_err());

    info!("Waiting 5 seconds");
    Timer::after_millis(5000).await;

    let then = time_provider.read().unwrap();
    let seconds = then.duration_since(now).as_secs();

    info!("measured = {}", seconds);

    assert!(seconds > 3 && seconds < 7);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
