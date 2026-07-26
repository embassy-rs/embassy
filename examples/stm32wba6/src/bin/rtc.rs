#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rtc::{DateTime, DayOfWeek, Rtc, RtcConfig};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

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
    let initial_datetime = DateTime::from(1970, 1, 1, DayOfWeek::Thursday, 0, 00, 00, 0).unwrap();
    match rtc.0.set_datetime(initial_datetime) {
        Ok(()) => info!("RTC set successfully."),
        Err(e) => error!("Failed to set RTC date/time: {:?}", e),
    }

    // Reading datetime every 1s
    loop {
        match rtc.1.now() {
            Ok(result) => info!("{}", result),
            Err(e) => error!("Failed to set RTC date/time: {:?}", e),
        }

        Timer::after_millis(1000).await;
    }
}
