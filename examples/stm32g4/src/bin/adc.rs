#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul85,
            divp: None,
            divq: None,
            // Main system clock at 170 MHz
            divr: Some(PllRDiv::Div2),
        });
        config.rcc.mux.adc12sel = mux::Adcsel::Sys;
        config.rcc.sys = Sysclk::Pll1R;
    }
    let mut p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC2, Default::default());

    let mut adc_temp = Adc::new(p.ADC1, Default::default());
    let mut temperature = adc_temp.enable_temperature();

    loop {
        let measured = adc.blocking_read(&mut p.PA7, SampleTime::Cycles245);
        let temperature = adc_temp.blocking_read(&mut temperature, SampleTime::Cycles245);
        info!("measured: {}", measured);
        info!("temperature: {}", temperature);
        Timer::after_millis(500).await;
    }
}
