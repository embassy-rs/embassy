#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Error, I2c};
use embedded_hal_1::i2c::Operation;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Initializing I2C bus scanner...");

    // Configure SCL/SDA pins for I2C2 (PB10 and PB11 are typical for Nucleo-L476RG)
    let mut i2c = I2c::new_blocking(p.I2C2, p.PB10, p.PB11, Default::default());

    info!("Starting bus scan using blocking zero-length writes...");
    for addr in 0x08u8..0x78u8 {
        let mut ops = [Operation::Write(&[])];
        match i2c.blocking_transaction(addr, &mut ops) {
            Ok(_) => {
                info!("Found device at address: 0x{:02x}", addr);
            }
            Err(Error::Nack) => {
                // Address not acknowledged (no device present)
            }
            Err(e) => {
                warn!("Address 0x{:02x} failed with error: {:?}", addr, e);
            }
        }
    }
    info!("Scan complete!");
}
