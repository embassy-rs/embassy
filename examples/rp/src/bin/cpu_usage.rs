//! Example demonstrating how busy CPU time may be measured in applications where a single thread based
//! executor is used.
//!
//! This example was heavily based on this blog post: https://www.giacomocaironi.dev/posts/measuring-cpu-usage-with-rust-embassy/

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, unwrap};
use embassy_executor::raw::Executor;
use embassy_time::{Duration, Instant, Ticker, Timer};
use portable_atomic::{AtomicU64, Ordering};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static SLEEP_TICKS: AtomicU64 = AtomicU64::new(0);

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let _p = embassy_rp::init(Default::default());

    // The cortex m pender expects context to be usize::MAX for a thread based executor
    let executor = EXECUTOR.init(Executor::new(usize::MAX as *mut ()));
    let spawner = executor.spawner();

    unwrap!(spawner.spawn(run_periodic_computation()));
    unwrap!(spawner.spawn(report_cpu_usage()));

    loop {
        // Sleep until woken by an event, measuring the time spent sleeping
        let before = Instant::now().as_ticks();
        cortex_m::asm::wfe();
        let after = Instant::now().as_ticks();
        SLEEP_TICKS.fetch_add(after - before, Ordering::Relaxed);
        unsafe { executor.poll() };
    }
}

#[embassy_executor::task]
async fn run_periodic_computation() {
    loop {
        info!("Starting long computation");

        // Spin-wait to simulate a long CPU computation
        embassy_time::block_for(embassy_time::Duration::from_secs(1));

        info!("Long computation done");

        Timer::after_secs(3).await;
    }
}

#[embassy_executor::task]
async fn report_cpu_usage() {
    let mut previous_tick = 0u64;
    let mut previous_sleep_tick = 0u64;

    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        ticker.next().await;

        let current_tick = Instant::now().as_ticks();
        let current_sleep_tick = SLEEP_TICKS.load(Ordering::Relaxed);

        // Calculate the ratio of time spent sleeping to total time since last report, the inverse
        // of which is the time spent busy
        let sleep_tick_difference = (current_sleep_tick - previous_sleep_tick) as f32;
        let tick_difference = (current_tick - previous_tick) as f32;
        let usage = 1f32 - sleep_tick_difference / tick_difference;

        previous_tick = current_tick;
        previous_sleep_tick = current_sleep_tick;

        info!("CPU usage: {}%", usage * 100.);
    }
}
