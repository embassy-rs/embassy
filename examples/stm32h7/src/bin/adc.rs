#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, Resolution, SampleTime};
use embassy_stm32::rcc::AdcClockSource;
use embassy_stm32::time::mhz;
use embassy_stm32::Config;
use embassy_time::{Delay, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(mhz(400));
    config.rcc.hclk = Some(mhz(200));
    config.rcc.per_ck = Some(mhz(64));
    config.rcc.adc_clock_source = AdcClockSource::PerCk;
    let mut p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut adc = Adc::new(p.ADC3, &mut Delay);

    let mut vrefint_channel = adc.enable_vrefint();

    loop {
        let vrefint = adc.read_internal(&mut vrefint_channel, SampleTime::Cycles32_5, Resolution::TwelveBit);
        info!("vrefint: {}", vrefint);
        let measured = adc.read(&mut p.PC0, SampleTime::Cycles32_5, Resolution::TwelveBit);
        info!("measured: {}", measured);
        Timer::after(Duration::from_millis(500)).await;
    }
}
