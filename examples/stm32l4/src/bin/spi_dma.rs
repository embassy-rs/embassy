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
use embassy::executor::Executor;
use embassy::time::Clock;
use embassy::util::Forever;
use embassy_stm32::pac;
use example_common::*;
use embassy_stm32::spi::{Spi, Config};
use embassy_traits::spi::FullDuplex;
use embassy_stm32::time::Hertz;
use embassy_stm32::gpio::{Output, Level, Speed, Input, Pull};
use embedded_hal::digital::v2::{OutputPin, InputPin};

#[embassy::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let mut spi = Spi::new(
        p.SPI3,
        p.PC10,
        p.PC12,
        p.PC11,
        p.DMA1_CH0,
        p.DMA1_CH1,
        Hertz(1_000_000),
        Config::default(),
    );


    // These are the pins for the Inventek eS-Wifi SPI Wifi Adapter.

    let _boot = Output::new(p.PB12, Level::Low, Speed::VeryHigh);
    let _wake = Output::new(p.PB13, Level::Low, Speed::VeryHigh);
    let mut reset = Output::new(p.PE8, Level::Low, Speed::VeryHigh);
    let mut cs = Output::new(p.PE0, Level::High, Speed::VeryHigh);
    let ready = Input::new(p.PE1, Pull::Up);

    cortex_m::asm::delay(100_000);
    reset.set_high().unwrap();
    cortex_m::asm::delay(100_000);

    while ready.is_low().unwrap() {
        info!("waiting for ready");
    }

    let write = [0x0A; 10];
    let mut read = [0; 10];
    unwrap!(cs.set_low());
    spi.read_write(&mut read, &write).await.ok();
    unwrap!(cs.set_high());
    info!("xfer {=[u8]:x}", read);
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

        //pac::RCC.apbenr().modify(|w| {
        //w.set_spi3en(true);
        // });

        pac::RCC.apb2enr().modify(|w| {
            w.set_syscfgen(true);
        });

        pac::RCC.ahb1enr().modify(|w| {
            w.set_dmamux1en(true);
            w.set_dma1en(true);
            w.set_dma2en(true);
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

    unsafe { embassy::time::set_clock(&ZeroClock) };

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}
