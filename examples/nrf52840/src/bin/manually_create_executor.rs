// This example showcases how to manually create an executor.
// This is what the #[embassy::main] macro does behind the scenes.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, unwrap};
use embassy_executor::Executor;
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn run1() {
    loop {
        info!("BIG INFREQUENT TICK");
        Timer::after_ticks(64000).await;
    }
}

#[embassy_executor::task]
async fn run2() {
    loop {
        info!("tick");
        Timer::after_ticks(13000).await;
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let _p = embassy_nrf::init(Default::default());

    // Create the executor and put it in a StaticCell, because `run` needs `&'static mut Executor`.
    let executor = EXECUTOR.init(Executor::new());

    // Run it.
    // `run` calls the closure then runs the executor forever. It never returns.
    executor.run(|spawner| {
        // Here we get access to a spawner to spawn the initial tasks.
        unwrap!(spawner.spawn(run1()));
        unwrap!(spawner.spawn(run2()));
    });
}
