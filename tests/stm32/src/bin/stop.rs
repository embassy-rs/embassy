// required-features: stop,jiff

//! Stop mode with the RTC and TIM time driver.
//!
//! Every board runs from a PLL where it has one, so a clock not restored after Stop shows up: the
//! timer then counts at the wrong rate and embassy-time drifts from the RTC.
#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{LsConfig, StopMode, get_stop_mode};
use embassy_stm32::rtc::{Rtc, RtcTimeProvider};
use embassy_time::{Duration, Instant, Timer, block_for};
use jiff::civil::{DateTime, date};

teleprobe_meta::timeout!(60);

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
        assert!(stop_ready(StopMode::Stop2));
        Timer::after_millis(500).await;
        // Run on the restored clocks for a while: if they're wrong, the timer drifts from the RTC.
        block_for(Duration::from_millis(20));
    }
}

#[embassy_executor::task]
async fn task_2(time_provider: RtcTimeProvider, start_rtc: DateTime, start: Instant) {
    for _ in 0..5 {
        info!("task 2: waiting for 1000ms...");
        assert!(stop_ready(StopMode::Stop2));
        Timer::after_millis(1000).await;
        block_for(Duration::from_millis(20));
    }

    // The time spent in stop mode is measured with the RTC, so embassy-time must track it.
    let rtc_elapsed = time_provider.read().unwrap().duration_since(start_rtc).as_micros() as i64;
    let elapsed = start.elapsed().as_micros() as i64;
    info!("elapsed: rtc {} us, embassy-time {} us", rtc_elapsed, elapsed);
    assert!((rtc_elapsed - elapsed).abs() < 20_000);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let mut config = config();
    config.rcc.ls = LsConfig::default_lse();

    // CRS would keep the chip out of Stop 2.
    #[cfg(any(feature = "stm32u585ai", feature = "stm32u5a5zj", feature = "stm32u083rc"))]
    {
        config.rcc.hsi48 = Some(embassy_stm32::rcc::Hsi48Config { sync_from_usb: false });
    }

    // The other tests run WBA from HSI. Run from the PLL here, so it has to be restored after Stop.
    #[cfg(any(feature = "stm32wba52cg", feature = "stm32wba65ri"))]
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            prescaler: HsePrescaler::Div1,
            trim: None,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div2, // 16 MHz
            mul: PllMul::Mul30,      // 480 MHz
            divp: None,
            divq: None,
            divr: Some(PllDiv::Div5), // 96 MHz
            frac: None,
        });
        config.rcc.sys = Sysclk::Pll1R;
        config.rcc.ahb5_pre = AHB5Prescaler::Div4; // 24 MHz
        config.rcc.voltage_scale = VoltageScale::Range1;
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
