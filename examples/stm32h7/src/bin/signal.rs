#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::Peripherals;

use defmt_rtt as _;
use embassy::channel::Signal;
// global logger
use panic_probe as _;


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

#[embassy::task]
async fn my_receiving_task() {

    loop {
        let received_counter = SIGNAL.wait().await;

        info!("signalled, counter: {}", received_counter);
    }
}

#[embassy::main]
async fn main(spawner: Spawner, _p: Peripherals) {
    unwrap!(spawner.spawn(my_receiving_task()));
    unwrap!(spawner.spawn(my_sending_task()));

    loop {
        Timer::after(Duration::from_secs(1)).await;
        info!(".");
    }
}
