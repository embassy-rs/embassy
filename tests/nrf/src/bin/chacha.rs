// required-features: crypto
#![no_std]
#![no_main]

//! ChaCha20 and ChaCha20-Poly1305 through the direct API, against the shared vectors, with the
//! data fed in many chunk patterns.

#[path = "../common.rs"]
mod common;

use defmt::{assert, assert_eq, info, unwrap};
use defmt_rtt as _;
use embassy_crypto_test::vectors::{self, Expected, MESSAGE};
use embassy_executor::Spawner;
use embassy_nrf::crypto::symmetric::{ChaChaPolyContext, ChaChaVariant, Direction, Error, Symmetric};
use embassy_nrf::mode::Blocking;
use panic_probe as _;

teleprobe_meta::timeout!(120);

type S = Symmetric<'static, Blocking>;

fn aad_chunked(chacha: &mut S, ctx: &mut ChaChaPolyContext, aad: &[u8], split: &[usize], mark_last: bool) {
    let mut i = 0;
    let mut k = 0;
    while i < aad.len() {
        let n = split[k % split.len()].min(aad.len() - i);
        i += n;
        k += 1;
        unwrap!(chacha.chachapoly_blocking_aad(ctx, &aad[i - n..i], mark_last && i == aad.len()));
    }
    if aad.is_empty() && mark_last {
        unwrap!(chacha.chachapoly_blocking_aad(ctx, &[], true));
    }
}

fn payload_chunked(chacha: &mut S, ctx: &mut ChaChaPolyContext, input: &[u8], output: &mut [u8], split: &[usize]) {
    let mut i = 0;
    let mut k = 0;
    while i < input.len() {
        let n = split[k % split.len()].min(input.len() - i);
        i += n;
        k += 1;
        unwrap!(chacha.chachapoly_blocking_payload(ctx, &input[i - n..i], &mut output[i - n..i], i == input.len()));
    }
    if input.is_empty() {
        unwrap!(chacha.chachapoly_blocking_payload(ctx, &[], &mut [], true));
    }
}

/// Wycheproof ChaCha20-Poly1305. Non-final payload chunks must be whole 64-byte blocks.
fn test_aead(chacha: &mut S) {
    const MAX: usize = 1024;
    let suite = &vectors::CHACHA20_POLY1305;
    info!("{}", suite.name);
    let mut passed = 0;
    for v in suite.cases {
        let (Ok(key), Ok(nonce)) = (<&[u8; 32]>::try_from(v.key), <&[u8; 12]>::try_from(v.nonce)) else {
            continue;
        };
        if v.msg.len() > MAX {
            continue;
        }
        let mut out = [0u8; MAX + 1];
        let out = &mut out[1..1 + v.ct.len()];
        let mut back = [0u8; MAX + 3];
        let back = &mut back[3..3 + v.ct.len()];
        for (aad_split, pt_split, mark_last) in [
            (&[4096][..], &[1024][..], true),
            (&[1, 5, 100][..], &[64, 128, 1024][..], false),
            (&[16][..], &[64][..], true),
        ] {
            // Decrypt: the computed tag must match exactly for valid cases only.
            let mut ctx = chacha.chachapoly_start(ChaChaVariant::ChaCha20, key, nonce, Direction::Decrypt);
            aad_chunked(chacha, &mut ctx, v.aad, aad_split, mark_last);
            payload_chunked(chacha, &mut ctx, v.ct, back, pt_split);
            let tag = unwrap!(chacha.chachapoly_blocking_finish(ctx));
            let accepted = tag[..] == v.tag[..] && back[..] == v.msg[..];
            match v.result {
                Expected::Valid => assert!(accepted, "tc {} rejected", v.tc_id),
                Expected::Invalid => assert!(!accepted, "tc {} accepted", v.tc_id),
                Expected::Acceptable => {}
            }
            if v.result != Expected::Valid {
                continue;
            }
            let mut ctx = chacha.chachapoly_start(ChaChaVariant::ChaCha20, key, nonce, Direction::Encrypt);
            aad_chunked(chacha, &mut ctx, v.aad, aad_split, mark_last);
            out.fill(0);
            payload_chunked(chacha, &mut ctx, v.msg, out, pt_split);
            let tag = unwrap!(chacha.chachapoly_blocking_finish(ctx));
            assert_eq!(out[..], v.ct[..], "tc {}", v.tc_id);
            assert_eq!(tag[..], v.tag[..], "tc {}", v.tc_id);
        }
        passed += 1;
    }
    info!("{} cases", passed);

    // Additional data fed after the payload is a usage error.
    let key = [0u8; 32];
    let nonce = [0u8; 12];
    let mut ctx = chacha.chachapoly_start(ChaChaVariant::ChaCha20, &key, &nonce, Direction::Encrypt);
    unwrap!(chacha.chachapoly_blocking_payload(&mut ctx, &[], &mut [], false));
    assert_eq!(
        chacha.chachapoly_blocking_aad(&mut ctx, &[1], true),
        Err(Error::AadAfterPayload)
    );
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut chacha = Symmetric::new_blocking(p.CRYPTO_SYMMETRIC);

    info!("{}", vectors::CHACHA20.name);
    // Unaligned input.
    let mut pt = [0u8; 2048 + 1];
    pt[1..].copy_from_slice(&MESSAGE[..2048]);
    let pt = &pt[1..];
    let mut out = [0u8; 2048];
    for v in vectors::CHACHA20.cases {
        let key = unwrap!(<&[u8; 32]>::try_from(v.key));
        let nonce = unwrap!(<&[u8; 12]>::try_from(v.nonce));
        let len = v.pt_len;
        for split in [&[4096][..], &[1], &[63, 64, 65], &[1, 100, 3], &[64], &[128, 7]] {
            info!("counter {:#x} len {} split {}", v.counter, len, split);
            let mut ctx = chacha.chacha_start(ChaChaVariant::ChaCha20, key, nonce, v.counter);
            let out = &mut out[..len];
            out.fill(0);
            let mut i = 0;
            let mut k = 0;
            while i < len {
                let n = split[k % split.len()].min(len - i);
                unwrap!(chacha.chacha_blocking_apply_keystream(&mut ctx, &pt[i..i + n], &mut out[i..i + n]));
                i += n;
                k += 1;
            }
            assert_eq!(out[..], v.ct[..]);

            // Decrypt in place.
            let mut ctx = chacha.chacha_start(ChaChaVariant::ChaCha20, key, nonce, v.counter);
            let mut i = 0;
            let mut k = 0;
            while i < len {
                let n = split[k % split.len()].min(len - i);
                chacha.chacha_blocking_apply_keystream_in_place(&mut ctx, &mut out[i..i + n]);
                i += n;
                k += 1;
            }
            assert_eq!(out[..], pt[..len]);
        }
    }
    let mut ctx = chacha.chacha_start(ChaChaVariant::ChaCha20, &[0; 32], &[0; 12], 0);
    assert_eq!(
        chacha.chacha_blocking_apply_keystream(&mut ctx, &pt[..16], &mut out[..15]),
        Err(Error::InvalidLength)
    );

    test_aead(&mut chacha);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
