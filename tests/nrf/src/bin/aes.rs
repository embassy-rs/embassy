// required-features: crypto
#![no_std]
#![no_main]

//! AES-128 through the direct API, against the shared vectors, plus the parameter checks of the
//! API. See `aes_common.rs`; AES-256 and GCM run in `aes256`.

#[path = "../aes_common.rs"]
mod aes_common;
#[path = "../common.rs"]
mod common;

use aes_common::*;
use defmt::{assert_eq, info, unwrap};
use defmt_rtt as _;
use embassy_crypto_test::vectors::{self, MESSAGE};
use embassy_executor::Spawner;
use embassy_nrf::crypto::symmetric::{AesCbc, AesEcb, Direction, Error, Symmetric};
use panic_probe as _;

teleprobe_meta::timeout!(120);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut aes = Symmetric::new_blocking(p.CRYPTO_SYMMETRIC);

    assert_eq!(AesEcb::new(&[0; 15]).err(), Some(Error::InvalidKeyLength));
    assert_eq!(AesEcb::new(&[0; 33]).err(), Some(Error::InvalidKeyLength));
    #[cfg(not(feature = "aes256"))]
    assert_eq!(AesEcb::new(&[0; 32]).err(), Some(Error::InvalidKeyLength));

    // Block-length errors.
    let key = &MESSAGE[..16];
    let pt = &MESSAGE[..32];
    let mut out = [0u8; 32];
    let mut ctx = aes.aes_start(unwrap!(AesEcb::new(key)), Direction::Encrypt);
    assert_eq!(
        aes.aes_blocking_payload(&mut ctx, &pt[..17], &mut out[..17], true),
        Err(Error::InvalidLength)
    );
    assert_eq!(
        aes.aes_blocking_payload(&mut ctx, &pt[..16], &mut out[..32], true),
        Err(Error::InvalidLength)
    );
    let mut ctx = aes.aes_start(unwrap!(AesCbc::new(key, &[0; 16])), Direction::Encrypt);
    assert_eq!(
        aes.aes_blocking_payload(&mut ctx, &pt[..1], &mut out[..1], true),
        Err(Error::InvalidLength)
    );

    test_ecb(&mut aes, &vectors::AES_ECB_128);
    test_cbc(&mut aes, &vectors::AES_CBC_128);
    test_ctr(&mut aes, &vectors::AES_CTR_128);
    test_cmac(&mut aes, &vectors::AES_CMAC_128);
    test_ccm(&mut aes, &vectors::AES_CCM_128);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
