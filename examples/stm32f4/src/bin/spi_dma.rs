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
use cortex_m_rt::entry;
use embassy::executor::Executor;
use embassy::time::Clock;
use embassy::util::Forever;
use example_common::*;
use embassy_traits::spi::FullDuplex;
use heapless::String;
use embassy_stm32::spi::{Spi, Config};
use embassy_stm32::pac;
use embassy_stm32::time::Hertz;
use core::str::from_utf8;

#[embassy::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let mut spi = Spi::new(
        p.SPI1,
        p.PB3,
        p.PA7,
        p.PA6,
        p.DMA2_CH3,
        p.DMA2_CH2,
        Hertz(1_000_000),
        Config::default(),
    );

    for n in 0u32.. {
        let mut write: String<128> = String::new();
        let mut read = [0;128];
        core::write!(&mut write, "Hello DMA World {}!\r\n", n).unwrap();
        spi.read_write(&mut read[0..write.len()], write.as_bytes()).await.ok();
        info!("read via spi+dma: {}", from_utf8(&read).unwrap());
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
    unsafe {
        pac::DBGMCU.cr().modify(|w| {
            w.set_dbg_sleep(true);
            w.set_dbg_standby(true);
            w.set_dbg_stop(true);
        });

        pac::RCC.ahb1enr().modify(|w| {
            w.set_gpioaen(true);
            w.set_gpioben(true);
            w.set_gpiocen(true);
            w.set_gpioden(true);
            w.set_gpioeen(true);
            w.set_gpiofen(true);
        });
    }

    unsafe { embassy::time::set_clock(&ZeroClock) };

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}
