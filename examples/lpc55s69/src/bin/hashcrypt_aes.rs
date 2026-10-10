#![no_std]
#![no_main]

use cortex_m as _;
use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::hashcrypt::{Aes, AesPadded, GenericHashcrypt, KeySize};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nxp::init(Default::default());
    info!("Device started !");

    let mut generic = GenericHashcrypt::new(p.HASHCRYPT);

    info!("ECB Example");
    // F.1.1 ECB-AES128.Encrypt / F.1.2 ECB-AES128.Decrypt (NIST SP 800-38A)
    // Key 2b7e151628aed2a6abf7158809cf4f3c
    // Block #1
    // Plaintext 6bc1bee22e409f96e93d7e117393172a
    // Input Block 6bc1bee22e409f96e93d7e117393172a
    // Output Block 3ad77bb40d7a3660a89ecaf32466ef97
    // Ciphertext 3ad77bb40d7a3660a89ecaf32466ef97
    //
    // Arbitrary length (Tests 3-4), PKCS#7 padded, reference from pycryptodome / RustCrypto `ecb`
    // Key 42424242424242424242424242424242
    // Plaintext "hello world! this is my plaintext." (34 bytes)
    // Ciphertext 42b153410851a931eb3e6c048867ae5f95eb20b42e176b07840db75688be9c70e4670ea0d87a71be5f9f3099b4fff3dc

    let mut ecb = generic.aes_ecb();
    ecb.set_key_size(KeySize::Bits128);

    info!("Example1: F.1.1 ECB-AES128.Encrypt, block 1");
    let key1: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];
    let pt1: [u8; 16] = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    ];
    let ct1: [u8; 16] = [
        0x3a, 0xd7, 0x7b, 0xb4, 0x0d, 0x7a, 0x36, 0x60, 0xa8, 0x9e, 0xca, 0xf3, 0x24, 0x66, 0xef, 0x97,
    ];

    ecb.set_key(&key1).unwrap();
    let mut out = [0u8; 16];
    match ecb.encrypt(&pt1, &mut out) {
        Ok(()) => {
            info!("Plain text:  {:02x}", pt1);
            info!("Cipher text: {:02x}", out);
        }
        Err(e) => error!("{}", &e),
    }

    info!("Example2: F.1.2 ECB-AES128.Decrypt, block 1");
    // Reuses key1 and ct1 from Test1
    let mut out = [0u8; 16];
    match ecb.decrypt(&ct1, &mut out) {
        Ok(()) => {
            info!("Plain text:  {:02x}", out);
            info!("Cipher text: {:02x}", ct1);
        }
        Err(e) => error!("{}", &e),
    }

    info!("Test3: Arbitrary length encrypt");
    let key3 = [0x42u8; 16];
    let pt3: &[u8; 34] = b"hello world! this is my plaintext.";
    let ct3: [u8; 48] = [
        0x42, 0xb1, 0x53, 0x41, 0x08, 0x51, 0xa9, 0x31, 0xeb, 0x3e, 0x6c, 0x04, 0x88, 0x67, 0xae, 0x5f, 0x95, 0xeb,
        0x20, 0xb4, 0x2e, 0x17, 0x6b, 0x07, 0x84, 0x0d, 0xb7, 0x56, 0x88, 0xbe, 0x9c, 0x70, 0xe4, 0x67, 0x0e, 0xa0,
        0xd8, 0x7a, 0x71, 0xbe, 0x5f, 0x9f, 0x30, 0x99, 0xb4, 0xff, 0xf3, 0xdc,
    ];

    ecb.set_key(&key3).unwrap();
    let mut out = [0u8; 48];
    match ecb.encrypt_padded(pt3, &mut out) {
        Ok(ct) => {
            info!("Plain text:  {:02x}", pt3);
            info!("Cipher text: {:02x}", ct);
        }
        Err(e) => error!("{}", &e),
    }

    info!("Example4: Arbitrary length decrypt");
    // Reuses key3 and ct3 from Test3
    let mut out = [0u8; 48];
    match ecb.decrypt_padded(&ct3, &mut out) {
        Ok(pt) => {
            info!("Plain text:  {:02x}", pt);
            info!("Cipher text: {:02x}", ct3);
        }
        Err(e) => error!("{}", &e),
    }

    info!("CBC Examples");
    // F.2.1 CBC-AES128.Encrypt / F.2.2 CBC-AES128.Decrypt (NIST SP 800-38A)
    // Key 2b7e151628aed2a6abf7158809cf4f3c
    // IV 000102030405060708090a0b0c0d0e0f
    // Block #1
    // Plaintext 6bc1bee22e409f96e93d7e117393172a
    // Input Block 6bc0bce12a459991e134741a7f9e1925
    // Output Block 7649abac8119b246cee98e9b12e9197d
    // Ciphertext 7649abac8119b246cee98e9b12e9197d
    //
    // Arbitrary length (Tests 3-4), PKCS#7 padded, same key and IV, reference from pycryptodome
    // Plaintext "hello world! this is my plaintext." (34 bytes)
    // Ciphertext bbdc0c6b15317b603c5bed2e77305d9ef4c7a971e2d53abdfc9a7eff920bc7d63b7c5a5bb85dc49d72b810663f420ef1

    let mut cbc = generic.aes_cbc();
    cbc.set_key_size(KeySize::Bits128);

    info!("Example1: F.2.1 CBC-AES128.Encrypt, block 1");
    let key1: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];
    let iv1: [u8; 16] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ];
    let pt1: [u8; 16] = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    ];
    let ct1: [u8; 16] = [
        0x76, 0x49, 0xab, 0xac, 0x81, 0x19, 0xb2, 0x46, 0xce, 0xe9, 0x8e, 0x9b, 0x12, 0xe9, 0x19, 0x7d,
    ];

    cbc.set_key(&key1).unwrap();
    cbc.set_iv(&iv1);
    let mut out = [0u8; 16];
    match cbc.encrypt(&pt1, &mut out) {
        Ok(()) => {
            info!("Plain text:  {:02x}", pt1);
            info!("Cipher text: {:02x}", out);
        }
        Err(e) => error!("{}", &e),
    }

    info!("Example2: F.2.2 CBC-AES128.Decrypt, block 1");
    // Reuses key1, iv1 and ct1 from Test1
    let mut out = [0u8; 16];
    match cbc.decrypt(&ct1, &mut out) {
        Ok(()) => {
            info!("Plain text:  {:02x}", out);
            info!("Cipher text: {:02x}", ct1);
        }
        Err(e) => error!("{}", &e),
    }

    info!("Example3: Arbitrary length encrypt");
    // Reuses key1 and iv1 from Test1
    let pt3: &[u8; 34] = b"hello world! this is my plaintext.";
    let ct3: [u8; 48] = [
        0xbb, 0xdc, 0x0c, 0x6b, 0x15, 0x31, 0x7b, 0x60, 0x3c, 0x5b, 0xed, 0x2e, 0x77, 0x30, 0x5d, 0x9e, 0xf4, 0xc7,
        0xa9, 0x71, 0xe2, 0xd5, 0x3a, 0xbd, 0xfc, 0x9a, 0x7e, 0xff, 0x92, 0x0b, 0xc7, 0xd6, 0x3b, 0x7c, 0x5a, 0x5b,
        0xb8, 0x5d, 0xc4, 0x9d, 0x72, 0xb8, 0x10, 0x66, 0x3f, 0x42, 0x0e, 0xf1,
    ];

    let mut out = [0u8; 48];
    match cbc.encrypt_padded(pt3, &mut out) {
        Ok(ct) => {
            info!("Plain text:  {:02x}", pt3);
            info!("Cipher text: {:02x}", ct);
        }
        Err(e) => error!("{}", &e),
    }

    info!("Example4: Arbitrary length decrypt");
    // Reuses key1 and iv1 from Test1, ct3 from Test3
    let mut out = [0u8; 48];
    match cbc.decrypt_padded(&ct3, &mut out) {
        Ok(pt) => {
            info!("Plain text:  {:02x}", pt);
            info!("Cipher text: {:02x}", ct3);
        }
        Err(e) => error!("{}", &e),
    }

    // ---------------------------------------------------------------- CTR
    info!("CTR Examples");
    // F.5.1 CTR-AES128.Encrypt / F.5.2 CTR-AES128.Decrypt (NIST SP 800-38A)
    // Key 2b7e151628aed2a6abf7158809cf4f3c
    // Init. Counter f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff
    // Block #1
    // Input Block f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff
    // Output Block ec8cdf7398607cb0f2d21675ea9ea1e4
    // Plaintext 6bc1bee22e409f96e93d7e117393172a
    // Ciphertext 874d6191b620e3261bef6864990db6ce
    // Block #2
    // Input Block f0f1f2f3f4f5f6f7f8f9fafbfcfdff00
    // Output Block 362b7c3c6773516318a077d7fc5073ae
    // Plaintext ae2d8a571e03ac9c9eb76fac45af8e51
    // Ciphertext 9806f66b7970fdff8617187bb9fffdff
    //
    // Arbitrary length (Tests 3-4), same key and counter, reference from pycryptodome
    // Plaintext "hello world! this is my plaintext." (34 bytes)
    // Ciphertext 84e9b31ff7400bdf80be7254caeac98d450b154f471e284368cc16be922416d61e02

    let mut ctr = generic.aes_ctr();
    ctr.set_key_size(KeySize::Bits128);

    info!("Example1: F.5.1 CTR-AES128.Encrypt, blocks 1-2");
    let key1: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];
    let counter1: [u8; 16] = [
        0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
    ];
    let pt1: [u8; 32] = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a, 0xae, 0x2d,
        0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51,
    ];
    let ct1: [u8; 32] = [
        0x87, 0x4d, 0x61, 0x91, 0xb6, 0x20, 0xe3, 0x26, 0x1b, 0xef, 0x68, 0x64, 0x99, 0x0d, 0xb6, 0xce, 0x98, 0x06,
        0xf6, 0x6b, 0x79, 0x70, 0xfd, 0xff, 0x86, 0x17, 0x18, 0x7b, 0xb9, 0xff, 0xfd, 0xff,
    ];

    ctr.set_key(&key1).unwrap();
    ctr.set_counter(&counter1);
    let mut out = [0u8; 32];
    match ctr.encrypt(&pt1, &mut out) {
        Ok(()) => {
            info!("Plain text:  {:02x}", pt1);
            info!("Cipher text: {:02x}", out);
        }
        Err(e) => error!("{}", &e),
    }

    info!("Test2: F.5.2 CTR-AES128.Decrypt, blocks 1-2");
    // Reuses key1, counter1 and ct1 from Test1
    let mut out = [0u8; 32];
    match ctr.decrypt(&ct1, &mut out) {
        Ok(()) => {
            info!("Plain text:  {:02x}", out);
            info!("Cipher text: {:02x}", ct1);
        }
        Err(e) => error!("{}", &e),
    }

    info!("Example3: Arbitrary length encrypt");
    // Reuses key1 and counter1 from Test1
    let pt3: &[u8; 34] = b"hello world! this is my plaintext.";
    let ct3: [u8; 34] = [
        0x84, 0xe9, 0xb3, 0x1f, 0xf7, 0x40, 0x0b, 0xdf, 0x80, 0xbe, 0x72, 0x54, 0xca, 0xea, 0xc9, 0x8d, 0x45, 0x0b,
        0x15, 0x4f, 0x47, 0x1e, 0x28, 0x43, 0x68, 0xcc, 0x16, 0xbe, 0x92, 0x24, 0x16, 0xd6, 0x1e, 0x02,
    ];

    let mut out = [0u8; 34];
    match ctr.encrypt(pt3, &mut out) {
        Ok(()) => {
            info!("Plain text:  {:02x}", pt3);
            info!("Cipher text: {:02x}", out);
        }
        Err(e) => error!("{}", &e),
    }

    info!("Example4: Arbitrary length decrypt");
    // Reuses key1 and counter1 from Test1, ct3 from Test3
    let mut out = [0u8; 34];
    match ctr.decrypt(&ct3, &mut out) {
        Ok(()) => {
            info!("Plain text:  {:02x}", out);
            info!("Cipher text: {:02x}", ct3);
        }
        Err(e) => error!("{}", &e),
    }

    loop {
        Timer::after_millis(100).await;
    }
}
