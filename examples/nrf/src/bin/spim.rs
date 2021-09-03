#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::panic;
use embassy::executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::Peripherals;
use embassy_nrf::{interrupt, spim};
use embassy_traits::spi::FullDuplex;
use embedded_hal::digital::v2::*;
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("running!");

    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M16;

    let irq = interrupt::take!(SPIM3);
    let mut spim = spim::Spim::new(p.SPI3, irq, p.P0_29, p.P0_28, p.P0_30, config);

    let mut ncs = Output::new(p.P0_31, Level::High, OutputDrive::Standard);

    // Example on how to talk to an ENC28J60 chip

    // softreset
    cortex_m::asm::delay(10);
    unwrap!(ncs.set_low());
    cortex_m::asm::delay(5);
    let tx = [0xFF];
    unwrap!(spim.read_write(&mut [], &tx).await);
    cortex_m::asm::delay(10);
    unwrap!(ncs.set_high());

    cortex_m::asm::delay(100000);

    let mut rx = [0; 2];

    // read ESTAT
    cortex_m::asm::delay(5000);
    unwrap!(ncs.set_low());
    cortex_m::asm::delay(5000);
    let tx = [0b000_11101, 0];
    unwrap!(spim.read_write(&mut rx, &tx).await);
    cortex_m::asm::delay(5000);
    unwrap!(ncs.set_high());
    info!("estat: {=[?]}", rx);

    // Switch to bank 3
    cortex_m::asm::delay(10);
    unwrap!(ncs.set_low());
    cortex_m::asm::delay(5);
    let tx = [0b100_11111, 0b11];
    unwrap!(spim.read_write(&mut rx, &tx).await);
    cortex_m::asm::delay(10);
    unwrap!(ncs.set_high());

    // read EREVID
    cortex_m::asm::delay(10);
    unwrap!(ncs.set_low());
    cortex_m::asm::delay(5);
    let tx = [0b000_10010, 0];
    unwrap!(spim.read_write(&mut rx, &tx).await);
    cortex_m::asm::delay(10);
    unwrap!(ncs.set_high());

    info!("erevid: {=[?]}", rx);
}
