#![no_std]
#![no_main]

use cortex_m as _;
use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp;
use embassy_nxp::hashcrypt::KeySize::Bits128;
use embassy_nxp::hashcrypt::{Aes, AesPadded, GenericDriver, KeySize};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nxp::init(Default::default());
    info!("Device started !");

    let mut generic = GenericDriver::new(p.HASHCRYPT);

    let mut ecb = generic.aes_ecb();
    ecb.set_key_size(KeySize::Bits128);

    let key = [0x42; 16];
    let pt = b"hello world! this is my plaintext.";

    let _ = ecb.set_key(key.as_slice());
    let mut out = [0u8; 48];
    let result = ecb.encrypt_padded(pt, &mut out);

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

    let _ = ecb.set_key(key.as_slice());

    let pt: [u8; 16] = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    ];

    let mut out: [u8; 32] = [0u8; 32];

    let result = ecb.encrypt_padded(pt.as_slice(), out.as_mut_slice());

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

    info!("Test 3: Decrypting a padded block, wrong key");
    // If the wrong key was ussed, the decrypted cyphertext might have invalid padding

    let ct: [u8; 48] = [
        0x42, 0xb1, 0x53, 0x41, 0x08, 0x51, 0xa9, 0x31, 0xeb, 0x3e, 0x6c, 0x04, 0x88, 0x67, 0xae, 0x5f, 0x95, 0xeb,
        0x20, 0xb4, 0x2e, 0x17, 0x6b, 0x07, 0x84, 0x0d, 0xb7, 0x56, 0x88, 0xbe, 0x9c, 0x70, 0xe4, 0x67, 0x0e, 0xa0,
        0xd8, 0x7a, 0x71, 0xbe, 0x5f, 0x9f, 0x30, 0x99, 0xb4, 0xff, 0xf3, 0xdc,
    ];

    let mut out = [0u8; 48];

    let result = ecb.decrypt_padded(ct.as_slice(), out.as_mut_slice());
    match result {
        Ok(pt) => {
            info!("Plain text: {=[u8]:a}", pt);
            info!("Cipher text: {:02x}", ct);
        }
        Err(e) => {
            error!("{}", defmt::Debug2Format(&e));
        }
    }

    info!("Test 4: Decrypting a padded block, correct key");

    let key = [0x42; 16];
    let _ = ecb.set_key(&key.as_slice());

    let result = ecb.decrypt_padded(ct.as_slice(), out.as_mut_slice());
    match result {
        Ok(pt) => {
            info!("Plain text: {=[u8]:a}", pt);
            info!("Cipher text: {}", ct);
        }
        Err(e) => {
            error!("{}", defmt::Debug2Format(&e));
        }
    }

    info!("Test5: Decrypt an unpadded block: NIST test vector F.1.1 ECB-AES128.Encrypt, block 1");

    let ct: [u8; 32] = [
        0x3a, 0xd7, 0x7b, 0xb4, 0x0d, 0x7a, 0x36, 0x60, 0xa8, 0x9e, 0xca, 0xf3, 0x24, 0x66, 0xef, 0x97, 0xa2, 0x54,
        0xbe, 0x88, 0xe0, 0x37, 0xdd, 0xd9, 0xd7, 0x9f, 0xb6, 0x41, 0x1c, 0x3f, 0x9d, 0xf8,
    ];

    let key: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];

    let _ = ecb.set_key(key.as_slice());
    let mut out = [0u8; 32];

    let result = ecb.decrypt_padded(ct.as_slice(), out.as_mut_slice());
    match result {
        Ok(pt) => {
            info!("Plain text:  {:02x}", pt);
            info!("Cipher text: {:02x}", ct);
        }
        Err(e) => {
            error!("{}", defmt::Debug2Format(&e));
        }
    }

    info!("Test6: Using NIST test vector F.1.1 ECB-AES128.Encrypt, block 2 with .encrypt");
    info!("");

    ecb.set_key(&key).unwrap();

    // Block #2
    // Plaintext ae2d8a571e03ac9c9eb76fac45af8e51
    // Input Block ae2d8a571e03ac9c9eb76fac45af8e51
    // Output Block f5d3d58503b9699de785895a96fdbaaf
    // Ciphertext f5d3d58503b9699de785895a96fdbaaf
    let pt2: [u8; 16] = [
        0xAE, 0x2D, 0x8A, 0x57, 0x1E, 0x03, 0xAC, 0x9C, 0x9E, 0xB7, 0x6F, 0xAC, 0x45, 0xAF, 0x8E, 0x51,
    ];

    let ct2 = [
        0xf5, 0xd3, 0xd5, 0x85, 0x03, 0xb9, 0x69, 0x9d, 0xe7, 0x85, 0x89, 0x5a, 0x96, 0xfd, 0xba, 0xaf,
    ];

    let mut out = [0u8; 16];
    let result = ecb.encrypt(pt2.as_slice(), out.as_mut_slice());
    match result {
        Ok(_) => {
            info!("Plain text:  {:02x}", pt2);
            info!("Cipher text: {:02x}", out);
        }
        Err(e) => {
            error!("{}", defmt::Debug2Format(&e));
        }
    }

    ecb.set_key(&key).unwrap();
    info!("Test7: Decrypt unpadded block");

    let mut out = [0u8; 16];
    let result = ecb.decrypt(ct2.as_slice(), out.as_mut_slice());
    match result {
        Ok(_) => {
            info!("Plain text:  {:02x}", out);
            info!("Cipher text: {:02x}", ct2);
        }
        Err(e) => {
            error!("{}", defmt::Debug2Format(&e));
        }
    }

    info!("CBC TESTS");
    info!("");

    let mut cbc = generic.aes_cbc();
    // F.2.1 CBC-AES128.Encrypt / F.2.2 CBC-AES128.Decrypt, block 1
    // Key 2b7e151628aed2a6abf7158809cf4f3c
    // IV 000102030405060708090a0b0c0d0e0f
    // Block #1
    // Plaintext 6bc1bee22e409f96e93d7e117393172a
    // Input Block 6bc0bce12a459991e134741a7f9e1925
    // Output Block 7649abac8119b246cee98e9b12e9197d
    // Ciphertext 7649abac8119b246cee98e9b12e9197d

    let key = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];

    let iv = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ];

    let pt1 = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    ];

    let ct1 = [
        0x76, 0x49, 0xab, 0xac, 0x81, 0x19, 0xb2, 0x46, 0xce, 0xe9, 0x8e, 0x9b, 0x12, 0xe9, 0x19, 0x7d,
    ];

    cbc.set_key_size(Bits128);
    cbc.set_key(key.as_slice()).unwrap();
    cbc.set_iv(&iv);
    let mut out = [0u8; 16];

    info!("Test 1: F.2.1 CBC-AES128.Encrypt");
    let result = cbc.encrypt(pt1.as_slice(), out.as_mut_slice());
    match result {
        Ok(_) => {
            info!("Plain text: {:02x}", &pt1);
            info!("Cipher text: {:02x}", &out);
        }
        Err(e) => {
            error!("{}", defmt::Debug2Format(&e));
        }
    }

    info!("Test 2: F.2.1 CBC-AES128.Decrypt");
    let mut out = [0u8; 16];
    let result = cbc.decrypt(ct1.as_slice(), out.as_mut_slice());
    match result {
        Ok(_) => {
            info!("Plain text: {:02x}", &out);
            info!("Cipher text: {:02x}", &ct1);
        }
        Err(e) => {
            error!("{}", defmt::Debug2Format(&e));
        }
    }

    info!("Test3: Encrypting an aritrary length message");
    // resut the same key and iv as the NIST tests

    let pt2 = b"hello world! this is my plaintext.";
    let ct2 = [
        0xbb, 0xdc, 0x0c, 0x6b, 0x15, 0x31, 0x7b, 0x60, 0x3c, 0x5b, 0xed, 0x2e, 0x77, 0x30, 0x5d, 0x9e, 0xf4, 0xc7,
        0xa9, 0x71, 0xe2, 0xd5, 0x3a, 0xbd, 0xfc, 0x9a, 0x7e, 0xff, 0x92, 0x0b, 0xc7, 0xd6, 0x3b, 0x7c, 0x5a, 0x5b,
        0xb8, 0x5d, 0xc4, 0x9d, 0x72, 0xb8, 0x10, 0x66, 0x3f, 0x42, 0x0e, 0xf1,
    ];
    let mut out = [0u8; 48];
    let result = cbc.encrypt_padded(pt2, out.as_mut_slice());
    match result {
        Ok(c) => {
            info!("Plain text: {:02x}", pt2);
            info!("Cipher text: {:02x}", &c);
        }
        Err(e) => {
            error!("{}", defmt::Debug2Format(&e));
        }
    }

    info!("Test4: Decrypting a padded ciphertext");
    let mut out = [0u8; 48];
    let result = cbc.decrypt_padded(ct2.as_slice(), out.as_mut_slice());
    match result {
        Ok(p) => {
            info!("Plain text: {:02x}", &p);
            info!("Cipher text: {:02x}", ct2);
        }
        Err(e) => {
            error!("{}", defmt::Debug2Format(&e));
        }
    }
    loop {
        Timer::after_millis(100).await;
    }
}
