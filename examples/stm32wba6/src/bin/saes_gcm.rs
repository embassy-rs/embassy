//! SAES-GCM (Secure AES - Galois/Counter Mode) Example
//!
//! # Known Limitation: GCM with AAD on SAES v1a (WBA6x)
//!
//! SAES GCM without AAD produces consistent encrypt/decrypt authentication tags
//! and is tested here. However, when AAD (Additional Authenticated Data) is
//! provided, encrypt and decrypt produce different authentication tags on
//! SAES v1a devices (STM32WBA6x). The plaintext is recovered correctly but
//! the AEAD authentication property does not hold.
//!
//! **If you need authenticated GCM with AAD, use the plain AES peripheral.**
//!
//! The root cause appears to be a SAES v1a hardware behaviour in the GCM
//! header phase that has not been documented by ST and cannot be worked around
//! in the driver.
//!
//! # What This Example Exercises
//! - SAES peripheral initialisation (busy-wait + RNG auto-enable on WBA6)
//! - GCM mode detection via chmod_bits() (the IV-length heuristic fix)
//! - GCMPH sequencing: init (0) → payload (2) → final (3) (no AAD)
//! - CCF polling via ISR register
//! - No-AAD GCM round-trip: encrypt tag == decrypt tag ✓

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::saes::{AesGcm, Direction, Saes};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SAES => embassy_stm32::saes::InterruptHandler<peripherals::SAES>;
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
    info!("SAES-GCM Example");

    let mut saes = Saes::new_blocking(p.SAES, Irqs);

    // ─── GCM round-trip: no AAD, 16-byte plaintext ───────────────────────────
    // Uses zero key, IV, and plaintext for a clean baseline test.
    // Ciphertext and tag are device-specific (SAES key mask is non-zero).
    info!("=== SAES-GCM 128-bit (no AAD) ===");
    let key = [0u8; 16];
    let iv = [0u8; 12];
    let pt = [0u8; 16];

    let cipher = AesGcm::new(&key, &iv);
    let mut ctx = saes.start(&cipher, Direction::Encrypt);
    let mut ct = [0u8; 16];
    match saes.payload_blocking(&mut ctx, &pt, &mut ct, true) {
        Ok(()) => info!("Ciphertext: {:02x}", ct),
        Err(e) => {
            error!("✗ Encrypt failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }
    let enc_tag = match saes.finish_blocking(ctx) {
        Ok(Some(tag)) => {
            info!("Encrypt tag: {:02x}", tag);
            tag
        }
        _ => {
            error!("✗ No tag");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let cipher = AesGcm::new(&key, &iv);
    let mut ctx = saes.start(&cipher, Direction::Decrypt);
    let mut recovered = [0u8; 16];
    match saes.payload_blocking(&mut ctx, &ct, &mut recovered, true) {
        Ok(()) => info!("Recovered:  {:02x}", recovered),
        Err(e) => error!("✗ Decrypt failed: {:?}", e),
    }
    match saes.finish_blocking(ctx) {
        Ok(Some(dec_tag)) => {
            info!("Decrypt tag: {:02x}", dec_tag);
            if recovered == pt && dec_tag == enc_tag {
                info!("✓ GCM no-AAD round-trip PASSED (plaintext + tag both match)");
            } else {
                if recovered != pt {
                    error!("✗ Plaintext mismatch");
                }
                if dec_tag != enc_tag {
                    error!("✗ Tag mismatch");
                }
            }
        }
        Ok(None) => error!("✗ No tag returned"),
        Err(e) => error!("✗ Finish failed: {:?}", e),
    }

    // ─── GCM round-trip: larger plaintext, no AAD ────────────────────────────
    info!("=== SAES-GCM 128-bit (no AAD, 60-byte PT) ===");
    let key2 = [
        0xfe, 0xff, 0xe9, 0x92, 0x86, 0x65, 0x73, 0x1c, 0x6d, 0x6a, 0x8f, 0x94, 0x67, 0x30, 0x83, 0x08u8,
    ];
    let iv2 = [0xca, 0xfe, 0xba, 0xbe, 0xfa, 0xce, 0xdb, 0xad, 0xde, 0xca, 0xf8, 0x88u8];
    let pt2 = [
        0xd9, 0x31, 0x32, 0x25, 0xf8, 0x84, 0x06, 0xe5, 0xa5, 0x59, 0x09, 0xc5, 0xaf, 0xf5, 0x26, 0x9a, 0x86, 0xa7,
        0xa9, 0x53, 0x15, 0x34, 0xf7, 0xda, 0x2e, 0x4c, 0x30, 0x3d, 0x8a, 0x31, 0x8a, 0x72, 0x1c, 0x3c, 0x0c, 0x95,
        0x95, 0x68, 0x09, 0x53, 0x2f, 0xcf, 0x0e, 0x24, 0x49, 0xa6, 0xb5, 0x25, 0xb1, 0x6a, 0xed, 0xf5, 0xaa, 0x0d,
        0xe6, 0x57, 0xba, 0x63, 0x7b, 0x39u8,
    ];

    let cipher = AesGcm::new(&key2, &iv2);
    let mut ctx = saes.start(&cipher, Direction::Encrypt);
    let mut ct2 = [0u8; 60];
    saes.payload_blocking(&mut ctx, &pt2, &mut ct2, true).ok();
    let enc_tag2 = match saes.finish_blocking(ctx) {
        Ok(Some(tag)) => {
            info!("Encrypt tag: {:02x}", tag);
            tag
        }
        _ => {
            error!("✗ Encrypt failed");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let cipher = AesGcm::new(&key2, &iv2);
    let mut ctx = saes.start(&cipher, Direction::Decrypt);
    let mut recovered2 = [0u8; 60];
    saes.payload_blocking(&mut ctx, &ct2, &mut recovered2, true).ok();
    match saes.finish_blocking(ctx) {
        Ok(Some(dec_tag)) => {
            if recovered2 == pt2 && dec_tag == enc_tag2 {
                info!("✓ GCM 60-byte no-AAD round-trip PASSED");
            } else {
                if recovered2 != pt2 {
                    error!("✗ Plaintext mismatch");
                }
                if dec_tag != enc_tag2 {
                    error!("✗ Tag mismatch");
                }
            }
        }
        _ => error!("✗ Decrypt failed"),
    }

    info!("=== SAES-GCM complete ===");
    loop {
        cortex_m::asm::wfi();
    }
}
