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
use embassy::util::Forever;
use embassy_stm32::dbgmcu::Dbgmcu;
use embassy_stm32::dma::NoDma;
use embassy_stm32::usart::{Config, Uart};
use embassy_traits::uart::Write as _;
use example_common::*;
use heapless::String;

#[embassy::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let config = Config::default();
    let mut usart = Uart::new(p.USART3, p.PD9, p.PD8, p.DMA1_CH3, NoDma, config);

    for n in 0u32.. {
        let mut s: String<128> = String::new();
        core::write!(&mut s, "Hello DMA World {}!\r\n", n).unwrap();

        usart.write(s.as_bytes()).await.unwrap();
        info!("wrote DMA");
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        Dbgmcu::enable_all();
    }

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}
