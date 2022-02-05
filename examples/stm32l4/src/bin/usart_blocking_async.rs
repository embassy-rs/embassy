#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::Peripherals;
use embassy_traits::adapter::BlockingAsync;
use embedded_hal_async::serial::{Read, Write};
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let config = Config::default();
    let usart = Uart::new(p.UART4, p.PA1, p.PA0, NoDma, NoDma, config);
    let mut usart = BlockingAsync::new(usart);

    unwrap!(usart.write(b"Hello Embassy World!\r\n").await);
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        unwrap!(usart.read(&mut buf).await);
        unwrap!(usart.write(&buf).await);
    }
}
