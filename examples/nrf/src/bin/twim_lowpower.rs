//! Example on how to read a 24C/24LC i2c eeprom with low power consumption.
//! The eeprom is read every 1 second, while ensuring lowest possible power while
//! sleeping between reads.
//!
//! Connect SDA to P0.03, SCL to P0.04

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::mem;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{interrupt, Peripherals};

use defmt_rtt as _; // global logger
use panic_probe as _;

const ADDRESS: u8 = 0x50;

#[embassy::main]
async fn main(_spawner: Spawner, mut p: Peripherals) {
    info!("Started!");
    let mut irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);

    loop {
        info!("Initializing TWI...");
        let config = twim::Config::default();

        // Create the TWIM instance with borrowed singletons, so they're not consumed.
        let mut twi = Twim::new(&mut p.TWISPI0, &mut irq, &mut p.P0_03, &mut p.P0_04, config);

        info!("Reading...");

        let mut buf = [0u8; 16];
        unwrap!(twi.blocking_write_read(ADDRESS, &mut [0x00], &mut buf));

        info!("Read: {=[u8]:x}", buf);

        // Drop the TWIM instance. This disables the peripehral and deconfigures the pins.
        // This clears the borrow on the singletons, so they can now be used again.
        mem::drop(twi);

        // Sleep for 1 second. The executor ensures the core sleeps with a WFE when it has nothing to do.
        // During this sleep, the nRF chip should only use ~3uA
        Timer::after(Duration::from_secs(1)).await;
    }
}
