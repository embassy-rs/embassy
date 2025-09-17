#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use embassy_microchip::i2c::I2c;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_microchip::init(Default::default());

    info!("Hello, world!");

    let mut i2c = I2c::new_blocking(p.SMB0, p.GPIO73, p.GPIO72, Default::default());

    Timer::after_secs(1).await;

    let mut read = [0_u8; 1];

    loop {
        for addr in (0..0x7f_u8).into_iter() {
            if i2c.blocking_read(addr, &mut read).is_ok() {
                info!("Found device at addr {:02x}", addr);
            }
        }

        Timer::after_secs(1).await;
    }
}
