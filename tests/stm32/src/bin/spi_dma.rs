#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{into_ref, Peripheral as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    let spi_peri = peri!(p, SPI);
    let sck = peri!(p, SPI_SCK);
    let mosi = peri!(p, SPI_MOSI);
    let miso = peri!(p, SPI_MISO);
    let tx_dma = peri!(p, SPI_TX_DMA);
    let rx_dma = peri!(p, SPI_RX_DMA);

    into_ref!(spi_peri, sck, mosi, miso, tx_dma, rx_dma);

    let mut spi_config = spi::Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let mut spi = Spi::new(
        spi_peri.reborrow(),
        sck.reborrow(),  // Arduino D13
        mosi.reborrow(), // Arduino D11
        miso.reborrow(), // Arduino D12
        tx_dma.reborrow(),
        rx_dma.reborrow(),
        spi_config,
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

    core::mem::drop(spi);

    // test rx-only configuration

    // stm32f207zg - spi_v1
    // stm32f103c8 - spi_f1
    // stm32g491re - spi_v2
    // stm32h753zi - spi_v3
    // stm32h563zi - spi_v4
    // stm32wba52cg - spi_v5

    #[cfg(any(stm32f207zg, stm32f103c8, stm32g491re, stm32h753zi, stm32h563zi, stm32wba52cg))]
    {
        let mut spi = {
            #[cfg(stm32f207zg, stm32f103c8, stm32g491re)]
            {
                Spi::new_rxonly(
                    spi_peri.reborrow(),
                    sck.reborrow(),
                    miso.reborrow(),
                    tx_dma.reborrow(),
                    rx_dma.reborrow(),
                    spi_config,
                )
            }
            #[cfg(stm32h753zi, stm32h563zi, stm32wba52cg)]
            {
                Spi::new_rxonly(
                    spi_peri.reborrow(),
                    sck.reborrow(),
                    miso.reborrow(),
                    rx_dma.reborrow(),
                    spi_config,
                )
            }
        };

        use embassy_stm32::gpio;
        let mut mosi = gpio::Output::new(mosi.reborrow(), gpio::Level::Low, gpio::Speed::Low);

        mosi.set_high();
        spi.read(&mut buf).await.unwrap();
        assert_eq!(buf, [0xff; 9]);

        mosi.set_low();
        spi.read(&mut buf).await.unwrap();
        assert_eq!(buf, [0x00; 9]);
    };

    info!("Test OK");
    cortex_m::asm::bkpt();
}
