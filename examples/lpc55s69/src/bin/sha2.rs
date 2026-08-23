#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use embassy_executor::Spawner;
use embassy_nxp::hashcrypt::hashcrypt::Sha256;

#[embassy_executor::main]
async fn main(_spawner: Spawner ) {
    let p = embassy_nxp::init(Default::default());
    info!("Device started");

    let mut sha2 = Sha256::new(p.HASHCRYPT);

    let message = b"abcd";
    info!("{}", message);
    sha2.update(message);

    let digest = sha2.finalize();
    
    info!("My hashed message:");
    info!("My hashed message: {:02x}", digest);
}