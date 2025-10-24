#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, AdcChannel as _, SampleTime};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[unsafe(link_section = ".ram_d3")]
static mut DMA_BUF: [u16; 2] = [0; 2];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut read_buffer = unsafe { &mut DMA_BUF[..] };

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV8), // 100mhz
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.adcsel = mux::Adcsel::PLL2_P;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut adc = Adc::new(p.ADC3);

    let mut dma = p.DMA1_CH1;
    let mut vrefint_channel = adc.enable_vrefint().degrade_adc();
    let mut pc0 = p.PC0.degrade_adc();

    loop {
        adc.read(
            dma.reborrow(),
            [
                (&mut vrefint_channel, SampleTime::CYCLES387_5),
                (&mut pc0, SampleTime::CYCLES810_5),
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
