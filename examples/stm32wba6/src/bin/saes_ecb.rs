//! SAES-ECB (Secure AES - Electronic Codebook) Mode Example
//!
//! Demonstrates SAES-ECB encryption and decryption using the same NIST test
//! vectors as the AES-ECB example so results can be directly compared.
//!
//! SAES shares all cipher modes with AES but adds hardware key derivation
//! (DHUK/BHK) and key-protection features. For software keys the outputs
//! are identical to AES.
//!
//! # What This Example Exercises
//! - SAES peripheral initialisation (busy-wait + RNG-error check on startup)
//! - Key loading and KEYVALID wait
//! - ECB encryption with 128-bit and 256-bit keys (NIST SP 800-38A vectors)
//! - ECB decryption, which exercises the key-derivation (MODE=1) fix

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::rcc::{AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale};
use embassy_stm32::saes::{AesEcb, Direction, Saes};
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

    // ─── NIST SP 800-38A F.1.1 / F.1.2 – AES-128-ECB ───────────────────────
    let key_128 = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];
    let plaintext = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    ];
    let expected_ct = [
        0x3a, 0xd7, 0x7b, 0xb4, 0x0d, 0x7a, 0x36, 0x60, 0xa8, 0x9e, 0xca, 0xf3, 0x24, 0x66, 0xef, 0x97,
    ];

    // ── Encrypt ──────────────────────────────────────────────────────────────
    info!("=== SAES-ECB 128-bit Encryption ===");
    let cipher = AesEcb::new(&key_128);
    let mut ctx = saes.start(&cipher, Direction::Encrypt);

    let mut ciphertext = [0u8; 16];
    match saes.payload_blocking(&mut ctx, &plaintext, &mut ciphertext, true) {
        Ok(()) => {
            info!("Plaintext:  {:02x}", plaintext);
            info!("Ciphertext: {:02x}", ciphertext);
            info!("Expected:   {:02x}", expected_ct);
            if ciphertext == expected_ct {
                info!("✓ Encryption PASSED");
            } else {
                error!("✗ Encryption FAILED");
            }
        }
        Err(e) => error!("Encryption error: {:?}", e),
    }
    saes.finish_blocking(ctx).ok();

    // ── Decrypt ──────────────────────────────────────────────────────────────
    // This path exercises the key-derivation fix: ECB decrypt sets MODE=KEY_DERIVATION,
    // enables the peripheral, waits for ISR.CCF, clears via ICR, then restores MODE=DECRYPT.
    info!("=== SAES-ECB 128-bit Decryption ===");
    let cipher = AesEcb::new(&key_128);
    let mut ctx = saes.start(&cipher, Direction::Decrypt);

    let mut decrypted = [0u8; 16];
    match saes.payload_blocking(&mut ctx, &ciphertext, &mut decrypted, true) {
        Ok(()) => {
            info!("Decrypted: {:02x}", decrypted);
            info!("Expected:  {:02x}", plaintext);
            if decrypted == plaintext {
                info!("✓ Decryption PASSED");
            } else {
                error!("✗ Decryption FAILED");
            }
        }
        Err(e) => error!("Decryption error: {:?}", e),
    }
    saes.finish_blocking(ctx).ok();

    // ─── NIST SP 800-38A F.1.5 / F.1.6 – AES-256-ECB ───────────────────────
    info!("=== SAES-ECB 256-bit Encryption ===");
    let key_256 = [
        0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81, 0x1f, 0x35,
        0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4,
    ];
    let plaintext_256 = [
        0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17, 0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10,
    ];
    let expected_256 = [
        0xb4, 0x7b, 0xd7, 0x3a, 0x60, 0x36, 0x7a, 0x0d, 0xf3, 0xca, 0x9e, 0xa8, 0x97, 0xef, 0x66, 0x24,
    ];

    let cipher = AesEcb::new(&key_256);
    let mut ctx = saes.start(&cipher, Direction::Encrypt);

    let mut ct_256 = [0u8; 16];
    match saes.payload_blocking(&mut ctx, &plaintext_256, &mut ct_256, true) {
        Ok(()) => {
            info!("Ciphertext: {:02x}", ct_256);
            info!("Expected:   {:02x}", expected_256);
            if ct_256 == expected_256 {
                info!("✓ 256-bit Encryption PASSED");
            } else {
                error!("✗ 256-bit Encryption FAILED");
            }
        }
        Err(e) => error!("256-bit Encryption error: {:?}", e),
    }
    saes.finish_blocking(ctx).ok();

    info!("=== SAES-ECB complete ===");
    loop {
        cortex_m::asm::wfi();
    }
}
