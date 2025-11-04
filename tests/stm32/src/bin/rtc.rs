// required-features: chrono

#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use chrono::{NaiveDate, NaiveDateTime};
use common::*;
use defmt::assert;
use embassy_executor::Spawner;
use embassy_stm32::rcc::LsConfig;
#[cfg(feature = "stop")]
use embassy_stm32::rtc::Rtc;
#[cfg(not(feature = "stop"))]
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_time::Timer;
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = config();
    config.rcc.ls = LsConfig::default_lse();

    let p = init_with_config(config);
    info!("Hello World!");

    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();

    #[cfg(not(feature = "stop"))]
    let (mut rtc, time_provider) = Rtc::new(p.RTC, RtcConfig::default());

    #[cfg(feature = "stop")]
    let (rtc, time_provider) = Rtc::new(p.RTC);

    #[cfg(not(feature = "stop"))]
    rtc.set_datetime(now.into()).expect("datetime not set");

    #[cfg(feature = "stop")]
    critical_section::with(|cs| {
        rtc.borrow_mut(cs).set_datetime(now.into()).expect("datetime not set");
    });

    info!("Waiting 5 seconds");
    Timer::after_millis(5000).await;

    let then: NaiveDateTime = time_provider.now().unwrap().into();

    let seconds = (then - now).num_seconds();

    info!("measured = {}", seconds);

    assert!(seconds > 3 && seconds < 7);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
