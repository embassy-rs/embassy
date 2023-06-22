#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::rtc::{DateTime, DayOfWeek, RealTimeClock};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let now = DateTime {
        year: 2020,
        month: 5,
        day: 15,
        day_of_week: DayOfWeek::Monday,
        hour: 10,
        minute: 30,
        second: 50,
    };

    let rtc_result = RealTimeClock::new(p.RTC, now);
    if let Ok(rtc) = rtc_result {
        // In reality the delay would be much longer
        Timer::after(Duration::from_millis(20000)).await;

        let _then: DateTime = rtc.now().unwrap();
    }
}
