#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use chrono::{NaiveDate, NaiveDateTime};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{self, ClockSrc, PLLClkDiv, PLLMul, PLLSource, PLLSrcDiv};
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = {
        let mut config = Config::default();
        config.rcc.mux = ClockSrc::PLL(
            PLLSource::HSE(Hertz::mhz(8)),
            PLLClkDiv::Div2,
            PLLSrcDiv::Div1,
            PLLMul::Mul20,
            None,
        );
        config.rcc.rtc_mux = rcc::RtcClockSource::LSE32;
        embassy_stm32::init(config)
    };
    info!("Hello World!");

    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();

    let mut rtc = Rtc::new(
        p.RTC,
        RtcConfig::default().clock_config(embassy_stm32::rtc::RtcClockSource::LSE),
    );
    info!("Got RTC! {:?}", now.timestamp());

    rtc.set_datetime(now.into()).expect("datetime not set");

    // In reality the delay would be much longer
    Timer::after(Duration::from_millis(20000)).await;

    let then: NaiveDateTime = rtc.now().unwrap().into();
    info!("Got RTC! {:?}", then.timestamp());
}
