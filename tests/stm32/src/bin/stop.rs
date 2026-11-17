// required-features: stop,jiff

#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rcc::{LsConfig, StopMode, get_stop_mode};
use embassy_stm32::rtc::{Rtc, RtcTimeProvider};
use embassy_time::{Instant, Timer};
use jiff::civil::{DateTime, date};

/// Get whether the core is ready to enter the given stop mode.
///
/// This will return false if some peripheral driver is in use that
/// prevents entering the given stop mode.
fn stop_ready(stop_mode: StopMode) -> bool {
    critical_section::with(|cs| match get_stop_mode(cs) {
        Some(mode) => mode.at_least(stop_mode),
        None => false,
    })
}

#[embassy_executor::task]
async fn task_1() {
    for _ in 0..9 {
        info!("task 1: waiting for 500ms...");
        defmt::assert!(stop_ready(StopMode::Stop2));
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn task_2(time_provider: RtcTimeProvider, start_rtc: DateTime, start: Instant) {
    for _ in 0..5 {
        info!("task 2: waiting for 1000ms...");
        defmt::assert!(stop_ready(StopMode::Stop2));
        Timer::after_millis(1000).await;
    }

    // The time spent in stop mode is measured with the RTC, so embassy-time must track it.
    let rtc_elapsed = time_provider.read().unwrap().duration_since(start_rtc).as_micros() as i64;
    let elapsed = start.elapsed().as_micros() as i64;
    info!("elapsed: rtc {} us, embassy-time {} us", rtc_elapsed, elapsed);
    defmt::assert!((rtc_elapsed - elapsed).abs() < 20_000);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn async_main(spawner: Spawner) {
    let _ = config();

    let mut config = Config::default();
    config.rcc.ls = LsConfig::default_lse();

    // System Clock seems cannot be greater than 16 MHz
    #[cfg(any(feature = "stm32h563zi", feature = "stm32h503rb"))]
    {
        use embassy_stm32::rcc::HSIPrescaler;
        config.rcc.hsi = Some(HSIPrescaler::Div4); // 64 MHz HSI will need a /4
    }

    let p = init_with_config(config);
    info!("Hello World!");

    let now = date(2020, 5, 15).at(10, 30, 15, 0);

    let (rtc, time_provider) = Rtc::new(p.RTC);

    info!("set datetime");
    critical_section::with(|cs| {
        rtc.borrow_mut(cs).set(now).unwrap();
    });

    let start_rtc = time_provider.read().unwrap();
    let start = Instant::now();

    info!("spawn");
    spawner.spawn(task_1().unwrap());
    spawner.spawn(task_2(time_provider, start_rtc, start).unwrap());
}
