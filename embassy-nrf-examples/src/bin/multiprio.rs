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

use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use defmt::panic;
use nrf52840_hal::clocks;

use embassy::executor::{task, Executor, IrqExecutor};
use embassy::time::{Duration, Instant, Timer};
use embassy::util::Forever;
use embassy_nrf::interrupt::OwnedInterrupt;
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
static ALARM_HIGH: Forever<rtc::Alarm<pac::RTC1>> = Forever::new();
static EXECUTOR_HIGH: Forever<IrqExecutor<interrupt::SWI1_EGU1Interrupt>> = Forever::new();
static ALARM_MED: Forever<rtc::Alarm<pac::RTC1>> = Forever::new();
static EXECUTOR_MED: Forever<IrqExecutor<interrupt::SWI0_EGU0Interrupt>> = Forever::new();
static ALARM_LOW: Forever<rtc::Alarm<pac::RTC1>> = Forever::new();
static EXECUTOR_LOW: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = unwrap!(embassy_nrf::pac::Peripherals::take());

    clocks::Clocks::new(p.CLOCK)
        .enable_ext_hfosc()
        .set_lfclk_src_external(clocks::LfOscConfiguration::NoExternalNoBypass)
        .start_lfclk();

    let rtc = RTC.put(rtc::RTC::new(p.RTC1, interrupt::take!(RTC1)));
    rtc.start();
    unsafe { embassy::time::set_clock(rtc) };

    // High-priority executor: SWI1_EGU1, priority level 6
    let irq = interrupt::take!(SWI1_EGU1);
    irq.set_priority(interrupt::Priority::Level6);
    let alarm = ALARM_HIGH.put(rtc.alarm2());
    let executor = EXECUTOR_HIGH.put(IrqExecutor::new(irq));
    executor.set_alarm(alarm);
    executor.start(|spawner| {
        unwrap!(spawner.spawn(run_high()));
    });

    // Medium-priority executor: SWI0_EGU0, priority level 7
    let irq = interrupt::take!(SWI0_EGU0);
    irq.set_priority(interrupt::Priority::Level7);
    let alarm = ALARM_MED.put(rtc.alarm1());
    let executor = EXECUTOR_MED.put(IrqExecutor::new(irq));
    executor.set_alarm(alarm);
    executor.start(|spawner| {
        unwrap!(spawner.spawn(run_med()));
    });

    // Low priority executor: runs in thread mode, using WFE/SEV
    let alarm = ALARM_LOW.put(rtc.alarm0());
    let executor = EXECUTOR_LOW.put(Executor::new());
    executor.set_alarm(alarm);
    executor.run(|spawner| {
        unwrap!(spawner.spawn(run_low()));
    });
}
