#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    let mut spi_peri = peri!(p, SPI);
    let mut sck = peri!(p, SPI_SCK);
    let mut mosi = peri!(p, SPI_MOSI);
    let mut miso = peri!(p, SPI_MISO);

    let mut spi_config = spi::Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let mut spi = Spi::new_blocking(
        &mut spi_peri,
        &mut sck,  // Arduino D13
        &mut mosi, // Arduino D11
        &mut miso, // Arduino D12
        spi_config,
    );

    let data: [u8; 9] = [0x00, 0xFF, 0xAA, 0x55, 0xC0, 0xFF, 0xEE, 0xC0, 0xDE];

    // Arduino pins D11 and D12 (MOSI-MISO) are connected together with a 1K resistor.
    // so we should get the data we sent back.
    let mut buf = [0; 9];
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

    // Assert the RCC bit gets disabled on drop.
    #[cfg(feature = "stm32f429zi")]
    defmt::assert!(embassy_stm32::pac::RCC.apb2enr().read().spi1en());
    drop(spi);
    #[cfg(feature = "stm32f429zi")]
    defmt::assert!(!embassy_stm32::pac::RCC.apb2enr().read().spi1en());

    // test rx-only configuration
    let mut spi = Spi::new_blocking_rxonly(&mut spi_peri, &mut sck, &mut miso, spi_config);
    let mut mosi_out = Output::new(&mut mosi, Level::Low, Speed::VeryHigh);
    mosi_out.set_high();
    spi.blocking_read(&mut buf).unwrap();
    assert_eq!(buf, [0xff; 9]);
    mosi_out.set_low();
    spi.blocking_read(&mut buf).unwrap();
    assert_eq!(buf, [0x00; 9]);
    spi.blocking_read::<u8>(&mut []).unwrap();
    spi.blocking_read::<u8>(&mut []).unwrap();
    drop(mosi_out);
    drop(spi);

    // Test tx-only. Just check it doesn't hang, not much else we can do without using SPI slave.
    let mut spi = Spi::new_blocking_txonly(&mut spi_peri, &mut sck, &mut mosi, spi_config);
    spi.blocking_transfer(&mut buf, &data).unwrap();
    spi.blocking_transfer_in_place(&mut buf).unwrap();
    spi.blocking_write(&buf).unwrap();
    spi.blocking_read(&mut buf).unwrap();
    spi.blocking_transfer::<u8>(&mut [], &[]).unwrap();
    spi.blocking_transfer_in_place::<u8>(&mut []).unwrap();
    spi.blocking_read::<u8>(&mut []).unwrap();
    spi.blocking_write::<u8>(&[]).unwrap();
    drop(spi);

    // Test tx-only nosck.
    let mut spi = Spi::new_blocking_txonly_nosck(&mut spi_peri, &mut mosi, spi_config);
    spi.blocking_write(&buf).unwrap();
    spi.blocking_write::<u8>(&[]).unwrap();
    spi.blocking_write(&buf).unwrap();
    drop(spi);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
