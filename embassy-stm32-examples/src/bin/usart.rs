#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use cortex_m::prelude::_embedded_hal_blocking_serial_Write;
use embassy::executor::Executor;
use embassy::time::Clock;
use embassy::util::Forever;
use embassy_stm32::usart::{Config, Uart};
use example_common::*;

use cortex_m_rt::entry;
use stm32f4::stm32f429 as pac;

#[embassy::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let config = Config::default();
    let mut usart = Uart::new(p.USART3, p.PD9, p.PD8, config, 16_000_000);

    usart.bwrite_all(b"Hello Embassy World!\r\n").unwrap();
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        usart.read(&mut buf).unwrap();
        usart.bwrite_all(&buf).unwrap();
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

    pp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });
    pp.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());

    pp.RCC.ahb1enr.modify(|_, w| {
        w.gpioaen().enabled();
        w.gpioben().enabled();
        w.gpiocen().enabled();
        w.gpioden().enabled();
        w.gpioeen().enabled();
        w.gpiofen().enabled();
        w
    });
    pp.RCC.apb2enr.modify(|_, w| {
        w.syscfgen().enabled();
        w
    });
    pp.RCC.apb1enr.modify(|_, w| {
        w.usart3en().enabled();
        w
    });

    unsafe { embassy::time::set_clock(&ZeroClock) };

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}
