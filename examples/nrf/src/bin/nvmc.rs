#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Duration, Timer};
use embassy_nrf::nvmc::Nvmc;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Hello NVMC!");

    // probe-run breaks without this, I'm not sure why.
    Timer::after(Duration::from_secs(1)).await;

    let mut f = Nvmc::new(p.NVMC);
    const ADDR: u32 = 0x80000;

    info!("Reading...");
    let mut buf = [0u8; 4];
    unwrap!(f.read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing...");
    unwrap!(f.erase(ADDR, ADDR + 4096));

    info!("Reading...");
    let mut buf = [0u8; 4];
    unwrap!(f.read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(f.write(ADDR, &[1, 2, 3, 4]));

    info!("Reading...");
    let mut buf = [0u8; 4];
    unwrap!(f.read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);
}
