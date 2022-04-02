#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

use defmt::*;

use embassy::executor::Spawner;
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy_stm32::dma::NoDma;
use embassy_stm32::interrupt;
use embassy_stm32::usart::{BufferedUart, Config, State, Uart};
use embassy_stm32::Peripherals;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    static mut TX_BUFFER: [u8; 8] = [0; 8];
    static mut RX_BUFFER: [u8; 256] = [0; 256];

    let mut config = Config::default();
    config.baudrate = 9600;

    let usart = Uart::new(p.USART1, p.PA10, p.PA9, NoDma, NoDma, config);
    let mut state = State::new();
    let mut usart = unsafe {
        BufferedUart::new(
            &mut state,
            usart,
            interrupt::take!(USART1),
            &mut TX_BUFFER,
            &mut RX_BUFFER,
        )
    };

    usart.write_all(b"Hello Embassy World!\r\n").await.unwrap();
    info!("wrote Hello, starting echo");

    let mut buf = [0; 4];
    loop {
        usart.read_exact(&mut buf[..]).await.unwrap();
        usart.write_all(&buf[..]).await.unwrap();
    }
}
