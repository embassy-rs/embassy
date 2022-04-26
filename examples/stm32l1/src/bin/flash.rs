#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use embassy::executor::Spawner;
use embassy_stm32::flash::Flash;
use embassy_stm32::Peripherals;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello Flash!");

    const ADDR: u32 = 0x26000;

    let mut f = Flash::unlock(p.FLASH);

    info!("Reading...");
    let mut buf = [0u8; 8];
    unwrap!(f.read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing...");
    unwrap!(f.erase(ADDR, ADDR + 256));

    info!("Reading...");
    let mut buf = [0u8; 8];
    unwrap!(f.read(ADDR, &mut buf));
    info!("Read after erase: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(f.write(ADDR, &[1, 2, 3, 4, 5, 6, 7, 8]));

    info!("Reading...");
    let mut buf = [0u8; 8];
    unwrap!(f.read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);
    assert_eq!(&buf[..], &[1, 2, 3, 4, 5, 6, 7, 8]);
}
