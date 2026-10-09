//! I2C blocking driver example (MPU6500 sensor)

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::i2c::{Config, I2c};
use embassy_time::Timer;
use panic_halt as _;

const MPU6500_ADDR: u8 = 0x68; // 0x69 if AD0 is grounded
const WHO_AM_I: u8 = 0x75; // chip ID register

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());

    let scl = p.PIO1_20; // D15 on the board
    let sda = p.PIO1_21; // D14 on the board
    let mut i2c = I2c::new_blocking(p.I2C4, scl, sda, Config::default());

    loop {
        // write_read example
        info!("Reading MPU6500 chip ID...");
        let mut buf = [0u8; 1];
        let res = i2c.blocking_write_read(MPU6500_ADDR, &[WHO_AM_I], &mut buf);
        match res {
            Ok(_) => {
                info!("Read successfull! MPU6500 ID: {:#04x}", buf[0]); // should be 0x70
            }
            Err(e) => {
                error!("Read error: {}", e);
            }
        }

        Timer::after_secs(2).await;
    }
}
