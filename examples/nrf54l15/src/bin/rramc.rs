#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::nvmc::{Nvmc, PAGE_SIZE};
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Hello RRAMC NVMC!");

    let mut f = Nvmc::new(p.RRAMC);

    const ADDR: u32 = 0x80000;
    let mut buf = [0u8; 4];

    info!("Reading...");
    unwrap!(f.read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing...");
    unwrap!(f.erase(ADDR, ADDR + PAGE_SIZE as u32));

    info!("Reading...");
    unwrap!(f.read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Writing...");
    // 16 B (128-bit) write minimum
    let out: [u8; 16] = [
        0xaa, 0xaa, 0xaa, 0xaa, 0xbb, 0xbb, 0xbb, 0xbb, 0xcc, 0xcc, 0xcc, 0xcc, 0xdd, 0xdd, 0xdd, 0xdd,
    ];
    unwrap!(f.write(ADDR, &out));

    info!("Reading...");
    // Can read arbitrary sizes
    for addr in (ADDR..ADDR + 16).step_by(4) {
        unwrap!(f.read(addr, &mut buf));
        info!("Read: {=[u8]:x}", buf);
    }
}
