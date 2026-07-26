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

    let mut backup_ram = BackupMemory::new(p.BKPSRAM);

    match backup_ram.is_retained() {
        false => info!("BKPSRAM just enabled"),
        true => info!("BKPSRAM already enabled"),
    }

    loop {
        let mut bytes = [0];
        backup_ram.read(0, &mut bytes);
        info!("byte0: {}", bytes[0]);
        bytes[0] = bytes[0].wrapping_add(1);
        backup_ram.write(0, &bytes);
        Timer::after_millis(500).await;
    }
}
