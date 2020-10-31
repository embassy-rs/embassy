//! This example showcases how to create multiple Executor instances to run tasks at
//! different priority levels.
//!
//! Low priority executor runs in thread mode (not interrupt), and uses `sev` for signaling
//! there's work in the queue, and `wfe` for waiting for work.
//!
//! Medium and high priority executors run in two interrupts with different priorities.
//! Signaling work is done by pending the interrupt. No "waiting" needs to be done explicitly, since
//! when there's work the interrupt will trigger and run the executor.
//!
//! Sample output below. Note that high priority ticks can interrupt everything else, and
//! medium priority computations can interrupt low priority computations, making them to appear
//! to take significantly longer time.
//!
//! ```not_rust
//!     [med] Starting long computation
//!     [med] done in 992 ms
//!         [high] tick!
//! [low] Starting long computation
//!     [med] Starting long computation
//!         [high] tick!
//!         [high] tick!
//!     [med] done in 993 ms
//!     [med] Starting long computation
//!         [high] tick!
//!         [high] tick!
//!     [med] done in 993 ms
//! [low] done in 3972 ms
//!     [med] Starting long computation
//!         [high] tick!
//!         [high] tick!
//!     [med] done in 993 ms
//! ```
//!
//! For comparison, try changing the code so all 3 tasks get spawned on the low priority executor.
//! You will get an output like the following. Note that no computation is ever interrupted.
//!
//! ```not_rust
//!         [high] tick!
//!     [med] Starting long computation
//!     [med] done in 496 ms
//! [low] Starting long computation
//! [low] done in 992 ms
//!     [med] Starting long computation
//!     [med] done in 496 ms
//!         [high] tick!
//! [low] Starting long computation
//! [low] done in 992 ms
//!         [high] tick!
//!     [med] Starting long computation
//!     [med] done in 496 ms
//!         [high] tick!
//! ```
//!

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use nrf52840_hal::clocks;

use embassy::executor::{task, TimerExecutor};
use embassy::time::{Duration, Instant, Timer};
use embassy::util::Forever;
use embassy_nrf::{interrupt, pac, rtc};

#[task]
async fn run_high() {
    loop {
        info!("        [high] tick!");
        Timer::after(Duration::from_ticks(27374)).await;
    }
}

#[task]
async fn run_med() {
    loop {
        let start = Instant::now();
        info!("    [med] Starting long computation");

        // Spin-wait to simulate a long CPU computation
        cortex_m::asm::delay(32_000_000); // ~1 second

        let end = Instant::now();
        let ms = end.duration_since(start).as_ticks() / 33;
        info!("    [med] done in {:u64} ms", ms);

        Timer::after(Duration::from_ticks(23421)).await;
    }
}

#[task]
async fn run_low() {
    loop {
        let start = Instant::now();
        info!("[low] Starting long computation");

        // Spin-wait to simulate a long CPU computation
        cortex_m::asm::delay(64_000_000); // ~2 seconds

        let end = Instant::now();
        let ms = end.duration_since(start).as_ticks() / 33;
        info!("[low] done in {:u64} ms", ms);

        Timer::after(Duration::from_ticks(32983)).await;
    }
}

static RTC: Forever<rtc::RTC<pac::RTC1>> = Forever::new();
static EXECUTOR_LOW: Forever<TimerExecutor<rtc::Alarm<pac::RTC1>>> = Forever::new();
static EXECUTOR_MED: Forever<TimerExecutor<rtc::Alarm<pac::RTC1>>> = Forever::new();
static EXECUTOR_HIGH: Forever<TimerExecutor<rtc::Alarm<pac::RTC1>>> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_nrf::pac::Peripherals::take().dewrap();

    clocks::Clocks::new(p.CLOCK)
        .enable_ext_hfosc()
        .set_lfclk_src_external(clocks::LfOscConfiguration::NoExternalNoBypass)
        .start_lfclk();

    let rtc = RTC.put(rtc::RTC::new(p.RTC1));
    rtc.start();
    unsafe { embassy::time::set_clock(rtc) };

    let executor_low = EXECUTOR_LOW.put(TimerExecutor::new(rtc.alarm0(), cortex_m::asm::sev));
    let executor_med = EXECUTOR_MED.put(TimerExecutor::new(rtc.alarm1(), cortex_m::asm::sev));
    let executor_high = EXECUTOR_HIGH.put(TimerExecutor::new(rtc.alarm2(), cortex_m::asm::sev));

    interrupt::set_priority(interrupt::SWI0_EGU0, interrupt::Priority::Level7);
    interrupt::set_priority(interrupt::SWI1_EGU1, interrupt::Priority::Level6);
    interrupt::enable(interrupt::SWI0_EGU0);
    interrupt::enable(interrupt::SWI1_EGU1);

    executor_low.spawn(run_low()).dewrap();
    executor_med.spawn(run_med()).dewrap();
    executor_high.spawn(run_high()).dewrap();

    loop {
        executor_low.run();
        cortex_m::asm::wfe();
    }
}

#[interrupt]
unsafe fn SWI0_EGU0() {
    EXECUTOR_MED.steal().run()
}

#[interrupt]
unsafe fn SWI1_EGU1() {
    EXECUTOR_HIGH.steal().run()
}
