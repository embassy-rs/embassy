// required-features: crypto, aes256
#![no_std]
#![no_main]

//! AES-256, and GCM with both key sizes, through the direct API, against the shared vectors.
//! See `aes_common.rs`.

#[path = "../aes_common.rs"]
mod aes_common;
#[path = "../common.rs"]
mod common;

use aes_common::*;
use defmt::info;
use defmt_rtt as _;
use embassy_crypto_test::vectors;
use embassy_executor::Spawner;
use embassy_nrf::crypto::symmetric::Symmetric;
use panic_probe as _;

teleprobe_meta::timeout!(120);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut aes = Symmetric::new_blocking(p.CRYPTO_SYMMETRIC);

    test_ecb(&mut aes, &vectors::AES_ECB_256);
    test_cbc(&mut aes, &vectors::AES_CBC_256);
    test_ctr(&mut aes, &vectors::AES_CTR_256);
    test_cmac(&mut aes, &vectors::AES_CMAC_256);
    test_ccm(&mut aes, &vectors::AES_CCM_256);
    test_gcm(&mut aes, &vectors::AES_GCM_128);
    test_gcm(&mut aes, &vectors::AES_GCM_256);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
