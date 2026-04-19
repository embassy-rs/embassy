#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_mcxa::{bind_interrupts, peripherals};
use embassy_time::Timer;
use hal::config::Config;
use hal::flexspi::{self, ClockConfig as FlexspiClockConfig, Flexspi, NorFlash};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[path = "../flexspi_common.rs"]
mod flexspi_common;

const FLASH_OFFSET: usize = 1000 * 0x1000;
const FLASH_PAGE_SIZE: usize = flexspi_common::FLASH_PAGE_SIZE;

bind_interrupts!(struct Irqs {
    FLEXSPI0 => flexspi::InterruptHandler<peripherals::FLEXSPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Config::default());

    info!("FlexSPI example");

    let flexspi = Flexspi::new_with_dma(
        p.FLEXSPI0,
        p.P3_0,
        p.P3_1,
        p.P3_6,
        p.P3_7,
        p.P3_8,
        p.P3_9,
        p.P3_10,
        p.P3_11,
        p.DMA0_CH0,
        p.DMA0_CH1,
        Irqs,
        FlexspiClockConfig::default(),
        flexspi_common::FLASH_CONFIG,
    )
    .unwrap();

    let mut flash = NorFlash::from_flexspi(flexspi);

    let vendor_id = flash.read_vendor_id_async().await.unwrap();
    info!("Vendor ID: 0x{:02x}", vendor_id);

    let sector_address = FLASH_OFFSET;
    let mut program = [0xffu8; FLASH_PAGE_SIZE];
    let mut readback = [0u8; FLASH_PAGE_SIZE];

    info!("Erasing sector at 0x{:08x}", sector_address as u32);
    flash.erase_sector_async(sector_address as u32).await.unwrap();
    flash.read_async(sector_address as u32, &mut readback).await.unwrap();
    assert!(readback.iter().all(|byte| *byte == u8::MAX));
    info!("Erase verified");

    for (index, byte) in program.iter_mut().enumerate() {
        *byte = index as u8;
    }

    info!("Programming page at 0x{:08x}", sector_address as u32);
    flash.page_program_async(sector_address as u32, &program).await.unwrap();
    flash.read_async(sector_address as u32, &mut readback).await.unwrap();
    assert_eq!(readback, program);
    info!("Page program verified");

    loop {
        Timer::after_secs(1).await;
        info!("FlexSPI EDMA demo heartbeat");
    }
}
