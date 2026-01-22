//! PKA RSA Encryption/Decryption Example
//!
//! Demonstrates RSA encryption and decryption using the PKA hardware accelerator.
//!
//! # What This Example Shows
//! - RSA encryption: ciphertext = plaintext^e mod n
//! - RSA decryption: plaintext = ciphertext^d mod n
//! - Using pre-computed Montgomery parameter for faster operations
//!
//! # RSA Basics
//! - Public key: (n, e) where n is the modulus and e is the public exponent
//! - Private key: d where d*e ≡ 1 (mod φ(n))
//! - Encryption: C = M^e mod n
//! - Decryption: M = C^d mod n
//!
//! # Note
//! This example uses a small 512-bit key for demonstration purposes.
//! In production, use at least 2048-bit keys!

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::pka::Pka;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::{bind_interrupts, peripherals, Config};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PKA => embassy_stm32::pka::InterruptHandler<peripherals::PKA>;
});

// RSA-512 test key (for demonstration only - use 2048+ bits in production!)
// These values are from a test vector set

// Modulus n (512 bits = 64 bytes)
const RSA_N: [u8; 64] = [
    0xd0, 0x36, 0x62, 0x5e, 0x57, 0x6e, 0x9d, 0x8e, 0x32, 0x6e, 0xf6, 0x34, 0xb2, 0x3a, 0xf5, 0x74,
    0x88, 0x87, 0x18, 0x8f, 0x74, 0x31, 0x22, 0x52, 0x9a, 0x6c, 0x30, 0x20, 0xb9, 0x8a, 0xd0, 0x5b,
    0xb0, 0x6e, 0xcd, 0x7f, 0x86, 0x31, 0x2e, 0xf0, 0x2c, 0x50, 0x25, 0x11, 0x3b, 0x3a, 0x0a, 0x5b,
    0x4f, 0x25, 0x0f, 0xd0, 0xd7, 0x38, 0x90, 0x82, 0x4d, 0x0c, 0x2b, 0x47, 0x93, 0x0a, 0x0f, 0x83,
];

// Public exponent e (typically 65537 = 0x10001)
const RSA_E: [u8; 3] = [0x01, 0x00, 0x01];

// Private exponent d (512 bits = 64 bytes)
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
    info!("PKA RSA Encryption/Decryption Example");

    let mut pka = Pka::new_blocking(p.PKA, Irqs);

    // Plaintext message (padded to modulus size)
    // In real applications, use proper padding like OAEP or PKCS#1 v1.5
    let mut plaintext = [0u8; 64];
    // Simple message: "Hello RSA!" followed by zeros
    plaintext[54..64].copy_from_slice(b"Hello RSA!");

    info!("=== RSA Encryption/Decryption Example ===");
    info!("Key size: 512 bits (demonstration only!)");
    info!("Plaintext (last 10 bytes): {:02x}", &plaintext[54..]);

    // Step 1: Compute Montgomery parameter for the modulus
    // This speeds up subsequent operations with the same modulus
    info!("Computing Montgomery parameter...");
    let mut montgomery_param = [0u32; 16]; // 64 bytes / 4 = 16 words
    match pka.montgomery_param(&RSA_N, &mut montgomery_param) {
        Ok(()) => {
            info!("Montgomery parameter computed successfully");
        }
        Err(e) => {
            error!("Failed to compute Montgomery parameter: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Step 2: Encrypt using public key (n, e)
    // C = M^e mod n
    info!("=== RSA Encryption ===");
    info!("Computing ciphertext = plaintext^e mod n");

    let mut ciphertext = [0u8; 64];
    match pka.modular_exp(&plaintext, &RSA_E, &RSA_N, &mut ciphertext) {
        Ok(()) => {
            info!("Encryption successful!");
            info!("Ciphertext: {:02x}", ciphertext);
        }
        Err(e) => {
            error!("Encryption failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Step 3: Decrypt using private key d
    // M = C^d mod n
    info!("=== RSA Decryption ===");
    info!("Computing plaintext = ciphertext^d mod n");

    let mut decrypted = [0u8; 64];
    match pka.modular_exp(&ciphertext, &RSA_D, &RSA_N, &mut decrypted) {
        Ok(()) => {
            info!("Decryption successful!");
            info!("Decrypted (last 10 bytes): {:02x}", &decrypted[54..]);

            // Verify decryption
            if decrypted == plaintext {
                info!("Decryption verified: plaintext matches!");
            } else {
                error!("Decryption verification FAILED!");
            }
        }
        Err(e) => {
            error!("Decryption failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Step 4: Demonstrate fast decryption with pre-computed Montgomery parameter
    info!("=== RSA Fast Decryption (with Montgomery param) ===");

    let mut decrypted_fast = [0u8; 64];
    match pka.modular_exp_fast(&ciphertext, &RSA_D, &RSA_N, &montgomery_param, &mut decrypted_fast) {
        Ok(()) => {
            info!("Fast decryption successful!");
            if decrypted_fast == plaintext {
                info!("Fast decryption verified: plaintext matches!");
            } else {
                error!("Fast decryption verification FAILED!");
            }
        }
        Err(e) => {
            error!("Fast decryption failed: {:?}", e);
        }
    }

    info!("=== RSA example complete ===");

    loop {
        cortex_m::asm::wfi();
    }
}
