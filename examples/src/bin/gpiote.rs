#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::pin::Pin;
use cortex_m_rt::entry;
use embassy::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};
use embassy_nrf::gpiote;
use futures::pin_mut;
use nrf52840_hal::gpio;

use embassy::executor::{task, Executor};
static EXECUTOR: Executor = Executor::new(|| cortex_m::asm::sev());

#[task]
async fn run() {
    let p = embassy_nrf::pac::Peripherals::take().dewrap();
    let port0 = gpio::p0::Parts::new(p.P0);

    let g = gpiote::Gpiote::new(p.GPIOTE);

    info!("Starting!");

    let pin1 = port0.p0_11.into_pullup_input().degrade();
    let button1 = async {
        let ch = g
            .new_input_channel(&pin1, gpiote::EventPolarity::HiToLo)
            .dewrap();

        loop {
            ch.wait().await;
            info!("Button 1 pressed")
        }
    };

    let pin2 = port0.p0_12.into_pullup_input().degrade();
    let button2 = async {
        let ch = g
            .new_input_channel(&pin2, gpiote::EventPolarity::LoToHi)
            .dewrap();

        loop {
            ch.wait().await;
            info!("Button 2 released")
        }
    };

    let pin3 = port0.p0_24.into_pullup_input().degrade();
    let button3 = async {
        let ch = g
            .new_input_channel(&pin3, gpiote::EventPolarity::Toggle)
            .dewrap();

        loop {
            ch.wait().await;
            info!("Button 3 toggled")
        }
    };

    let pin4 = port0.p0_25.into_pullup_input().degrade();
    let button4 = async {
        let ch = g
            .new_input_channel(&pin4, gpiote::EventPolarity::Toggle)
            .dewrap();

        loop {
            ch.wait().await;
            info!("Button 4 toggled")
        }
    };

    futures::join!(button1, button2, button3, button4);
}

#[entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        EXECUTOR.spawn(run()).dewrap();

        loop {
            EXECUTOR.run();
            cortex_m::asm::wfe();
        }
    }
}
