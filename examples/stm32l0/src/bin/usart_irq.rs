#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::interrupt;
use embassy_stm32::usart::{BufferedUart, Config, State};
use embedded_io::asynch::{Read, Write};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hi!");

    static mut TX_BUFFER: [u8; 8] = [0; 8];
    static mut RX_BUFFER: [u8; 256] = [0; 256];

    let mut config = Config::default();
    config.baudrate = 9600;

    let mut state = State::new();
    let irq = interrupt::take!(USART2);
    let mut usart = unsafe {
        BufferedUart::new(
            &mut state,
            p.USART2,
            p.PA3,
            p.PA2,
            irq,
            &mut TX_BUFFER,
            &mut RX_BUFFER,
            config,
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
