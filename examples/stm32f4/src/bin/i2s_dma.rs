#![no_std]
#![no_main]

use core::fmt::Write;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2s::{Config, I2S};
use embassy_stm32::time::Hertz;
use heapless::String;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut i2s = I2S::new(
        p.SPI2,
        p.PC3,  // sd
        p.PB12, // ws
        p.PB10, // ck
        p.PC6,  // mck
        p.DMA1_CH4,
        p.DMA1_CH3,
        Hertz(1_000_000),
        Config::default(),
    );

    for n in 0u32.. {
        let mut write: String<128> = String::new();
        core::write!(&mut write, "Hello DMA World {}!\r\n", n).unwrap();
        i2s.write(&mut write.as_bytes()).await.ok();
    }
}
