#![no_std]
#![no_main]

use chrono::{NaiveDate, NaiveDateTime};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux = ClockSrc::PLL1_R;
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(8),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL20,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2), // sysclk 80Mhz clock (8 / 1 * 20 / 2)
        });
        config.rcc.ls = LsConfig::default_lse();
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());
    info!("Got RTC! {:?}", now.timestamp());

    rtc.set_datetime(now.into()).expect("datetime not set");

    // In reality the delay would be much longer
    Timer::after_millis(20000).await;

    let then: NaiveDateTime = rtc.now().unwrap().into();
    info!("Got RTC! {:?}", then.timestamp());
}
