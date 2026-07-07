#![no_std]
#![no_main]

use core::array;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::flash::Flash;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello Flash!");

    const ADDR: u32 = 0; // This is the offset into bank 2

    // wait a bit before accessing the flash
    Timer::after_millis(300).await;

    let mut f = Flash::new_blocking(p.FLASH).into_blocking_regions().bank2_region;

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing...");
    unwrap!(f.blocking_erase(ADDR, ADDR + 128 * 1024));

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read after erase: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(f.blocking_write(ADDR, array::from_fn::<u8, 32, _>(|i| i as u8).as_slice()));

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);
    assert_eq!(&buf[..], array::from_fn::<u8, 32, _>(|i| i as u8).as_slice());
    info!("Success!");
}
