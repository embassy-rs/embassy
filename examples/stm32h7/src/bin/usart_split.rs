#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _; // global logger
use embassy::blocking_mutex::raw::ThreadModeRawMutex;
use embassy::channel::channel::Channel;
use embassy::executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::{
    peripherals::{DMA1_CH1, UART7},
    usart::{Config, Uart, UartRx},
    Peripherals,
};
use panic_probe as _;

#[embassy::task]
async fn writer(mut usart: Uart<'static, UART7, NoDma, NoDma>) {
    unwrap!(usart.blocking_write(b"Hello Embassy World!\r\n"));
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        unwrap!(usart.blocking_read(&mut buf));
        unwrap!(usart.blocking_write(&buf));
    }
}

static CHANNEL: Channel<ThreadModeRawMutex, [u8; 8], 1> = Channel::new();

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) -> ! {
    info!("Hello World!");

    let config = Config::default();
    let mut usart = Uart::new(p.UART7, p.PF6, p.PF7, p.DMA1_CH0, p.DMA1_CH1, config);
    unwrap!(usart.blocking_write(b"Type 8 chars to echo!\r\n"));

    let (mut tx, rx) = usart.split();

    unwrap!(spawner.spawn(reader(rx)));

    loop {
        let buf = CHANNEL.recv().await;
        info!("writing...");
        unwrap!(tx.write(&buf).await);
    }
}

#[embassy::task]
async fn reader(mut rx: UartRx<'static, UART7, DMA1_CH1>) {
    let mut buf = [0; 8];
    loop {
        info!("reading...");
        unwrap!(rx.read(&mut buf).await);
        CHANNEL.send(buf).await;
    }
}
