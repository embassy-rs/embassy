#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::cell::RefCell;

use defmt_rtt as _;
use embassy_boot_rp::*;
use embassy_executor::Spawner;
use embassy_rp::flash::Flash;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::watchdog::Watchdog;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::{Duration, Timer};
use embedded_storage::nor_flash::NorFlash;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
#[cfg(feature = "panic-reset")]
use panic_reset as _;

#[cfg(feature = "skip-include")]
static APP_B: &[u8] = &[0, 1, 2, 3];
#[cfg(not(feature = "skip-include"))]
static APP_B: &[u8] = include_bytes!("../../b.bin");

const FLASH_SIZE: usize = 2 * 1024 * 1024;

#[embassy_executor::main]
async fn main(_s: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    // Override bootloader watchdog
    let mut watchdog = Watchdog::new(p.WATCHDOG);
    watchdog.start(Duration::from_secs(8));

    let flash = Flash::<_, _, FLASH_SIZE>::new_blocking(p.FLASH);
    let flash = Mutex::new(RefCell::new(flash));

    let config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash);
    let mut aligned = AlignedBuffer([0; 1]);
    let mut updater = BlockingFirmwareUpdater::new(config, &mut aligned.0);

    Timer::after(Duration::from_secs(5)).await;
    watchdog.feed();
    led.set_high();
    let mut offset = 0;
    let mut buf: AlignedBuffer<4096> = AlignedBuffer([0; 4096]);
    defmt::info!("preparing update");
    let writer = updater
        .prepare_update()
        .map_err(|e| defmt::warn!("E: {:?}", defmt::Debug2Format(&e)))
        .unwrap();
    defmt::info!("writer created, starting write");
    for chunk in APP_B.chunks(4096) {
        buf.0[..chunk.len()].copy_from_slice(chunk);
        defmt::info!("writing block at offset {}", offset);
        writer.write(offset, &buf.0[..]).unwrap();
        offset += chunk.len() as u32;
    }
    watchdog.feed();
    defmt::info!("firmware written, marking update");
    updater.mark_updated().unwrap();
    Timer::after(Duration::from_secs(2)).await;
    led.set_low();
    defmt::info!("update marked, resetting");
    cortex_m::peripheral::SCB::sys_reset();
}
