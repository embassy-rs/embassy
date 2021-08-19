#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use cortex_m_rt::entry;
use embassy_stm32::dma::NoDma;
use embassy_stm32::usart::{Config, Uart};
use embedded_hal::blocking::serial::Write;
use example_common::*;

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    let config = Config::default();
    let mut usart = Uart::new(p.USART3, p.PD9, p.PD8, NoDma, NoDma, config);

    unwrap!(usart.bwrite_all(b"Hello Embassy World!\r\n"));
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        unwrap!(usart.read_blocking(&mut buf));
        unwrap!(usart.bwrite_all(&buf));
    }
}
