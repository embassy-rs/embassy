#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::{self, Spi, Word};
use embassy_stm32::time::Hertz;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init();
    info!("Hello World!");

    let mut spi_peri = peri!(p, SPI);
    let mut sck = peri!(p, SPI_SCK);
    let mut mosi = peri!(p, SPI_MOSI);
    let mut miso = peri!(p, SPI_MISO);

    let mut spi_config = spi::Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let mut spi = Spi::new_blocking(
        spi_peri.reborrow(),
        sck.reborrow(),  // Arduino D13
        mosi.reborrow(), // Arduino D11
        miso.reborrow(), // Arduino D12
        spi_config,
    );

    test_txrx::<u8>(&mut spi);
    test_txrx::<u16>(&mut spi);

    // Assert the RCC bit gets disabled on drop.
    #[cfg(feature = "stm32f429zi")]
    defmt::assert!(embassy_stm32::pac::RCC.apb2enr().read().spi1en());
    drop(spi);
    #[cfg(feature = "stm32f429zi")]
    defmt::assert!(!embassy_stm32::pac::RCC.apb2enr().read().spi1en());

    // test rx-only configuration
    let mut spi = Spi::new_blocking_rxonly(spi_peri.reborrow(), sck.reborrow(), miso.reborrow(), spi_config);
    let mut mosi_out = Output::new(mosi.reborrow(), Level::Low, Speed::VeryHigh);

    test_rx::<u8>(&mut spi, &mut mosi_out);
    test_rx::<u16>(&mut spi, &mut mosi_out);
    drop(spi);
    drop(mosi_out);

    let mut spi = Spi::new_blocking_txonly(spi_peri.reborrow(), sck.reborrow(), mosi.reborrow(), spi_config);
    test_tx::<u8>(&mut spi);
    test_tx::<u16>(&mut spi);
    drop(spi);

    let mut spi = Spi::new_blocking_txonly_nosck(spi_peri.reborrow(), mosi.reborrow(), spi_config);
    test_tx::<u8>(&mut spi);
    test_tx::<u16>(&mut spi);
    drop(spi);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

fn test_txrx<W: Word + From<u8> + defmt::Format + Eq>(spi: &mut Spi<'_, Blocking>)
where
    W: core::ops::Not<Output = W>,
{
    let data: [W; 9] = [
        0x00u8.into(),
        0xFFu8.into(),
        0xAAu8.into(),
        0x55u8.into(),
        0xC0u8.into(),
        0xFFu8.into(),
        0xEEu8.into(),
        0xC0u8.into(),
        0xDEu8.into(),
    ];

    // Arduino pins D11 and D12 (MOSI-MISO) are connected together with a 1K resistor.
    // so we should get the data we sent back.
    let mut buf = [W::default(); 9];
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
}

fn test_rx<W: Word + From<u8> + defmt::Format + Eq>(spi: &mut Spi<'_, Blocking>, mosi_out: &mut Output<'_>)
where
    W: core::ops::Not<Output = W>,
{
    let mut buf = [W::default(); 9];

    mosi_out.set_high();
    spi.blocking_read(&mut buf).unwrap();
    assert_eq!(buf, [!W::default(); 9]);
    mosi_out.set_low();
    spi.blocking_read(&mut buf).unwrap();
    assert_eq!(buf, [W::default(); 9]);
    spi.blocking_read::<u8>(&mut []).unwrap();
    spi.blocking_read::<u8>(&mut []).unwrap();
}

fn test_tx<W: Word + From<u8> + defmt::Format + Eq>(spi: &mut Spi<'_, Blocking>)
where
    W: core::ops::Not<Output = W>,
{
    let buf = [W::default(); 9];

    // Test tx-only. Just check it doesn't hang, not much else we can do without using SPI slave.
    spi.blocking_write(&buf).unwrap();
    spi.blocking_write::<u8>(&[]).unwrap();
    spi.blocking_write(&buf).unwrap();
    spi.blocking_write::<u8>(&[]).unwrap();
}
