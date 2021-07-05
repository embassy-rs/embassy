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
use stm32h7::stm32h743 as pac;
use stm32h7xx_hal::prelude::*;
use embassy_stm32::dac::{Dac, Value, Channel};

#[entry]
fn main() -> ! {
    info!("Hello World, dude!");

    let pp = pac::Peripherals::take().unwrap();

    let pwrcfg = pp.PWR.constrain()
        .freeze();

    let rcc = pp.RCC.constrain();

    rcc
        .sys_ck(96.mhz())
        .pclk1(48.mhz())
        .pclk2(48.mhz())
        .pclk3(48.mhz())
        .pclk4(48.mhz())
        .pll1_q_ck(48.mhz())
        .freeze(pwrcfg, &pp.SYSCFG);

    let pp = unsafe { pac::Peripherals::steal() };

    pp.DBGMCU.cr.modify(|_, w| {
        w.dbgsleep_d1().set_bit();
        w.dbgstby_d1().set_bit();
        w.dbgstop_d1().set_bit();
        w.d1dbgcken().set_bit();
        w
    });

    pp.RCC.apb1lenr.modify(|_, w|{
        w.dac12en().set_bit();
        w
    });

    pp.RCC.ahb4enr.modify(|_, w| {
        w.gpioaen().set_bit();
        w.gpioben().set_bit();
        w.gpiocen().set_bit();
        w.gpioden().set_bit();
        w.gpioeen().set_bit();
        w.gpiofen().set_bit();
        w
    });

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
        let r = 3.14 * ( (v-128) as f32/ 128.0) ;
        (r.sin() * 128.0 + 127.0) as u8
    } else {
        // bottom half
        let r = 3.14 + 3.14 * (v as f32/ 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    }
}
