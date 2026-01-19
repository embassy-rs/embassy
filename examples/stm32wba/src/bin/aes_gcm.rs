//! AES-GCM (Galois/Counter Mode) - Authenticated Encryption Example
//!
//! Demonstrates modern authenticated encryption using AES-GCM mode.
//! This is the RECOMMENDED mode for new applications.
//!
//! # Cipher Mode: GCM (Galois/Counter Mode)
//! - Combines encryption + authentication in one operation
//! - No padding required - works with any data length
//! - Generates authentication tag to detect tampering
//! - Supports Additional Authenticated Data (AAD) - metadata that's authenticated but not encrypted
//! - Industry standard for TLS, IPsec, disk encryption
//!
//! # What This Example Shows
//! - Complete GCM encryption with AAD
//! - Tag generation and verification
//! - Decryption with authentication check
//! - GCM without AAD (optional AAD)
//! - NIST GCM test vector validation
//!
//! # Three-Phase GCM Process
//! 1. AAD Phase: Process metadata to authenticate (optional)
//! 2. Payload Phase: Encrypt/decrypt data
//! 3. Final Phase: Generate/verify authentication tag
//!
//! # Why Use GCM
//! - Provides both confidentiality AND authenticity
//! - Detects any tampering or corruption
//! - AAD protects metadata without encryption overhead
//! - Single-pass operation (efficient)
//! - Widely supported and standardized
//!
//! # Security Notes
//! - IV/nonce must be unique for each encryption with same key
//! - Recommended: 96-bit (12-byte) random nonce
//! - ALWAYS verify tag before using decrypted data
//! - Tag verification failure means: reject the data (tampering detected)

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::aes::{Aes, AesGcm, Direction};
use embassy_stm32::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    AES => embassy_stm32::aes::InterruptHandler<peripherals::AES>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("AES-GCM Authenticated Encryption Example");

    let mut aes = Aes::new_blocking(p.AES, Irqs);

    // Test vector from NIST GCM test suite
    let key = [
        0xfe, 0xff, 0xe9, 0x92, 0x86, 0x65, 0x73, 0x1c, 0x6d, 0x6a, 0x8f, 0x94, 0x67, 0x30, 0x83, 0x08,
    ];

    let iv = [0xca, 0xfe, 0xba, 0xbe, 0xfa, 0xce, 0xdb, 0xad, 0xde, 0xca, 0xf8, 0x88];

    let plaintext = [
        0xd9, 0x31, 0x32, 0x25, 0xf8, 0x84, 0x06, 0xe5, 0xa5, 0x59, 0x09, 0xc5, 0xaf, 0xf5, 0x26, 0x9a, 0x86, 0xa7,
        0xa9, 0x53, 0x15, 0x34, 0xf7, 0xda, 0x2e, 0x4c, 0x30, 0x3d, 0x8a, 0x31, 0x8a, 0x72, 0x1c, 0x3c, 0x0c, 0x95,
        0x95, 0x68, 0x09, 0x53, 0x2f, 0xcf, 0x0e, 0x24, 0x49, 0xa6, 0xb5, 0x25, 0xb1, 0x6a, 0xed, 0xf5, 0xaa, 0x0d,
        0xe6, 0x57, 0xba, 0x63, 0x7b, 0x39, 0x1a, 0xaf, 0xd2, 0x55,
    ];

    let aad = [
        0xfe, 0xed, 0xfa, 0xce, 0xde, 0xad, 0xbe, 0xef, 0xfe, 0xed, 0xfa, 0xce, 0xde, 0xad, 0xbe, 0xef, 0xab, 0xad,
        0xda, 0xd2,
    ];

    let expected_ciphertext = [
        0x42, 0x83, 0x1e, 0xc2, 0x21, 0x77, 0x74, 0x24, 0x4b, 0x72, 0x21, 0xb7, 0x84, 0xd0, 0xd4, 0x9c, 0xe3, 0xaa,
        0x21, 0x2f, 0x2c, 0x02, 0xa4, 0xe0, 0x35, 0xc1, 0x7e, 0x23, 0x29, 0xac, 0xa1, 0x2e, 0x21, 0xd5, 0x14, 0xb2,
        0x54, 0x66, 0x93, 0x1c, 0x7d, 0x8f, 0x6a, 0x5a, 0xac, 0x84, 0xaa, 0x05, 0x1b, 0xa3, 0x0b, 0x39, 0x6a, 0x0a,
        0xac, 0x97, 0x3d, 0x58, 0xe0, 0x91, 0x47, 0x3f, 0x59, 0x85,
    ];

    let expected_tag = [
        0x4d, 0x5c, 0x2a, 0xf3, 0x27, 0xcd, 0x64, 0xa6, 0x2c, 0xf3, 0x5a, 0xbd, 0x2b, 0xa6, 0xfa, 0xb4,
    ];

    // ========== GCM Encryption with AAD ==========
    info!("=== AES-GCM Encryption ===");
    info!("Key:       {:02x}", key);
    info!("IV (12b):  {:02x}", iv);
    info!("AAD:       {:02x}", aad);
    info!("Plaintext: {} bytes", plaintext.len());

    let cipher = AesGcm::new(&key, &iv);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    // Process AAD (Additional Authenticated Data)
    match aes.aad_blocking(&mut ctx, &aad, true) {
        Ok(()) => info!("✓ AAD processed"),
        Err(e) => {
            error!("✗ AAD processing failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Encrypt payload
    let mut ciphertext = [0u8; 64];
    match aes.payload_blocking(&mut ctx, &plaintext, &mut ciphertext, true) {
        Ok(()) => {
            info!("✓ Payload encrypted");
            info!("Ciphertext: {:02x}", ciphertext);
        }
        Err(e) => {
            error!("✗ Encryption failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Get authentication tag
    match aes.finish_blocking(ctx) {
        Ok(Some(tag)) => {
            info!("Auth Tag:     {:02x}", tag);
            info!("Expected Tag: {:02x}", expected_tag);

            // Verify results
            let ciphertext_ok = ciphertext == expected_ciphertext;
            let tag_ok = tag == expected_tag;

            if ciphertext_ok && tag_ok {
                info!("✓ GCM Encryption PASSED!");
            } else {
                if !ciphertext_ok {
                    error!("✗ Ciphertext mismatch!");
                }
                if !tag_ok {
                    error!("✗ Tag mismatch!");
                }
            }
        }
        Ok(None) => {
            error!("✗ No tag returned!");
        }
        Err(e) => {
            error!("✗ Finish failed: {:?}", e);
        }
    }

    // ========== GCM Decryption and Verification ==========
    info!("=== AES-GCM Decryption ===");

    let cipher = AesGcm::new(&key, &iv);
    let mut ctx = aes.start(&cipher, Direction::Decrypt);

    // Process AAD
    match aes.aad_blocking(&mut ctx, &aad, true) {
        Ok(()) => info!("✓ AAD processed"),
        Err(e) => {
            error!("✗ AAD processing failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Decrypt payload
    let mut decrypted = [0u8; 64];
    match aes.payload_blocking(&mut ctx, &ciphertext, &mut decrypted, true) {
        Ok(()) => {
            info!("✓ Payload decrypted");
        }
        Err(e) => {
            error!("✗ Decryption failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Get and verify authentication tag
    match aes.finish_blocking(ctx) {
        Ok(Some(tag)) => {
            info!("Computed Tag: {:02x}", tag);

            // In real applications, you would compare the computed tag with the received tag
            if tag == expected_tag {
                info!("✓ Tag verification PASSED!");

                // Check decrypted plaintext
                if decrypted == plaintext {
                    info!("✓ Decryption PASSED!");
                    info!("Decrypted: {:02x}", decrypted[..16]);
                } else {
                    error!("✗ Decryption mismatch!");
                }
            } else {
                error!("✗ Tag verification FAILED - message authentication failed!");
                // In real code, you would reject the decrypted data
            }
        }
        Ok(None) => {
            error!("✗ No tag returned!");
        }
        Err(e) => {
            error!("✗ Finish failed: {:?}", e);
        }
    }

    // ========== GCM with No AAD ==========
    info!("=== AES-GCM without AAD ===");

    let simple_plaintext = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ];

    let cipher = AesGcm::new(&key, &iv);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    // No AAD - skip aad_blocking call

    let mut simple_ciphertext = [0u8; 16];
    match aes.payload_blocking(&mut ctx, &simple_plaintext, &mut simple_ciphertext, true) {
        Ok(()) => {
            info!("✓ Simple encryption complete");
            info!("Plaintext:  {:02x}", simple_plaintext);
            info!("Ciphertext: {:02x}", simple_ciphertext);
        }
        Err(e) => {
            error!("✗ Simple encryption failed: {:?}", e);
        }
    }

    if let Ok(Some(tag)) = aes.finish_blocking(ctx) {
        info!("Tag: {:02x}", tag);
    }

    info!("=== All AES-GCM tests complete ===");

    loop {
        cortex_m::asm::wfi();
    }
}
