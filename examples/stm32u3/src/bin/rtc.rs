#![no_std]
#![no_main]

use chrono::{NaiveDate, NaiveDateTime};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rtc::Rtc;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main()]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    info!("Hello World!");

    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();

    let (rtc, time_provider) = Rtc::new(p.RTC);
    info!("Got RTC! {:?}", now.and_utc().timestamp());

    critical_section::with(|cs| {
        rtc.borrow_mut(cs).set_datetime(now.into()).expect("datetime not set");
    });

    // In reality the delay would be much longer
    Timer::after_millis(20000).await;

    let then: NaiveDateTime = time_provider.now().unwrap().into();
    info!("Got RTC! {:?}", then.and_utc().timestamp());
}
