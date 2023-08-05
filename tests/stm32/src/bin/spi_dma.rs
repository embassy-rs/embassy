#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    #[cfg(feature = "stm32f103c8")]
    let (spi, sck, mosi, miso, tx_dma, rx_dma) = (p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH3, p.DMA1_CH2);
    #[cfg(feature = "stm32f429zi")]
    let (spi, sck, mosi, miso, tx_dma, rx_dma) = (p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA2_CH3, p.DMA2_CH2);
    #[cfg(feature = "stm32h755zi")]
    let (spi, sck, mosi, miso, tx_dma, rx_dma) = (p.SPI1, p.PA5, p.PB5, p.PA6, p.DMA1_CH0, p.DMA1_CH1);
    #[cfg(feature = "stm32g491re")]
    let (spi, sck, mosi, miso, tx_dma, rx_dma) = (p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH1, p.DMA1_CH2);
    #[cfg(feature = "stm32g071rb")]
    let (spi, sck, mosi, miso, tx_dma, rx_dma) = (p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH1, p.DMA1_CH2);
    #[cfg(feature = "stm32wb55rg")]
    let (spi, sck, mosi, miso, tx_dma, rx_dma) = (p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH1, p.DMA1_CH2);
    #[cfg(feature = "stm32u585ai")]
    let (spi, sck, mosi, miso, tx_dma, rx_dma) = (p.SPI1, p.PE13, p.PE15, p.PE14, p.GPDMA1_CH0, p.GPDMA1_CH1);
    #[cfg(feature = "stm32h563zi")]
    let (spi, sck, mosi, miso, tx_dma, rx_dma) = (p.SPI4, p.PE12, p.PE14, p.PE13, p.GPDMA1_CH0, p.GPDMA1_CH1);
    #[cfg(feature = "stm32c031c6")]
    let (spi, sck, mosi, miso, tx_dma, rx_dma) = (p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH1, p.DMA1_CH2);

    let mut spi_config = spi::Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let mut spi = Spi::new(
        spi, sck,  // Arduino D13
        mosi, // Arduino D11
        miso, // Arduino D12
        tx_dma, rx_dma, spi_config,
    );

    let data: [u8; 9] = [0x00, 0xFF, 0xAA, 0x55, 0xC0, 0xFF, 0xEE, 0xC0, 0xDE];

    // Arduino pins D11 and D12 (MOSI-MISO) are connected together with a 1K resistor.
    // so we should get the data we sent back.
    let mut buf = [0; 9];
    spi.transfer(&mut buf, &data).await.unwrap();
    assert_eq!(buf, data);

    spi.transfer_in_place(&mut buf).await.unwrap();
    assert_eq!(buf, data);

    // Check read/write don't hang. We can't check they transfer the right data
    // without fancier test mechanisms.
    spi.write(&buf).await.unwrap();
    spi.read(&mut buf).await.unwrap();
    spi.write(&buf).await.unwrap();
    spi.read(&mut buf).await.unwrap();
    spi.write(&buf).await.unwrap();

    // Check transfer doesn't break after having done a write, due to garbage in the FIFO
    spi.transfer(&mut buf, &data).await.unwrap();
    assert_eq!(buf, data);

    // Check zero-length operations, these should be noops.
    spi.transfer::<u8>(&mut [], &[]).await.unwrap();
    spi.transfer_in_place::<u8>(&mut []).await.unwrap();
    spi.read::<u8>(&mut []).await.unwrap();
    spi.write::<u8>(&[]).await.unwrap();

    // === Check mixing blocking with async.
    spi.blocking_transfer(&mut buf, &data).unwrap();
    assert_eq!(buf, data);
    spi.transfer(&mut buf, &data).await.unwrap();
    assert_eq!(buf, data);
    spi.blocking_write(&buf).unwrap();
    spi.transfer(&mut buf, &data).await.unwrap();
    assert_eq!(buf, data);
    spi.blocking_read(&mut buf).unwrap();
    spi.blocking_write(&buf).unwrap();
    spi.write(&buf).await.unwrap();
    spi.read(&mut buf).await.unwrap();
    spi.blocking_write(&buf).unwrap();
    spi.blocking_read(&mut buf).unwrap();
    spi.write(&buf).await.unwrap();

    info!("Test OK");
    cortex_m::asm::bkpt();
}
