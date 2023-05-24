#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::{flash::Flash, interrupt};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello Flash!");

    const ADDR: u32 = 0x36000;

    let mut f = Flash::new(p.FLASH, interrupt::take!(FLASH)).into_blocking_regions().bank1_region;

    info!("Reading...");
    let mut buf = [0u8; 8];
    unwrap!(f.read_blocking(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing...");
    unwrap!(f.erase_blocking(ADDR, ADDR + 2048));

    info!("Reading...");
    let mut buf = [0u8; 8];
    unwrap!(f.read_blocking(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(f.write_blocking(ADDR, &[1, 2, 3, 4, 5, 6, 7, 8]));

    info!("Reading...");
    let mut buf = [0u8; 8];
    unwrap!(f.read_blocking(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);
    assert_eq!(&buf[..], &[1, 2, 3, 4, 5, 6, 7, 8]);
}
