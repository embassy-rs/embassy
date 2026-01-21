// This example is written for a nucleo-g491re board
//
// NOTE: This example outputs potentially loud audio. Please run responsibly.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::i2s::{Config, Format, I2S};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // stereo wavetable generation
    let mut wavetable = [0u16; 1200];
    for (i, frame) in wavetable.chunks_mut(2).enumerate() {
        frame[0] = ((((i / 150) % 2) * 2048) as i16 - 1024) as u16;
        frame[1] = ((((i / 100) % 2) * 2048) as i16 - 1024) as u16;
    }

    // i2s configuration
    let mut dma_buffer = [0u16; 2400];

    let mut i2s_config = Config::default();
    i2s_config.format = Format::Data16Channel32;
    let mut i2s = I2S::new_txonly(
        p.SPI2,
        p.PB15, // sd
        p.PB12, // ws
        p.PB13, // ck
        p.PC6,
        p.DMA1_CH1,
        &mut dma_buffer,
        i2s_config,
    );
    i2s.start();

    loop {
        i2s.write(&wavetable).await.ok();
    }
}
