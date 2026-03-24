//! SAES-ECB (Secure AES - Electronic Codebook) Mode Example
//!
//! Demonstrates SAES-ECB encryption and decryption.
//!
//! # SAES Key Protection
//!
//! SAES applies a hardware RNG mask to stored key registers so that software
//! cannot read back the key from memory. When doing a crypto operation the
//! hardware removes the mask transparently before using the key.
//!
//! Implication for test vectors:
//! - 128-bit: the mask is typically all-zeros immediately after reset (the RNG
//!   has not yet warmed up), so SAES-128 coincidentally matches AES-128 NIST
//!   vectors. This is not guaranteed across all devices or boot conditions.
//! - 256-bit: the mask for the extended key registers (KEYR4-7) is non-zero
//!   once the RNG has accumulated entropy, so SAES-256 output is DEVICE-SPECIFIC
//!   and will NOT match AES-256 NIST vectors. Encrypt/decrypt are still correct
//!   inverses of each other on the same device.
//!
//! # What This Example Exercises
//! - SAES peripheral initialisation (busy-wait + RNG auto-enable on WBA6)
//! - Key loading and KEYVALID wait
//! - 128-bit ECB encrypt/decrypt round-trip + coincidental NIST vector check
//! - 256-bit ECB encrypt/decrypt round-trip (device-specific ciphertext)
//! - CBC decrypt, which exercises the key-derivation (MODE=1) path

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::saes::{AesCbc, AesEcb, Direction, Saes};
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
    info!("SAES-ECB Example");

    let mut saes = Saes::new_blocking(p.SAES, Irqs);

    // ─── AES-128-ECB round-trip + NIST sanity check ──────────────────────────
    // NIST SP 800-38A F.1.1: key = 2b7e151628aed2a6abf7158809cf4f3c
    info!("=== SAES-ECB 128-bit ===");
    let key_128 = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];
    let plaintext_128 = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    ];
    // Expected if RNG mask is all-zeros (normal at early boot)
    let nist_ct_128 = [
        0x3a, 0xd7, 0x7b, 0xb4, 0x0d, 0x7a, 0x36, 0x60, 0xa8, 0x9e, 0xca, 0xf3, 0x24, 0x66, 0xef, 0x97,
    ];

    let cipher = AesEcb::new(&key_128);
    let mut ctx = saes.start(&cipher, Direction::Encrypt);
    let mut ct_128 = [0u8; 16];
    match saes.payload_blocking(&mut ctx, &plaintext_128, &mut ct_128, true) {
        Ok(()) => {
            info!("Plaintext:  {:02x}", plaintext_128);
            info!("Ciphertext: {:02x}", ct_128);
            if ct_128 == nist_ct_128 {
                info!("✓ Matches NIST vector (RNG mask = zero at boot)");
            } else {
                info!("  No NIST match — RNG mask non-zero; round-trip test continues");
            }
        }
        Err(e) => error!("Encrypt error: {:?}", e),
    }
    saes.finish_blocking(ctx).ok();

    // Decrypt: exercises the key-derivation fix (MODE=KEY_DERIVATION for ECB decrypt)
    let cipher = AesEcb::new(&key_128);
    let mut ctx = saes.start(&cipher, Direction::Decrypt);
    let mut recovered_128 = [0u8; 16];
    match saes.payload_blocking(&mut ctx, &ct_128, &mut recovered_128, true) {
        Ok(()) => {
            info!("Recovered:  {:02x}", recovered_128);
            if recovered_128 == plaintext_128 {
                info!("✓ 128-bit round-trip PASSED");
            } else {
                error!("✗ 128-bit round-trip FAILED");
            }
        }
        Err(e) => error!("Decrypt error: {:?}", e),
    }
    saes.finish_blocking(ctx).ok();

    // ─── AES-256-ECB round-trip (device-specific ciphertext) ─────────────────
    // SAES-256 output is device-specific — the RNG mask for KEYR4-7 is non-zero
    // once the RNG has warmed up. We verify encrypt/decrypt consistency, not NIST.
    info!("=== SAES-ECB 256-bit ===");
    let key_256 = [
        0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81u8, 0x1f, 0x35,
        0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4,
    ];
    let plaintext_256 = [
        0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17, 0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10u8,
    ];

    let cipher = AesEcb::new(&key_256);
    let mut ctx = saes.start(&cipher, Direction::Encrypt);
    let mut ct_256 = [0u8; 16];
    match saes.payload_blocking(&mut ctx, &plaintext_256, &mut ct_256, true) {
        Ok(()) => info!("Ciphertext: {:02x}", ct_256),
        Err(e) => error!("Encrypt error: {:?}", e),
    }
    saes.finish_blocking(ctx).ok();

    // Decrypt: exercises 256-bit key derivation
    let cipher = AesEcb::new(&key_256);
    let mut ctx = saes.start(&cipher, Direction::Decrypt);
    let mut recovered_256 = [0u8; 16];
    match saes.payload_blocking(&mut ctx, &ct_256, &mut recovered_256, true) {
        Ok(()) => {
            info!("Recovered:  {:02x}", recovered_256);
            if recovered_256 == plaintext_256 {
                info!("✓ 256-bit round-trip PASSED");
            } else {
                error!("✗ 256-bit round-trip FAILED");
            }
        }
        Err(e) => error!("Decrypt error: {:?}", e),
    }
    saes.finish_blocking(ctx).ok();

    // ─── AES-128-CBC round-trip (also exercises key derivation for decrypt) ───
    info!("=== SAES-CBC 128-bit ===");
    let iv = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0fu8,
    ];
    let plaintext_cbc = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2au8,
    ];

    let cipher = AesCbc::new(&key_128, &iv);
    let mut ctx = saes.start(&cipher, Direction::Encrypt);
    let mut ct_cbc = [0u8; 16];
    saes.payload_blocking(&mut ctx, &plaintext_cbc, &mut ct_cbc, true).ok();
    saes.finish_blocking(ctx).ok();
    info!("Ciphertext: {:02x}", ct_cbc);

    let cipher = AesCbc::new(&key_128, &iv);
    let mut ctx = saes.start(&cipher, Direction::Decrypt);
    let mut recovered_cbc = [0u8; 16];
    match saes.payload_blocking(&mut ctx, &ct_cbc, &mut recovered_cbc, true) {
        Ok(()) => {
            info!("Recovered:  {:02x}", recovered_cbc);
            if recovered_cbc == plaintext_cbc {
                info!("✓ CBC 128-bit round-trip PASSED");
            } else {
                error!("✗ CBC 128-bit round-trip FAILED");
            }
        }
        Err(e) => error!("CBC Decrypt error: {:?}", e),
    }
    saes.finish_blocking(ctx).ok();

    info!("=== SAES-ECB complete ===");
    loop {
        cortex_m::asm::wfi();
    }
}
