//! PKA ECDH Key Agreement Example
//!
//! Demonstrates Elliptic Curve Diffie-Hellman (ECDH) key agreement using the
//! PKA hardware accelerator.
//!
//! # What This Example Shows
//! - Generating key pairs (private + public key)
//! - Computing shared secrets between two parties
//! - Verifying that both parties derive the same shared secret
//! - Point validation for security
//!
//! # ECDH Key Agreement Process
//! 1. Alice generates private key (a) and public key (A = a*G)
//! 2. Bob generates private key (b) and public key (B = b*G)
//! 3. Alice computes shared secret: S = a * B = a * b * G
//! 4. Bob computes shared secret: S = b * A = b * a * G
//! 5. Both parties now have the same shared secret S
//!
//! # Security Notes
//! - Always validate received public keys (use point_check)
//! - Use the x-coordinate of the shared point as the shared secret
//! - Derive session keys from the shared secret using a KDF (HKDF, SHA-256)
//! - Private keys must be randomly generated and kept secret

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::pka::{EccPoint, EcdsaCurveParams, Pka};
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
    info!("PKA ECDH Key Agreement Example");

    let mut pka = Pka::new_blocking(p.PKA, Irqs);
    let mut rng = Rng::new(p.RNG, Irqs);

    // Use NIST P-256 curve parameters
    let curve = EcdsaCurveParams::nist_p256();

    // ========== Generate Alice's Key Pair ==========
    info!("=== Generating Alice's Key Pair ===");

    // Generate Alice's private key
    let mut alice_private = [0u8; 32];
    if let Err(e) = rng.async_fill_bytes(&mut alice_private).await {
        error!("Failed to generate Alice's private key: {:?}", e);
        loop {
            cortex_m::asm::wfi();
        }
    }
    // Ensure private key is in valid range (1 < d < n)
    alice_private[0] &= 0x7F;
    alice_private[31] |= 0x01;

    info!("Alice private key: {:02x}", alice_private);

    // Compute Alice's public key: A = alice_private * G
    let mut alice_public = EccPoint::new(32);
    match pka.ecc_mul(
        &curve,
        &alice_private,
        curve.generator_x,
        curve.generator_y,
        &mut alice_public,
    ) {
        Ok(()) => {
            info!("Alice public key X: {:02x}", alice_public.x[..32]);
            info!("Alice public key Y: {:02x}", alice_public.y[..32]);
        }
        Err(e) => {
            error!("Failed to generate Alice's public key: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // ========== Generate Bob's Key Pair ==========
    info!("=== Generating Bob's Key Pair ===");

    // Generate Bob's private key
    let mut bob_private = [0u8; 32];
    if let Err(e) = rng.async_fill_bytes(&mut bob_private).await {
        error!("Failed to generate Bob's private key: {:?}", e);
        loop {
            cortex_m::asm::wfi();
        }
    }
    // Ensure private key is in valid range
    bob_private[0] &= 0x7F;
    bob_private[31] |= 0x01;

    info!("Bob private key: {:02x}", bob_private);

    // Compute Bob's public key: B = bob_private * G
    let mut bob_public = EccPoint::new(32);
    match pka.ecc_mul(
        &curve,
        &bob_private,
        curve.generator_x,
        curve.generator_y,
        &mut bob_public,
    ) {
        Ok(()) => {
            info!("Bob public key X: {:02x}", bob_public.x[..32]);
            info!("Bob public key Y: {:02x}", bob_public.y[..32]);
        }
        Err(e) => {
            error!("Failed to generate Bob's public key: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // ========== Validate Public Keys ==========
    info!("=== Validating Public Keys ===");

    // Alice validates Bob's public key
    match pka.point_check(&curve, &bob_public.x[..32], &bob_public.y[..32]) {
        Ok(true) => {
            info!("Bob's public key is valid (on curve)");
        }
        Ok(false) => {
            error!("Bob's public key is INVALID (not on curve)!");
            loop {
                cortex_m::asm::wfi();
            }
        }
        Err(e) => {
            error!("Point check failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Bob validates Alice's public key
    match pka.point_check(&curve, &alice_public.x[..32], &alice_public.y[..32]) {
        Ok(true) => {
            info!("Alice's public key is valid (on curve)");
        }
        Ok(false) => {
            error!("Alice's public key is INVALID (not on curve)!");
            loop {
                cortex_m::asm::wfi();
            }
        }
        Err(e) => {
            error!("Point check failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // ========== Compute Shared Secrets ==========
    info!("=== Computing Shared Secrets ===");

    // Alice computes shared secret: S_alice = alice_private * bob_public
    let mut alice_shared = EccPoint::new(32);
    match pka.ecc_mul(
        &curve,
        &alice_private,
        &bob_public.x[..32],
        &bob_public.y[..32],
        &mut alice_shared,
    ) {
        Ok(()) => {
            info!("Alice's shared secret X: {:02x}", alice_shared.x[..32]);
        }
        Err(e) => {
            error!("Alice failed to compute shared secret: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Bob computes shared secret: S_bob = bob_private * alice_public
    let mut bob_shared = EccPoint::new(32);
    match pka.ecc_mul(
        &curve,
        &bob_private,
        &alice_public.x[..32],
        &alice_public.y[..32],
        &mut bob_shared,
    ) {
        Ok(()) => {
            info!("Bob's shared secret X: {:02x}", bob_shared.x[..32]);
        }
        Err(e) => {
            error!("Bob failed to compute shared secret: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // ========== Verify Shared Secrets Match ==========
    info!("=== Verifying Key Agreement ===");

    if alice_shared.x[..32] == bob_shared.x[..32] && alice_shared.y[..32] == bob_shared.y[..32] {
        info!("SUCCESS: Both parties derived the SAME shared secret!");
        info!("Shared secret (x-coord): {:02x}", alice_shared.x[..32]);
        info!("");
        info!("This shared secret can now be used to derive:");
        info!("- AES encryption keys (using SHA-256 or HKDF)");
        info!("- HMAC authentication keys");
        info!("- Session keys for secure communication");
    } else {
        error!("FAILURE: Shared secrets do not match!");
        error!("Alice X: {:02x}", alice_shared.x[..32]);
        error!("Bob X:   {:02x}", bob_shared.x[..32]);
    }

    // ========== Example with Pre-defined Test Vectors ==========
    info!("=== NIST ECDH Test Vector ===");

    // NIST CAVP test vector for P-256 ECDH
    let test_private: [u8; 32] = [
        0xc9, 0xaf, 0xa9, 0xd8, 0x45, 0xba, 0x75, 0x16, 0x6b, 0x5c, 0x21, 0x57, 0x67, 0xb1, 0xd6, 0x93, 0x4e, 0x50,
        0xc3, 0xdb, 0x36, 0xe8, 0x9b, 0x12, 0x7b, 0x8a, 0x62, 0x2b, 0x12, 0x0f, 0x67, 0x21,
    ];

    let test_peer_pub_x: [u8; 32] = [
        0x70, 0x04, 0x0a, 0xcd, 0x89, 0x8e, 0xb2, 0x3d, 0xfa, 0x85, 0x9a, 0x16, 0x53, 0x31, 0x9c, 0xa8, 0xd1, 0xb0,
        0x81, 0xf6, 0x0f, 0x3e, 0x05, 0x97, 0xc7, 0xfd, 0xd6, 0x29, 0x32, 0x4b, 0xe6, 0x2c,
    ];

    let test_peer_pub_y: [u8; 32] = [
        0x5f, 0x67, 0x94, 0x7f, 0x9c, 0x63, 0x8f, 0x63, 0xd7, 0xba, 0x35, 0x73, 0xb8, 0xbd, 0xb5, 0x5a, 0x83, 0x62,
        0xb3, 0x9c, 0x23, 0x4e, 0x7d, 0x36, 0x7f, 0xc1, 0xd5, 0xcd, 0x8c, 0x82, 0xc9, 0x25,
    ];

    info!("Computing shared secret with test vector...");
    let mut test_shared = EccPoint::new(32);
    match pka.ecc_mul(
        &curve,
        &test_private,
        &test_peer_pub_x,
        &test_peer_pub_y,
        &mut test_shared,
    ) {
        Ok(()) => {
            info!("Test shared secret X: {:02x}", test_shared.x[..32]);
            info!("Test shared secret Y: {:02x}", test_shared.y[..32]);
        }
        Err(e) => {
            error!("Test vector computation failed: {:?}", e);
        }
    }

    info!("=== ECDH key agreement example complete ===");

    loop {
        cortex_m::asm::wfi();
    }
}
