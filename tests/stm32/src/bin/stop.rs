// required-features: stop,chrono

#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use chrono::NaiveDate;
use common::*;
use cortex_m_rt::entry;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::low_power::{Executor, StopMode, reconfigure_rtc, stop_ready};
use embassy_stm32::rcc::LsConfig;
use embassy_time::Timer;

#[entry]
fn main() -> ! {
    Executor::take().run(|spawner| {
        spawner.spawn(unwrap!(async_main(spawner)));
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

    // System Clock seems cannot be greater than 16 MHz
    #[cfg(any(feature = "stm32h563zi", feature = "stm32h503rb"))]
    {
        use embassy_stm32::rcc::HSIPrescaler;
        config.rcc.hsi = Some(HSIPrescaler::DIV4); // 64 MHz HSI will need a /4
    }

    let _p = init_with_config(config);
    info!("Hello World!");

    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();

    reconfigure_rtc(|rtc| rtc.set_datetime(now.into()).expect("datetime not set"));

    spawner.spawn(task_1().unwrap());
    spawner.spawn(task_2().unwrap());
}
