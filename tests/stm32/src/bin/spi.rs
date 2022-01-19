#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::assert_eq;
use embassy::executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Peripherals;
use example_common::*;

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    #[cfg(not(feature = "stm32h755zi"))]
    let (sck, mosi, miso) = (p.PA5, p.PA7, p.PA6);
    #[cfg(feature = "stm32h755zi")]
    let (sck, mosi, miso) = (p.PA5, p.PB5, p.PA6);

    let mut spi = Spi::new(
        p.SPI1,
        sck,  // Arduino D13
        mosi, // Arduino D11
        miso, // Arduino D12
        NoDma,
        NoDma,
        Hertz(1_000_000),
        spi::Config::default(),
    );

    let data: [u8; 9] = [0x00, 0xFF, 0xAA, 0x55, 0xC0, 0xFF, 0xEE, 0xC0, 0xDE];

    // Arduino pins D11 and D12 (MOSI-MISO) are connected together with a 1K resistor.
    // so we should get the data we sent back.
    let mut buf = data;
    spi.blocking_transfer_in_place(&mut buf).unwrap();
    assert_eq!(buf, data);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
