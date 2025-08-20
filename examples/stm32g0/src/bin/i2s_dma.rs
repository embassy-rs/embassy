// This example is written for an STM32G0B1 chip communicating with an external
// DAC. Remap pins, change clock speeds, etc. as necessary for your own
// hardware.
//
// NOTE: This example outputs potentially loud audio. Please run responsibly.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::i2s::{Config, Format, I2S};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = {
        use embassy_stm32::rcc::*;

        let mut config = embassy_stm32::Config::default();
        config.rcc.hsi = Some(Hsi {
            sys_div: HsiSysDiv::DIV1,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL16,
            divp: None,
            divq: Some(PllQDiv::DIV2), // 16 / 1 * 16 / 2 = 128 MHz
            divr: Some(PllRDiv::DIV4), // 16 / 1 * 16 / 4 = 64 MHz
        });
        config.rcc.sys = Sysclk::PLL1_R;

        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV1;

        config.enable_debug_during_sleep = true;

        config
    };

    let p = embassy_stm32::init(config);

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
        p.SPI2,
        p.PB15,  // sd
        p.PA8,   // ws
        p.PB10,  // ck
        p.DMA1_CH1,
        &mut dma_buffer,
        i2s_config,
    );
    i2s.start();

    loop {
        i2s.write(&wavetable).await.ok();
    }
}
