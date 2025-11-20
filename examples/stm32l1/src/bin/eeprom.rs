#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::flash::{EEPROM_BASE, EEPROM_SIZE, Flash};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("Hello Eeprom! Start: {}, Size: {}", EEPROM_BASE, EEPROM_SIZE);

    const ADDR: u32 = 0x0;

    let mut f = Flash::new_blocking(p.FLASH);

    info!("Reading...");
    let mut buf = [0u8; 8];
    unwrap!(f.eeprom_read_slice(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(f.eeprom_write_slice(ADDR, &[1, 2, 3, 4, 5, 6, 7, 8]));

    info!("Reading...");
    let mut buf = [0u8; 8];
    unwrap!(f.eeprom_read_slice(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);
    assert_eq!(&buf[..], &[1, 2, 3, 4, 5, 6, 7, 8]);
}
