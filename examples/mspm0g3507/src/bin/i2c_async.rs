//! The example uses FIFO and interrupts, wrapped in async API.
//!
//! This uses the virtual COM port provided on the LP-MSPM0G3507 board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::bind_interrupts;
use embassy_mspm0::i2c::{Config, I2c, InterruptHandler};
use embassy_mspm0::peripherals::I2C1;
use {defmt_rtt as _, panic_halt as _};

const ADDRESS: u8 = 0x6a;

bind_interrupts!(struct Irqs {
    I2C1 => InterruptHandler<I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_mspm0::init(Default::default());

    let instance = p.I2C1;
    let scl = p.PB2;
    let sda = p.PB3;

    let mut i2c = unwrap!(I2c::new_async(instance, scl, sda, Irqs, Config::default()));

    let mut to_read = [1u8; 17];
    let to_write = [0u8; 17];

    match i2c.async_write_read(ADDRESS, &to_write, &mut to_read).await {
        Ok(()) => info!("Register {}: {}", to_write, to_read[0]),
        Err(e) => error!("I2c Error: {:?}", e),
    }

    loop {}
}
