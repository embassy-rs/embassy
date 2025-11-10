#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rtc::{DateTime, DayOfWeek, Rtc, RtcConfig};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let now = DateTime::from(2023, 6, 14, DayOfWeek::Friday, 15, 59, 10, 0);

    let (mut rtc, time_provider) = Rtc::new(p.RTC, RtcConfig::default());

    rtc.set_datetime(now.unwrap()).expect("datetime not set");

    loop {
        let now: DateTime = time_provider.now().unwrap().into();

        info!("{}:{}:{}", now.hour(), now.minute(), now.second());

        Timer::after_millis(1000).await;
    }
}
