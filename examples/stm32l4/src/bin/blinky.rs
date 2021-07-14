#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use cortex_m_rt::entry;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::pac;
use embedded_hal::digital::v2::OutputPin;
use example_common::*;

#[entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        pac::DBGMCU.cr().modify(|w| {
            w.set_dbg_sleep(true);
            w.set_dbg_standby(true);
            w.set_dbg_stop(true);
        });

        pac::RCC.ahb2enr().modify(|w| {
            w.set_gpioaen(true);
            w.set_gpioben(true);
            w.set_gpiocen(true);
            w.set_gpioden(true);
            w.set_gpioeen(true);
            w.set_gpiofen(true);
        });
    }

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
