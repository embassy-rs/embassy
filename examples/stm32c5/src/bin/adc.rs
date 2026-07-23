#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, AdcConfig, SampleTime};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux.adcdacsel = mux::Adcdacsel::Hclk1;
        config.rcc.adcdac_pre = embassy_stm32::pac::rcc::vals::Adcdacpre::Div2;
    }
    let mut p = embassy_stm32::init(config);

    info!("Hello World!");

    let adc1_config = AdcConfig::default();
    let adc2_config = AdcConfig::default();
    let mut adc1 = Adc::new_with_config(p.ADC1, adc1_config);
    let mut adc2 = Adc::new_with_config(p.ADC2, adc2_config);

    let mut vrefint_channel = adc1.enable_vrefint();

    loop {
        let vrefint = adc1.blocking_read(&mut vrefint_channel, SampleTime::Cycles289);
        info!("vrefint: {}", vrefint);
        let measured = adc2.blocking_read(&mut p.PH4, SampleTime::Cycles289);
        info!("PH4 aka A0: {}", measured);
        Timer::after_millis(500).await;
    }
}
