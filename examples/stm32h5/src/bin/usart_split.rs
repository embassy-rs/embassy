#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::peripherals::{GPDMA1_CH1, UART7};
use embassy_stm32::usart::{Config, Uart, UartRx};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART7 => usart::InterruptHandler<peripherals::UART7>;
});

#[embassy_executor::task]
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

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let config = Config::default();
    let mut usart = Uart::new(p.UART7, p.PF6, p.PF7, Irqs, p.GPDMA1_CH0, p.GPDMA1_CH1, config);
    unwrap!(usart.blocking_write(b"Type 8 chars to echo!\r\n"));

    let (mut tx, rx) = usart.split();

    unwrap!(spawner.spawn(reader(rx)));

    loop {
        let buf = CHANNEL.receive().await;
        info!("writing...");
        unwrap!(tx.write(&buf).await);
    }
}

#[embassy_executor::task]
async fn reader(mut rx: UartRx<'static, UART7, GPDMA1_CH1>) {
    let mut buf = [0; 8];
    loop {
        info!("reading...");
        unwrap!(rx.read(&mut buf).await);
        CHANNEL.send(buf).await;
    }
}
