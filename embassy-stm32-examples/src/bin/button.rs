#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_stm32::gpio::{Input, Level, Output, Pull};
use embedded_hal::digital::v2::{InputPin, OutputPin};
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

    let p = embassy_stm32::Peripherals::take().unwrap();
    let button = Input::new(p.PC13, Pull::Down);
    let mut led1 = Output::new(p.PB0, Level::High);
    let _led2 = Output::new(p.PB7, Level::High);
    let mut led3 = Output::new(p.PB14, Level::High);

    loop {
        if button.is_high().unwrap() {
            info!("high");
            led1.set_high().unwrap();
            led3.set_low().unwrap();
        } else {
            info!("low");
            led1.set_low().unwrap();
            led3.set_high().unwrap();
        }
    }
}
