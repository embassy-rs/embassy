//! PKA ECDSA Signature Verification Example
//!
//! Demonstrates ECDSA signature verification using the PKA hardware accelerator.
//!
//! # What This Example Shows
//! - Loading NIST P-256 curve parameters
//! - Verifying an ECDSA signature over a message hash
//! - Using NIST test vectors for validation
//!
//! # ECDSA Verification Process
//! The PKA hardware verifies that the signature (r, s) is valid for the given
//! message hash and public key on the specified elliptic curve.
//!
//! # Security Notes
//! - Always validate public keys before use (point_check)
//! - Hash the message before verification (PKA expects the hash, not raw data)
//! - Use approved hash functions (SHA-256 for P-256)

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::pka::{EcdsaCurveParams, EcdsaPublicKey, EcdsaSignature, Pka};
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
    // RNG clock is required for PKA to initialize properly on WBA
    config.rcc.mux.rngsel = mux::Rngsel::HSI;

    let p = embassy_stm32::init(config);
    info!("PKA ECDSA Signature Verification Example");

    // Initialize RNG to enable its clock - PKA requires RNG clock on AHB5
    let _rng = Rng::new(p.RNG, Irqs);

    let mut pka = Pka::new_blocking(p.PKA, Irqs);

    // Use NIST P-256 curve parameters
    let curve = EcdsaCurveParams::nist_p256();

    // NIST CAVP test vector for P-256 ECDSA verification
    // From NIST CAVP SigVer.rsp [P-256,SHA-256]
    // This test vector is verified to work with the ST-HAL

    // Public key Qx, Qy (big-endian)
    let pub_key_x: [u8; 32] = [
        0xe4, 0x24, 0xdc, 0x61, 0xd4, 0xbb, 0x3c, 0xb7, 0xef, 0x43, 0x44, 0xa7, 0xf8, 0x95, 0x7a, 0x0c, 0x51, 0x34,
        0xe1, 0x6f, 0x7a, 0x67, 0xc0, 0x74, 0xf8, 0x2e, 0x6e, 0x12, 0xf4, 0x9a, 0xbf, 0x3c,
    ];
    let pub_key_y: [u8; 32] = [
        0x97, 0x0e, 0xed, 0x7a, 0xa2, 0xbc, 0x48, 0x65, 0x15, 0x45, 0x94, 0x9d, 0xe1, 0xdd, 0xda, 0xf0, 0x12, 0x7e,
        0x59, 0x65, 0xac, 0x85, 0xd1, 0x24, 0x3d, 0x6f, 0x60, 0xe7, 0xdf, 0xae, 0xe9, 0x27,
    ];

    // SHA-256 hash of the test message (pre-computed)
    let message_hash: [u8; 32] = [
        0xd1, 0xb8, 0xef, 0x21, 0xeb, 0x41, 0x82, 0xee, 0x27, 0x06, 0x38, 0x06, 0x10, 0x63, 0xa3, 0xf3, 0xc1, 0x6c,
        0x11, 0x4e, 0x33, 0x93, 0x7f, 0x69, 0xfb, 0x23, 0x2c, 0xc8, 0x33, 0x96, 0x5a, 0x94,
    ];

    // ECDSA signature (r, s) - valid signature for the above hash and public key
    let sig_r: [u8; 32] = [
        0xbf, 0x96, 0xb9, 0x9a, 0xa4, 0x9c, 0x70, 0x5c, 0x91, 0x0b, 0xe3, 0x31, 0x42, 0x01, 0x7c, 0x64, 0x2f, 0xf5,
        0x40, 0xc7, 0x63, 0x49, 0xb9, 0xda, 0xb7, 0x2f, 0x98, 0x1f, 0xd9, 0x34, 0x7f, 0x4f,
    ];
    let sig_s: [u8; 32] = [
        0x17, 0xc5, 0x50, 0x95, 0x81, 0x90, 0x89, 0xc2, 0xe0, 0x3b, 0x9c, 0xd4, 0x15, 0xab, 0xdf, 0x12, 0x44, 0x4e,
        0x32, 0x30, 0x75, 0xd9, 0x8f, 0x31, 0x92, 0x0b, 0x9e, 0x0f, 0x57, 0xec, 0x87, 0x1c,
    ];

    info!("=== ECDSA Signature Verification ===");
    info!("Curve: NIST P-256 (secp256r1)");
    info!("Public Key X: {:02x}", pub_key_x);
    info!("Public Key Y: {:02x}", pub_key_y);
    info!("Message Hash: {:02x}", message_hash);
    info!("Signature R:  {:02x}", sig_r);
    info!("Signature S:  {:02x}", sig_s);

    let public_key = EcdsaPublicKey {
        x: &pub_key_x,
        y: &pub_key_y,
    };

    let signature = EcdsaSignature { r: &sig_r, s: &sig_s };

    // Verify the signature
    info!("Verifying signature...");
    match pka.ecdsa_verify(&curve, &public_key, &signature, &message_hash) {
        Ok(true) => {
            info!("Signature is VALID");
        }
        Ok(false) => {
            info!("Signature is INVALID");
        }
        Err(e) => {
            error!("Verification failed with error: {:?}", e);
        }
    }

    // Test with a tampered signature (modify one byte)
    info!("=== Testing with tampered signature ===");
    let mut tampered_sig_r = sig_r;
    tampered_sig_r[0] ^= 0x01; // Flip one bit

    let tampered_signature = EcdsaSignature {
        r: &tampered_sig_r,
        s: &sig_s,
    };

    match pka.ecdsa_verify(&curve, &public_key, &tampered_signature, &message_hash) {
        Ok(true) => {
            error!("Tampered signature incorrectly verified as VALID!");
        }
        Ok(false) => {
            info!("Tampered signature correctly detected as INVALID");
        }
        Err(e) => {
            error!("Verification failed with error: {:?}", e);
        }
    }

    info!("=== ECDSA verification example complete ===");

    loop {
        cortex_m::asm::wfi();
    }
}
