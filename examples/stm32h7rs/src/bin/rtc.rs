#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rcc::LsConfig;
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_time::Timer;
use jiff::civil::{DateTime, date};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.ls = LsConfig::default_lse();

    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let now = date(2020, 5, 15).at(10, 30, 15, 0);

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());
    info!("Got RTC!");

    rtc.set(now).unwrap();

    // In reality the delay would be much longer
    Timer::after_millis(20000).await;

    let then = rtc.read().unwrap();
    print_datetime(then);
}

fn print_datetime(t: DateTime) {
    info!(
        "{}-{}-{} {}:{}:{}",
        t.year(),
        t.month(),
        t.day(),
        t.hour(),
        t.minute(),
        t.second()
    );
}
