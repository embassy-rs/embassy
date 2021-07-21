#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use core::fmt::Write;
use embassy::executor::Executor;
use embassy::time::Clock;
use embassy::util::Forever;
use embassy_stm32::dma::NoDma;
use example_common::*;
use embedded_hal::blocking::spi::Transfer;

use hal::prelude::*;
use stm32h7xx_hal as hal;

use cortex_m_rt::entry;
use stm32h7::stm32h743 as pac;
use heapless::String;
use embassy_stm32::spi::{Spi, Config};
use embassy_stm32::time::Hertz;

#[embassy::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let mut spi = Spi::new(
        p.SPI3,
        p.PB3,
        p.PB5,
        p.PB4,
        NoDma,
        NoDma,
        Hertz(1_000_000),
        Config::default(),
    );

    for n in 0u32.. {
        let mut write: String<128> = String::new();
        core::write!(&mut write, "Hello DMA World {}!\r\n", n).unwrap();
        unsafe {
            let result = spi.transfer(write.as_bytes_mut());
            if let Err(_) = result {
                defmt::panic!("crap");
            }
        }
        info!("read via spi: {}", write.as_bytes());
    }
}

struct ZeroClock;

impl Clock for ZeroClock {
    fn now(&self) -> u64 {
        0
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let pp = pac::Peripherals::take().unwrap();

    let pwrcfg = pp.PWR.constrain().freeze();

    let rcc = pp.RCC.constrain();

    rcc.sys_ck(96.mhz())
        .pclk1(48.mhz())
        .pclk2(48.mhz())
        .pclk3(48.mhz())
        .pclk4(48.mhz())
        .pll1_q_ck(48.mhz())
        .freeze(pwrcfg, &pp.SYSCFG);

    let pp = unsafe { pac::Peripherals::steal() };

    pp.DBGMCU.cr.modify(|_, w| {
        w.dbgsleep_d1().set_bit();
        w.dbgstby_d1().set_bit();
        w.dbgstop_d1().set_bit();
        w.d1dbgcken().set_bit();
        w
    });

    pp.RCC.ahb4enr.modify(|_, w| {
        w.gpioaen().set_bit();
        w.gpioben().set_bit();
        w.gpiocen().set_bit();
        w.gpioden().set_bit();
        w.gpioeen().set_bit();
        w.gpiofen().set_bit();
        w
    });

    unsafe { embassy::time::set_clock(&ZeroClock) };

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}
