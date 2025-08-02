#![no_std]
#![no_main]

use core::str::from_utf8_mut;
use cortex_m::asm::nop;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nxp::usart::{Config, Usart};
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());
    let mut usart = Usart::new_blocking(p.USART2, p.PIO0_27, p.PIO1_24, Config::default());
    let tx_buf: &[u8; 14] = b"Hello, Ferris!";
    let mut rx_buf: [u8; 14] = [0u8; 14];
    loop {
        info!("Write a message");
        usart.blocking_write(tx_buf).unwrap();
        usart.blocking_flush().unwrap();
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

        // Time driver should be used here instead

        for _ in 0..500_000 {
            nop();
        }
    }
}
