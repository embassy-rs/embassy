//! Example of using blocking I2C
//!
//! This uses the virtual COM port provided on the LP-MSPM0G3507 board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::i2c::{BusSpeed, ClockSel, Config, I2c};
use {defmt_rtt as _, panic_halt as _};

const ADDRESS: u8 = 0x6a;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_mspm0::init(Default::default());

    let instance = p.I2C1;
    let scl = p.PB2;
    let sda = p.PB3;

    let mut config = Config::default();
    config.clock_source = ClockSel::BusClk;
    config.bus_speed = BusSpeed::FastMode;
    let mut i2c = unwrap!(I2c::new_blocking(instance, scl, sda, config));

    let mut to_read = [0u8; 1];
    let to_write: u8 = 0x0F;

    match i2c.blocking_write_read(ADDRESS, &[to_write], &mut to_read) {
        Ok(()) => info!("Register {}: {}", to_write, to_read[0]),
        Err(e) => error!("I2c Error: {:?}", e),
    }

    loop {}
}
