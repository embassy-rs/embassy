#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, Resolution};
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

    let mut adc = Adc::new(p.ADC1);
    adc.set_resolution(Resolution::BITS8);
    let mut channel = p.PC0;

    loop {
        let v = adc.blocking_read(&mut channel);
        info!("--> {}", v);
        embassy_time::block_for(Duration::from_millis(200));
    }
}
