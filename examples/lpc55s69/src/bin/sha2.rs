//! This example has been made with the LPCXpresso55S69 board in mind and has been verified against
//! FIPS PUB 180-4, Appendix A.2 (Implementation Notes), via the worked SHA-256 examples
//! at NIST's Cryptographic Standards and Guidelines site, as well as against Apple's sha256sum
//! utility on macOS.

#![no_std]
#![no_main]

use cortex_m::asm::nop;
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::hashcrypt::Sha256;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Initialize board
    let p = embassy_nxp::init(Default::default());
    info!("Device started");

    // Take ownership of the HASHCRYPT peripheral
    let mut sha2 = Sha256::new(p.HASHCRYPT);

    let message = b"abc";
    info!("{}", message);
    // Feed the unhashed message into the buffer. Once the buffer is filled,
    // it is drained into the FIFO. The update() method can be called as many
    // times as is necessary until the entire message is processed.
    sha2.update(message);

    // finalize() returns the final hashed message in its entirety
    let digest = sha2.finalize();

    info!("Message 1 Digest: {:02x}", digest);

    let message = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    sha2.update(message);
    let digest = sha2.finalize();
    info!("Message 2 Digest: {:02x}", digest);

    // Calling finalize() twice in a row with no update() calls in between results
    // in the hash of an empty message
    info!("Empty Digest:     {:02x}", sha2.finalize());

    loop {
        nop();
    }
}
