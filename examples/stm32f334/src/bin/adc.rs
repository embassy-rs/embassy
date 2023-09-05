#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::adc::Adc;
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

    let mut adc = Adc::new(p.ADC1, &mut Delay);

    let mut vrefint = adc.enable_vref(&mut Delay);

    let _vref = adc.read(&mut vrefint);
    let _pin = adc.read(&mut p.PA0);

    loop {
        info!("Hello World!");
        Timer::after(Duration::from_secs(1)).await;
    }
}
