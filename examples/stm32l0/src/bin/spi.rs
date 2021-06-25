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
use embassy_stm32::rcc;
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embedded_hal::blocking::spi::Transfer;

#[entry]
fn main() -> ! {
    info!("Hello World, dude!");

    let mut p = embassy_stm32::init(Default::default());
    let mut rcc = rcc::Rcc::new(p.RCC);
    rcc.enable_debug_wfe(&mut p.DBGMCU, true);

    let mut spi = Spi::new(
        p.SPI1,
        p.PB3,
        p.PA7,
        p.PA6,
        Hertz(1_000_000),
        Config::default(),
    );

    let mut cs = Output::new(p.PA15, Level::High, Speed::VeryHigh);

    loop {
        let mut buf = [0x0A; 4];
        unwrap!(cs.set_low());
        unwrap!(spi.transfer(&mut buf));
        unwrap!(cs.set_high());
        info!("xfer {=[u8]:x}", buf);
    }
}
