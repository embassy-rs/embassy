#![no_std]
#![no_main]
#![allow(incomplete_features)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::time::Delay;
use embassy_stm32::adc::{Adc, Resolution};
use embassy_stm32::dbgmcu::Dbgmcu;
use embassy_stm32::pac;
use example_common::*;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        Dbgmcu::enable_all();

        pac::RCC.ccipr().modify(|w| {
            w.set_adcsel(0b11);
        });
        pac::RCC.ahb2enr().modify(|w| w.set_adcen(true));
    }

    let p = embassy_stm32::init(Default::default());

    let mut adc = Adc::new(p.ADC1, &mut Delay);
    //adc.enable_vref();
    adc.set_resolution(Resolution::EightBit);
    let mut channel = p.PC0;

    loop {
        let v = adc.read(&mut channel);
        info!("--> {}", v);
    }
}
