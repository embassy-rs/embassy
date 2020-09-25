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

use core::mem::MaybeUninit;
use cortex_m_rt::entry;
use embassy::executor::{task, Executor};
use embassy::time::{Duration, Instant, Timer};
use embassy_nrf::{interrupt, pac, rtc};
use nrf52840_hal::clocks;

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
        let ms = end.duration_since(start).into_ticks() / 33;
        info!("    [med] done in {:u32} ms", ms);

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
        let ms = end.duration_since(start).into_ticks() / 33;
        info!("[low] done in {:u32} ms", ms);

        Timer::after(Duration::from_ticks(32983)).await;
    }
}

static mut RTC: MaybeUninit<rtc::RTC<pac::RTC1>> = MaybeUninit::uninit();
static mut EXECUTOR_LOW: MaybeUninit<Executor<rtc::Alarm<pac::RTC1>>> = MaybeUninit::uninit();
static mut EXECUTOR_MED: MaybeUninit<Executor<rtc::Alarm<pac::RTC1>>> = MaybeUninit::uninit();
static mut EXECUTOR_HIGH: MaybeUninit<Executor<rtc::Alarm<pac::RTC1>>> = MaybeUninit::uninit();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_nrf::pac::Peripherals::take().dewrap();

    clocks::Clocks::new(p.CLOCK)
        .enable_ext_hfosc()
        .set_lfclk_src_external(clocks::LfOscConfiguration::NoExternalNoBypass)
        .start_lfclk();

    let rtc: &'static _ = unsafe {
        let ptr = RTC.as_mut_ptr();
        ptr.write(rtc::RTC::new(p.RTC1));
        &*ptr
    };

    rtc.start();
    unsafe { embassy::time::set_clock(rtc) };

    let executor_low: &'static _ = unsafe {
        let ptr = EXECUTOR_LOW.as_mut_ptr();
        ptr.write(Executor::new(rtc.alarm0(), cortex_m::asm::sev));
        &*ptr
    };

    let executor_med: &'static _ = unsafe {
        let ptr = EXECUTOR_MED.as_mut_ptr();
        ptr.write(Executor::new(rtc.alarm1(), || {
            interrupt::pend(interrupt::SWI0_EGU0)
        }));
        &*ptr
    };

    let executor_high: &'static _ = unsafe {
        let ptr = EXECUTOR_HIGH.as_mut_ptr();
        ptr.write(Executor::new(rtc.alarm2(), || {
            interrupt::pend(interrupt::SWI1_EGU1)
        }));
        &*ptr
    };

    interrupt::set_priority(interrupt::SWI0_EGU0, interrupt::Priority::Level7);
    interrupt::set_priority(interrupt::SWI1_EGU1, interrupt::Priority::Level6);
    interrupt::enable(interrupt::SWI0_EGU0);
    interrupt::enable(interrupt::SWI1_EGU1);

    unsafe {
        executor_low.spawn(run_low()).dewrap();
        executor_med.spawn(run_med()).dewrap();
        executor_high.spawn(run_high()).dewrap();

        loop {
            executor_low.run();
            cortex_m::asm::wfe();
        }
    }
}

#[interrupt]
unsafe fn SWI0_EGU0() {
    EXECUTOR_MED.as_ptr().as_ref().unwrap().run()
}

#[interrupt]
unsafe fn SWI1_EGU1() {
    EXECUTOR_HIGH.as_ptr().as_ref().unwrap().run()
}
