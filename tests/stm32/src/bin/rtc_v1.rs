#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::{assert, assert_eq};
use embassy_executor::Spawner;
use embassy_stm32::rcc::LsConfig;
use embassy_stm32::rtc::Rtc;
use embassy_time::{Instant, Timer};

teleprobe_meta::timeout!(30);

/// Busy-wait for the next counter increment.
fn next_tick(rtc: &Rtc) -> Instant {
    let c = rtc.counter();
    while rtc.counter() == c {}
    Instant::now()
}

#[embassy_executor::main(executor = "common::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut config = config();
    config.rcc.ls = LsConfig::default_lse();
    let mut p = init_with_config(config);
    info!("Hello World!");

    let mut rtc = Rtc::new(p.RTC.reborrow());
    info!("counter at boot: {}", rtc.counter());

    rtc.set_counter(1000);
    assert_eq!(rtc.counter(), 1000);

    // Setting the counter restarts the second.
    Timer::after_millis(900).await;
    assert_eq!(rtc.counter(), 1000);
    Timer::after_millis(200).await;
    assert_eq!(rtc.counter(), 1001);

    // Carry from the low into the high half of the counter.
    rtc.set_counter(0xFFFF);
    Timer::after_millis(1100).await;
    assert_eq!(rtc.counter(), 0x1_0000);

    // The counter wraps.
    rtc.set_counter(u32::MAX);
    Timer::after_millis(1100).await;
    assert_eq!(rtc.counter(), 0);

    // Creating the driver again doesn't restart the second.
    rtc.set_counter(5);
    Timer::after_millis(600).await;
    drop(rtc);
    let rtc = Rtc::new(p.RTC.reborrow());
    assert_eq!(rtc.counter(), 5);
    Timer::after_millis(500).await;
    assert_eq!(rtc.counter(), 6);

    // The counter increments once per second.
    let start = next_tick(&rtc);
    for _ in 0..2 {
        next_tick(&rtc);
    }
    let ms = next_tick(&rtc).duration_since(start).as_millis();
    info!("3 ticks took {} ms", ms);
    assert!((2990..=3010).contains(&ms));

    info!("Test OK");
    cortex_m::asm::bkpt();
}
