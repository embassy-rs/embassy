#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{BufferedUart, Config};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embedded_io_async::{Read, Write};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART2 => usart::BufferedInterruptHandler<peripherals::USART2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hi!");

    let mut config = Config::default();
    config.baudrate = 9600;
    let mut tx_buf = [0u8; 256];
    let mut rx_buf = [0u8; 256];
    let mut usart = BufferedUart::new(p.USART2, Irqs, p.PA3, p.PA2, &mut tx_buf, &mut rx_buf, config).unwrap();

    usart.write_all(b"Hello Embassy World!\r\n").await.unwrap();
    info!("wrote Hello, starting echo");

    let mut buf = [0; 4];
    loop {
        usart.read_exact(&mut buf[..]).await.unwrap();
        usart.write_all(&buf[..]).await.unwrap();
    }
}
