//! Example demonstrating how busy CPU time may be measured in applications where one thread based
//! executor per core is used.
//!
//! This example was heavily based on this blog post: https://www.giacomocaironi.dev/posts/measuring-cpu-usage-with-rust-embassy/

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::raw::Executor;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_time::{Duration, Instant, Ticker, Timer};
use portable_atomic::{AtomicU64, Ordering};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static CORE0_SLEEP_TICKS: AtomicU64 = AtomicU64::new(0);
static CORE1_SLEEP_TICKS: AtomicU64 = AtomicU64::new(0);

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_rp::init(Default::default());

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            // The cortex m pender expects context to be usize::MAX for a thread based executor
            let executor1 = EXECUTOR1.init(Executor::new(usize::MAX as *mut ()));
            let spawner = executor1.spawner();
            unwrap!(spawner.spawn(core1_task()));

            loop {
                // Sleep until woken by an event, measuring the time spent sleeping
                let before = Instant::now().as_ticks();
                cortex_m::asm::wfe();
                let after = Instant::now().as_ticks();
                CORE1_SLEEP_TICKS.fetch_add(after - before, Ordering::Relaxed);
                unsafe { executor1.poll() };
            }
        },
    );

    // The cortex m pender expects context to be usize::MAX for a thread based executor
    let executor0 = EXECUTOR0.init(Executor::new(usize::MAX as *mut ()));
    let spawner = executor0.spawner();
    unwrap!(spawner.spawn(core0_task()));
    unwrap!(spawner.spawn(report_cpu_usage()));

    loop {
        // Sleep until woken by an event, measuring the time spent sleeping
        let before = Instant::now().as_ticks();
        cortex_m::asm::wfe();
        let after = Instant::now().as_ticks();
        CORE0_SLEEP_TICKS.fetch_add(after - before, Ordering::Relaxed);
        unsafe { executor0.poll() };
    }
}

#[embassy_executor::task]
async fn core0_task() {
    loop {
        info!("[core 0] starting work");

        // Spin-wait to simulate a long CPU computation
        embassy_time::block_for(embassy_time::Duration::from_millis(500));

        Timer::after_secs(3).await;
    }
}

#[embassy_executor::task]
async fn core1_task() {
    loop {
        info!("[core 1] starting work");

        // Spin-wait to simulate a long CPU computation
        embassy_time::block_for(embassy_time::Duration::from_millis(1500));

        Timer::after_secs(3).await;
    }
}

#[embassy_executor::task]
async fn report_cpu_usage() {
    let mut previous_tick = 0u64;
    let mut previous_sleep_tick_core0 = 0u64;
    let mut previous_sleep_tick_core1 = 0u64;

    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        ticker.next().await;

        let current_tick = Instant::now().as_ticks();
        let current_sleep_tick_core0 = CORE0_SLEEP_TICKS.load(Ordering::Relaxed);
        let current_sleep_tick_core1 = CORE1_SLEEP_TICKS.load(Ordering::Relaxed);

        let tick_difference = (current_tick - previous_tick) as f32;

        // Calculate the ratio of time spent sleeping to total time since last report, the inverse
        // of which is the time spent busy
        let calc_cpu_usage = |current_sleep_tick: u64, previous_sleep_tick: u64| -> f32 {
            let sleep_tick_difference = (current_sleep_tick - previous_sleep_tick) as f32;
            let usage = 1f32 - sleep_tick_difference / tick_difference;
            usage
        };
        let usage_core0 = calc_cpu_usage(current_sleep_tick_core0, previous_sleep_tick_core0);
        let usage_core1 = calc_cpu_usage(current_sleep_tick_core1, previous_sleep_tick_core1);

        previous_tick = current_tick;
        previous_sleep_tick_core0 = current_sleep_tick_core0;
        previous_sleep_tick_core1 = current_sleep_tick_core1;

        info!(
            "CPU usage: core0 {}%, core1 {}%",
            usage_core0 * 100.,
            usage_core1 * 100.
        );
    }
}
