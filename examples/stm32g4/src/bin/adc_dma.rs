#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel as _, SampleTime};
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

static mut DMA_BUF: [u16; 2] = [0; 2];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut read_buffer = unsafe { &mut DMA_BUF[..] };

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL85,
            divp: None,
            divq: None,
            // Main system clock at 170 MHz
            divr: Some(PllRDiv::DIV2),
        });
        config.rcc.mux.adc12sel = mux::Adcsel::SYS;
        config.rcc.sys = Sysclk::PLL1_R;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1);

    let mut dma = p.DMA1_CH1;
    let mut vrefint_channel = adc.enable_vrefint().degrade_adc();
    let mut pa0 = p.PA0.degrade_adc();

    loop {
        adc.read(
            &mut dma,
            [
                (&mut vrefint_channel, SampleTime::CYCLES247_5),
                (&mut pa0, SampleTime::CYCLES247_5),
            ]
            .into_iter(),
            &mut read_buffer,
        )
        .await;

        let vrefint = read_buffer[0];
        let measured = read_buffer[1];
        info!("vrefint: {}", vrefint);
        info!("measured: {}", measured);
        Timer::after_millis(500).await;
    }
}
