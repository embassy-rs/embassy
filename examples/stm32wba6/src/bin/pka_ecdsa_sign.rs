//! PKA ECDSA Signature Generation Example
//!
//! Demonstrates ECDSA signature generation using the PKA hardware accelerator.
//!
//! # What This Example Shows
//! - Generating an ECDSA signature with a private key
//! - Using the hardware RNG for the random nonce (k value)
//! - Verifying the generated signature
//!
//! # ECDSA Signing Process
//! 1. Hash the message (SHA-256 for P-256)
//! 2. Generate a random nonce k (CRITICAL: must be unique per signature!)
//! 3. Compute signature (r, s) = Sign(hash, private_key, k)
//!
//! # Security Notes
//! - The k value MUST be cryptographically random and unique for EVERY signature
//! - Reusing k values completely compromises the private key!
//! - Use hardware RNG for k generation
//! - Private keys should be stored securely

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::pka::{EccPoint, EcdsaCurveParams, EcdsaPublicKey, EcdsaSignature, Pka};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32::rng::Rng;
use embassy_stm32::{Config, bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PKA => embassy_stm32::pka::InterruptHandler<peripherals::PKA>;
    RNG => embassy_stm32::rng::InterruptHandler<peripherals::RNG>;
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
    // RNG requires HSI clock source on WBA
    config.rcc.mux.rngsel = mux::Rngsel::HSI;

    let p = embassy_stm32::init(config);
    info!("PKA ECDSA Signature Generation Example");

    let mut pka = Pka::new_blocking(p.PKA, Irqs);
    let mut rng = Rng::new(p.RNG, Irqs);

    // Use NIST P-256 curve parameters
    let curve = EcdsaCurveParams::nist_p256();

    // Test private key (32 bytes, big-endian)
    // In a real application, this would be securely stored
    // WARNING: Never use this key in production!
    let private_key: [u8; 32] = [
        0xc9, 0xaf, 0xa9, 0xd8, 0x45, 0xba, 0x75, 0x16, 0x6b, 0x5c, 0x21, 0x57, 0x67, 0xb1, 0xd6, 0x93, 0x4e, 0x50,
        0xc3, 0xdb, 0x36, 0xe8, 0x9b, 0x12, 0x7b, 0x8a, 0x62, 0x2b, 0x12, 0x0f, 0x67, 0x21,
    ];

    // Corresponding public key (derived from private key)
    let pub_key_x: [u8; 32] = [
        0x60, 0xfe, 0xd4, 0xba, 0x25, 0x5a, 0x9d, 0x31, 0xc9, 0x61, 0xeb, 0x74, 0xc6, 0x35, 0x6d, 0x68, 0xc0, 0x49,
        0xb8, 0x92, 0x3b, 0x61, 0xfa, 0x6c, 0xe6, 0x69, 0x62, 0x2e, 0x60, 0xf2, 0x9f, 0xb6,
    ];
    let pub_key_y: [u8; 32] = [
        0x79, 0x03, 0xfe, 0x10, 0x08, 0xb8, 0xbc, 0x99, 0xa4, 0x1a, 0xe9, 0xe9, 0x56, 0x28, 0xbc, 0x64, 0xf2, 0xf1,
        0xb2, 0x0c, 0x2d, 0x7e, 0x9f, 0x51, 0x77, 0xa3, 0xc2, 0x94, 0xd4, 0x46, 0x22, 0x99,
    ];

    // Message hash (SHA-256 of the message)
    // In a real application, you would compute this from the message
    let message_hash: [u8; 32] = [
        0xaf, 0x2b, 0xdb, 0xe1, 0xaa, 0x9b, 0x6e, 0xc1, 0xe2, 0xad, 0xe1, 0xd6, 0x94, 0xf4, 0x1f, 0xc7, 0x1a, 0x83,
        0x1d, 0x02, 0x68, 0xe9, 0x89, 0x15, 0x62, 0x11, 0x3d, 0x8a, 0x62, 0xad, 0xd1, 0xbf,
    ];

    info!("=== ECDSA Signature Generation ===");
    info!("Curve: NIST P-256 (secp256r1)");
    info!("Message Hash: {:02x}", message_hash);

    // Generate random k value using hardware RNG
    // CRITICAL: k must be random and unique for every signature!
    let mut k = [0u8; 32];
    if let Err(e) = rng.async_fill_bytes(&mut k).await {
        error!("Failed to generate random k: {:?}", e);
        loop {
            cortex_m::asm::wfi();
        }
    }

    // Ensure k is in valid range (1 < k < n)
    // For simplicity, we set the MSB to ensure it's less than n
    k[0] &= 0x7F;
    // Ensure k is not zero
    k[31] |= 0x01;

    info!("Random k:     {:02x}", k);

    // Generate signature
    let mut sig_r = [0u8; 32];
    let mut sig_s = [0u8; 32];

    info!("Signing message...");
    match pka.ecdsa_sign(&curve, &private_key, &k, &message_hash, &mut sig_r, &mut sig_s) {
        Ok(()) => {
            info!("Signature generated successfully!");
            info!("Signature R: {:02x}", sig_r);
            info!("Signature S: {:02x}", sig_s);
        }
        Err(e) => {
            error!("Signing failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Verify the signature we just generated
    info!("=== Verifying Generated Signature ===");

    let public_key = EcdsaPublicKey {
        x: &pub_key_x,
        y: &pub_key_y,
    };

    let signature = EcdsaSignature { r: &sig_r, s: &sig_s };

    match pka.ecdsa_verify(&curve, &public_key, &signature, &message_hash) {
        Ok(true) => {
            info!("Generated signature verified successfully!");
        }
        Ok(false) => {
            error!("Generated signature verification FAILED!");
        }
        Err(e) => {
            error!("Verification error: {:?}", e);
        }
    }

    // Demonstrate public key derivation from private key using scalar multiplication
    info!("=== Deriving Public Key from Private Key ===");

    // Get generator point from curve parameters
    let generator_x = curve.generator_x;
    let generator_y = curve.generator_y;

    let mut derived_pub = EccPoint::new(32);
    match pka.ecc_mul(&curve, &private_key, generator_x, generator_y, &mut derived_pub) {
        Ok(()) => {
            info!("Public key derived from private key:");
            info!("Derived X:  {:02x}", derived_pub.x[..32]);
            info!("Expected X: {:02x}", pub_key_x);
            info!("Derived Y:  {:02x}", derived_pub.y[..32]);
            info!("Expected Y: {:02x}", pub_key_y);

            // Compare derived with expected
            if derived_pub.x[..32] == pub_key_x && derived_pub.y[..32] == pub_key_y {
                info!("Derived public key matches expected!");
            } else {
                warn!("Derived public key does not match!");
            }
        }
        Err(e) => {
            error!("Public key derivation failed: {:?}", e);
        }
    }

    info!("=== ECDSA signing example complete ===");

    loop {
        cortex_m::asm::wfi();
    }
}
