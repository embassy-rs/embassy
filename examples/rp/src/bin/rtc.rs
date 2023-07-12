#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::pac::rtc::regs::{Rtc0, Rtc1};
use embassy_rp::rtc::{DateTime, DayOfWeek, Rtc};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Wait for 20s");

    let mut watchdog = embassy_rp::watchdog::Watchdog::new(p.WATCHDOG);
    let mut rtc = Rtc::new(p.RTC);

    let rtc0 = Rtc0(watchdog.get_scratch0());
    let rtc1 = Rtc1(watchdog.get_scratch1());
    if rtc1.year() >= 2020 {
        rtc.restore(rtc1, rtc0);
    } else {
        let now = DateTime {
            year: 2020,
            month: 5,
            day: 15,
            day_of_week: DayOfWeek::Monday,
            hour: 10,
            minute: 30,
            second: 50,
        };
        rtc.set_datetime(now).unwrap();
    }

    Timer::after(Duration::from_millis(20000)).await;

    if let Ok(dt) = rtc.now() {
        info!(
            "Now: {}-{:02}-{:02} {}:{:02}:{:02}",
            dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
        );
        let (rtc1, rtc0) = rtc.save();
        watchdog.set_scratch0(rtc0.0);
        watchdog.set_scratch1(rtc1.0);
    }

    info!("Reboot.");
    Timer::after(Duration::from_millis(200)).await;
    cortex_m::peripheral::SCB::sys_reset();
}
