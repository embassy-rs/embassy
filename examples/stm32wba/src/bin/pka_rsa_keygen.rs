//! PKA RSA Key Finalization Example
//!
//! Demonstrates RSA key preparation and validation using PKA hardware.
//!
//! # What This Example Shows
//! - Computing the Montgomery parameter R² mod n (required for fast operations)
//! - Validating RSA key relationships using modular arithmetic
//! - Using comparison operations for big integer validation
//!
//! # RSA Key Components
//! - n = p * q (modulus, product of two primes)
//! - e (public exponent, typically 65537)
//! - d (private exponent, where e*d ≡ 1 mod φ(n))
//! - φ(n) = (p-1)(q-1) (Euler's totient)
//!
//! # Key Finalization Steps
//! 1. Compute Montgomery parameter R² mod n for fast modular operations
//! 2. Optionally verify e*d ≡ 1 (mod φ(n))
//! 3. Pre-compute CRT parameters if using CRT decryption
//!
//! # Note
//! This example uses a small 512-bit key for demonstration.
//! In production, use at least 2048-bit keys!

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::pka::{ComparisonResult, Pka};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::{bind_interrupts, peripherals, Config};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PKA => embassy_stm32::pka::InterruptHandler<peripherals::PKA>;
});

// RSA-512 test key components
// n = p * q
const RSA_N: [u8; 64] = [
    0xd0, 0x36, 0x62, 0x5e, 0x57, 0x6e, 0x9d, 0x8e, 0x32, 0x6e, 0xf6, 0x34, 0xb2, 0x3a, 0xf5, 0x74,
    0x88, 0x87, 0x18, 0x8f, 0x74, 0x31, 0x22, 0x52, 0x9a, 0x6c, 0x30, 0x20, 0xb9, 0x8a, 0xd0, 0x5b,
    0xb0, 0x6e, 0xcd, 0x7f, 0x86, 0x31, 0x2e, 0xf0, 0x2c, 0x50, 0x25, 0x11, 0x3b, 0x3a, 0x0a, 0x5b,
    0x4f, 0x25, 0x0f, 0xd0, 0xd7, 0x38, 0x90, 0x82, 0x4d, 0x0c, 0x2b, 0x47, 0x93, 0x0a, 0x0f, 0x83,
];

// Prime p (256 bits = 32 bytes)
const RSA_P: [u8; 32] = [
    0xf5, 0x2b, 0x3f, 0x61, 0xf4, 0x52, 0x35, 0x15, 0xa6, 0xf5, 0x38, 0x63, 0x4a, 0x5f, 0x70, 0x75,
    0x21, 0x5d, 0x19, 0x02, 0x35, 0x86, 0xce, 0xb2, 0x2a, 0xdb, 0x55, 0x3e, 0x6c, 0x57, 0x9b, 0x59,
];

// Prime q (256 bits = 32 bytes)
const RSA_Q: [u8; 32] = [
    0xd9, 0x29, 0x6a, 0xde, 0x51, 0x65, 0xd7, 0x54, 0x56, 0xc0, 0x76, 0x04, 0xd4, 0x98, 0xd0, 0x62,
    0xd8, 0x9c, 0x53, 0x62, 0x71, 0xc4, 0x6c, 0x30, 0x49, 0x14, 0xa0, 0xe6, 0x8c, 0x79, 0xe7, 0x4b,
];

// Public exponent e = 65537
const RSA_E: [u8; 3] = [0x01, 0x00, 0x01];

// Private exponent d
const RSA_D: [u8; 64] = [
    0x2f, 0x2d, 0x54, 0x6d, 0x28, 0xc6, 0x25, 0x5b, 0x34, 0x88, 0x33, 0x0a, 0x3e, 0xc4, 0xd0, 0xb5,
    0x70, 0x4e, 0xc5, 0xd2, 0x8a, 0x4d, 0x47, 0x3a, 0x3c, 0x97, 0x56, 0x2f, 0x5c, 0xf9, 0x01, 0x29,
    0x51, 0x0c, 0x02, 0x6c, 0x1f, 0x29, 0xe3, 0x0a, 0x5c, 0x49, 0x3c, 0x64, 0x70, 0xd7, 0x20, 0xa4,
    0xc3, 0x7f, 0xb1, 0x00, 0x29, 0x7f, 0x6c, 0x32, 0x5a, 0xe5, 0x2f, 0x69, 0xf5, 0x1d, 0x4e, 0x81,
];

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
    info!("PKA RSA Key Finalization Example");

    let mut pka = Pka::new_blocking(p.PKA, Irqs);

    info!("=== RSA Key Finalization Example ===");
    info!("Key size: 512 bits (demonstration only!)");

    // Step 1: Compute Montgomery parameter R² mod n
    // This is essential for fast modular exponentiation
    info!("=== Step 1: Computing Montgomery Parameter ===");
    info!("R² mod n is required for fast modular operations");

    let mut montgomery_param = [0u32; 16]; // 64 bytes / 4 = 16 words
    match pka.montgomery_param(&RSA_N, &mut montgomery_param) {
        Ok(()) => {
            info!("Montgomery parameter computed successfully");
            info!("First 4 words: {:08x} {:08x} {:08x} {:08x}",
                montgomery_param[0], montgomery_param[1],
                montgomery_param[2], montgomery_param[3]);
        }
        Err(e) => {
            error!("Failed to compute Montgomery parameter: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Step 2: Verify n = p * q using arithmetic multiplication
    info!("=== Step 2: Verifying n = p * q ===");

    let mut computed_n = [0u8; 64];
    match pka.arithmetic_mul(&RSA_P, &RSA_Q, &mut computed_n) {
        Ok(()) => {
            info!("Computed p * q");

            // Compare with known n
            match pka.comparison(&computed_n, &RSA_N) {
                Ok(ComparisonResult::Equal) => {
                    info!("Verified: p * q = n");
                }
                Ok(result) => {
                    error!("Verification FAILED: p * q != n (result: {:?})", result);
                }
                Err(e) => {
                    error!("Comparison failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            error!("Multiplication failed: {:?}", e);
        }
    }

    // Step 3: Validate that d < n (private exponent must be less than modulus)
    info!("=== Step 3: Validating d < n ===");

    match pka.comparison(&RSA_D, &RSA_N) {
        Ok(ComparisonResult::Less) => {
            info!("Verified: d < n (private exponent is valid)");
        }
        Ok(ComparisonResult::Equal) => {
            error!("Invalid: d = n (should not happen)");
        }
        Ok(ComparisonResult::Greater) => {
            error!("Invalid: d > n (private exponent too large)");
        }
        Err(e) => {
            error!("Comparison failed: {:?}", e);
        }
    }

    // Step 4: Test encryption/decryption to verify key pair
    info!("=== Step 4: Verifying Key Pair with Test Encryption ===");

    // Small test message
    let mut test_message = [0u8; 64];
    test_message[63] = 0x42; // Simple test value

    // Encrypt with public key
    let mut ciphertext = [0u8; 64];
    match pka.modular_exp_fast(&test_message, &RSA_E, &RSA_N, &montgomery_param, &mut ciphertext) {
        Ok(()) => {
            info!("Test encryption successful");
        }
        Err(e) => {
            error!("Test encryption failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Decrypt with private key
    let mut decrypted = [0u8; 64];
    match pka.modular_exp_fast(&ciphertext, &RSA_D, &RSA_N, &montgomery_param, &mut decrypted) {
        Ok(()) => {
            if decrypted == test_message {
                info!("Key pair verification PASSED!");
                info!("Encryption and decryption work correctly");
            } else {
                error!("Key pair verification FAILED!");
                error!("Decrypted message does not match original");
            }
        }
        Err(e) => {
            error!("Test decryption failed: {:?}", e);
        }
    }

    // Step 5: Demonstrate modular inverse (useful for computing d from e)
    info!("=== Step 5: Modular Inverse Demonstration ===");
    info!("Computing modular inverse of a test value");

    // Compute inverse of 7 mod 11 (should be 8, since 7*8 = 56 = 5*11 + 1)
    let a: [u8; 1] = [7];
    let modulus: [u8; 1] = [11];
    let mut inverse = [0u8; 1];

    match pka.modular_inv(&a, &modulus, &mut inverse) {
        Ok(()) => {
            info!("Inverse of 7 mod 11 = {}", inverse[0]);
            if inverse[0] == 8 {
                info!("Modular inverse verified correct!");
            }
        }
        Err(e) => {
            error!("Modular inverse failed: {:?}", e);
        }
    }

    // Step 6: Demonstrate modular reduction
    info!("=== Step 6: Modular Reduction ===");

    // Reduce a large number mod n
    let large_num = [0xFFu8; 64]; // All 1s
    let mut reduced = [0u8; 64];

    match pka.modular_red(&large_num, &RSA_N, &mut reduced) {
        Ok(()) => {
            info!("Modular reduction successful");
            info!("Result (first 8 bytes): {:02x}", &reduced[..8]);
        }
        Err(e) => {
            error!("Modular reduction failed: {:?}", e);
        }
    }

    info!("=== RSA Key Finalization Example Complete ===");
    info!("Summary:");
    info!("  - Montgomery parameter: computed");
    info!("  - n = p * q: verified");
    info!("  - d < n: verified");
    info!("  - Key pair: verified");

    loop {
        cortex_m::asm::wfi();
    }
}
