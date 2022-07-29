#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Delay, Duration, Timer};
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::rcc::AdcClockSource;
use embassy_stm32::time::mhz;
use embassy_stm32::{Config, Peripherals};
use {defmt_rtt as _, panic_probe as _};

pub fn config() -> Config {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(mhz(400));
    config.rcc.hclk = Some(mhz(200));
    config.rcc.per_ck = Some(mhz(64));
    config.rcc.adc_clock_source = AdcClockSource::PerCk;
    config
}

#[embassy_executor::main(config = "config()")]
async fn main(_spawner: Spawner, mut p: Peripherals) {
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
