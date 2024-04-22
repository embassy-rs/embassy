#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::flash::Flash;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let addr: u32 = 0x8000;

    let mut f = Flash::new_blocking(p.FLASH).into_blocking_regions().bank1_region;

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(addr, &mut buf));
    info!("Read: {=[u8]:x}", buf);
    info!("Erasing...");
    unwrap!(f.blocking_erase(addr, addr + 2 * 1024));

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(addr, &mut buf));
    info!("Read after erase: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(f.blocking_write(
        addr,
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32
        ]
    ));

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(addr, &mut buf));
    info!("Read: {=[u8]:x}", buf);
}
