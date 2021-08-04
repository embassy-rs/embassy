//! Example on how to read a 24C/24LC i2c eeprom.
//!
//! Connect SDA to P0.03, SCL to P0.04

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::{panic, *};
use embassy::executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{interrupt, Peripherals};

const ADDRESS: u8 = 0x50;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Initializing TWI...");
    let config = twim::Config::default();
    let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    let mut twi = Twim::new(p.TWISPI0, irq, p.P0_03, p.P0_04, config);

    info!("Reading...");

    let mut buf = [0u8; 16];
    twi.write_then_read(ADDRESS, &mut [0x00], &mut buf).unwrap();

    info!("Read: {=[u8]:x}", buf);
}
