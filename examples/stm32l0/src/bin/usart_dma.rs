#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use example_common::*;

use defmt::panic;
use embassy::executor::Spawner;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{rcc, Peripherals};
use embassy_traits::uart::{Read, Write};

#[embassy::main]
async fn main(_spawner: Spawner, mut p: Peripherals) {
    let mut rcc = rcc::Rcc::new(p.RCC);
    rcc.enable_debug_wfe(&mut p.DBGMCU, true);

    let mut usart = Uart::new(
        p.USART1,
        p.PB7,
        p.PB6,
        p.DMA1_CH2,
        p.DMA1_CH3,
        Config::default(),
    );

    usart.write(b"Hello Embassy World!\r\n").await.unwrap();
    info!("wrote Hello, starting echo");

    let mut buf = [0; 1];
    loop {
        usart.read(&mut buf[..]).await.unwrap();
        usart.write(&buf[..]).await.unwrap();
    }
}
