#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::crc::{Config, Crc, InputReverseConfig, PolySize};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut crc = Crc::new(
        p.CRC,
        unwrap!(Config::new(
            InputReverseConfig::Byte,
            true,
            PolySize::Width32,
            0xFFFF_FFFF,
            0x04C1_1DB7,
        )),
    );

    // CRC-32/ISO-HDLC("123456789") == 0xcbf4_3926
    crc.feed_bytes(b"1234");
    crc.feed_bytes(b"56789");
    let result = crc.read() ^ 0xFFFF_FFFF;

    info!("CRC-32 = {=u32:#010x}", result);
    defmt::assert_eq!(result, 0xCBF4_3926);

    cortex_m::asm::bkpt();
}