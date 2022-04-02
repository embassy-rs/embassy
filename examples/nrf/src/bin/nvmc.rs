#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::nvmc::Nvmc;
use embassy_nrf::Peripherals;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
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
