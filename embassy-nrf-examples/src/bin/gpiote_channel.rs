#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use defmt::panic;
use nrf52840_hal::gpio;

use embassy::executor::{task, Executor};
use embassy::util::Forever;
use embassy_nrf::gpiote::{Gpiote, InputChannel, InputChannelPolarity};
use embassy_nrf::interrupt;

#[task]
async fn run() {
    let p = unwrap!(embassy_nrf::pac::Peripherals::take());
    let port0 = gpio::p0::Parts::new(p.P0);

    let (g, chs) = Gpiote::new(p.GPIOTE, interrupt::take!(GPIOTE));

    info!("Starting!");

    let pin1 = port0.p0_11.into_pullup_input().degrade();
    let pin2 = port0.p0_12.into_pullup_input().degrade();
    let pin3 = port0.p0_24.into_pullup_input().degrade();
    let pin4 = port0.p0_25.into_pullup_input().degrade();

    let ch1 = InputChannel::new(g, chs.ch0, pin1, InputChannelPolarity::HiToLo);
    let ch2 = InputChannel::new(g, chs.ch1, pin2, InputChannelPolarity::LoToHi);
    let ch3 = InputChannel::new(g, chs.ch2, pin3, InputChannelPolarity::Toggle);
    let ch4 = InputChannel::new(g, chs.ch3, pin4, InputChannelPolarity::Toggle);

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

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        unwrap!(spawner.spawn(run()));
    });
}
