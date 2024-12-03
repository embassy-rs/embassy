//! This example shows how to use SPI (Serial Peripheral Interface) in the RP2040 chip.
//! No specific hardware is specified in this example. If you connect pin 11 and 12 you should get the same data back.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::spi::{Config, Spi};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let clk = p.PIN_10;

    let mut spi = Spi::new(p.SPI1, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Config::default());

    loop {
        let tx_buf = [1_u8, 2, 3, 4, 5, 6];
        let mut rx_buf = [0_u8; 6];
        spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
        info!("{:?}", rx_buf);
        Timer::after_secs(1).await;
    }
}
