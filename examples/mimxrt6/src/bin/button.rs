#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_imxrt::gpio;
use {defmt_rtt as _, embassy_imxrt_examples as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    let mut user1 = gpio::Input::new(p.PIO1_1, gpio::Pull::None, gpio::Inverter::Disabled);
    let mut user2 = gpio::Input::new(p.PIO0_10, gpio::Pull::None, gpio::Inverter::Disabled);

    loop {
        let res = select(user1.wait_for_falling_edge(), user2.wait_for_falling_edge()).await;

        match res {
            Either::First(()) => {
                info!("Button `USER1' pressed");
            }
            Either::Second(()) => {
                info!("Button `USER2' pressed");
            }
        }
    }
}
