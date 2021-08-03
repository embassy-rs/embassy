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
use embassy_stm32::dma::NoDma;
use embassy_stm32::usart::{Config, Uart};
use example_common::*;

use cortex_m_rt::entry;
use embassy_stm32::dbgmcu::Dbgmcu;

#[embassy::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let config = Config::default();
    let mut usart = Uart::new(p.UART7, p.PF6, p.PF7, NoDma, NoDma, config);

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

    unsafe {
        Dbgmcu::enable_all();
    }

    unsafe { embassy::time::set_clock(&ZeroClock) };

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}
