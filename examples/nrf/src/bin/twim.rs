//! Example on how to read a 24C/24LC i2c eeprom.
//!
//! Connect SDA to P0.03, SCL to P0.04

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy::executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{interrupt, Peripherals};

use defmt_rtt as _; // global logger
use panic_probe as _;

const ADDRESS: u8 = 0x50;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Initializing TWI...");
    let config = twim::Config::default();
    let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    let mut twi = Twim::new(p.TWISPI0, irq, p.P0_03, p.P0_04, config);

    info!("Reading...");

    let mut buf = [0u8; 16];
    unwrap!(twi.blocking_write_read(ADDRESS, &mut [0x00], &mut buf));

    info!("Read: {=[u8]:x}", buf);
}
