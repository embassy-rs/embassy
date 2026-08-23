//! This example uses FIFO with polling, and the maximum FIFO size is 8.
//! Refer to async example to handle larger packets.
//!
//! This example reads all registers (0-19) of DS3231 Real-Time Clock via I2C with the LP-MSPM0G3507 board.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mspm0::i2c::{Config, I2c};
use embassy_time::Timer;
use panic_halt as _;

const ADDRESS: u8 = 0x68;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_mspm0::init(Default::default());

    let instance = p.I2C1;
    let scl = p.PB2;
    let sda = p.PB3;

    let mut i2c = unwrap!(I2c::new_blocking(instance, scl, sda, Config::default()));

    loop {
        for reg in 0..20u8 {
            let reg_addr = [reg];
            let mut val = [0u8];

            match i2c.blocking_write_read(ADDRESS, &reg_addr, &mut val) {
                Ok(()) => info!("reg {}: {}", reg, val[0]),
                Err(e) => error!("I2c Error: {:?}", e),
            }
            Timer::after_millis(100).await;
        }

        Timer::after_millis(2000).await;
    }
}
