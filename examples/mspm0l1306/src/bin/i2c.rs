//! Example of using blocking I2C
//!
//! This uses the virtual COM port provided on the LP-MSPM0L1306 board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::i2c::{Config, I2c};
use {defmt_rtt as _, panic_halt as _};

const ADDRESS: u8 = 0x6a;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_mspm0::init(Default::default());

    let instance = p.I2C0;
    let scl = p.PA1;
    let sda = p.PA0;

    let mut i2c = unwrap!(I2c::new_blocking(instance, scl, sda, Config::default()));

    let mut to_read = [0u8; 1];
    let to_write: u8 = 0x0F;

    match i2c.blocking_write_read(ADDRESS, &[to_write], &mut to_read) {
        Ok(()) => info!("Register {}: {}", to_write, to_read[0]),
        Err(e) => error!("I2c Error: {:?}", e),
    }

    loop {}
}
