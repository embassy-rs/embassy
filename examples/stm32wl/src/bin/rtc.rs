#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, SharedData};
use embassy_time::Timer;
use jiff::civil::{DateTime, date};
use panic_probe as _;

#[unsafe(link_section = ".shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.ls = LsConfig::default_lse();
        config.rcc.hse = Some(Hse {
            freq: Hertz(32_000_000),
            mode: HseMode::Bypass,
            prescaler: HsePrescaler::Div1,
        });
        config.rcc.sys = Sysclk::Pll1R;
        config.rcc.pll = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div2,
            mul: PllMul::Mul6,
            divp: None,
            divq: Some(PllQDiv::Div2), // PLL1_Q clock (32 / 2 * 6 / 2), used for RNG
            divr: Some(PllRDiv::Div2), // sysclk 48Mhz clock (32 / 2 * 6 / 2)
        });
    }
    let p = embassy_stm32::init_primary(config, &SHARED_DATA);
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
