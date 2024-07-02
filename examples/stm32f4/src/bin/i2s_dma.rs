#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2s::{Config, I2S};
use embassy_stm32::time::Hertz;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut dma_buffer = [0x00_u16; 128];

    let mut i2s = I2S::new_txonly(
        p.SPI2,
        p.PC3,  // sd
        p.PB12, // ws
        p.PB10, // ck
        p.PC6,  // mck
        p.DMA1_CH4,
        &mut dma_buffer,
        Hertz(1_000_000),
        Config::default(),
    );

    for i in 0_u16.. {
        i2s.write(&mut [i * 2; 64]).await.ok();
        i2s.write(&mut [i * 2 + 1; 64]).await.ok();
    }
}
