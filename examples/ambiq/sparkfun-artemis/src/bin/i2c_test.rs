#![no_std]
#![no_main]

use embassy_ambiq::i2c::{Config, I2c};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_ambiq::init();

    let config = Config::default();
    let mut i2c = I2c::new(p.IOM4, p.P39, p.P40, config);

    loop {
        // Send a test packet to address 0x42
        let data = [0xde, 0xad, 0xbe, 0xef];

        match embedded_hal_async::i2c::I2c::write(&mut i2c, 0x42, &data).await {
            Ok(_) => {
                // Transaction successful (ACK received)
            }
            Err(_e) => {
                // Transaction failed (e.g., NACK)
                // In a real application we would handle the error here
            }
        }

        Timer::after_millis(500).await;
    }
}
