#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use embassy_stm32::hal::prelude::*;

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_stm32::pac::Peripherals::take().unwrap();

    p.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });
    p.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());

    let gpioa = p.GPIOA.split();
    let gpioc = p.GPIOC.split();

    let mut led = gpioc.pc13.into_push_pull_output();
    let button = gpioa.pa0.into_pull_up_input();
    led.set_low().unwrap();

    loop {
        if button.is_high().unwrap() {
            led.set_low().unwrap();
        } else {
            led.set_high().unwrap();
        }
    }
}
