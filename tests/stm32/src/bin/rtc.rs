// required-features: chrono

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use chrono::{NaiveDate, NaiveDateTime};
use common::*;
use defmt::assert;
use embassy_executor::Spawner;
use embassy_stm32::pac;
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_time::{Duration, Timer};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();

    info!("Starting LSI");

    pac::RCC.csr().modify(|w| w.set_lsion(true));
    while !pac::RCC.csr().read().lsirdy() {}

    info!("Started LSI");

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());

    rtc.set_datetime(now.into()).expect("datetime not set");

    info!("Waiting 5 seconds");
    Timer::after(Duration::from_millis(5000)).await;

    let then: NaiveDateTime = rtc.now().unwrap().into();
    let seconds = (then - now).num_seconds();

    defmt::info!("measured = {}", seconds);

    assert!(seconds > 3 && seconds < 7);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
