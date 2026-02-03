//! AES-CBC (Cipher Block Chaining) Mode Example
//!
//! Demonstrates AES encryption/decryption using CBC mode with initialization vectors.
//!
//! # Cipher Mode: CBC (Cipher Block Chaining)
//! - Each plaintext block is XORed with previous ciphertext block before encryption
//! - Requires initialization vector (IV) - must be random and unique per message
//! - Requires 16-byte aligned data (padding necessary)
//! - Common for file and disk encryption
//!
//! # What This Example Shows
//! - CBC mode encryption/decryption with IV
//! - Multi-block processing (streaming)
//! - 128-bit and 256-bit keys
//! - Processing data in multiple calls
//! - Error handling for unaligned data
//! - NIST test vector validation
//!
//! # Security Notes
//! - IV must be random and unpredictable
//! - Never reuse the same IV with the same key
//! - CBC provides confidentiality but NOT authentication
//! - Consider GCM mode for authenticated encryption

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::aes::{Aes, AesCbc, Direction};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    AES => embassy_stm32::aes::InterruptHandler<peripherals::AES>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let mut config = Config::default();
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL30,
        divr: Some(PllDiv::DIV5),
        divq: None,
        divp: Some(PllDiv::DIV30),
        frac: Some(0),
    });
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    config.rcc.sys = Sysclk::PLL1_R;

    let p = embassy_stm32::init(config);
    info!("AES-CBC Example");

    let mut aes = Aes::new_blocking(p.AES, Irqs);

    // Test vectors from NIST SP 800-38A (F.2.1 CBC-AES128.Encrypt)
    // Using NO_SWAP mode for direct byte array compatibility
    let key = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];

    let iv = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ];

    // Two blocks of plaintext from NIST
    let plaintext = [
        // Block 1
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
        // Block 2
        0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51,
    ];

    // Expected ciphertext from NIST SP 800-38A (big-endian, NO_SWAP mode)
    let expected_ciphertext = [
        // Block 1
        0x76, 0x49, 0xab, 0xac, 0x81, 0x19, 0xb2, 0x46, 0xce, 0xe9, 0x8e, 0x9b, 0x12, 0xe9, 0x19, 0x7d,
        // Block 2
        0x50, 0x86, 0xcb, 0x9b, 0x50, 0x72, 0x19, 0xee, 0x95, 0xdb, 0x11, 0x3a, 0x91, 0x76, 0x78, 0xb2,
    ];

    // ========== CBC Encryption ==========
    info!("=== AES-CBC 128-bit Encryption ===");
    info!("Key: {:02x}", key);
    info!("IV:  {:02x}", iv);

    let cipher = AesCbc::new(&key, &iv);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    let mut ciphertext = [0u8; 32];
    match aes.payload_blocking(&mut ctx, &plaintext, &mut ciphertext, true) {
        Ok(()) => {
            info!("Plaintext:  {:02x}", plaintext);
            info!("Ciphertext: {:02x}", ciphertext);
            info!("Expected:   {:02x}", expected_ciphertext);

            if ciphertext == expected_ciphertext {
                info!("✓ CBC Encryption PASSED!");
            } else {
                error!("✗ CBC Encryption FAILED!");
                // Show which bytes differ
                for i in 0..32 {
                    if ciphertext[i] != expected_ciphertext[i] {
                        error!(
                            "  Byte {}: got {:02x}, expected {:02x}",
                            i, ciphertext[i], expected_ciphertext[i]
                        );
                    }
                }
            }
        }
        Err(e) => {
            error!("Encryption error: {:?}", e);
        }
    }

    aes.finish_blocking(ctx).ok();

    // ========== CBC Decryption ==========
    info!("=== AES-CBC 128-bit Decryption ===");

    let cipher = AesCbc::new(&key, &iv);
    let mut ctx = aes.start(&cipher, Direction::Decrypt);

    let mut decrypted = [0u8; 32];
    match aes.payload_blocking(&mut ctx, &ciphertext, &mut decrypted, true) {
        Ok(()) => {
            info!("Ciphertext: {:02x}", ciphertext);
            info!("Decrypted:  {:02x}", decrypted);
            info!("Expected:   {:02x}", plaintext);

            if decrypted == plaintext {
                info!("✓ CBC Decryption PASSED!");
            } else {
                error!("✗ CBC Decryption FAILED!");
            }
        }
        Err(e) => {
            error!("Decryption error: {:?}", e);
        }
    }

    aes.finish_blocking(ctx).ok();

    // ========== Multi-block Processing ==========
    info!("=== AES-CBC Multi-block Processing ===");

    // Encrypt in multiple calls (simulating streaming)
    let cipher = AesCbc::new(&key, &iv);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    let mut ciphertext_multi = [0u8; 32];

    // First block
    match aes.payload_blocking(&mut ctx, &plaintext[..16], &mut ciphertext_multi[..16], false) {
        Ok(()) => info!("✓ Block 1 encrypted"),
        Err(e) => error!("✗ Block 1 failed: {:?}", e),
    }

    // Second block (last=true)
    match aes.payload_blocking(&mut ctx, &plaintext[16..32], &mut ciphertext_multi[16..32], true) {
        Ok(()) => info!("✓ Block 2 encrypted"),
        Err(e) => error!("✗ Block 2 failed: {:?}", e),
    }

    aes.finish_blocking(ctx).ok();

    if ciphertext_multi == expected_ciphertext {
        info!("✓ Multi-block encryption PASSED!");
    } else {
        error!("✗ Multi-block encryption FAILED!");
    }

    // ========== 256-bit Key Test ==========
    info!("=== AES-CBC 256-bit Test ===");

    // NIST SP 800-38A F.2.5 CBC-AES256.Encrypt
    let key_256 = [
        0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81, 0x1f, 0x35,
        0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4,
    ];

    let iv_256 = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ];

    let plaintext_256 = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    ];

    // Expected from NIST (NO_SWAP mode, big-endian)
    let expected_256 = [
        0xf5, 0x8c, 0x4c, 0x04, 0xd6, 0xe5, 0xf1, 0xba, 0x77, 0x9e, 0xab, 0xfb, 0x5f, 0x7b, 0xfb, 0xd6,
    ];

    let cipher = AesCbc::new(&key_256, &iv_256);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    let mut ciphertext_256 = [0u8; 16];
    match aes.payload_blocking(&mut ctx, &plaintext_256, &mut ciphertext_256, true) {
        Ok(()) => {
            info!("256-bit Ciphertext: {:02x}", ciphertext_256);
            info!("Expected:           {:02x}", expected_256);

            if ciphertext_256 == expected_256 {
                info!("✓ 256-bit CBC Encryption PASSED!");
            } else {
                error!("✗ 256-bit CBC Encryption FAILED!");
            }
        }
        Err(e) => {
            error!("256-bit encryption error: {:?}", e);
        }
    }

    aes.finish_blocking(ctx).ok();

    // ========== Error Handling Example ==========
    info!("=== Testing Error Handling ===");

    let cipher = AesCbc::new(&key, &iv);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    // Try to encrypt non-aligned data (should fail with REQUIRES_PADDING modes)
    let unaligned_data = [0u8; 15]; // Not a multiple of 16
    let mut output = [0u8; 15];

    match aes.payload_blocking(&mut ctx, &unaligned_data, &mut output, false) {
        Ok(()) => {
            warn!("Unexpected success with unaligned data");
        }
        Err(e) => {
            info!("✓ Correctly rejected unaligned data: {:?}", e);
        }
    }

    aes.finish_blocking(ctx).ok();

    info!("=== All AES-CBC tests complete ===");

    loop {
        cortex_m::asm::wfi();
    }
}
