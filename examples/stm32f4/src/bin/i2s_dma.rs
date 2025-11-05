// This example is written for an STM32F411 chip communicating with an external
// PCM5102a DAC. Remap pins, change clock speeds, etc. as necessary for your own
// hardware.
//
// NOTE: This example outputs potentially loud audio. Please run responsibly.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::i2s::{Config, Format, I2S};
use embassy_stm32::time::Hertz;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = {
        use embassy_stm32::rcc::*;

        let mut config = embassy_stm32::Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(25),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV25,
            mul: PllMul::MUL192,
            divp: Some(PllPDiv::DIV2),
            divq: Some(PllQDiv::DIV4),
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P;

        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;

        // reference your chip's manual for proper clock settings; this config
        // is recommended for a 32 bit frame at 48 kHz sample rate
        config.rcc.plli2s = Some(Pll {
            prediv: PllPreDiv::DIV25,
            mul: PllMul::MUL384,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV5),
        });
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
        p.SPI3,
        p.PB5,  // sd
        p.PA15, // ws
        p.PB3,  // ck
        p.DMA1_CH7,
        &mut dma_buffer,
        i2s_config,
    );
    i2s.start();

    loop {
        i2s.write(&wavetable).await.ok();
    }
}
