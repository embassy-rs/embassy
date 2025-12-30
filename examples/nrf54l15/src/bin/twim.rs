//! Example on how to read a 24C/24LC i2c eeprom.
//!
//! Connect SDA to P0.03, SCL to P0.04

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

const ADDRESS: u8 = 0x18;
const WHOAMI: u8 = 0x0F;

bind_interrupts!(struct Irqs {
    SERIAL20 => twim::InterruptHandler<peripherals::SERIAL20>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Initializing TWI...");
    let config = twim::Config::default();
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    let mut twi = Twim::new(p.SERIAL20, Irqs, p.P1_13, p.P1_12, config, RAM_BUFFER.take());

    info!("Reading...");

    let mut data = [0u8; 1];
    match twi.write_read(ADDRESS, &[WHOAMI], &mut data).await {
        Ok(()) => info!("Whoami: {}", data[0]),
        Err(e) => error!("I2c Error: {:?}", e),
    }
}
