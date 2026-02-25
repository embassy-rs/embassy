//! SAES-GCM (Secure AES - Galois/Counter Mode) Authenticated Encryption Example
//!
//! Demonstrates SAES-GCM using the same NIST test vectors as the AES-GCM
//! example so results can be directly compared.
//!
//! # What This Example Exercises
//! - SAES peripheral initialisation (busy-wait + RNG-error check on startup)
//! - Key loading and KEYVALID wait
//! - GCM mode detection via chmod_bits() — the IV-length heuristic would have
//!   mis-detected GCM as CBC; this verifies the fix
//! - AAD processing without spurious NPBLB writes in the header phase
//! - GCMPH sequencing: init (0) → header (1) → payload (2) → final (3)
//! - CCF polling via ISR register (not SR.busy)
//! - Authentication tag generation and verification

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::saes::{AesGcm, Direction, Saes};
use embassy_stm32::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SAES => embassy_stm32::saes::InterruptHandler<peripherals::SAES>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("SAES-GCM Authenticated Encryption Example");

    let mut saes = Saes::new_blocking(p.SAES, Irqs);

    // ─── NIST SP 800-38D Test Case 4 – 128-bit key, 60-byte PT, 20-byte AAD ─
    let key = [
        0xfe, 0xff, 0xe9, 0x92, 0x86, 0x65, 0x73, 0x1c, 0x6d, 0x6a, 0x8f, 0x94, 0x67, 0x30, 0x83, 0x08,
    ];
    let iv = [0xca, 0xfe, 0xba, 0xbe, 0xfa, 0xce, 0xdb, 0xad, 0xde, 0xca, 0xf8, 0x88u8];
    let aad = [
        0xfe, 0xed, 0xfa, 0xce, 0xde, 0xad, 0xbe, 0xef, 0xfe, 0xed, 0xfa, 0xce, 0xde, 0xad, 0xbe, 0xef, 0xab, 0xad,
        0xda, 0xd2u8,
    ];
    let plaintext = [
        0xd9, 0x31, 0x32, 0x25, 0xf8, 0x84, 0x06, 0xe5, 0xa5, 0x59, 0x09, 0xc5, 0xaf, 0xf5, 0x26, 0x9a, 0x86, 0xa7,
        0xa9, 0x53, 0x15, 0x34, 0xf7, 0xda, 0x2e, 0x4c, 0x30, 0x3d, 0x8a, 0x31, 0x8a, 0x72, 0x1c, 0x3c, 0x0c, 0x95,
        0x95, 0x68, 0x09, 0x53, 0x2f, 0xcf, 0x0e, 0x24, 0x49, 0xa6, 0xb5, 0x25, 0xb1, 0x6a, 0xed, 0xf5, 0xaa, 0x0d,
        0xe6, 0x57, 0xba, 0x63, 0x7b, 0x39u8,
    ];
    let expected_ct = [
        0x42, 0x83, 0x1e, 0xc2, 0x21, 0x77, 0x74, 0x24, 0x4b, 0x72, 0x21, 0xb7, 0x84, 0xd0, 0xd4, 0x9c, 0xe3, 0xaa,
        0x21, 0x2f, 0x2c, 0x02, 0xa4, 0xe0, 0x35, 0xc1, 0x7e, 0x23, 0x29, 0xac, 0xa1, 0x2e, 0x21, 0xd5, 0x14, 0xb2,
        0x54, 0x66, 0x93, 0x1c, 0x7d, 0x8f, 0x6a, 0x5a, 0xac, 0x84, 0xaa, 0x05, 0x1b, 0xa3, 0x0b, 0x39, 0x6a, 0x0a,
        0xac, 0x97, 0x3d, 0x58, 0xe0, 0x91u8,
    ];
    let expected_tag = [
        0x5b, 0xc9, 0x4f, 0xbc, 0x32, 0x21, 0xa5, 0xdb, 0x94, 0xfa, 0xe9, 0x5a, 0xe7, 0x12, 0x1a, 0x47u8,
    ];

    // ── Encrypt ──────────────────────────────────────────────────────────────
    info!("=== SAES-GCM Encryption (NIST TC4, AAD+60-byte PT) ===");

    let cipher = AesGcm::new(&key, &iv);
    let mut ctx = saes.start(&cipher, Direction::Encrypt);

    match saes.aad_blocking(&mut ctx, &aad, true) {
        Ok(()) => info!("✓ AAD processed"),
        Err(e) => {
            error!("✗ AAD failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    let mut ciphertext = [0u8; 60];
    match saes.payload_blocking(&mut ctx, &plaintext, &mut ciphertext, true) {
        Ok(()) => info!("✓ Payload encrypted"),
        Err(e) => {
            error!("✗ Payload failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    match saes.finish_blocking(ctx) {
        Ok(Some(tag)) => {
            let ct_ok = ciphertext == expected_ct;
            let tag_ok = tag == expected_tag;
            info!("Ciphertext matches: {}", ct_ok);
            info!("Tag:      {:02x}", tag);
            info!("Expected: {:02x}", expected_tag);
            if ct_ok && tag_ok {
                info!("✓ GCM Encryption PASSED");
            } else {
                if !ct_ok {
                    error!("✗ Ciphertext mismatch");
                }
                if !tag_ok {
                    error!("✗ Tag mismatch");
                }
            }
        }
        Ok(None) => error!("✗ No tag returned"),
        Err(e) => error!("✗ Finish failed: {:?}", e),
    }

    // ── Decrypt ──────────────────────────────────────────────────────────────
    info!("=== SAES-GCM Decryption ===");

    let cipher = AesGcm::new(&key, &iv);
    let mut ctx = saes.start(&cipher, Direction::Decrypt);

    saes.aad_blocking(&mut ctx, &aad, true).ok();

    let mut decrypted = [0u8; 60];
    match saes.payload_blocking(&mut ctx, &ciphertext, &mut decrypted, true) {
        Ok(()) => info!("✓ Payload decrypted"),
        Err(e) => error!("✗ Payload failed: {:?}", e),
    }

    match saes.finish_blocking(ctx) {
        Ok(Some(tag)) => {
            if tag == expected_tag && decrypted == plaintext {
                info!("✓ GCM Decryption + tag verification PASSED");
            } else {
                if tag != expected_tag {
                    error!("✗ Tag mismatch on decrypt");
                }
                if decrypted != plaintext {
                    error!("✗ Plaintext mismatch");
                }
            }
        }
        Ok(None) => error!("✗ No tag returned"),
        Err(e) => error!("✗ Finish failed: {:?}", e),
    }

    // ─── NIST SP 800-38D Test Case 2 – zero key/IV/PT, no AAD ───────────────
    info!("=== SAES-GCM NIST TC2 (no AAD, 16-byte PT) ===");

    let nist_key = [0u8; 16];
    let nist_iv = [0u8; 12];
    let nist_pt = [0u8; 16];
    let nist_expected_ct = [
        0x03, 0x88, 0xda, 0xce, 0x60, 0xb6, 0xa3, 0x92, 0xf3, 0x28, 0xc2, 0xb9, 0x71, 0xb2, 0xfe, 0x78u8,
    ];
    let nist_expected_tag = [
        0xab, 0x6e, 0x47, 0xd4, 0x2c, 0xec, 0x13, 0xbd, 0xf5, 0x3a, 0x67, 0xb2, 0x12, 0x57, 0xbd, 0xdfu8,
    ];

    let cipher = AesGcm::new(&nist_key, &nist_iv);
    let mut ctx = saes.start(&cipher, Direction::Encrypt);

    let mut nist_ct = [0u8; 16];
    match saes.payload_blocking(&mut ctx, &nist_pt, &mut nist_ct, true) {
        Ok(()) => {
            info!("Ciphertext: {:02x}", nist_ct);
            info!("Expected:   {:02x}", nist_expected_ct);
            if nist_ct != nist_expected_ct {
                error!("✗ NIST TC2 ciphertext mismatch");
            }
        }
        Err(e) => error!("✗ NIST TC2 payload failed: {:?}", e),
    }

    match saes.finish_blocking(ctx) {
        Ok(Some(tag)) => {
            info!("Tag:      {:02x}", tag);
            info!("Expected: {:02x}", nist_expected_tag);
            if nist_ct == nist_expected_ct && tag == nist_expected_tag {
                info!("✓ NIST TC2 PASSED");
            } else if tag != nist_expected_tag {
                error!("✗ NIST TC2 tag mismatch");
            }
        }
        Ok(None) => error!("✗ No tag returned"),
        Err(e) => error!("✗ Finish failed: {:?}", e),
    }

    info!("=== SAES-GCM complete ===");
    loop {
        cortex_m::asm::wfi();
    }
}
