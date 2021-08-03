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
use embassy_stm32::rcc;
use embassy_stm32::time::U32Ext;
use embassy_stm32::Config;

#[entry]
fn main() -> ! {
    info!("Hello World, dude!");

    let p = embassy_stm32::init(
        Config::default().rcc(rcc::Config::default().sys_ck(400.mhz()).pll1_q(100.mhz())),
    );

    unsafe {
        Dbgmcu::enable_all();
    }

    let mut dac = Dac::new(p.DAC1, p.PA4, NoPin);

    loop {
        for v in 0..=255 {
            unwrap!(dac.set(Channel::Ch1, Value::Bit8(to_sine_wave(v))));
            unwrap!(dac.trigger(Channel::Ch1));
        }
    }
}

use embassy_stm32::dbgmcu::Dbgmcu;
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
