#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::flash::Flash;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello Flash!");

    const ADDR: u32 = 0x8_0000; // This is the offset into the third region, the absolute address is 4x32K + 128K + 0x8_0000.

    // wait a bit before accessing the flash
    Timer::after_millis(300).await;

    let mut f = Flash::new_blocking(p.FLASH).into_blocking_regions().bank1_region3;

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing...");
    unwrap!(f.blocking_erase(ADDR, ADDR + 256 * 1024));

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read after erase: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(f.blocking_write(
        ADDR,
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32
        ]
    ));

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);
    assert_eq!(
        &buf[..],
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32
        ]
    );
}
