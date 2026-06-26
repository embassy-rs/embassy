//! AES-ECB (Electronic Codebook) Mode Example
//!
//! Demonstrates basic AES encryption/decryption using ECB mode.
//!
//! # Cipher Mode: ECB (Electronic Codebook)
//! - Simplest AES mode - each block encrypted independently
//! - Requires 16-byte aligned data (padding necessary)
//! - WARNING: Not recommended for most data - identical plaintext blocks produce
//!   identical ciphertext blocks, revealing patterns
//! - Use only for encrypting random data like keys
//!
//! # What This Example Shows
//! - Basic AES peripheral initialization
//! - 128-bit and 256-bit key encryption
//! - Encryption and decryption operations
//! - NIST test vector validation
//!
//! # Test Vectors
//! Uses official NIST SP 800-38A test vectors for validation.

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::aes::{Aes, AesEcb, Direction};
use embassy_stm32::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    AES => embassy_stm32::aes::InterruptHandler<peripherals::AES>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("AES-ECB Example");

    let mut aes = Aes::new_blocking(p.AES, Irqs);

    // Test vector from NIST SP 800-38A
    let key_128 = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];

    let plaintext = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    ];

    let expected_ciphertext = [
        0x3a, 0xd7, 0x7b, 0xb4, 0x0d, 0x7a, 0x36, 0x60, 0xa8, 0x9e, 0xca, 0xf3, 0x24, 0x66, 0xef, 0x97,
    ];

    // ========== Encryption Test ==========
    info!("=== AES-ECB 128-bit Encryption ===");
    let cipher = AesEcb::new(&key_128);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    let mut ciphertext = [0u8; 16];
    match aes.payload_blocking(&mut ctx, &plaintext, &mut ciphertext, true) {
        Ok(()) => {
            info!("Plaintext:  {:02x}", plaintext);
            info!("Ciphertext: {:02x}", ciphertext);
            info!("Expected:   {:02x}", expected_ciphertext);

            if ciphertext == expected_ciphertext {
                info!("✓ Encryption PASSED!");
            } else {
                error!("✗ Encryption FAILED!");
            }
        }
        Err(e) => {
            error!("Encryption error: {:?}", e);
        }
    }

    aes.finish_blocking(ctx).ok();

    // ========== Decryption Test ==========
    info!("=== AES-ECB 128-bit Decryption ===");
    let cipher = AesEcb::new(&key_128);
    let mut ctx = aes.start(&cipher, Direction::Decrypt);

    let mut decrypted = [0u8; 16];
    match aes.payload_blocking(&mut ctx, &ciphertext, &mut decrypted, true) {
        Ok(()) => {
            info!("Ciphertext: {:02x}", ciphertext);
            info!("Decrypted:  {:02x}", decrypted);
            info!("Expected:   {:02x}", plaintext);

            if decrypted == plaintext {
                info!("✓ Decryption PASSED!");
            } else {
                error!("✗ Decryption FAILED!");
            }
        }
        Err(e) => {
            error!("Decryption error: {:?}", e);
        }
    }

    aes.finish_blocking(ctx).ok();

    // ========== 256-bit Key Test ==========
    info!("=== AES-ECB 256-bit Test ===");
    let key_256 = [
        0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81, 0x1f, 0x35,
        0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4,
    ];

    let plaintext_256 = [
        0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17, 0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10,
    ];

    let expected_256 = [
        0xb4, 0x7b, 0xd7, 0x3a, 0x60, 0x36, 0x7a, 0x0d, 0xf3, 0xca, 0x9e, 0xa8, 0x97, 0xef, 0x66, 0x24,
    ];

    let cipher = AesEcb::new(&key_256);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    let mut ciphertext_256 = [0u8; 16];
    match aes.payload_blocking(&mut ctx, &plaintext_256, &mut ciphertext_256, true) {
        Ok(()) => {
            info!("256-bit Ciphertext: {:02x}", ciphertext_256);
            info!("Expected:           {:02x}", expected_256);

            if ciphertext_256 == expected_256 {
                info!("✓ 256-bit Encryption PASSED!");
            } else {
                error!("✗ 256-bit Encryption FAILED!");
            }
        }
        Err(e) => {
            error!("256-bit Encryption error: {:?}", e);
        }
    }

    aes.finish_blocking(ctx).ok();

    info!("=== All AES-ECB tests complete ===");

    loop {
        cortex_m::asm::wfi();
    }
}
