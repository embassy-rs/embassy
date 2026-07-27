#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::crc::{Config, Crc, InputReverseConfig, PolySize};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // DK uses external SMPS (UM3300 Tab.6); embassy default = internal SMPS hangs init() at VOSRDY.
    let mut config = embassy_stm32::Config::default();
    config.rcc.supply_config = embassy_stm32::rcc::SupplyConfig::External;
    let p = embassy_stm32::init(config);
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

    crc.feed_bytes(b"Life, it never die\nWomen are my favorite guy");
    let output = crc.read() ^ 0xFFFFFFFF;

    defmt::assert_eq!(output, 0x33F0E26B);

    cortex_m::asm::bkpt();
}
