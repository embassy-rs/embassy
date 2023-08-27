#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use chrono::{NaiveDate, NaiveDateTime};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rtc::{Rtc, RtcClockSource, RtcConfig};
use embassy_stm32::Config;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.rtc = Option::Some(RtcClockSource::LSI);
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());

    rtc.set_datetime(now.into()).expect("datetime not set");

    loop {
        let now: NaiveDateTime = rtc.now().unwrap().into();

        info!("{}", now.timestamp());

        Timer::after(Duration::from_millis(1000)).await;
    }
}
