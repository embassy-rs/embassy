//! AES-CCM (Counter with CBC-MAC) - Authenticated Encryption Example
//!
//! Demonstrates authenticated encryption using AES-CCM mode.
//! CCM is an alternative to GCM that's commonly used in constrained environments.
//!
//! # Cipher Mode: CCM (Counter with CBC-MAC)
//! - Combines CTR encryption with CBC-MAC authentication
//! - Provides both confidentiality AND authenticity
//! - Requires knowing data lengths in advance (AAD and payload)
//! - Used in IEEE 802.15.4 (Zigbee), Bluetooth LE, IPsec
//!
//! # Key Differences from GCM
//! - CCM requires knowing payload/AAD lengths before encryption
//! - CCM uses CBC-MAC for authentication (vs GHASH in GCM)
//! - CCM may have variable tag sizes (4-16 bytes)
//! - CCM AAD has a specific format with length prefix
//!
//! # B0 Block Format (computed by AesCcm::new)
//! - Byte 0: Flags (tag size, AAD present, L value)
//! - Bytes 1-N: Nonce
//! - Bytes N+1-15: Payload length (big-endian)
//!
//! # AAD Format for CCM
//! For CCM, AAD must be formatted with a length prefix:
//! - If AAD_len < 2^16-2^8: 2-byte length || AAD || padding
//! - If AAD_len < 2^32: 0xFFFE || 4-byte length || AAD || padding
//!
//! # Security Notes
//! - Nonce must be unique for each message with same key
//! - Tag verification failure means: reject the data
//! - Variable tag size allows security/overhead tradeoff

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::aes::{Aes, AesCcm, Direction};
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
    info!("AES-CCM Authenticated Encryption Example");

    let mut aes = Aes::new_blocking(p.AES, Irqs);

    // ========== CCM Test Case (from ST HAL) ==========
    // Note: CCM requires pre-formatted AAD with length prefix

    // 256-bit key
    let key: [u8; 32] = [
        0xD3, 0x46, 0xD1, 0x1A, 0x71, 0x17, 0xCE, 0x04, 0x08, 0x08, 0x95, 0x70, 0x77, 0x78, 0x28, 0x7C, 0x40, 0xF5,
        0xF4, 0x73, 0xA9, 0xA8, 0xF2, 0xB1, 0x57, 0x0F, 0x61, 0x37, 0x46, 0x69, 0x75, 0x1A,
    ];

    // 12-byte nonce (extracted from ST HAL B0 block bytes 1-12)
    // Note: The first byte of B0 (0x7A) is the FLAGS byte, computed by AesCcm::new()
    // The nonce is bytes 1-12 of the B0 block
    let nonce: [u8; 12] = [0x05, 0xC8, 0xCC, 0x77, 0x32, 0xB3, 0xB4, 0x7F, 0x08, 0xAF, 0x1D, 0xAF];

    // Actual AAD (7 bytes): 0x34, 0x21, 0x5F, 0x03, 0x25, 0x67, 0x0B
    // For CCM, AAD must be formatted with length prefix
    // Formatted AAD (B1 block): 2-byte length (0x0007) || 7 bytes AAD || 7 bytes padding
    let formatted_aad: [u8; 16] = [
        0x00, 0x07, // 2-byte length encoding (7 bytes of AAD)
        0x34, 0x21, 0x5F, 0x03, 0x25, 0x67, 0x0B, // Actual AAD (7 bytes)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Zero padding
    ];

    // Plaintext: 17 bytes (not block-aligned)
    let plaintext: [u8; 17] = [
        0xBB, 0xD8, 0x83, 0x34, 0x00, 0x00, 0x75, 0xF6, 0xF4, 0xE8, 0x9F, 0x9D, 0xDA, 0x50, 0xF5, 0xEA, 0xB1,
    ];

    // Expected ciphertext: 17 bytes
    let expected_ciphertext: [u8; 17] = [
        0xA7, 0xB7, 0x65, 0x3C, 0x5D, 0x60, 0x0A, 0xF3, 0x9C, 0xA0, 0xDB, 0x48, 0x0F, 0x4F, 0x5C, 0xCE, 0x99,
    ];

    // Expected tag (16 bytes)
    let expected_tag: [u8; 16] = [
        0x35, 0x2C, 0x36, 0xD3, 0x93, 0x5B, 0x88, 0x94, 0x04, 0x26, 0xA0, 0x04, 0x3B, 0xBA, 0xB7, 0xEE,
    ];

    // AAD length and payload length must be known in advance for CCM
    let aad_len = 7; // Actual AAD length (not including format prefix)
    let payload_len = 17;

    info!("=== AES-CCM Encryption ===");
    info!("Key (256-bit): {:02x}", key[..16]);
    info!("Nonce (12 bytes): {:02x}", nonce);
    info!("AAD length: {} bytes", aad_len);
    info!("Payload length: {} bytes", payload_len);

    // Create CCM cipher
    // Parameters: key, nonce, AAD length, payload length
    // IV_SIZE = 12, TAG_SIZE = 16 (full tag)
    let cipher: AesCcm<32, 12, 16> = AesCcm::new(&key, &nonce, aad_len, payload_len);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    // Process formatted AAD (with length prefix for CCM)
    // Note: CCM requires AAD to be pre-formatted with length encoding
    match aes.aad_blocking(&mut ctx, &formatted_aad, true) {
        Ok(()) => info!("✓ AAD processed"),
        Err(e) => {
            error!("✗ AAD processing failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Encrypt payload
    let mut ciphertext = [0u8; 17];
    match aes.payload_blocking(&mut ctx, &plaintext, &mut ciphertext, true) {
        Ok(()) => {
            info!("✓ Payload encrypted");
            info!("Plaintext:  {:02x}", plaintext);
            info!("Ciphertext: {:02x}", ciphertext);
            info!("Expected:   {:02x}", expected_ciphertext);

            if ciphertext == expected_ciphertext {
                info!("✓ Ciphertext MATCHES!");
            } else {
                error!("✗ Ciphertext mismatch!");
            }
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

            if tag == expected_tag {
                info!("✓ CCM Encryption PASSED!");
            } else {
                error!("✗ Tag mismatch!");
            }
        }
        Ok(None) => {
            error!("✗ No tag returned!");
        }
        Err(e) => {
            error!("✗ Finish failed: {:?}", e);
        }
    }

    // ========== CCM Decryption ==========
    info!("=== AES-CCM Decryption ===");

    let cipher: AesCcm<32, 12, 16> = AesCcm::new(&key, &nonce, aad_len, payload_len);
    let mut ctx = aes.start(&cipher, Direction::Decrypt);

    // Process AAD
    match aes.aad_blocking(&mut ctx, &formatted_aad, true) {
        Ok(()) => info!("✓ AAD processed"),
        Err(e) => {
            error!("✗ AAD processing failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Decrypt ciphertext
    let mut decrypted = [0u8; 17];
    match aes.payload_blocking(&mut ctx, &ciphertext, &mut decrypted, true) {
        Ok(()) => {
            info!("✓ Payload decrypted");
            info!("Decrypted: {:02x}", decrypted);

            if decrypted == plaintext {
                info!("✓ Decryption matches plaintext!");
            } else {
                error!("✗ Decryption mismatch!");
            }
        }
        Err(e) => {
            error!("✗ Decryption failed: {:?}", e);
        }
    }

    // Verify tag
    match aes.finish_blocking(ctx) {
        Ok(Some(tag)) => {
            info!("Computed Tag: {:02x}", tag);

            if tag == expected_tag {
                info!("✓ CCM Decryption and Verification PASSED!");
            } else {
                error!("✗ Tag verification FAILED - data may be tampered!");
            }
        }
        Ok(None) => {
            error!("✗ No tag returned!");
        }
        Err(e) => {
            error!("✗ Finish failed: {:?}", e);
        }
    }

    info!("=== All CCM tests complete ===");

    loop {
        cortex_m::asm::wfi();
    }
}
