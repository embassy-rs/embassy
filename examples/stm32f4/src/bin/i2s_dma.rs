#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2s::{Config, I2S};
use embassy_stm32::time::Hertz;
use {defmt_rtt as _, panic_probe as _};

/// This example is written for the nucleo-stm32f429zi, with a stm32f429zi chip.
///
/// If you are using a different board or chip, make sure you update the following:
///
/// * [ ] Update .cargo/config.toml with the correct `probe-rs run --chip STM32F429ZITx`chip name.
/// * [ ] Update Cargo.toml to have the correct `embassy-stm32` feature, it is
///       currently `stm32f429zi`.
/// * [ ] If your board has a special clock or power configuration, make sure that it is
///       set up appropriately.
/// * [ ] If your board has different pin mapping, update any pin numbers or peripherals
///       to match your schematic
///
/// If you are unsure, please drop by the Embassy Matrix chat for support, and let us know:
///
/// * Which example you are trying to run
/// * Which chip and board you are using
///
/// Embassy Chat: https://matrix.to/#/#embassy-rs:matrix.org
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
