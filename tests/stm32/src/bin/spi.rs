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

    #[cfg(feature = "stm32f103c8")]
    let (sck, mosi, miso) = (p.PA5, p.PA7, p.PA6);
    #[cfg(feature = "stm32f429zi")]
    let (sck, mosi, miso) = (p.PA5, p.PA7, p.PA6);
    #[cfg(feature = "stm32h755zi")]
    let (sck, mosi, miso) = (p.PA5, p.PB5, p.PA6);
    #[cfg(feature = "stm32g491re")]
    let (sck, mosi, miso) = (p.PA5, p.PA7, p.PA6);
    #[cfg(feature = "stm32g071rb")]
    let (sck, mosi, miso) = (p.PA5, p.PA7, p.PA6);
    #[cfg(feature = "stm32wb55rg")]
    let (sck, mosi, miso) = (p.PA5, p.PA7, p.PA6);
    #[cfg(feature = "stm32u585ai")]
    let (sck, mosi, miso) = (p.PE13, p.PE15, p.PE14);

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
    spi.blocking_transfer(&mut buf, &data).unwrap();
    assert_eq!(buf, data);

    spi.blocking_transfer_in_place(&mut buf).unwrap();
    assert_eq!(buf, data);

    // Check read/write don't hang. We can't check they transfer the right data
    // without fancier test mechanisms.
    spi.blocking_write(&buf).unwrap();
    spi.blocking_read(&mut buf).unwrap();
    spi.blocking_write(&buf).unwrap();
    spi.blocking_read(&mut buf).unwrap();
    spi.blocking_write(&buf).unwrap();

    // Check transfer doesn't break after having done a write, due to garbage in the FIFO
    spi.blocking_transfer(&mut buf, &data).unwrap();
    assert_eq!(buf, data);

    // Check zero-length operations, these should be noops.
    spi.blocking_transfer::<u8>(&mut [], &[]).unwrap();
    spi.blocking_transfer_in_place::<u8>(&mut []).unwrap();
    spi.blocking_read::<u8>(&mut []).unwrap();
    spi.blocking_write::<u8>(&[]).unwrap();

    info!("Test OK");
    cortex_m::asm::bkpt();
}
