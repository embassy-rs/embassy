#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::nvmc::Nvmc;
use embedded_storage::nor_flash::ReadNorFlash;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut nvmc = Nvmc::new(p.NVMC);

    defmt::println!("Reading data from flash...");

    let mut data = [0u8; 32];
    nvmc.read(0, &mut data).unwrap();

    defmt::println!("Read data: {:X}", data);
}
