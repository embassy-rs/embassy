// required-features: i2s_f4

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::i2s::{Config, Format, I2S};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, dma, peripherals};
use {defmt_rtt as _, panic_probe as _};

teleprobe_meta::target!(b"nucleo-stm32f429zi");

bind_interrupts!(struct Irqs {
    DMA1_STREAM7 => dma::InterruptHandler<peripherals::DMA1_CH7>;
});

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let config = {
        use embassy_stm32::rcc::*;

        let mut config = embassy_stm32::Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::Hse;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul180,
            divp: Some(PllPDiv::Div2), // 8mhz / 4 * 180 / 2 = 180Mhz.
            divq: None,
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div4;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.sys = Sysclk::Pll1P;

        // reference your chip's manual for proper clock settings; this config
        // is recommended for a 32 bit frame at 48 kHz sample rate
        config.rcc.plli2s = Some(Pll {
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul60,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::Div2),
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
        Irqs,
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
