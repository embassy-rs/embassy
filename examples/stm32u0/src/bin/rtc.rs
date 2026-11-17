#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_time::Timer;
use jiff::civil::{DateTime, date};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.sys = Sysclk::Pll1R;
        config.rcc.hsi = true;
        config.rcc.pll = Some(Pll {
            source: PllSource::Hsi, // 16 MHz
            prediv: PllPreDiv::Div1,
            mul: PllMul::Mul7, // 16 * 7 = 112 MHz
            divp: None,
            divq: None,
            divr: Some(PllRDiv::Div2), // 112 / 2 = 56 MHz
        });
        config.rcc.ls = LsConfig::default();
    }

    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let now = date(2020, 5, 15).at(10, 30, 15, 0);

    let (mut rtc, time_provider) = Rtc::new(p.RTC, RtcConfig::default());
    info!("Got RTC!");

    rtc.set(now).unwrap();

    // In reality the delay would be much longer
    Timer::after_millis(20000).await;

    let then = time_provider.read().unwrap();
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
