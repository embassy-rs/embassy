#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());
    info!("nRF initialized");

    // Phase 1: tight-loop Instant::now() to stress syscounter().
    info!("Phase 1: 100_000 Instant::now() calls");
    let start = Instant::now();
    info!("  first now() = {} us", start.as_micros());
    for i in 1..=100_000u32 {
        let _now = Instant::now();
        if i % 10_000 == 0 {
            info!("  iter {}", i);
        }
    }
    let phase1_elapsed = (Instant::now() - start).as_micros();
    info!("Phase 1 OK ({} us)", phase1_elapsed);

    // Phase 2: scheduled medium timer to stress set_alarm() once.
    info!("Phase 2: 100 ms timer");
    let t0 = Instant::now();
    Timer::after_millis(100).await;
    let slept = (Instant::now() - t0).as_millis();
    info!("Phase 2 OK (slept {} ms)", slept);
    defmt::assert!(slept >= 99 && slept <= 110);

    // Phase 3: short-timer stress to drive set_alarm() repeatedly.
    info!("Phase 3: 2_000 x 500 us timers");
    for i in 1..=2_000u32 {
        Timer::after_micros(500).await;
        if i % 200 == 0 {
            info!("  iter {}", i);
        }
    }
    info!("Phase 3 OK");

    loop {
        Timer::after_secs(500).await;
    }
}
