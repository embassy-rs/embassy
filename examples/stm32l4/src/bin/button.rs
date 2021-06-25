#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embedded_hal::digital::v2::{InputPin, OutputPin};
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

    let button = Input::new(p.PC13, Pull::Up);
    let mut led1 = Output::new(p.PA5, Level::High, Speed::Low);
    let mut led2 = Output::new(p.PB14, Level::High, Speed::Low);

    loop {
        if button.is_high().unwrap() {
            info!("high");
            led1.set_high().unwrap();
            led2.set_low().unwrap();
        } else {
            info!("low");
            led1.set_low().unwrap();
            led2.set_high().unwrap();
        }
    }
}
