#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

const TMP117_ADDR: u8 = 0x48;
const TMP117_TEMP_RESULT: u8 = 0x00;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello world");

    let p = embassy_stm32::init(Default::default());

    let mut data = [0u8; 2];
    let mut i2c = I2c::new(p.I2C1, p.PB8, p.PB9, Irqs, p.DMA1_CH1, p.DMA1_CH2, Default::default());

    loop {
        match i2c.write_read(TMP117_ADDR, &[TMP117_TEMP_RESULT], &mut data).await {
            Ok(()) => {
                let temp = f32::from(i16::from_be_bytes(data)) * 7.8125 / 1000.0;
                info!("Temperature {}", temp);
            }
            Err(_) => error!("I2C Error"),
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}
