#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::Config;
use embassy_time::{Delay, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(Hsi::Mhz64);
        config.rcc.csi = true;
        config.rcc.pll_src = PllSource::Hsi;
        config.rcc.pll1 = Some(Pll {
            prediv: 4,
            mul: 50,
            divp: Some(2),
            divq: Some(8), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            prediv: 4,
            mul: 50,
            divp: Some(8), // 100mhz
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.adc_clock_source = AdcClockSource::PLL2_P;
    }
    let mut p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut adc = Adc::new(p.ADC3, &mut Delay);

    adc.set_sample_time(SampleTime::Cycles32_5);

    let mut vrefint_channel = adc.enable_vrefint();

    loop {
        let vrefint = adc.read_internal(&mut vrefint_channel);
        info!("vrefint: {}", vrefint);
        let measured = adc.read(&mut p.PC0);
        info!("measured: {}", measured);
        Timer::after(Duration::from_millis(500)).await;
    }
}
