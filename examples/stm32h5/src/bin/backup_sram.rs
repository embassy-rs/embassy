#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::backup_sram::BackupMemory;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.ls.enable_backup_sram = true;

    let p = embassy_stm32::init(config);
    info!("Started!");

    let (bytes, status) = BackupMemory::new(p.BKPSRAM);

    match status {
        false => info!("BKPSRAM just enabled"),
        true => info!("BKPSRAM already enabled"),
    }

    loop {
        info!("byte0: {}", bytes[0]);
        bytes[0] = bytes[0].wrapping_add(1);
        Timer::after_millis(500).await;
    }
}
