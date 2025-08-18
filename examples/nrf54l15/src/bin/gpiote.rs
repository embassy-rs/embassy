#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::{select4, Either4};
use embassy_nrf::gpio::{Input, Pull};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Starting!");

    // Buttons on nrf54l15 PDK
    let mut button0 = Input::new(p.P1_13, Pull::Up);
    let mut button1 = Input::new(p.P1_09, Pull::Up);
    let mut button2 = Input::new(p.P1_08, Pull::Up);
    let mut button3 = Input::new(p.P0_04, Pull::Up);

    loop {
        match select4(
            button0.wait_for_low(),
            button1.wait_for_falling_edge(),
            button2.wait_for_any_edge(),
            button3.wait_for_low(),
        )
        .await
        {
            Either4::First(_) => {
                info!("Button 0 pressed");
                button0.wait_for_high().await;
                info!("Button 0 released");
            }
            Either4::Second(_) => {
                info!("Button 1 pressed");
                button1.wait_for_rising_edge().await;
                info!("Button 1 released");
            }
            Either4::Third(_) => {
                info!("Button 2 pressed");
                button2.wait_for_any_edge().await;
                info!("Button 2 released");
            }
            Either4::Fourth(_) => {
                info!("Button 3 pressed");
                button3.wait_for_high().await;
                info!("Button 3 released");
            }
        }
    }
}
