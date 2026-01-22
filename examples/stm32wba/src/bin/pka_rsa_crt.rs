//! PKA RSA-CRT Decryption Example
//!
//! Demonstrates RSA decryption using the Chinese Remainder Theorem (CRT)
//! optimization with the PKA hardware accelerator.
//!
//! # What This Example Shows
//! - CRT-based RSA decryption (approximately 4x faster than standard RSA)
//! - Computing and using CRT parameters (dp, dq, qinv)
//! - Comparing performance with standard RSA decryption
//!
//! # CRT-RSA Basics
//! Instead of computing M = C^d mod n directly, CRT computes:
//! - M1 = C^dp mod p (where dp = d mod (p-1))
//! - M2 = C^dq mod q (where dq = d mod (q-1))
//! - M = M2 + q * (qinv * (M1 - M2) mod p)
//!
//! This is faster because:
//! - Exponents dp and dq are half the size of d
//! - Modular operations use smaller moduli p and q
//!
//! # CRT Parameters
//! - p, q: Prime factors of n
//! - dp = d mod (p-1)
//! - dq = d mod (q-1)
//! - qinv = q^(-1) mod p
//!
//! # Note
//! This example uses a small 512-bit key for demonstration.
//! In production, use at least 2048-bit keys!

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::pka::{Pka, RsaCrtParams};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::{bind_interrupts, peripherals, Config};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PKA => embassy_stm32::pka::InterruptHandler<peripherals::PKA>;
});

// RSA-512 test key components

// Modulus n (512 bits = 64 bytes)
const RSA_N: [u8; 64] = [
    0xd0, 0x36, 0x62, 0x5e, 0x57, 0x6e, 0x9d, 0x8e, 0x32, 0x6e, 0xf6, 0x34, 0xb2, 0x3a, 0xf5, 0x74,
    0x88, 0x87, 0x18, 0x8f, 0x74, 0x31, 0x22, 0x52, 0x9a, 0x6c, 0x30, 0x20, 0xb9, 0x8a, 0xd0, 0x5b,
    0xb0, 0x6e, 0xcd, 0x7f, 0x86, 0x31, 0x2e, 0xf0, 0x2c, 0x50, 0x25, 0x11, 0x3b, 0x3a, 0x0a, 0x5b,
    0x4f, 0x25, 0x0f, 0xd0, 0xd7, 0x38, 0x90, 0x82, 0x4d, 0x0c, 0x2b, 0x47, 0x93, 0x0a, 0x0f, 0x83,
];

// Public exponent e = 65537
const RSA_E: [u8; 3] = [0x01, 0x00, 0x01];

// Private exponent d (for comparison with standard RSA)
const RSA_D: [u8; 64] = [
    0x2f, 0x2d, 0x54, 0x6d, 0x28, 0xc6, 0x25, 0x5b, 0x34, 0x88, 0x33, 0x0a, 0x3e, 0xc4, 0xd0, 0xb5,
    0x70, 0x4e, 0xc5, 0xd2, 0x8a, 0x4d, 0x47, 0x3a, 0x3c, 0x97, 0x56, 0x2f, 0x5c, 0xf9, 0x01, 0x29,
    0x51, 0x0c, 0x02, 0x6c, 0x1f, 0x29, 0xe3, 0x0a, 0x5c, 0x49, 0x3c, 0x64, 0x70, 0xd7, 0x20, 0xa4,
    0xc3, 0x7f, 0xb1, 0x00, 0x29, 0x7f, 0x6c, 0x32, 0x5a, 0xe5, 0x2f, 0x69, 0xf5, 0x1d, 0x4e, 0x81,
];

// CRT Parameters

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

// dp = d mod (p-1) (256 bits = 32 bytes)
const RSA_DP: [u8; 32] = [
    0x58, 0x57, 0x3f, 0x42, 0x77, 0x7a, 0xf5, 0x6e, 0x4f, 0x16, 0x58, 0x32, 0xde, 0x55, 0x8e, 0x24,
    0x48, 0x01, 0x25, 0x81, 0xd4, 0x0f, 0x4d, 0x11, 0xe3, 0x42, 0x2c, 0xa4, 0x81, 0x40, 0xbe, 0x59,
];

// dq = d mod (q-1) (256 bits = 32 bytes)
const RSA_DQ: [u8; 32] = [
    0x71, 0x8a, 0x96, 0x53, 0x66, 0x4a, 0xa3, 0xb2, 0x05, 0xc4, 0xe0, 0x08, 0xae, 0x3b, 0x45, 0xb0,
    0x73, 0xde, 0x07, 0x0e, 0xb0, 0xc1, 0x32, 0x07, 0x3a, 0x41, 0xf9, 0x48, 0x20, 0x43, 0x95, 0x41,
];

// qinv = q^(-1) mod p (256 bits = 32 bytes)
const RSA_QINV: [u8; 32] = [
    0xb5, 0x13, 0x9f, 0x95, 0xf3, 0x32, 0x09, 0xa9, 0x02, 0x30, 0xde, 0xde, 0xd0, 0xf3, 0x97, 0x19,
    0x7a, 0x1a, 0x77, 0x21, 0x9c, 0x00, 0xf7, 0x4d, 0x2f, 0xd7, 0x6c, 0x1f, 0x27, 0x96, 0x19, 0xe0,
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
    info!("PKA RSA-CRT Decryption Example");

    let mut pka = Pka::new_blocking(p.PKA, Irqs);

    info!("=== RSA-CRT Decryption Example ===");
    info!("Key size: 512 bits (demonstration only!)");
    info!("CRT provides ~4x speedup over standard RSA decryption");

    // Create plaintext message
    let mut plaintext = [0u8; 64];
    // Simple test message
    plaintext[54..64].copy_from_slice(b"CRT-RSA!!!");

    info!("Original message (last 10 bytes): {:02x}", &plaintext[54..]);

    // Step 1: Encrypt with public key (standard RSA encryption)
    info!("=== Step 1: RSA Encryption (Public Key) ===");

    let mut ciphertext = [0u8; 64];
    match pka.modular_exp(&plaintext, &RSA_E, &RSA_N, &mut ciphertext) {
        Ok(()) => {
            info!("Encryption successful");
            info!("Ciphertext (first 16 bytes): {:02x}", &ciphertext[..16]);
        }
        Err(e) => {
            error!("Encryption failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Step 2: Decrypt using standard RSA (for comparison)
    info!("=== Step 2: Standard RSA Decryption ===");
    info!("M = C^d mod n (full exponent)");

    let mut decrypted_standard = [0u8; 64];
    match pka.modular_exp(&ciphertext, &RSA_D, &RSA_N, &mut decrypted_standard) {
        Ok(()) => {
            info!("Standard decryption complete");
            if decrypted_standard == plaintext {
                info!("Standard decryption verified!");
            } else {
                error!("Standard decryption FAILED!");
            }
        }
        Err(e) => {
            error!("Standard decryption failed: {:?}", e);
        }
    }

    // Step 3: Decrypt using RSA-CRT (faster!)
    info!("=== Step 3: RSA-CRT Decryption ===");
    info!("Uses smaller exponents dp, dq for ~4x speedup");
    info!("CRT parameters:");
    info!("  - p: {} bytes", RSA_P.len());
    info!("  - q: {} bytes", RSA_Q.len());
    info!("  - dp = d mod (p-1): {} bytes", RSA_DP.len());
    info!("  - dq = d mod (q-1): {} bytes", RSA_DQ.len());
    info!("  - qinv = q^(-1) mod p: {} bytes", RSA_QINV.len());

    let crt_params = RsaCrtParams {
        prime_p: &RSA_P,
        prime_q: &RSA_Q,
        dp: &RSA_DP,
        dq: &RSA_DQ,
        qinv: &RSA_QINV,
    };

    let mut decrypted_crt = [0u8; 64];
    match pka.rsa_crt_exp(&ciphertext, &crt_params, &mut decrypted_crt) {
        Ok(()) => {
            info!("CRT decryption complete");
            info!("Decrypted (last 10 bytes): {:02x}", &decrypted_crt[54..]);

            if decrypted_crt == plaintext {
                info!("CRT decryption verified!");
                info!("CRT result matches original plaintext");
            } else {
                error!("CRT decryption verification FAILED!");
            }
        }
        Err(e) => {
            error!("CRT decryption failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Step 4: Verify both methods produce the same result
    info!("=== Step 4: Comparing Standard vs CRT Results ===");

    if decrypted_standard == decrypted_crt {
        info!("Both decryption methods produce identical results!");
    } else {
        error!("Standard and CRT results differ!");
    }

    // Step 5: Multiple decryption demonstration
    info!("=== Step 5: Multiple CRT Decryptions ===");
    info!("Demonstrating CRT decryption with different messages");

    for i in 0..3u8 {
        // Create different test messages
        let mut msg = [0u8; 64];
        msg[63] = i + 1;

        // Encrypt
        let mut ct = [0u8; 64];
        if pka.modular_exp(&msg, &RSA_E, &RSA_N, &mut ct).is_ok() {
            // Decrypt with CRT
            let mut pt = [0u8; 64];
            if pka.rsa_crt_exp(&ct, &crt_params, &mut pt).is_ok() {
                if pt == msg {
                    info!("Message {}: CRT decryption OK", i + 1);
                } else {
                    error!("Message {}: CRT decryption FAILED", i + 1);
                }
            }
        }
    }

    info!("=== RSA-CRT Example Complete ===");
    info!("Summary:");
    info!("  - Standard RSA: C^d mod n (512-bit exponent)");
    info!("  - CRT RSA: Uses dp, dq (256-bit exponents)");
    info!("  - CRT is ~4x faster for private key operations");
    info!("  - Both methods produce identical results");

    loop {
        cortex_m::asm::wfi();
    }
}
