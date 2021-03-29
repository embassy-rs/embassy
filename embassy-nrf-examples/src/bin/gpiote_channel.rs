#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use defmt::panic;
use embassy::executor::Spawner;
use embassy_nrf::gpio::{Input, Pull};
use embassy_nrf::gpiote::{self, InputChannel, InputChannelPolarity};
use embassy_nrf::{interrupt, Peripherals};

#[embassy::main]
async fn main(spawner: Spawner) {
    let p = Peripherals::take().unwrap();
    let g = gpiote::initialize(p.GPIOTE, interrupt::take!(GPIOTE));

    info!("Starting!");

    let ch1 = InputChannel::new(
        g,
        p.GPIOTE_CH0,
        Input::new(p.P0_11, Pull::Up),
        InputChannelPolarity::HiToLo,
    );
    let ch2 = InputChannel::new(
        g,
        p.GPIOTE_CH1,
        Input::new(p.P0_12, Pull::Up),
        InputChannelPolarity::LoToHi,
    );
    let ch3 = InputChannel::new(
        g,
        p.GPIOTE_CH2,
        Input::new(p.P0_24, Pull::Up),
        InputChannelPolarity::Toggle,
    );
    let ch4 = InputChannel::new(
        g,
        p.GPIOTE_CH3,
        Input::new(p.P0_25, Pull::Up),
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
