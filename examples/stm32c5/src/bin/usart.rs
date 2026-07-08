#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config, Uart};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let config = Config::default();
    let mut usart = Uart::new_blocking(p.USART2, p.PD6, p.PD5, config).unwrap();

    unwrap!(usart.blocking_write(b"Hi"));
    info!("wrote 'Hi', starting echo");

    let mut buf = [0u8; 1];
    loop {
        match usart.blocking_read(&mut buf) {
            Ok(_) => info!("{}", buf[0]),
            Err(e) => error!("{}", e),
        }
        unwrap!(usart.blocking_write(&buf));
    }
}
