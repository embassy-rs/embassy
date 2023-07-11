#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::rtc::{DateTime, DayOfWeek, Rtc};
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

    let mut rtc = Rtc::new(p.RTC);
    if rtc.set_datetime(now).is_ok() {
        // In reality the delay would be much longer
        Timer::after(Duration::from_millis(20000)).await;

        if let Ok(dt) = rtc.now() {
            info!(
                "Now: {}-{}-{} {}:{}:{}",
                dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
            );
        }
    }
    info!("Done.");
}
