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
//use stm32l4::stm32l4x5 as pac;
use embassy_stm32::spi::{ByteOrder, Config, Spi, MODE_0};
use embassy_stm32::time::Hertz;
use embedded_hal::blocking::spi::Transfer;

#[entry]
fn main() -> ! {
    info!("Hello World, dude!");

    let pp = pac::Peripherals::take().unwrap();

    pp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });
    pp.RCC.ahb1enr.modify(|_, w| w.dma1en().set_bit());

    pp.RCC.apb1enr.modify(|_, w| {
        w.spi3en().enabled();
        w
    });

    pp.RCC.ahb1enr.modify(|_, w| {
        w.gpioaen().enabled();
        w.gpioben().enabled();
        w.gpiocen().enabled();
        w.gpioden().enabled();
        w.gpioeen().enabled();
        w.gpiofen().enabled();
        w
    });

    let rc = pp.RCC.cfgr.read().sws().bits();
    let p = embassy_stm32::init(Default::default());

    let mut spi = Spi::new(
        Hertz(16_000_000),
        p.SPI3,
        p.PC10,
        p.PC12,
        p.PC11,
        Hertz(1_000_000),
        Config::default(),
    );

    let mut cs = Output::new(p.PE0, Level::High);

    loop {
        let mut buf = [0x0A; 4];
        cs.set_low();
        spi.transfer(&mut buf);
        cs.set_high();
        info!("xfer {=[u8]:x}", buf);
    }
}
