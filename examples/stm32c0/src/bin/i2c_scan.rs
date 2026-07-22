//! I2C bus scan — NUCLEO-C092RC
//!
//! Scans the 7-bit address space (0x08..=0x77) using zero-length write transactions.
//! A device ACKs its address → printed as found. NACK → no device present.
//!
//! Hardware: I2C1 on PB8 (SCL) / PB9 (SDA). Add 4.7 kΩ pull-ups to 3.3 V.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Error, I2c};
use embedded_hal::i2c::Operation;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("I2C1 bus scan — PB8 (SCL) / PB9 (SDA)");

    let mut i2c = I2c::new_blocking(p.I2C1, p.PB8, p.PB9, Default::default());

    let mut found = 0u8;
    for addr in 0x08u8..0x78u8 {
        let mut ops = [Operation::Write(&[])];
        match i2c.blocking_transaction(addr, &mut ops) {
            Ok(_) => {
                info!("Found device at 0x{:02x}", addr);
                found += 1;
            }
            Err(Error::Nack) => {}
            Err(e) => warn!("0x{:02x}: {:?}", addr, e),
        }
    }
    info!("Scan complete: {} device(s) found", found);
}
