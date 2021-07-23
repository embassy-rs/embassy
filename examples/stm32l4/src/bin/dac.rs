#![no_std]
#![no_main]
#![allow(incomplete_features)]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy_stm32::gpio::NoPin;
use example_common::*;

use cortex_m_rt::entry;
use embassy_stm32::dac::{Channel, Dac, Value};
use embassy_stm32::pac;
use stm32l4xx_hal::prelude::*;
use stm32l4xx_hal::rcc::PllSource;

#[entry]
fn main() -> ! {
    info!("Hello World, dude!");
    //let pp = pac::Peripherals::take().unwrap();
    let pp = stm32l4xx_hal::stm32::Peripherals::take().unwrap();
    let mut flash = pp.FLASH.constrain();
    let mut rcc = pp.RCC.constrain();
    let mut pwr = pp.PWR.constrain(&mut rcc.apb1r1);

    // TRY the other clock configuration
    // let clocks = rcc.cfgr.freeze(&mut flash.acr);
    rcc.cfgr
        .sysclk(80.mhz())
        .pclk1(80.mhz())
        .pclk2(80.mhz())
        .pll_source(PllSource::HSI16)
        .freeze(&mut flash.acr, &mut pwr);

    unsafe {
        pac::DBGMCU.cr().modify(|w| {
            w.set_dbg_sleep(true);
            w.set_dbg_standby(true);
            w.set_dbg_stop(true);
        });
        pac::RCC.apb1enr1().modify(|w| {
            w.set_dac1en(true);
        });
    }

    let p = embassy_stm32::init(Default::default());

    let mut dac = Dac::new(p.DAC1, p.PA4, NoPin);

    loop {
        for v in 0..=255 {
            unwrap!(dac.set(Channel::Ch1, Value::Bit8(to_sine_wave(v))));
            unwrap!(dac.trigger(Channel::Ch1));
        }
    }
}

use micromath::F32Ext;

fn to_sine_wave(v: u8) -> u8 {
    if v >= 128 {
        // top half
        let r = 3.14 * ((v - 128) as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    } else {
        // bottom half
        let r = 3.14 + 3.14 * (v as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    }
}
