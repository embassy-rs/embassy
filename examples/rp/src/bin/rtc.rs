//! This example shows how to use RTC (Real Time Clock) in the RP2040 chip.

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
    info!("Wait for 20s");

    let mut rtc = Rtc::new(p.RTC);

    if !rtc.is_running() {
        info!("Start RTC");
        let now = DateTime {
            year: 2000,
            month: 1,
            day: 1,
            day_of_week: DayOfWeek::Saturday,
            hour: 0,
            minute: 0,
            second: 0,
        };
        rtc.set_datetime(now).unwrap();
    }

    Timer::after(Duration::from_millis(20000)).await;

    if let Ok(dt) = rtc.now() {
        info!(
            "Now: {}-{:02}-{:02} {}:{:02}:{:02}",
            dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
        );
    }

    info!("Reboot.");
    Timer::after(Duration::from_millis(200)).await;
    cortex_m::peripheral::SCB::sys_reset();
}
