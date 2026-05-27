#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::gpio::Pull;
use embassy_nrf::gpiote::{InputChannel, InputChannelPolarity};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Starting!");

    let mut ch1 = InputChannel::new(p.GPIOTE20_CH0, p.P1_26, Pull::Up, InputChannelPolarity::HiToLo);
    let mut ch2 = InputChannel::new(p.GPIOTE20_CH1, p.P1_09, Pull::Up, InputChannelPolarity::LoToHi);
    let mut ch3 = InputChannel::new(p.GPIOTE20_CH2, p.P1_08, Pull::Up, InputChannelPolarity::Toggle);
    let mut ch4 = InputChannel::new(p.GPIOTE30_CH0, p.P0_05, Pull::Up, InputChannelPolarity::Toggle);

    let button1 = async {
        loop {
            ch1.wait().await;
            info!("Button 1 pressed")
        }
    };

    let button2 = async {
        loop {
            ch2.wait().await;
            info!("Button 2 released")
        }
    };

    let button3 = async {
        loop {
            ch3.wait().await;
            info!("Button 3 toggled")
        }
    };

    let button4 = async {
        loop {
            ch4.wait().await;
            info!("Button 4 toggled")
        }
    };

    embassy_futures::join::join4(button1, button2, button3, button4).await;
}
