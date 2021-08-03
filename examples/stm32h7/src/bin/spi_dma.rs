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
use embassy_stm32::time::U32Ext;
use embassy::util::Forever;
use example_common::*;
use embassy_traits::spi::FullDuplex;

use cortex_m_rt::entry;
use heapless::String;
use embassy_stm32::spi;
use embassy_stm32::Config;
use core::str::from_utf8;
use embassy_stm32::dbgmcu::Dbgmcu;
use embassy_stm32::rcc;
use embassy_stm32::peripherals::{DMA1_CH4, DMA1_CH3, SPI3};

#[embassy::task]
async fn main_task(mut spi: spi::Spi<'static, SPI3, DMA1_CH3, DMA1_CH4>) {

    for n in 0u32.. {
        let mut write: String<128> = String::new();
        let mut read = [0;128];
        core::write!(&mut write, "Hello DMA World {}!\r\n", n).unwrap();
        // read_write will slice the &mut read down to &write's actual length.
        spi.read_write(&mut read, write.as_bytes()).await.ok();
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
        Dbgmcu::enable_all();
    }

    let p = embassy_stm32::init(Config::default().rcc(
        rcc::Config::default()
            .sys_ck(400.mhz())
            .pll1_q(100.mhz())
    ));

    let spi = spi::Spi::new(
        p.SPI3,
        p.PB3,
        p.PB5,
        p.PB4,
        p.DMA1_CH3,
        p.DMA1_CH4,
        1.mhz(),
        spi::Config::default(),
    );

    unsafe { embassy::time::set_clock(&ZeroClock) };
    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task(spi)));
    })
}
