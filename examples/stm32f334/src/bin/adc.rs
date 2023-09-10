#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, VREF_INT};
use embassy_stm32::rcc::{ADCClock, ADCPrescaler};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_time::{Delay, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut config = Config::default();
    config.rcc.hse = Some(Hertz(8_000_000));
    config.rcc.sysclk = Some(Hertz(16_000_000));
    config.rcc.adc = Some(ADCClock::PLL(ADCPrescaler::Div1));

    let mut p = embassy_stm32::init(config);

    info!("create adc...");

    let mut adc = Adc::new(p.ADC1, &mut Delay);

    info!("enable vrefint...");

    let mut vrefint = adc.enable_vref(&mut Delay);
    let mut temperature = adc.enable_temperature();

    loop {
        let vref = adc.read(&mut vrefint);
        info!("read vref: {}", vref);

        let temp = adc.read(&mut temperature);
        info!("read temperature: {}", temp);

        let pin = adc.read(&mut p.PA0);
        info!("read pin: {}", pin);

        let pin_mv = pin as u32 * VREF_INT as u32 / vref as u32;
        info!("computed pin mv: {}", pin_mv);

        Timer::after(Duration::from_secs(1)).await;
    }
}
