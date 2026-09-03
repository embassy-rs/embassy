// required-features: crypto
#![no_std]
#![no_main]

//! Exercises the hardware accelerators through `embassy-crypto`'s RustCrypto-style types.

#[path = "../common.rs"]
mod common;
#[path = "../crypto_vectors.rs"]
mod vectors;

use aead::AeadInOut;
use cipher::{BlockCipherDecrypt, BlockCipherEncrypt, BlockModeDecrypt, BlockModeEncrypt, KeyIvInit, StreamCipher};
use defmt::{assert, assert_eq, info, unwrap};
use defmt_rtt as _;
use digest::consts::{U8, U13};
use digest::{Digest, KeyInit, Mac};
use embassy_crypto::{
    Aes128, Aes128CbcDecrypt, Aes128CbcEncrypt, Aes128Ccm, Aes128Cmac, Aes128Ctr, HmacSha256, Sha1, Sha256,
};
use embassy_executor::Spawner;
use panic_probe as _;
use vectors::*;

fn blocks<const N: usize>(data: &[u8]) -> [cipher::Block<Aes128>; N] {
    core::array::from_fn(|i| {
        let mut b = [0u8; 16];
        b.copy_from_slice(&data[i * 16..i * 16 + 16]);
        b.into()
    })
}

fn flatten<const N: usize>(blocks: &[cipher::Block<Aes128>; N]) -> [u8; 1024] {
    let mut out = [0u8; 1024];
    for (i, b) in blocks.iter().enumerate() {
        out[i * 16..i * 16 + 16].copy_from_slice(b.as_slice());
    }
    out
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());

    let mut msg = [0u8; 2049];
    pattern(&mut msg, 1);

    info!("digest");
    for &(len, ref digest) in SHA256 {
        assert_eq!(Sha256::digest(&msg[..len])[..], digest[..]);
        let mut h = Sha256::new();
        for chunk in msg[..len].chunks(37) {
            h.update(chunk);
        }
        assert_eq!(h.finalize()[..], digest[..]);
    }
    for &(len, ref digest) in SHA1 {
        assert_eq!(Sha1::digest(&msg[..len])[..], digest[..]);
    }

    info!("hmac");
    let mut key = [0u8; 200];
    pattern(&mut key, 0x55);
    let mut msg2 = [0u8; 1000];
    pattern(&mut msg2, 2);
    for &(key_len, len, ref digest) in HMAC_SHA256 {
        let mut mac = unwrap!(<HmacSha256 as KeyInit>::new_from_slice(&key[..key_len]).ok());
        mac.update(&msg2[..len]);
        assert_eq!(mac.finalize().into_bytes()[..], digest[..]);
    }

    info!("aes ecb");
    let mut pt = [0u8; 1024];
    pattern(&mut pt, 3);
    let aes = Aes128::new(&KEY128.into());
    let mut b: [cipher::Block<Aes128>; 64] = blocks(&pt);
    aes.encrypt_blocks(&mut b);
    assert_eq!(flatten(&b), AES128_ECB);
    aes.decrypt_blocks(&mut b);
    assert_eq!(flatten(&b), pt);

    info!("aes cbc");
    let mut enc = Aes128CbcEncrypt::new(&KEY128.into(), &CBC_IV.into());
    let mut b: [cipher::Block<Aes128>; 64] = blocks(&pt);
    enc.encrypt_blocks(&mut b[..10]);
    enc.encrypt_blocks(&mut b[10..]);
    assert_eq!(flatten(&b), AES128_CBC);
    let mut dec = Aes128CbcDecrypt::new(&KEY128.into(), &CBC_IV.into());
    dec.decrypt_blocks(&mut b[..1]);
    dec.decrypt_blocks(&mut b[1..]);
    assert_eq!(flatten(&b), pt);

    info!("aes ctr");
    let mut pt = [0u8; 1000];
    pattern(&mut pt, 5);
    let mut buf = pt;
    let mut ctr = Aes128Ctr::new(&KEY128.into(), &CTR_IV0.into());
    ctr.apply_keystream(&mut buf[..1]);
    ctr.apply_keystream(&mut buf[1..100]);
    ctr.apply_keystream(&mut buf[100..]);
    assert_eq!(buf, AES128_CTR0);

    info!("aes cmac");
    let mut msg6 = [0u8; 1024];
    pattern(&mut msg6, 6);
    for &(len, ref tag) in AES128_CMAC {
        let mut mac = <Aes128Cmac as KeyInit>::new(&KEY128.into());
        for chunk in msg6[..len].chunks(13) {
            mac.update(chunk);
        }
        assert_eq!(mac.finalize().into_bytes()[..], tag[..]);
    }

    info!("aes ccm");
    let mut nonce = [0u8; 13];
    pattern(&mut nonce, 7);
    let mut aad = [0u8; 300];
    pattern(&mut aad, 8);
    let mut pt = [0u8; 1000];
    pattern(&mut pt, 9);
    let v = unwrap!(AES128_CCM.iter().find(|v| v.nonce_len == 13 && v.tag_len == 8));
    let ccm = <Aes128Ccm<U8, U13> as KeyInit>::new(&KEY128.into());
    let mut buf = [0u8; 1000];
    buf[..v.pt_len].copy_from_slice(&pt[..v.pt_len]);
    let tag = unwrap!(
        ccm.encrypt_inout_detached(&nonce.into(), &aad[..v.aad_len], (&mut buf[..v.pt_len]).into())
            .ok()
    );
    assert_eq!(buf[..v.pt_len], v.ct[..]);
    assert_eq!(tag[..], v.tag[..]);
    unwrap!(
        ccm.decrypt_inout_detached(&nonce.into(), &aad[..v.aad_len], (&mut buf[..v.pt_len]).into(), &tag)
            .ok()
    );
    assert_eq!(buf[..v.pt_len], pt[..v.pt_len]);
    let mut bad_tag = tag;
    bad_tag[0] ^= 1;
    assert!(
        ccm.decrypt_inout_detached(
            &nonce.into(),
            &aad[..v.aad_len],
            (&mut buf[..v.pt_len]).into(),
            &bad_tag
        )
        .is_err()
    );

    info!("Test OK");
    cortex_m::asm::bkpt();
}
