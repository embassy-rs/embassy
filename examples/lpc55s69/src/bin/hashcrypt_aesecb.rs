#![no_std]
#![no_main]

use cortex_m as _;
use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp;
use embassy_nxp::hashcrypt::{AesPadded, GenericDriver, KeySize};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nxp::init(Default::default());
    info!("Device started !");

    let mut generic = GenericDriver::new(p.HASHCRYPT);

    let mut aes = generic.aes_ecb();
    aes.set_key_size(KeySize::Bits128);

    let key = [0x42; 16];
    let pt = b"hello world! this is my plaintext.";

    let _ = aes.set_key(key.as_slice());
    let mut out = [0u8; 48];
    let result = aes.encrypt_padded(pt, &mut out);

    info!("Test 1: Test arbitrary length message");
    info!("");

    match result {
        Ok(ct) => {
            info!("Plain text: {:02x}", &pt);
            info!("Ciphertext: {:02x}", &ct);
        }
        Err(e) => {
            error!("Error: {}", defmt::Debug2Format(&e));
        }
    }

    let key: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];

    let _ = aes.set_key(key.as_slice());

    let pt: [u8; 16] = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    ];

    let mut out: [u8; 32] = [0u8; 32];

    let result = aes.encrypt_padded(pt.as_slice(), out.as_mut_slice());

    info!("Test2: Using NIST test vector F.1.1 ECB-AES128.Encrypt, block 1");
    info!("");

    //     F.1.1 ECB-AES128.Encrypt
    // Key 2b7e151628aed2a6abf7158809cf4f3c
    // Block #1
    // Plaintext 6bc1bee22e409f96e93d7e117393172a
    // Input Block 6bc1bee22e409f96e93d7e117393172a
    // Output Block 3ad77bb40d7a3660a89ecaf32466ef97
    // Ciphertext 3ad77bb40d7a3660a89ecaf32466ef97

    match result {
        Ok(ct) => {
            info!("Plain text:  {:02x}", pt);
            info!("Cipher text: {:02x}", &ct);
        }
        Err(e) => {
            error!("{}", defmt::Debug2Format(&e));
        }
    }
    loop {
        Timer::after_millis(100).await;
    }
}
