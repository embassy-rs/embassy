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

use cortex_m_rt::entry;
use defmt::*;
use embassy_executor::{Executor, InterruptExecutor};
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::{InterruptExt, Priority};
use embassy_time::{Instant, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn run_high() {
    loop {
        // info!("        [high] tick!");
        Timer::after_ticks(27374).await;
    }
}

#[embassy_executor::task]
async fn run_med() {
    loop {
        let start = Instant::now();
        info!("    [med] Starting long computation");

        // Spin-wait to simulate a long CPU computation
        cortex_m::asm::delay(8_000_000); // ~1 second

        let end = Instant::now();
        let ms = end.duration_since(start).as_ticks() / 33;
        info!("    [med] done in {} ms", ms);

        Timer::after_ticks(23421).await;
    }
}

#[embassy_executor::task]
async fn run_low() {
    loop {
        let start = Instant::now();
        info!("[low] Starting long computation");

        // Spin-wait to simulate a long CPU computation
        cortex_m::asm::delay(16_000_000); // ~2 seconds

        let end = Instant::now();
        let ms = end.duration_since(start).as_ticks() / 33;
        info!("[low] done in {} ms", ms);

        Timer::after_ticks(32983).await;
    }
}

static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_MED: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_LOW: StaticCell<Executor> = StaticCell::new();

#[interrupt]
unsafe fn USART1() {
    EXECUTOR_HIGH.on_interrupt()
}

#[interrupt]
unsafe fn USART2() {
    EXECUTOR_MED.on_interrupt()
}

#[entry]
fn main() -> ! {
    // Initialize and create handle for devicer peripherals
    let _p = embassy_stm32::init(Default::default());

    // High-priority executor: USART1, priority level 6
    interrupt::USART1.set_priority(Priority::P6);
    let spawner = EXECUTOR_HIGH.start(interrupt::USART1);
    unwrap!(spawner.spawn(run_high()));

    // Medium-priority executor: USART2, priority level 7
    interrupt::USART2.set_priority(Priority::P7);
    let spawner = EXECUTOR_MED.start(interrupt::USART2);
    unwrap!(spawner.spawn(run_med()));

    // Low priority executor: runs in thread mode, using WFE/SEV
    let executor = EXECUTOR_LOW.init(Executor::new());
    executor.run(|spawner| {
        unwrap!(spawner.spawn(run_low()));
    });
}
