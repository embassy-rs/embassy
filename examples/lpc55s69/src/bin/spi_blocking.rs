#![no_std]
#![no_main]

use core::str::from_utf8_mut;

use defmt::*;
use embassy_executor::Spawner;
use embassy_nxp::spi::{Config, Spi};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());
    let mut spi = Spi::new_blocking(p.SPI2, p.PIO1_23, p.PIO1_24, p.PIO1_25, Config::default()).unwrap();
    let tx_buf = b"Hello, Ferris!";
    let mut rx_buf = [0u8; 14];

    loop {
        info!("Write a message");

        spi.blocking_transfer(&mut rx_buf, tx_buf).unwrap();
        spi.flush().unwrap();

        Timer::after_millis(500).await;

        info!("Read a message");

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
