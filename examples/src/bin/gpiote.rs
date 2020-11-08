#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use nrf52840_hal::gpio;

use embassy::executor::{task, Executor};
use embassy::util::Forever;
use embassy_nrf::gpiote;

#[task]
async fn run() {
    let p = unwrap!(embassy_nrf::pac::Peripherals::take());
    let port0 = gpio::p0::Parts::new(p.P0);

    let g = gpiote::Gpiote::new(p.GPIOTE);

    info!("Starting!");

    let pin1 = port0.p0_11.into_pullup_input().degrade();
    let button1 = async {
        let ch = unwrap!(g.new_input_channel(pin1, gpiote::InputChannelPolarity::HiToLo));

        loop {
            ch.wait().await;
            info!("Button 1 pressed")
        }
    };

    let pin2 = port0.p0_12.into_pullup_input().degrade();
    let button2 = async {
        let ch = unwrap!(g.new_input_channel(pin2, gpiote::InputChannelPolarity::LoToHi));

        loop {
            ch.wait().await;
            info!("Button 2 released")
        }
    };

    let pin3 = port0.p0_24.into_pullup_input().degrade();
    let button3 = async {
        let ch = unwrap!(g.new_input_channel(pin3, gpiote::InputChannelPolarity::Toggle));

        loop {
            ch.wait().await;
            info!("Button 3 toggled")
        }
    };

    let pin4 = port0.p0_25.into_pullup_input().degrade();
    let button4 = async {
        let ch = unwrap!(g.new_input_channel(pin4, gpiote::InputChannelPolarity::Toggle));

        loop {
            ch.wait().await;
            info!("Button 4 toggled")
        }
    };

    futures::join!(button1, button2, button3, button4);
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let executor = EXECUTOR.put(Executor::new(cortex_m::asm::sev));
    unwrap!(executor.spawn(run()));

    loop {
        executor.run();
        cortex_m::asm::wfe();
    }
}
