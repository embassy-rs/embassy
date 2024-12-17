#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::flash::Flash;
use embassy_stm32::SharedData;
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".shared_data"]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init_primary(Default::default(), &SHARED_DATA);
    info!("Hello Flash!");

    const ADDR: u32 = 0x36000;

    let mut f = Flash::new_blocking(p.FLASH).into_blocking_regions().bank1_region;

    info!("Reading...");
    let mut buf = [0u8; 8];
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing...");
    unwrap!(f.blocking_erase(ADDR, ADDR + 2048));

    info!("Reading...");
    let mut buf = [0u8; 8];
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(f.blocking_write(ADDR, &[1, 2, 3, 4, 5, 6, 7, 8]));

    info!("Reading...");
    let mut buf = [0u8; 8];
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);
    assert_eq!(&buf[..], &[1, 2, 3, 4, 5, 6, 7, 8]);
}
