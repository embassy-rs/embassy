//! This example has been made with the LPCXpresso55S69. The hashcrypt driver is built according to FIPS-180-4,
//! and tested against FIPS Cryptographic Standards and Guidelines

#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt;
use panic_probe as _;
use defmt_rtt as _;   // <- this line is missing
use defmt::info;
use embassy_nxp::{self, hashcrypt::{Digest}};
use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_nxp::hashcrypt;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> !
{
    let p = embassy_nxp::init(Default::default());
    info!("Device started !");
    
    // Creates an instance of a generic driver that lives for as long as the program does which
    // holds the HASHCRYPT peripheral and and passes a mutable reference to itself to
    // whichever specific driver instance is created at a time
    let mut generic = hashcrypt::GenericDriver::new(p.HASHCRYPT);

    let msg1 = "abc";
    let msg2 = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";

    info!("SHA1");
    // Creates an instance of a Sha1 driver
    let mut sha_1 = generic.sha1();

    // Buffers an arbitrary length message which is drained into the hashcrypt FIFO
    sha_1.update(msg1.as_bytes());
    // Returns the 5 word digest for SHA-1
    let digest = sha_1.finalise();
    info!("Input message:  {}", &msg1); 
    info!("Message Digest: {:02x}",&digest);

    // The instance of the specific SHA-1 driver is reusable after finalise() is called
    sha_1.update(msg2.as_bytes());
    let digest = sha_1.finalise();
    info!("Input message:  {}", &msg2);
    info!("Message Digest: {:02x}",&digest);

    // The generic driver instance still exists and can be used to create a different specific driver
    info!("SHA-256");
    let mut sha2 = generic.sha256();

    // Buffers an arbitrary length message which is drained in to the hashcrypt FIFO
    sha2.update(msg1.as_bytes());
    // Returns the 8 word digest for SHA-256
    let digest = sha2.finalise();
    info!("Input message:  {}", msg1);
    info!("Message digest: {:02x}", &digest);

    sha2.update(msg2.as_bytes());
    let digest = sha2.finalise();
    info!("Input message:  {}", &msg2);
    info!("Message digest: {:02x}", &digest);

    loop {
        Timer::after_millis(100).await;
    }
}