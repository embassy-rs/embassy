#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use embassy_microchip::i2c::{I2c, InterruptHandler};
use embassy_microchip::{bind_interrupts, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        I2CSMB0 => InterruptHandler<peripherals::SMB0>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_microchip::init(Default::default());

    info!("Hello, world!");

    let mut i2c = I2c::new_async(p.SMB0, p.GPIO73, p.GPIO72, Irqs, Default::default());

    Timer::after_secs(1).await;

    let mut read = [0_u8; 1];

    loop {
        for addr in (0..0x7f_u8).into_iter() {
            if i2c.read_async(addr, &mut read).await.is_ok() {
                info!("Found device at addr {:02x}", addr);
            }
        }

        Timer::after_secs(1).await;
    }
}
