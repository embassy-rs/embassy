// required-features: crypto
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
#[path = "../crypto_vectors.rs"]
mod vectors;

use defmt::{assert_eq, info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::chacha::{AeadContext, ChaCha, Direction, Error};
use embassy_nrf::mode::Blocking;
use panic_probe as _;
use vectors::*;

type C = ChaCha<'static, Blocking>;

fn aead_chunked(chacha: &mut C, ctx: &mut AeadContext, aad: &[u8], split: &[usize], mark_last: bool) {
    let mut i = 0;
    let mut k = 0;
    while i < aad.len() {
        let n = split[k % split.len()].min(aad.len() - i);
        i += n;
        k += 1;
        unwrap!(chacha.blocking_aad(ctx, &aad[i - n..i], mark_last && i == aad.len()));
    }
    if aad.is_empty() && mark_last {
        unwrap!(chacha.blocking_aad(ctx, &[], true));
    }
}

fn payload_chunked(chacha: &mut C, ctx: &mut AeadContext, input: &[u8], output: &mut [u8], split: &[usize]) {
    let mut i = 0;
    let mut k = 0;
    while i < input.len() {
        let n = split[k % split.len()].min(input.len() - i);
        i += n;
        k += 1;
        unwrap!(chacha.blocking_payload(ctx, &input[i - n..i], &mut output[i - n..i], i == input.len()));
    }
    if input.is_empty() {
        unwrap!(chacha.blocking_payload(ctx, &[], &mut [], true));
    }
}

fn test_aead(chacha: &mut C, vectors: &[AeadVector]) {
    let mut aad = [0u8; 300];
    pattern(&mut aad, 8);
    let mut pt = [0u8; 1000];
    pattern(&mut pt, 9);
    let mut out = [0u8; 1000];
    for v in vectors {
        let aad = &aad[..v.aad_len];
        let pt = &pt[..v.pt_len];
        let out = &mut out[..v.pt_len];
        for (aad_split, pt_split, mark_last) in [
            (&[4096][..], &[1024][..], true),
            (&[1, 5, 100][..], &[64, 128, 1024][..], false),
            (&[16][..], &[64][..], true),
        ] {
            info!(
                "chachapoly aad {} pt {} splits {} {}",
                v.aad_len, v.pt_len, aad_split, pt_split
            );
            let mut ctx = chacha.start_aead(&CHACHA_KEY, &CHACHA_NONCE, Direction::Encrypt);
            aead_chunked(chacha, &mut ctx, aad, aad_split, mark_last);
            out.fill(0);
            payload_chunked(chacha, &mut ctx, pt, out, pt_split);
            let tag = unwrap!(chacha.blocking_finish(ctx));
            assert_eq!(out[..], v.ct[..]);
            assert_eq!(tag, v.tag);

            let mut ctx = chacha.start_aead(&CHACHA_KEY, &CHACHA_NONCE, Direction::Decrypt);
            aead_chunked(chacha, &mut ctx, aad, aad_split, mark_last);
            let mut back = [0u8; 1000];
            let back = &mut back[..v.pt_len];
            payload_chunked(chacha, &mut ctx, out, back, pt_split);
            let tag = unwrap!(chacha.blocking_finish(ctx));
            assert_eq!(back[..], pt[..]);
            assert_eq!(tag, v.tag);
        }

        // Additional data fed after the payload is a usage error.
        let mut ctx = chacha.start_aead(&CHACHA_KEY, &CHACHA_NONCE, Direction::Encrypt);
        unwrap!(chacha.blocking_payload(&mut ctx, &[], &mut [], false));
        assert_eq!(chacha.blocking_aad(&mut ctx, &[1], true), Err(Error::AadAfterPayload));
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut chacha = ChaCha::new_blocking(p.CHACHA);

    let mut pt = [0u8; 1000 + 8];
    pattern(&mut pt[1..], 12);
    let pt = &pt[1..];
    let mut out = [0u8; 1000];

    for &(counter, ct) in CHACHA20 {
        let len = ct.len();
        for split in [&[4096][..], &[1], &[63, 64, 65], &[1, 100, 3], &[64], &[128, 7]] {
            info!("counter {:#x} len {} split {}", counter, len, split);
            let mut ctx = chacha.start(&CHACHA_KEY, &CHACHA_NONCE, counter);
            let out = &mut out[..len];
            out.fill(0);
            let mut i = 0;
            let mut k = 0;
            while i < len {
                let n = split[k % split.len()].min(len - i);
                unwrap!(chacha.blocking_apply_keystream(&mut ctx, &pt[i..i + n], &mut out[i..i + n]));
                i += n;
                k += 1;
            }
            assert_eq!(out[..], ct[..]);

            // Decrypt in place.
            let mut ctx = chacha.start(&CHACHA_KEY, &CHACHA_NONCE, counter);
            let mut i = 0;
            let mut k = 0;
            while i < len {
                let n = split[k % split.len()].min(len - i);
                chacha.blocking_apply_keystream_in_place(&mut ctx, &mut out[i..i + n]);
                i += n;
                k += 1;
            }
            assert_eq!(out[..], pt[..len]);
        }
    }

    info!("chacha20-poly1305");
    test_aead(&mut chacha, CHACHA20_POLY1305);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
