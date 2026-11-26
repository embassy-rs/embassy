#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rcc::LsConfig;
use embassy_stm32::rtc::Rtc;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.ls = LsConfig::default_lse();
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    // The counter is kept across resets, and while powered from VBAT.
    let rtc = Rtc::new(p.RTC);
    info!("counter at boot: {}", rtc.counter());

    loop {
        info!("counter: {}", rtc.counter());
        Timer::after_millis(1000).await;
    }
}
