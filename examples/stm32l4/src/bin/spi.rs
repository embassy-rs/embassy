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
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;
use example_common::*;

#[entry]
fn main() -> ! {
    info!("Hello World, dude!");

    unsafe {
        pac::DBGMCU.cr().modify(|w| {
            w.set_dbg_sleep(true);
            w.set_dbg_standby(true);
            w.set_dbg_stop(true);
        });

        pac::RCC.apb2enr().modify(|w| {
            w.set_syscfgen(true);
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

    let mut spi = Spi::new(
        p.SPI3,
        p.PC10,
        p.PC12,
        p.PC11,
        Hertz(1_000_000),
        Config::default(),
    );

    let mut cs = Output::new(p.PE0, Level::High, Speed::VeryHigh);

    loop {
        let mut buf = [0x0A; 4];
        unwrap!(cs.set_low());
        unwrap!(spi.transfer(&mut buf));
        unwrap!(cs.set_high());
        info!("xfer {=[u8]:x}", buf);
    }
}
