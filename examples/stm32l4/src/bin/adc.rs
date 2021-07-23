#![no_std]
#![no_main]
#![allow(incomplete_features)]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use example_common::*;

use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use embassy_stm32::adc::{Adc, Resolution};
use embassy_stm32::pac;
use stm32l4xx_hal::prelude::*;
use stm32l4xx_hal::rcc::PllSource;

#[entry]
fn main() -> ! {
    info!("Hello World, dude!");
    //let pp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let pp = stm32l4xx_hal::stm32::Peripherals::take().unwrap();
    let mut flash = pp.FLASH.constrain();
    let mut rcc = pp.RCC.constrain();
    let mut pwr = pp.PWR.constrain(&mut rcc.apb1r1);

    let mut delay = Delay::new(cp.SYST, 80_000_000);

    // TRY the other clock configuration
    // let clocks = rcc.cfgr.freeze(&mut flash.acr);
    rcc.cfgr
        .sysclk(80.mhz())
        .pclk1(80.mhz())
        .pclk2(80.mhz())
        .pll_source(PllSource::HSI16)
        .freeze(&mut flash.acr, &mut pwr);

    unsafe {
        pac::RCC.ccipr().modify(|w| {
            w.set_adcsel(0b11);
        });
        pac::DBGMCU.cr().modify(|w| {
            w.set_dbg_sleep(true);
            w.set_dbg_standby(true);
            w.set_dbg_stop(true);
        });
    }

    let p = embassy_stm32::init(Default::default());

    let mut adc = Adc::new(p.ADC1, &mut delay);
    //adc.enable_vref();
    adc.set_resolution(Resolution::EightBit);
    let mut channel = p.PC0;

    loop {
        let v = adc.read(&mut channel);
        info!("--> {}", v);
    }
}
