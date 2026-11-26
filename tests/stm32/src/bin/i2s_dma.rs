// required-features: i2s_f4

#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
use common::*;
use embassy_executor::Spawner;
use embassy_stm32::i2s::{Config, Format, I2S};
use embassy_stm32::rcc::{Pll, PllMul, PllPreDiv, PllRDiv};
use embassy_stm32::{bind_interrupts, dma, peripherals};

bind_interrupts!(struct Irqs {
    DMA1_STREAM7 => dma::InterruptHandler<peripherals::DMA1_CH7>;
});

#[embassy_executor::main(executor = "common::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut config = config();
    // reference your chip's manual for proper clock settings; this config
    // is recommended for a 32 bit frame at 48 kHz sample rate
    config.rcc.plli2s = Some(Pll {
        prediv: PllPreDiv::Div4,
        mul: PllMul::Mul60,
        divp: None,
        divq: None,
        divr: Some(PllRDiv::Div2),
    });
    let p = init_with_config(config);

    // stereo wavetable generation
    let mut wavetable = [0u16; 1200];
    for (i, frame) in wavetable.chunks_mut(2).enumerate() {
        frame[0] = ((((i / 150) % 2) * 2048) as i16 - 1024) as u16; // 160 Hz square wave in left channel
        frame[1] = ((((i / 100) % 2) * 2048) as i16 - 1024) as u16; // 240 Hz square wave in right channel
    }

    // i2s configuration
    let mut dma_buffer = [0u16; 2400];

    let mut i2s_config = Config::default();
    i2s_config.format = Format::Data16Channel32;
    i2s_config.master_clock = false;
    let mut i2s = I2S::new_txonly_nomck(
        p.SPI3,
        p.PB5,  // sd
        p.PA15, // ws
        p.PB3,  // ck
        p.DMA1_CH7,
        Irqs,
        &mut dma_buffer,
        i2s_config,
    );

    for _ in 0..3 {
        i2s.start();

        for _ in 0..10 {
            i2s.write(&wavetable).await.ok();
        }

        i2s.stop().await;
    }

    cortex_m::asm::bkpt();
}
