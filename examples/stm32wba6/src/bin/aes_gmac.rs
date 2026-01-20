//! AES-GMAC (Galois Message Authentication Code) Example
//!
//! Demonstrates message authentication without encryption using AES-GMAC mode.
//! GMAC authenticates data integrity without providing confidentiality.
//!
//! # Cipher Mode: GMAC (Galois Message Authentication Code)
//! - Provides authentication WITHOUT encryption
//! - Data remains in plaintext but tampering is detected
//! - Generates authentication tag for integrity verification
//! - Uses the same GHASH operation as GCM
//!
//! # Use Cases
//! - Authenticating packet headers in network protocols
//! - Verifying integrity of publicly-readable metadata
//! - Any scenario requiring authentication without encryption
//! - Protocol message authentication
//!
//! # How It Works
//! 1. Initialize with key and unique IV
//! 2. Process header data (AAD only - no payload)
//! 3. Generate authentication tag
//! 4. Use tag to verify data integrity
//!
//! # Security Notes
//! - IV/nonce must be unique for each authentication with same key
//! - GMAC alone does NOT encrypt data - use GCM if confidentiality needed
//! - Tag verification detects any tampering with the authenticated data

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::aes::{Aes, AesGmac, Direction};
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
    info!("AES-GMAC Message Authentication Example");

    let mut aes = Aes::new_blocking(p.AES, Irqs);

    // ========== GMAC Test Case (from ST HAL) ==========
    // This test authenticates a 68-byte header message

    // 256-bit key
    let key: [u8; 32] = [
        0x69, 0x1D, 0x3E, 0xE9, 0x09, 0xD7, 0xF5, 0x41, 0x67, 0xFD, 0x1C, 0xA0, 0xB5, 0xD7, 0x69, 0x08, 0x1F, 0x2B,
        0xDE, 0x1A, 0xEE, 0x65, 0x5F, 0xDB, 0xAB, 0x80, 0xBD, 0x52, 0x95, 0xAE, 0x6B, 0xE7,
    ];

    // 12-byte IV (nonce)
    let iv: [u8; 12] = [0xF0, 0x76, 0x1E, 0x8D, 0xCD, 0x3D, 0x00, 0x01, 0x76, 0xD4, 0x57, 0xED];

    // Header message to authenticate (68 bytes - not block aligned)
    let header: [u8; 68] = [
        0xE2, 0x01, 0x06, 0xD7, 0xCD, 0x0D, 0xF0, 0x76, 0x1E, 0x8D, 0xCD, 0x3D, 0x88, 0xE5, 0x40, 0x00, 0x76, 0xD4,
        0x57, 0xED, 0x08, 0x00, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C,
        0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E,
        0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x00, 0x03,
    ];

    // Expected tag from ST HAL test
    let expected_tag: [u8; 16] = [
        0x35, 0x21, 0x7C, 0x77, 0x4B, 0xBC, 0x31, 0xB6, 0x31, 0x66, 0xBC, 0xF9, 0xD4, 0xAB, 0xED, 0x07,
    ];

    info!("=== AES-GMAC Authentication ===");
    info!("Key (256-bit): {:02x}", key[..16]);
    info!("IV (12 bytes): {:02x}", iv);
    info!("Header: {} bytes", header.len());

    // Create GMAC cipher
    let cipher = AesGmac::new(&key, &iv);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    // Process header data (this is the data we're authenticating)
    match aes.aad_blocking(&mut ctx, &header, true) {
        Ok(()) => info!("✓ Header processed for authentication"),
        Err(e) => {
            error!("✗ Header processing failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // No payload phase for GMAC - we're just authenticating, not encrypting

    // Get authentication tag
    match aes.finish_blocking(ctx) {
        Ok(Some(tag)) => {
            info!("Computed Tag: {:02x}", tag);
            info!("Expected Tag: {:02x}", expected_tag);

            if tag == expected_tag {
                info!("✓ GMAC Authentication PASSED!");
            } else {
                error!("✗ GMAC Tag mismatch!");
            }
        }
        Ok(None) => {
            error!("✗ No tag returned!");
        }
        Err(e) => {
            error!("✗ Finish failed: {:?}", e);
        }
    }

    // ========== Simple GMAC Example ==========
    info!("=== Simple GMAC Example ===");

    // 128-bit key for simplicity
    let simple_key: [u8; 16] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ];

    let simple_iv: [u8; 12] = [0xca, 0xfe, 0xba, 0xbe, 0xfa, 0xce, 0xdb, 0xad, 0xde, 0xca, 0xf8, 0x88];

    // Message to authenticate
    let message = b"This message will be authenticated but NOT encrypted";

    info!("Message: \"{}\"", core::str::from_utf8(message).unwrap_or(""));
    info!("Message length: {} bytes", message.len());

    let cipher = AesGmac::new(&simple_key, &simple_iv);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    match aes.aad_blocking(&mut ctx, message, true) {
        Ok(()) => info!("✓ Message authenticated"),
        Err(e) => {
            error!("✗ Authentication failed: {:?}", e);
        }
    }

    if let Ok(Some(tag)) = aes.finish_blocking(ctx) {
        info!("Auth Tag: {:02x}", tag);
        info!("Note: This tag can be sent with the message to verify integrity");
    }

    info!("=== All GMAC tests complete ===");

    loop {
        cortex_m::asm::wfi();
    }
}
