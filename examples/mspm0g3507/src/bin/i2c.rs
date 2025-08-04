//! This example uses FIFO with polling, and the maximum FIFO size is 8.
//! Refer to async example to handle larger packets.
//!
//! This example controls AD5171 digital potentiometer via I2C with the LP-MSPM0G3507 board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::i2c::{Config, I2c};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

const ADDRESS: u8 = 0x6a;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_mspm0::init(Default::default());

    let instance = p.I2C1;
    let scl = p.PB2;
    let sda = p.PB3;

    let mut i2c = unwrap!(I2c::new_blocking(instance, scl, sda, Config::default()));

    let mut pot_value: u8 = 0;

    loop {
        let to_write = [0u8, pot_value];

        match i2c.blocking_write(ADDRESS, &to_write) {
            Ok(()) => info!("New potentioemter value: {}", pot_value),
            Err(e) => error!("I2c Error: {:?}", e),
        }

        pot_value += 1;
        // if reached 64th position (max)
        // start over from lowest value
        if pot_value == 64 {
            pot_value = 0;
        }
        Timer::after_millis(500).await;
    }
}
