#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_stm32::adc::{Adc, Resolution, SampleTime};
use embassy_stm32::{Config, adc};
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux.adcsel = mux::Adcsel::Sys;
    }
    let p = embassy_stm32::init(config);

    let mut config = adc::Config::default();
    config.resolution = Some(Resolution::Bits8);

    let mut adc = Adc::new_blocking(p.ADC1, config);
    //adc.enable_vrefint();

    let mut channel = p.PC0;

    loop {
        let v = adc.blocking_read(&mut channel, SampleTime::from_bits(0));
        info!("--> {}", v);
    }
}
