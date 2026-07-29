#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nxp::crc::{Crc, Config};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_nxp::init(Default::default());

    // Test CRC-32 standard
    {
        let mut crc32 = Crc::new(p.CRC_ENGINE.reborrow(), Config::crc32());
        crc32.feed(b"123456789");
        let result32 = crc32.sum32();
        info!("CRC-32 result: {:#010x} (expected 0xcbf43926)", result32);
        defmt::assert_eq!(result32, 0xCBF43926);
    } // crc32 e eliberat aici

    // Test CRC-16/CCITT-FALSE (default config)
    {
        let mut crc16 = Crc::new(p.CRC_ENGINE.reborrow(), Config::default());
        crc16.feed(b"123456789");
        let result16 = crc16.sum16();
        info!("CRC-16/CCITT result: {:#06x} (expected 0x29b1)", result16);
        defmt::assert_eq!(result16, 0x29B1);
    }

    info!("CRC driver validated successfully!");

    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}
