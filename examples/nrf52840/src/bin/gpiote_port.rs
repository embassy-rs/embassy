#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Pull};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task(pool_size = 4)]
async fn button_task(n: usize, mut pin: Input<'static>) {
    loop {
        pin.wait_for_low().await;
        info!("Button {:?} pressed!", n);
        pin.wait_for_high().await;
        info!("Button {:?} released!", n);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Starting!");

    let btn1 = Input::new(p.P0_11, Pull::Up);
    let btn2 = Input::new(p.P0_12, Pull::Up);
    let btn3 = Input::new(p.P0_24, Pull::Up);
    let btn4 = Input::new(p.P0_25, Pull::Up);

    unwrap!(spawner.spawn(button_task(1, btn1)));
    unwrap!(spawner.spawn(button_task(2, btn2)));
    unwrap!(spawner.spawn(button_task(3, btn3)));
    unwrap!(spawner.spawn(button_task(4, btn4)));
}
