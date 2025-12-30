#![no_std]
#![no_main]

use core::str::from_utf8_mut;

use defmt::*;
use embassy_executor::Spawner;
use embassy_nxp::usart::{Config, Usart};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());
    let mut usart = Usart::new_blocking(p.USART2, p.PIO0_27, p.PIO1_24, Config::default());
    let tx_buf = b"Hello, Ferris!";
    let mut rx_buf = [0u8; 14];

    loop {
        info!("Write a message");
        usart.blocking_write(tx_buf).unwrap();
        usart.blocking_flush().unwrap();

        Timer::after_millis(500).await;

        info!("Read a message");
        usart.blocking_read(&mut rx_buf).unwrap();

        match from_utf8_mut(&mut rx_buf) {
            Ok(str) => {
                info!("The message is: {}", str);
            }
            Err(_) => {
                error!("Error in converting to UTF8");
            }
        }

        Timer::after_millis(500).await;
    }
}
