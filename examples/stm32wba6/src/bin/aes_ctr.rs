//! AES-CTR (Counter) Mode Example - Stream Cipher
//!
//! Demonstrates AES in Counter mode, which converts the block cipher into a stream cipher.
//!
//! # Cipher Mode: CTR (Counter)
//! - Turns AES into a stream cipher by encrypting sequential counter values
//! - No padding required - works with any data length
//! - Encryption and decryption are identical operations
//! - Allows parallel processing and random access
//! - Ideal for streaming data
//!
//! # What This Example Shows
//! - Stream cipher operation (no padding needed)
//! - Partial block handling (13 bytes, not aligned to 16)
//! - Streaming with multiple calls of varying sizes
//! - 128-bit and 256-bit keys
//! - Encryption/decryption symmetry (same operation)
//! - NIST test vector validation
//!
//! # Advantages of CTR Mode
//! - Process any length data without padding overhead
//! - Can decrypt specific blocks without decrypting entire stream
//! - Encryption/decryption use the same hardware operation
//! - Suitable for real-time streaming applications
//!
//! # Security Notes
//! - Counter/nonce must NEVER be reused with the same key
//! - Typically: nonce (random) + counter (incremental)
//! - CTR provides confidentiality but NOT authentication
//! - Consider GCM mode for authenticated encryption

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::aes::{Aes, AesCtr, Direction};
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
    info!("AES-CTR Stream Cipher Example");

    let mut aes = Aes::new_blocking(p.AES, Irqs);

    // Test vectors from NIST SP 800-38A
    let key = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];

    // Counter/IV (nonce + counter)
    let counter = [
        0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
    ];

    let plaintext = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a, 0xae, 0x2d,
        0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51,
    ];

    let expected_ciphertext = [
        0x87, 0x4d, 0x61, 0x91, 0xb6, 0x20, 0xe3, 0x26, 0x1b, 0xef, 0x68, 0x64, 0x99, 0x0d, 0xb6, 0xce, 0x98, 0x06,
        0xf6, 0x6b, 0x79, 0x70, 0xfd, 0xff, 0x86, 0x17, 0x18, 0x7b, 0xb9, 0xff, 0xfd, 0xff,
    ];

    // ========== CTR Encryption ==========
    info!("=== AES-CTR 128-bit Encryption ===");
    info!("Key:     {:02x}", key);
    info!("Counter: {:02x}", counter);

    let cipher = AesCtr::new(&key, &counter);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    let mut ciphertext = [0u8; 32];
    match aes.payload_blocking(&mut ctx, &plaintext, &mut ciphertext, true) {
        Ok(()) => {
            info!("Plaintext:  {:02x}", plaintext);
            info!("Ciphertext: {:02x}", ciphertext);
            info!("Expected:   {:02x}", expected_ciphertext);

            if ciphertext == expected_ciphertext {
                info!("✓ CTR Encryption PASSED!");
            } else {
                error!("✗ CTR Encryption FAILED!");
            }
        }
        Err(e) => {
            error!("Encryption error: {:?}", e);
        }
    }

    aes.finish_blocking(ctx).ok();

    // ========== CTR Decryption (same as encryption for CTR mode) ==========
    info!("=== AES-CTR 128-bit Decryption ===");

    // In CTR mode, encryption and decryption are the same operation
    let cipher = AesCtr::new(&key, &counter);
    let mut ctx = aes.start(&cipher, Direction::Encrypt); // Note: can use Encrypt for both

    let mut decrypted = [0u8; 32];
    match aes.payload_blocking(&mut ctx, &ciphertext, &mut decrypted, true) {
        Ok(()) => {
            info!("Ciphertext: {:02x}", ciphertext);
            info!("Decrypted:  {:02x}", decrypted);
            info!("Expected:   {:02x}", plaintext);

            if decrypted == plaintext {
                info!("✓ CTR Decryption PASSED!");
            } else {
                error!("✗ CTR Decryption FAILED!");
            }
        }
        Err(e) => {
            error!("Decryption error: {:?}", e);
        }
    }

    aes.finish_blocking(ctx).ok();

    // ========== Stream Cipher Property - Partial Block ==========
    info!("=== AES-CTR Stream Cipher - Partial Block ===");

    // CTR mode can handle any data length (no padding required)
    let partial_data = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, // 13 bytes
    ];

    let expected_partial = [
        0x87, 0x4d, 0x61, 0x91, 0xb6, 0x20, 0xe3, 0x26, 0x1b, 0xef, 0x68, 0x64, 0x99, // 13 bytes
    ];

    let cipher = AesCtr::new(&key, &counter);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    let mut ciphertext_partial = [0u8; 13];
    match aes.payload_blocking(&mut ctx, &partial_data, &mut ciphertext_partial, true) {
        Ok(()) => {
            info!("Partial input (13 bytes): {:02x}", partial_data);
            info!("Output:                   {:02x}", ciphertext_partial);
            info!("Expected:                 {:02x}", expected_partial);

            if ciphertext_partial == expected_partial {
                info!("✓ Partial block encryption PASSED!");
            } else {
                error!("✗ Partial block encryption FAILED!");
            }
        }
        Err(e) => {
            error!("Partial block error: {:?}", e);
        }
    }

    aes.finish_blocking(ctx).ok();

    // ========== Streaming Multiple Calls ==========
    info!("=== AES-CTR Streaming Processing ===");
    // Note: Intermediate chunks must be block-aligned (16 bytes).
    // Only the final chunk (last=true) can be a partial block.

    let cipher = AesCtr::new(&key, &counter);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    let mut ciphertext_stream = [0u8; 32];

    // Encrypt first 16 bytes (block-aligned)
    match aes.payload_blocking(&mut ctx, &plaintext[..16], &mut ciphertext_stream[..16], false) {
        Ok(()) => info!("✓ Stream block 1 encrypted (16 bytes)"),
        Err(e) => error!("✗ Stream block 1 failed: {:?}", e),
    }

    // Encrypt final 16 bytes (last=true)
    match aes.payload_blocking(&mut ctx, &plaintext[16..32], &mut ciphertext_stream[16..32], true) {
        Ok(()) => info!("✓ Stream block 2 encrypted (16 bytes, final)"),
        Err(e) => error!("✗ Stream block 2 failed: {:?}", e),
    }

    aes.finish_blocking(ctx).ok();

    if ciphertext_stream == expected_ciphertext {
        info!("✓ Streaming encryption PASSED!");
    } else {
        error!("✗ Streaming encryption FAILED!");
        info!("Got:      {:02x}", ciphertext_stream);
        info!("Expected: {:02x}", expected_ciphertext);
    }

    // ========== 256-bit Key Test ==========
    info!("=== AES-CTR 256-bit Test ===");

    let key_256 = [
        0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81, 0x1f, 0x35,
        0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4,
    ];

    let counter_256 = [
        0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
    ];

    let plaintext_256 = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    ];

    let expected_256 = [
        0x60, 0x1e, 0xc3, 0x13, 0x77, 0x57, 0x89, 0xa5, 0xb7, 0xa7, 0xf5, 0x04, 0xbb, 0xf3, 0xd2, 0x28,
    ];

    let cipher = AesCtr::new(&key_256, &counter_256);
    let mut ctx = aes.start(&cipher, Direction::Encrypt);

    let mut ciphertext_256 = [0u8; 16];
    match aes.payload_blocking(&mut ctx, &plaintext_256, &mut ciphertext_256, true) {
        Ok(()) => {
            info!("256-bit Ciphertext: {:02x}", ciphertext_256);
            info!("Expected:           {:02x}", expected_256);

            if ciphertext_256 == expected_256 {
                info!("✓ 256-bit CTR Encryption PASSED!");
            } else {
                error!("✗ 256-bit CTR Encryption FAILED!");
            }
        }
        Err(e) => {
            error!("256-bit encryption error: {:?}", e);
        }
    }

    aes.finish_blocking(ctx).ok();

    // ========== Demonstrate Stream Cipher Advantage ==========
    info!("=== CTR Mode Advantages ===");
    info!("✓ No padding required - works with any data length");
    info!("✓ Encryption and decryption use same operation");
    info!("✓ Parallel processing possible (not shown in blocking mode)");
    info!("✓ Random access to encrypted data (can decrypt any block independently)");

    info!("=== All AES-CTR tests complete ===");

    loop {
        cortex_m::asm::wfi();
    }
}
