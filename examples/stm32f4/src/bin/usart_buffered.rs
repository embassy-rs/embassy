#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy::executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::usart::{BufferedUart, Config, State, Uart};
use embassy_stm32::{interrupt, Peripherals};
use embedded_io::asynch::BufRead;
use {defmt_rtt as _, panic_probe as _};

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let config = Config::default();
    let usart = Uart::new(p.USART3, p.PD9, p.PD8, NoDma, NoDma, config);

    let mut state = State::new();
    let irq = interrupt::take!(USART3);
    let mut tx_buf = [0u8; 32];
    let mut rx_buf = [0u8; 32];
    let mut buf_usart = BufferedUart::new(&mut state, usart, irq, &mut tx_buf, &mut rx_buf);

    loop {
        let buf = buf_usart.fill_buf().await.unwrap();
        info!("Received: {}", buf);

        // Read bytes have to be explicitly consumed, otherwise fill_buf() will return them again
        let n = buf.len();
        buf_usart.consume(n);
    }
}
