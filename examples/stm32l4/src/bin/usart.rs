#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy_stm32::dma::NoDma;
use embassy_stm32::usart::{Config, Uart};
use example_common::*;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    let config = Config::default();
    let mut usart = Uart::new(p.UART4, p.PA1, p.PA0, NoDma, NoDma, config);

    unwrap!(usart.blocking_write(b"Hello Embassy World!\r\n"));
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        unwrap!(usart.blocking_read(&mut buf));
        unwrap!(usart.blocking_write(&buf));
    }
}
