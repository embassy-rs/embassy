// required-features: stop,chrono

#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use chrono::NaiveDate;
use common::*;
use cortex_m_rt::entry;
use embassy_executor::Spawner;
use embassy_stm32::low_power::{stop_ready, stop_with_rtc, Executor, StopMode};
use embassy_stm32::rcc::LsConfig;
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_stm32::Config;
use embassy_time::Timer;
use static_cell::StaticCell;

#[entry]
fn main() -> ! {
    Executor::take().run(|spawner| {
        unwrap!(spawner.spawn(async_main(spawner)));
    });
}

#[embassy_executor::task]
async fn task_1() {
    for _ in 0..9 {
        info!("task 1: waiting for 500ms...");
        defmt::assert!(stop_ready(StopMode::Stop2));
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn task_2() {
    for _ in 0..5 {
        info!("task 2: waiting for 1000ms...");
        defmt::assert!(stop_ready(StopMode::Stop2));
        Timer::after_millis(1000).await;
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

#[embassy_executor::task]
async fn async_main(spawner: Spawner) {
    let _ = config();

    let mut config = Config::default();
    config.rcc.ls = LsConfig::default_lse();

    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());

    rtc.set_datetime(now.into()).expect("datetime not set");

    static RTC: StaticCell<Rtc> = StaticCell::new();
    let rtc = RTC.init(rtc);

    stop_with_rtc(rtc);

    spawner.spawn(task_1()).unwrap();
    spawner.spawn(task_2()).unwrap();
}
