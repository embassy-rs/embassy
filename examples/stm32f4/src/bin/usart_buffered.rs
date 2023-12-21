#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{BufferedUart, Config};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embedded_io_async::BufRead;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART3 => usart::BufferedInterruptHandler<peripherals::USART3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let config = Config::default();

    let mut tx_buf = [0u8; 32];
    let mut rx_buf = [0u8; 32];
    let mut buf_usart = BufferedUart::new(p.USART3, Irqs, p.PD9, p.PD8, &mut tx_buf, &mut rx_buf, config).unwrap();

    loop {
        let buf = buf_usart.fill_buf().await.unwrap();
        info!("Received: {}", buf);

        // Read bytes have to be explicitly consumed, otherwise fill_buf() will return them again
        let n = buf.len();
        buf_usart.consume(n);
    }
}
