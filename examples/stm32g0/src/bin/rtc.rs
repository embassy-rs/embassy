#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rtc::{RawDateTime, Rtc, RtcConfig};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    // The raw API does no validation: these values are written to the RTC registers as is.
    let now = RawDateTime {
        year: 23, // 2023
        month: 6,
        day: 14,
        day_of_week: 3, // Wednesday
        hour: 15,
        minute: 59,
        second: 10,
        subsecond: 0,
    };

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());

    rtc.set_raw(now);

    loop {
        let now = rtc.read_raw();

        info!("{}:{}:{}", now.hour, now.minute, now.second);

        Timer::after_millis(1000).await;
    }
}
