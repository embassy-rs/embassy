#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use core::fmt::Write;
use embassy::executor::Executor;
use embassy::time::Clock;
use embassy::util::Forever;
use embassy_stm32::dma::NoDma;
use embassy_stm32::spi;
use embedded_hal::blocking::spi::Transfer;
use example_common::*;

use core::str::from_utf8;
use cortex_m_rt::entry;
use embassy_stm32::dbgmcu::Dbgmcu;
use embassy_stm32::peripherals::SPI3;
use embassy_stm32::time::U32Ext;
use heapless::String;

#[embassy::task]
async fn main_task(mut spi: spi::Spi<'static, SPI3, NoDma, NoDma>) {
    for n in 0u32.. {
        let mut write: String<128> = String::new();
        core::write!(&mut write, "Hello DMA World {}!\r\n", n).unwrap();
        unsafe {
            let result = spi.transfer(write.as_bytes_mut());
            if let Err(_) = result {
                defmt::panic!("crap");
            }
        }
        info!("read via spi: {}", from_utf8(write.as_bytes()).unwrap());
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

    let p = embassy_stm32::init(config());

    let spi = spi::Spi::new(
        p.SPI3,
        p.PB3,
        p.PB5,
        p.PB4,
        NoDma,
        NoDma,
        1.mhz(),
        spi::Config::default(),
    );

    unsafe { embassy::time::set_clock(&ZeroClock) };
    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task(spi)));
    })
}
