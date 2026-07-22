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
    info!("Initializing I2C bus scanner...");

    // Adjust to match hardware
    let mut i2c = I2c::new_blocking(p.I2C1, p.PA15, p.PB7, Default::default());

    // Scan 0x08..=0x77: the valid 7-bit address space for regular devices.
    // 0x00..=0x07 are reserved (general call, CBUS, Hs-mode codes, etc.).
    // 0x78..=0x7F are reserved for 10-bit addressing and future use.
    info!("Starting bus scan using blocking zero-length writes...");
    let mut found = 0u8;
    for addr in 0x08u8..0x78u8 {
        let mut ops = [Operation::Write(&[])];
        match i2c.blocking_transaction(addr, &mut ops) {
            Ok(_) => {
                info!("Found device at address: 0x{:02x}", addr);
                found += 1;
            }
            Err(Error::Nack) => {
                debug!("No device found at address: 0x{:02x}", addr);
            }
            Err(e) => {
                warn!("Address 0x{:02x} failed with error: {:?}", addr, e);
            }
        }
    }
    info!("Scan complete: {} device(s) found.", found);
}
