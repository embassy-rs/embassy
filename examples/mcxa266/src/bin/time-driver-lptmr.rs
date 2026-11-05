//! LPTMR timestamps and WAKETIMER one-shot alarms on FRDM-MCXA266.
//!
//! Run with:
//! `cargo run --release --features time-driver-lptmr --bin time-driver-lptmr`
//!
//! Two tasks use independent deadlines. The short waits replace an outstanding
//! long alarm without resetting the LPTMR timebase. Timestamps have 1 us units;
//! alarm resolution is approximately 1 ms, not 1 us. There is no 1 ms polling IRQ.
//! Default clocks keep the 1 MHz timebase and WAKETIMER's FRO16K source enabled.
//! This example uses normal executor sleep, not Power Down or Deep Power Down.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::{Instant, Timer};
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use panic_probe as _;

#[embassy_executor::task]
async fn long_deadlines() {
    loop {
        let start = Instant::now();
        Timer::after_secs(10).await;
        defmt::info!("Long alarm: elapsed {} us", start.elapsed().as_micros());
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());
    let mut red = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Slow);
    spawner.spawn(long_deadlines().unwrap());

    defmt::info!("LPTMR timekeeping + WAKETIMER alarms");
    loop {
        for millis in [100, 7, 1_250, 250] {
            let start = Instant::now();
            Timer::after_millis(millis).await;
            red.toggle();
            defmt::info!("Requested {} ms, elapsed {} us", millis, start.elapsed().as_micros());
        }
    }
}
