#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_rp::spi::{Async, Config, Spi};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let clk = p.PIN_2;
    let mosi = p.PIN_3;
    let miso = p.PIN_4;

    let mut spi: Spi<'_, _, Async> = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Config::default());

    let tx_buf = [1_u8, 2, 3, 4, 5, 6];
    let mut rx_buf = [0_u8; 6];
    spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
    assert_eq!(rx_buf, tx_buf);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
