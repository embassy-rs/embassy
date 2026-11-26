//
// This example is written for a discovery-f303vc board
//
// NOTE: This example outputs potentially loud audio. Please run responsibly.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::i2s::{Config, Format, I2S};
use embassy_stm32::{bind_interrupts, dma, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
});

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
    i2s_config.format = Format::Data16Channel16;
    i2s_config.master_clock = false;

    let mut i2s = I2S::new_txonly_nomck(
        p.SPI2,
        p.PB15, // sd
        p.PB12, // ws
        p.PB13, // ck (~1.6Mhz by osciloscope)
        p.DMA1_CH5,
        Irqs,
        &mut dma_buffer,
        i2s_config,
    );
    i2s.start();

    loop {
        i2s.write(&wavetable).await.ok();
    }
}
