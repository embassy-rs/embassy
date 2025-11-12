#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, AdcConfig, Resolution, SampleTime};
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux.adcsel = mux::Adcsel::SYS;
    }
    let p = embassy_stm32::init(config);

    let mut config = AdcConfig::default();
    config.resolution = Some(Resolution::BITS8);
    let mut adc = Adc::new_with_config(p.ADC1, config);
    let mut channel = p.PC0;

    loop {
        let v = adc.blocking_read(&mut channel, SampleTime::CYCLES12_5);
        info!("--> {}", v);
        embassy_time::block_for(Duration::from_millis(200));
    }
}
