#![no_std]
#![no_main]

use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::flash::{EDATA_ERASE_SIZE, Flash};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello EDATA!");

    // The EDATA area of bank 2, so the CPU can keep running from bank 1 while it's written.
    let mut edata = Flash::new_blocking(p.FLASH, Default::default())
        .into_blocking_regions()
        .edata_bank2;

    if edata.capacity() == 0 {
        // For example, EDATA2_EN = 1 and EDATA2_STRT = 0 make the last sector of bank 2 a 6 KB
        // EDATA sector. It can't hold code anymore.
        defmt::panic!("EDATA is disabled, set EDATA2_EN and EDATA2_STRT in the option bytes");
    }
    info!("EDATA size: {} bytes", edata.capacity());

    info!("Erasing...");
    unwrap!(edata.blocking_erase(0, EDATA_ERASE_SIZE as u32));

    // Erased locations read as 0xFF.
    let mut buf = [0u8; 8];
    unwrap!(edata.blocking_read(0, &mut buf));
    info!("Read after erase: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(edata.blocking_write(0, &[1, 2, 3, 4, 5, 6, 7, 8]));

    unwrap!(edata.blocking_read(0, &mut buf));
    info!("Read: {=[u8]:x}", buf);
    assert_eq!(buf, [1, 2, 3, 4, 5, 6, 7, 8]);

    info!("Success!");
}
