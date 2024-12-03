#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

static SIGNAL: Signal<CriticalSectionRawMutex, u32> = Signal::new();

#[embassy_executor::task]
async fn my_sending_task() {
    let mut counter: u32 = 0;

    loop {
        Timer::after_secs(1).await;

        SIGNAL.signal(counter);

        counter = counter.wrapping_add(1);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_stm32::init(Default::default());
    unwrap!(spawner.spawn(my_sending_task()));

    loop {
        let received_counter = SIGNAL.wait().await;

        info!("signalled, counter: {}", received_counter);
    }
}
