#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Pull};
use embassy_nrf::gpiote::{InputChannel, InputChannelPolarity};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Starting!");

    let ch1 = InputChannel::new(
        p.GPIOTE_CH0,
        Input::new(p.P0_23, Pull::Up),
        InputChannelPolarity::HiToLo,
    );
    let ch2 = InputChannel::new(
        p.GPIOTE_CH1,
        Input::new(p.P0_24, Pull::Up),
        InputChannelPolarity::LoToHi,
    );
    let ch3 = InputChannel::new(
        p.GPIOTE_CH2,
        Input::new(p.P0_08, Pull::Up),
        InputChannelPolarity::Toggle,
    );
    let ch4 = InputChannel::new(
        p.GPIOTE_CH3,
        Input::new(p.P0_09, Pull::Up),
        InputChannelPolarity::Toggle,
    );

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

    futures::join!(button1, button2, button3, button4);
}
