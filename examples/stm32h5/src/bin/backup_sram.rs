#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::pac::pwr::vals::Retention;
use embassy_stm32::{backup_sram, Config};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.ls.backup_ram_retention = Retention::PRESERVED;

    let p = embassy_stm32::init(config);
    info!("Started!");

    let (bytes, status) = backup_sram::init(p.BKPSRAM);

    match status {
        backup_sram::Status::BackupRamDisabled => info!("BKPSRAM just enabled"),
        backup_sram::Status::AlreadyActive => info!("BKPSRAM already enabled"),
    }

    loop {
        info!("byte0: {}", bytes[0]);
        bytes[0] = bytes[0].wrapping_add(1);
        Timer::after_millis(500).await;
    }
}
