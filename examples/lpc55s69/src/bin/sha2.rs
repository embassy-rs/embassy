//! This example has been made with the LPCXpresso55S69 board in mind

#![no_std]
#![no_main]

use cortex_m::asm::nop;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use embassy_executor::Spawner;
use embassy_nxp::hashcrypt::hashcrypt::Sha256;

#[embassy_executor::main]
async fn main(_spawner: Spawner ) -> ! {
    // Initialize board
    let p = embassy_nxp::init(Default::default());
    info!("Device started");

    // Take ownership of the HASHCRYPT peripheral
    let mut sha2 = Sha256::new(p.HASHCRYPT);

    let message = b"abcd";
    info!("{}", message);
    // Feed the unhashed message into the buffer. Once the buffer is filled,
    // it is drained into the FIFO. The update() method can be called as many
    // times as is necessary until the entire message is processed.
    sha2.update(message);

    // finalize() returns the final hashed message in it entirety
    // It consumes self, so it can not be called more than once
    let digest = sha2.finalize();
    
    info!("My hashed message: {:02x}", digest);

    loop {
        nop();
    }
}