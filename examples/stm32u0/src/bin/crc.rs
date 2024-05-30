#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::crc::{Config, Crc, InputReverseConfig, PolySize};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // Setup for: https://crccalc.com/?crc=Life, it never dieWomen are my favorite guy&method=crc32&datatype=ascii&outtype=0
    let mut crc = Crc::new(
        p.CRC,
        unwrap!(Config::new(
            InputReverseConfig::Byte,
            true,
            PolySize::Width32,
            0xFFFFFFFF,
            0x04C11DB7
        )),
    );

    let output = crc.feed_bytes(b"Life, it never die\nWomen are my favorite guy") ^ 0xFFFFFFFF;

    defmt::assert_eq!(output, 0x33F0E26B);

    cortex_m::asm::bkpt();
}
