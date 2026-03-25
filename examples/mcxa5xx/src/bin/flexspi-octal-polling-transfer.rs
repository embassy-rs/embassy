#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use defmt::info;
use hal::config::Config;
use hal::flexspi::{FlexSpi, Port3Pins};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const FLASH_OFFSET: usize = 1000 * 0x1000;
const FLASH_PAGE_SIZE: usize = hal::flexspi::PAGE_SIZE;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Config::default());

    info!("FlexSPI example");

    let pins = Port3Pins::new(p.P3_0, p.P3_1, p.P3_6, p.P3_7, p.P3_8, p.P3_9, p.P3_10, p.P3_11);
    let mut flash = FlexSpi::new_blocking(pins, Default::default()).unwrap();

    let vendor_id = flash.read_vendor_id().unwrap();
    info!("Vendor ID: 0x{:02x}", vendor_id);

    let sector_address = FLASH_OFFSET;
    let mut program = [0xffu8; FLASH_PAGE_SIZE];
    let mut readback = [0u8; FLASH_PAGE_SIZE];

    info!("Erasing sector at 0x{:08x}", sector_address as u32);
    flash.erase_sector(sector_address as u32).unwrap();
    flash.read(sector_address as u32, &mut readback).unwrap();
    assert!(readback.iter().all(|byte| *byte == 0xff));
    info!("Erase verified");

    for (index, byte) in program.iter_mut().enumerate() {
        *byte = index as u8;
    }

    info!("Programming page at 0x{:08x}", sector_address as u32);
    flash.page_program(sector_address as u32, &program).unwrap();
    flash.read(sector_address as u32, &mut readback).unwrap();
    assert_eq!(readback, program);
    info!("Page program verified");

    loop {
        Timer::after_secs(1).await;
        info!("FlexSPI demo heartbeat");
    }
}
