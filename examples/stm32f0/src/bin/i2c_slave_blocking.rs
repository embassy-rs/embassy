//! I2C Blocking Slave Example
//!
//! Demonstrates the blocking I2C slave (multimaster) implementation.
//! The device listens for I2C transactions from an external master.
//!
//! # Hardware Setup
//!
//! - PB8 (SCL) and PB9 (SDA) with 4.7k pull-up resistors to 3.3V
//! - Connect an I2C master device with clock stretching enabled
//! - Default slave address: 0x42 (7-bit)
//!
//! # Behavior
//!
//! - Master WRITE: Receives up to BUFFER_SIZE bytes, excess bytes are drained
//! - Master READ: Sends an 8-byte response pattern (0xA0-0xA7)

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{self, I2c, SendStatus, SlaveAddrConfig, SlaveCommandKind};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

const I2C_ADDR: u8 = 0x42;
const BUFFER_SIZE: usize = 32;

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("I2C Blocking Slave Example");
    info!("Address: 0x{:02X}, Buffer: {} bytes", I2C_ADDR, BUFFER_SIZE);

    let mut config = i2c::Config::default();
    config.timeout = Duration::from_secs(30);

    let mut i2c =
        I2c::new_blocking(p.I2C1, p.PB8, p.PB9, config).into_slave_multimaster(SlaveAddrConfig::basic(I2C_ADDR));

    info!("Slave ready, listening...");

    let mut count: u32 = 0;
    loop {
        match i2c.blocking_listen() {
            Ok(cmd) => {
                count += 1;
                match cmd.kind {
                    SlaveCommandKind::Write => {
                        let mut buffer = [0u8; BUFFER_SIZE];
                        match i2c.blocking_respond_to_write(&mut buffer) {
                            Ok(len) => info!("[{}] Write: {} bytes: {:02X}", count, len, &buffer[..len]),
                            Err(e) => error!("[{}] Write error: {:?}", count, e),
                        }
                    }
                    SlaveCommandKind::Read => {
                        let response: [u8; 8] = [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7];
                        match i2c.blocking_respond_to_read(&response) {
                            Ok(SendStatus::Done) => info!("[{}] Read: {} bytes", count, response.len()),
                            Ok(SendStatus::LeftoverBytes(n)) => {
                                info!("[{}] Read: {} of {} bytes", count, response.len() - n, response.len())
                            }
                            Err(e) => error!("[{}] Read error: {:?}", count, e),
                        }
                    }
                }
            }
            Err(e) => error!("Listen error: {:?}", e),
        }
    }
}
