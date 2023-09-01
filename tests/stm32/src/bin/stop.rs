// required-features: stop,chrono

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use chrono::NaiveDate;
use common::*;
use cortex_m_rt::entry;
use embassy_executor::Spawner;
use embassy_stm32::low_power::{stop_with_rtc, Executor};
use embassy_stm32::rcc::RtcClockSource;
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_time::{Duration, Timer};
use static_cell::make_static;

#[entry]
fn main() -> ! {
    let executor = Executor::take();
    executor.run(|spawner| {
        unwrap!(spawner.spawn(async_main(spawner)));
    });
}

#[embassy_executor::task]
async fn async_main(_spawner: Spawner) {
    let mut config = config();

    config.rcc.rtc = Some(RtcClockSource::LSI);

    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());

    rtc.set_datetime(now.into()).expect("datetime not set");

    let rtc = make_static!(rtc);

    stop_with_rtc(rtc);

    info!("Waiting...");
    Timer::after(Duration::from_secs(2)).await;
    info!("Waiting...");
    Timer::after(Duration::from_secs(3)).await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}
