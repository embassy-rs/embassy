//! The example uses FIFO and interrupts, wrapped in async API.
//!
//! This example controls AD5171 digital potentiometer via I2C with the LP-MSPM0L1306 board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::bind_interrupts;
use embassy_mspm0::i2c::{Config, I2c, InterruptHandler};
use embassy_mspm0::peripherals::I2C0;
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

const ADDRESS: u8 = 0x6a;

bind_interrupts!(struct Irqs {
    I2C0 => InterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_mspm0::init(Default::default());

    let instance = p.I2C0;
    let scl = p.PA1;
    let sda = p.PA0;

    let mut i2c = unwrap!(I2c::new_async(instance, scl, sda, Irqs, Config::default()));

    let mut pot_value: u8 = 0;

    loop {
        let to_write = [0u8, pot_value];

        match i2c.async_write(ADDRESS, &to_write).await {
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
