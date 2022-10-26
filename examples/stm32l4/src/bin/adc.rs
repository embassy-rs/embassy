#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_stm32::adc::{Adc, Resolution, SampleTime};
use embassy_stm32::pac;
use embassy_time::Delay;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        pac::RCC.ccipr().modify(|w| {
            w.set_adcsel(0b11);
        });
        pac::RCC.ahb2enr().modify(|w| w.set_adcen(true));
    }

    let p = embassy_stm32::init(Default::default());

    let mut adc = Adc::new(p.ADC1, &mut Delay);
    let mut pin = p.PC0;

    let mut input = adc.single_channel(&mut pin, SampleTime::default(), Resolution::EightBit);

    loop {
        let v = input.read();
        info!("--> {}", v);
    }
}
