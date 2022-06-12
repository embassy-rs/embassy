#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// global logger
use defmt::{info, unwrap};
use defmt_rtt as _;

use panic_probe as _;

use embassy::channel::signal::Signal;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};

use embassy_stm32::Peripherals;

static SIGNAL: Signal<u32> = Signal::new();

#[embassy::task]
async fn my_sending_task() {
    let mut counter: u32 = 0;

    loop {
        Timer::after(Duration::from_secs(1)).await;

        SIGNAL.signal(counter);

        counter = counter.wrapping_add(1);
    }
}

#[embassy::main]
async fn main(spawner: Spawner, _p: Peripherals) {
    unwrap!(spawner.spawn(my_sending_task()));

    loop {
        let received_counter = SIGNAL.wait().await;

        info!("signalled, counter: {}", received_counter);
    }
}
