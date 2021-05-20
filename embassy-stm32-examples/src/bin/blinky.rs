#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_stm32::gpio::{Level, Output};
use embedded_hal::digital::v2::OutputPin;
use example_common::*;

use cortex_m_rt::entry;
use stm32f4::stm32f429 as pac;

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let pp = pac::Peripherals::take().unwrap();

    pp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });
    pp.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());

    pp.RCC.ahb1enr.modify(|_, w| {
        w.gpioaen().enabled();
        w.gpioben().enabled();
        w.gpiocen().enabled();
        w.gpioden().enabled();
        w.gpioeen().enabled();
        w.gpiofen().enabled();
        w
    });

    let p = embassy_stm32::init(Default::default());

    let mut led = Output::new(p.PB7, Level::High);

    loop {
        info!("high");
        led.set_high().unwrap();
        cortex_m::asm::delay(10_000_000);

        info!("low");
        led.set_low().unwrap();
        cortex_m::asm::delay(10_000_000);
    }
}
