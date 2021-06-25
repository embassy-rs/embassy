#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_stm32::gpio::{Level, Output, Speed};
use embedded_hal::digital::v2::OutputPin;
use example_common::*;

use cortex_m_rt::entry;
use stm32l4::stm32l4x5 as pac;

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let pp = pac::Peripherals::take().unwrap();

    pp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });

    pp.RCC.ahb2enr.modify(|_, w| {
        w.gpioaen().set_bit();
        w.gpioben().set_bit();
        w.gpiocen().set_bit();
        w.gpioden().set_bit();
        w.gpioeen().set_bit();
        w.gpiofen().set_bit();
        w
    });

    let p = embassy_stm32::init(Default::default());

    let mut led = Output::new(p.PB14, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high().unwrap();
        cortex_m::asm::delay(10_000_000);

        info!("low");
        led.set_low().unwrap();
        cortex_m::asm::delay(10_000_000);
    }
}
