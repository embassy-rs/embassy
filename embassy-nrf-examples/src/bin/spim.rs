#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::panic;
use embassy::executor::Spawner;
use embassy::util::Steal;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::Peripherals;
use embassy_nrf::{interrupt, spim};
use embassy_traits::spi::FullDuplex;
use embedded_hal::digital::v2::*;
use example_common::*;
use futures::pin_mut;

#[embassy::main]
async fn main(spawner: Spawner) {
    info!("running!");

    let p = unsafe { Peripherals::steal() };

    let config = spim::Config {
        frequency: spim::Frequency::M16,
        mode: spim::MODE_0,
        orc: 0x00,
    };

    let irq = interrupt::take!(SPIM3);
    let spim = spim::Spim::new(p.SPIM3, irq, p.P0_29, p.P0_28, p.P0_30, config);
    pin_mut!(spim);

    let mut ncs = Output::new(p.P0_31, Level::High, OutputDrive::Standard);

    // Example on how to talk to an ENC28J60 chip

    // softreset
    cortex_m::asm::delay(10);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5);
    let tx = [0xFF];
    unwrap!(spim.as_mut().read_write(&mut [], &tx).await);
    cortex_m::asm::delay(10);
    ncs.set_high().unwrap();

    cortex_m::asm::delay(100000);

    let mut rx = [0; 2];

    // read ESTAT
    cortex_m::asm::delay(5000);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5000);
    let tx = [0b000_11101, 0];
    unwrap!(spim.as_mut().read_write(&mut rx, &tx).await);
    cortex_m::asm::delay(5000);
    ncs.set_high().unwrap();
    info!("estat: {=[?]}", rx);

    // Switch to bank 3
    cortex_m::asm::delay(10);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5);
    let tx = [0b100_11111, 0b11];
    unwrap!(spim.as_mut().read_write(&mut rx, &tx).await);
    cortex_m::asm::delay(10);
    ncs.set_high().unwrap();

    // read EREVID
    cortex_m::asm::delay(10);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5);
    let tx = [0b000_10010, 0];
    unwrap!(spim.as_mut().read_write(&mut rx, &tx).await);
    cortex_m::asm::delay(10);
    ncs.set_high().unwrap();

    info!("erevid: {=[?]}", rx);
}
