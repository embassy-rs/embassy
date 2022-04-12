#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy::executor::Spawner;
use embassy::time::{Delay, Duration, Timer};
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::rcc::AdcClockSource;
use embassy_stm32::time::U32Ext;
use embassy_stm32::{Config, Peripherals};

use defmt::*;
use defmt_rtt as _; // global logger
use panic_probe as _;

pub fn config() -> Config {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(400.mhz().into());
    config.rcc.hclk = Some(200.mhz().into());
    config.rcc.per_ck = Some(64.mhz().into());
    config.rcc.adc_clock_source = AdcClockSource::PerCk;
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, mut p: Peripherals) {
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC3, &mut Delay);

    adc.set_sample_time(SampleTime::Cycles32_5);

    let mut vref_channel = adc.enable_vref();

    loop {
        let vref = adc.read_internal(&mut vref_channel);
        info!("vref: {}", vref);
        let measured = adc.read(&mut p.PC0);
        info!("measured: {}", measured);
        Timer::after(Duration::from_millis(500)).await;
    }
}
