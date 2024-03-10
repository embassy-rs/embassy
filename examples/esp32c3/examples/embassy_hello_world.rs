//! embassy hello world
//!
//! This is an example of running the embassy executor with multiple tasks
//! concurrently.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Executor;
use embassy_time::{Duration, Timer};
use esp32c3_hal::{clock::ClockControl, embassy, peripherals::Peripherals, prelude::*};
use esp_backtrace as _;
use static_cell::make_static;

#[embassy_executor::task]
async fn run1() {
    loop {
        esp_println::println!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[embassy_executor::task]
async fn run2() {
    loop {
        esp_println::println!("Bing!");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

#[entry]
fn main() -> ! {
    esp_println::println!("Init!");
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    #[cfg(feature = "embassy-time-systick")]
    embassy::init(
        &clocks,
        esp32c3_hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    #[cfg(feature = "embassy-time-timg0")]
    embassy::init(
        &clocks,
        esp32c3_hal::timer::TimerGroup::new(peripherals.TIMG0, &clocks).timer0,
    );

    let executor = make_static!(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(run1()).ok();
        spawner.spawn(run2()).ok();
    });
}
