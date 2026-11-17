#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rtc::{RawDateTime, Rtc, RtcConfig};
use embassy_time::Timer;
use panic_probe as _;

pub fn pll_init(_config: &mut Config) {
    // voltage scale for max performance
    // route PLL1_P into the USB‐OTG‐HS block
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    pll_init(&mut config);

    let p = embassy_stm32::init(config);

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());

    // Setting datetime
    // The raw API does no validation: these values are written to the RTC registers as is.
    let initial_datetime = RawDateTime {
        year: 0, // 2000
        month: 1,
        day: 1,
        day_of_week: 6, // Saturday
        hour: 0,
        minute: 0,
        second: 0,
        subsecond: 0,
    };
    rtc.0.set_raw(initial_datetime);

    // Reading datetime every 1s
    loop {
        info!("{}", rtc.1.read_raw());

        Timer::after_millis(1000).await;
    }
}
