#![no_std]
#![no_main]

// required-features: eeprom

#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_stm32::flash::Flash;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the board and obtain a Peripherals instance
    let p: embassy_stm32::Peripherals = init();

    let mut f = Flash::new_blocking(p.FLASH);
    const ADDR: u32 = 0x0;

    unwrap!(f.eeprom_write_slice(ADDR, &[1, 2, 3, 4, 5, 6, 7, 8]));
    let mut buf = [0u8; 8];
    unwrap!(f.eeprom_read_slice(ADDR, &mut buf));
    assert_eq!(&buf[..], &[1, 2, 3, 4, 5, 6, 7, 8]);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
