#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_boot_rp::*;
use embassy_executor::Spawner;
use embassy_rp::flash::Flash;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::watchdog::Watchdog;
use embassy_time::{Duration, Timer};
#[cfg(feature = "panic-probe")]
use panic_probe as _;
#[cfg(feature = "panic-reset")]
use panic_reset as _;

static APP_B: &[u8] = include_bytes!("../../b.bin");
const FLASH_SIZE: usize = 2 * 1024 * 1024;

#[embassy_executor::main]
async fn main(_s: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    // Override bootloader watchdog
    let mut watchdog = Watchdog::new(p.WATCHDOG);
    watchdog.start(Duration::from_secs(8));

    let mut flash: Flash<_, FLASH_SIZE> = Flash::new_blocking_only(p.FLASH);

    let mut updater = FirmwareUpdater::default();

    Timer::after(Duration::from_secs(5)).await;
    watchdog.feed();
    led.set_high();
    let mut offset = 0;
    let mut buf: AlignedBuffer<4096> = AlignedBuffer([0; 4096]);
    defmt::info!("preparing update");
    let mut writer = updater
        .prepare_update_blocking(&mut flash)
        .map_err(|e| defmt::warn!("E: {:?}", defmt::Debug2Format(&e)))
        .unwrap();
    defmt::info!("writer created, starting write");
    for chunk in APP_B.chunks(4096) {
        buf.0[..chunk.len()].copy_from_slice(chunk);
        defmt::info!("writing block at offset {}", offset);
        writer
            .write_block_blocking(offset, &buf.0[..], &mut flash, 256)
            .unwrap();
        offset += chunk.len();
    }
    watchdog.feed();
    defmt::info!("firmware written, marking update");
    updater.mark_updated_blocking(&mut flash, &mut buf.0[..1]).unwrap();
    Timer::after(Duration::from_secs(2)).await;
    led.set_low();
    defmt::info!("update marked, resetting");
    cortex_m::peripheral::SCB::sys_reset();
}
