//! Example on how to read a 24C/24LC i2c eeprom with low power consumption.
//! The eeprom is read every 1 second, while ensuring lowest possible power while
//! sleeping between reads.
//!
//! Connect SDA to P0.03, SCL to P0.04

#![no_std]
#![no_main]

use core::mem;

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const ADDRESS: u8 = 0x50;

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    info!("Started!");

    loop {
        info!("Initializing TWI...");
        let config = twim::Config::default();

        // Create the TWIM instance with borrowed singletons, so they're not consumed.
        let mut twi = Twim::new(&mut p.TWISPI0, Irqs, &mut p.P0_03, &mut p.P0_04, config);

        info!("Reading...");

        let mut buf = [0u8; 16];
        unwrap!(twi.blocking_write_read(ADDRESS, &mut [0x00], &mut buf));

        info!("Read: {=[u8]:x}", buf);

        // Drop the TWIM instance. This disables the peripehral and deconfigures the pins.
        // This clears the borrow on the singletons, so they can now be used again.
        mem::drop(twi);

        // Sleep for 1 second. The executor ensures the core sleeps with a WFE when it has nothing to do.
        // During this sleep, the nRF chip should only use ~3uA
        Timer::after_secs(1).await;
    }
}
