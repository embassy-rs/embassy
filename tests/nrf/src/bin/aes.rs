// required-features: crypto
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
#[path = "../crypto_vectors.rs"]
mod vectors;

use defmt::{assert, assert_eq, info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::aes::{Aes, AesCbc, AesCcm, AesCmac, AesCtr, AesEcb, Cipher, Context, Direction, Error};
use embassy_nrf::mode::Blocking;
use panic_probe as _;
use vectors::*;

type A = Aes<'static, Blocking>;

/// Feeds `input` in chunks whose sizes cycle through `split`.
fn payload_chunked<C: Cipher>(aes: &mut A, ctx: &mut Context<C>, input: &[u8], output: &mut [u8], split: &[usize]) {
    let mut i = 0;
    let mut k = 0;
    while i < input.len() {
        let n = split[k % split.len()].min(input.len() - i);
        let last = i + n == input.len();
        if output.is_empty() {
            unwrap!(aes.blocking_payload(ctx, &input[i..i + n], &mut [], last));
        } else {
            unwrap!(aes.blocking_payload(ctx, &input[i..i + n], &mut output[i..i + n], last));
        }
        i += n;
        k += 1;
    }
}

fn payload_chunked_in_place<C: Cipher>(aes: &mut A, ctx: &mut Context<C>, data: &mut [u8], split: &[usize]) {
    let mut i = 0;
    let mut k = 0;
    while i < data.len() {
        let n = split[k % split.len()].min(data.len() - i);
        let last = i + n == data.len();
        unwrap!(aes.blocking_payload_in_place(ctx, &mut data[i..i + n], last));
        i += n;
        k += 1;
    }
}

fn aad_chunked<C: embassy_nrf::aes::AuthenticatedCipher>(
    aes: &mut A,
    ctx: &mut Context<C>,
    aad: &[u8],
    split: &[usize],
    mark_last: bool,
) {
    let mut i = 0;
    let mut k = 0;
    while i < aad.len() {
        let n = split[k % split.len()].min(aad.len() - i);
        let last = mark_last && i + n == aad.len();
        unwrap!(aes.blocking_aad(ctx, &aad[i..i + n], last));
        i += n;
        k += 1;
    }
}

fn test_block_modes(aes: &mut A, key: &[u8], ecb: &[u8; 1024], cbc: &[u8; 1024], ctr: [&[u8; 1000]; 3]) {
    let mut pt = [0u8; 1024];
    pattern(&mut pt, 3);
    let mut out = [0u8; 1024];

    // ECB, CBC: encrypt into a separate buffer, decrypt in place.
    for split in [&[1024][..], &[16], &[48, 16, 96, 1024]] {
        info!("ecb/cbc split {}", split);
        let mut ctx = aes.start(unwrap!(AesEcb::new(key)), Direction::Encrypt);
        out.fill(0);
        payload_chunked(aes, &mut ctx, &pt, &mut out, split);
        assert!(unwrap!(aes.blocking_finish(ctx)).is_none());
        assert_eq!(out, *ecb);
        let mut ctx = aes.start(unwrap!(AesEcb::new(key)), Direction::Decrypt);
        payload_chunked_in_place(aes, &mut ctx, &mut out, split);
        unwrap!(aes.blocking_finish(ctx));
        assert_eq!(out, pt);

        let mut ctx = aes.start(unwrap!(AesCbc::new(key, &CBC_IV)), Direction::Encrypt);
        out.fill(0);
        payload_chunked(aes, &mut ctx, &pt, &mut out, split);
        unwrap!(aes.blocking_finish(ctx));
        assert_eq!(out, *cbc);
        let mut ctx = aes.start(unwrap!(AesCbc::new(key, &CBC_IV)), Direction::Decrypt);
        payload_chunked_in_place(aes, &mut ctx, &mut out, split);
        unwrap!(aes.blocking_finish(ctx));
        assert_eq!(out, pt);
    }

    // Unaligned buffers.
    let mut ubuf = [0u8; 1024 + 8];
    let mut uout = [0u8; 1024 + 8];
    ubuf[1..1025].copy_from_slice(&pt);
    let mut ctx = aes.start(unwrap!(AesEcb::new(key)), Direction::Encrypt);
    unwrap!(aes.blocking_payload(aes_ctx_ref(&mut ctx), &ubuf[1..1025], &mut uout[3..1027], true));
    unwrap!(aes.blocking_finish(ctx));
    assert_eq!(uout[3..1027], ecb[..]);

    // Block-length errors.
    let mut ctx = aes.start(unwrap!(AesEcb::new(key)), Direction::Encrypt);
    assert_eq!(
        aes.blocking_payload(&mut ctx, &pt[..17], &mut out[..17], true),
        Err(Error::InvalidLength)
    );
    assert_eq!(
        aes.blocking_payload(&mut ctx, &pt[..16], &mut out[..32], true),
        Err(Error::InvalidLength)
    );
    let mut ctx = aes.start(unwrap!(AesCbc::new(key, &CBC_IV)), Direction::Encrypt);
    assert_eq!(
        aes.blocking_payload(&mut ctx, &pt[..1], &mut out[..1], true),
        Err(Error::InvalidLength)
    );

    // CTR: counters crossing 16-bit, 32-bit and 128-bit boundaries, arbitrary chunk sizes.
    let mut pt = [0u8; 1000];
    pattern(&mut pt, 5);
    let mut out = [0u8; 1000];
    for (iv, expected) in [(&CTR_IV0, ctr[0]), (&CTR_IV1, ctr[1]), (&CTR_IV2, ctr[2])] {
        for split in [&[1000][..], &[16], &[1, 15, 16, 17, 100, 3], &[7], &[64, 1]] {
            info!("ctr split {}", split);
            let mut ctx = aes.start(unwrap!(AesCtr::new(key, iv)), Direction::Encrypt);
            out.fill(0);
            payload_chunked(aes, &mut ctx, &pt, &mut out, split);
            assert!(unwrap!(aes.blocking_finish(ctx)).is_none());
            assert_eq!(out, *expected);
            let mut ctx = aes.start(unwrap!(AesCtr::new(key, iv)), Direction::Decrypt);
            payload_chunked_in_place(aes, &mut ctx, &mut out, split);
            unwrap!(aes.blocking_finish(ctx));
            assert_eq!(out, pt);
        }
    }
}

fn aes_ctx_ref<C: Cipher>(ctx: &mut Context<C>) -> &mut Context<C> {
    ctx
}

fn test_cmac(aes: &mut A, key: &[u8], vectors: &[(usize, [u8; 16])]) {
    let mut msg = [0u8; 1024];
    pattern(&mut msg, 6);
    for &(len, ref tag) in vectors {
        for split in [&[1024][..], &[16], &[1, 15, 16, 17, 100, 3], &[64, 1]] {
            info!("cmac len {} split {}", len, split);
            let mut ctx = aes.start(unwrap!(AesCmac::new(key)), Direction::Encrypt);
            payload_chunked(aes, &mut ctx, &msg[..len], &mut [], split);
            let out = unwrap!(unwrap!(aes.blocking_finish(ctx)));
            assert_eq!(&out, tag);
        }
    }
    // Interleaved contexts.
    let mut ctx_a = aes.start(unwrap!(AesCmac::new(key)), Direction::Encrypt);
    let mut ctx_b = aes.start(unwrap!(AesCmac::new(key)), Direction::Encrypt);
    unwrap!(aes.blocking_payload(&mut ctx_a, &msg[..500], &mut [], false));
    unwrap!(aes.blocking_payload(&mut ctx_b, &msg[..40], &mut [], false));
    unwrap!(aes.blocking_payload(&mut ctx_a, &msg[500..1000], &mut [], false));
    unwrap!(aes.blocking_payload(&mut ctx_b, &msg[40..100], &mut [], false));
    let tag_a = unwrap!(unwrap!(aes.blocking_finish(ctx_a)));
    let tag_b = unwrap!(unwrap!(aes.blocking_finish(ctx_b)));
    assert_eq!(&tag_a, &unwrap!(vectors.iter().find(|v| v.0 == 1000)).1);
    assert_eq!(&tag_b, &unwrap!(vectors.iter().find(|v| v.0 == 100)).1);
}

fn test_ccm(aes: &mut A, key: &[u8], vectors: &[CcmVector]) {
    let mut nonce = [0u8; 13];
    pattern(&mut nonce, 7);
    let mut aad = [0u8; 0xff00 + 5];
    pattern(&mut aad, 8);
    let mut pt = [0u8; 1000];
    pattern(&mut pt, 9);
    let mut out = [0u8; 1000];

    for v in vectors {
        let nonce = &nonce[..v.nonce_len];
        let aad = &aad[..v.aad_len];
        let pt = &pt[..v.pt_len];
        let out = &mut out[..v.pt_len];
        for (aad_split, pt_split, mark_last) in [
            (&[4096][..], &[1024][..], true),
            (&[1, 5, 100][..], &[1, 15, 16, 17, 100, 3][..], false),
            (&[16][..], &[7][..], true),
        ] {
            info!(
                "ccm nonce {} tag {} aad {} pt {} splits {} {}",
                v.nonce_len, v.tag_len, v.aad_len, v.pt_len, aad_split, pt_split
            );
            let cipher = unwrap!(AesCcm::new(key, nonce, v.aad_len, v.pt_len, v.tag_len));

            let mut ctx = aes.start(cipher, Direction::Encrypt);
            aad_chunked(aes, &mut ctx, aad, aad_split, mark_last);
            out.fill(0);
            payload_chunked(aes, &mut ctx, pt, out, pt_split);
            let tag = unwrap!(unwrap!(aes.blocking_finish(ctx)));
            assert_eq!(out[..], v.ct[..]);
            assert_eq!(tag[..v.tag_len], v.tag[..]);

            let mut ctx = aes.start(cipher, Direction::Decrypt);
            aad_chunked(aes, &mut ctx, aad, aad_split, mark_last);
            payload_chunked_in_place(aes, &mut ctx, out, pt_split);
            let tag = unwrap!(unwrap!(aes.blocking_finish(ctx)));
            assert_eq!(out[..], pt[..]);
            assert_eq!(tag[..v.tag_len], v.tag[..]);
        }
    }

    // Parameter validation.
    assert_eq!(
        AesCcm::new(key, &nonce[..6], 0, 0, 16).err(),
        Some(Error::InvalidNonceLength)
    );
    assert_eq!(
        AesCcm::new(key, &nonce[..13], 0, 0, 3).err(),
        Some(Error::InvalidTagLength)
    );
    assert_eq!(
        AesCcm::new(key, &nonce[..13], 0, 0, 5).err(),
        Some(Error::InvalidTagLength)
    );
    assert_eq!(
        AesCcm::new(key, &nonce[..13], 0, 0x1_0000, 8).err(),
        Some(Error::InvalidLength)
    );
    assert!(AesCcm::new(key, &nonce[..13], 0, 0xffff, 8).is_ok());
    // Length mismatches and AAD after payload.
    let cipher = unwrap!(AesCcm::new(key, &nonce[..13], 8, 24, 8));
    let mut ctx = aes.start(cipher, Direction::Encrypt);
    assert_eq!(aes.blocking_aad(&mut ctx, &aad[..9], true), Err(Error::InvalidLength));
    let mut ctx = aes.start(cipher, Direction::Encrypt);
    unwrap!(aes.blocking_aad(&mut ctx, &aad[..4], false));
    assert_eq!(
        aes.blocking_payload(&mut ctx, &pt[..24], &mut out[..24], true),
        Err(Error::InvalidLength)
    );
    let mut ctx = aes.start(cipher, Direction::Encrypt);
    unwrap!(aes.blocking_aad(&mut ctx, &aad[..8], true));
    unwrap!(aes.blocking_payload(&mut ctx, &pt[..10], &mut out[..10], false));
    assert_eq!(aes.blocking_aad(&mut ctx, &aad[..1], true), Err(Error::AadAfterPayload));
    assert_eq!(aes.blocking_finish(ctx), Err(Error::InvalidLength));
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut aes = Aes::new_blocking(p.AES);

    assert_eq!(AesEcb::new(&[0; 15]).err(), Some(Error::InvalidKeyLength));
    assert_eq!(AesEcb::new(&[0; 33]).err(), Some(Error::InvalidKeyLength));
    #[cfg(not(feature = "aes256"))]
    assert_eq!(AesEcb::new(&[0; 32]).err(), Some(Error::InvalidKeyLength));

    info!("aes-128");
    test_block_modes(
        &mut aes,
        &KEY128,
        &AES128_ECB,
        &AES128_CBC,
        [&AES128_CTR0, &AES128_CTR1, &AES128_CTR2],
    );
    test_cmac(&mut aes, &KEY128, AES128_CMAC);
    test_ccm(&mut aes, &KEY128, AES128_CCM);

    #[cfg(feature = "aes256")]
    {
        info!("aes-256");
        test_block_modes(
            &mut aes,
            &KEY256,
            &AES256_ECB,
            &AES256_CBC,
            [&AES256_CTR0, &AES256_CTR1, &AES256_CTR2],
        );
        test_cmac(&mut aes, &KEY256, AES256_CMAC);
        test_ccm(&mut aes, &KEY256, AES256_CCM);
        test_gcm(&mut aes, &KEY128, AES128_GCM);
        test_gcm(&mut aes, &KEY256, AES256_GCM);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

#[cfg(feature = "aes256")]
fn test_gcm(aes: &mut A, key: &[u8], vectors: &[AeadVector]) {
    use embassy_nrf::aes::AesGcm;
    let mut nonce = [0u8; 12];
    pattern(&mut nonce, 7);
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
            (&[1, 5, 100][..], &[16, 48, 1024][..], false),
            (&[16][..], &[32][..], true),
        ] {
            info!(
                "gcm aad {} pt {} splits {} {}",
                v.aad_len, v.pt_len, aad_split, pt_split
            );
            let cipher = unwrap!(AesGcm::new(key, &nonce));
            let mut ctx = aes.start(cipher, Direction::Encrypt);
            aad_chunked(aes, &mut ctx, aad, aad_split, mark_last);
            out.fill(0);
            payload_chunked(aes, &mut ctx, pt, out, pt_split);
            let tag = unwrap!(unwrap!(aes.blocking_finish(ctx)));
            assert_eq!(out[..], v.ct[..]);
            assert_eq!(tag, v.tag);

            let mut ctx = aes.start(cipher, Direction::Decrypt);
            aad_chunked(aes, &mut ctx, aad, aad_split, mark_last);
            payload_chunked_in_place(aes, &mut ctx, out, pt_split);
            let tag = unwrap!(unwrap!(aes.blocking_finish(ctx)));
            assert_eq!(out[..], pt[..]);
            assert_eq!(tag, v.tag);
        }
    }
}
