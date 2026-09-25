//! Shared code of the AES tests: every mode with the data fed in many chunk patterns, in place
//! and to a separate buffer, against the shared vectors.
#![allow(dead_code)]

use defmt::{assert, assert_eq, info, unwrap};
use embassy_crypto_test::vectors::{self, Expected, MESSAGE, Suite};
use embassy_nrf::crypto::symmetric::{
    AesCbc, AesCcm, AesCmac, AesContext, AesCtr, AesEcb, AuthenticatedCipher, Cipher, Direction, Error, Symmetric,
};
use embassy_nrf::mode::Blocking;

pub type S = Symmetric<'static, Blocking>;

const MAX: usize = 1024;

/// Feeds `input` in chunks whose sizes cycle through `split`.
fn payload_chunked<C: Cipher>(aes: &mut S, ctx: &mut AesContext<C>, input: &[u8], output: &mut [u8], split: &[usize]) {
    let mut i = 0;
    let mut k = 0;
    while i < input.len() {
        let n = split[k % split.len()].min(input.len() - i);
        let last = i + n == input.len();
        if output.is_empty() {
            unwrap!(aes.aes_blocking_payload(ctx, &input[i..i + n], &mut [], last));
        } else {
            unwrap!(aes.aes_blocking_payload(ctx, &input[i..i + n], &mut output[i..i + n], last));
        }
        i += n;
        k += 1;
    }
}

fn payload_chunked_in_place<C: Cipher>(aes: &mut S, ctx: &mut AesContext<C>, data: &mut [u8], split: &[usize]) {
    let mut i = 0;
    let mut k = 0;
    while i < data.len() {
        let n = split[k % split.len()].min(data.len() - i);
        let last = i + n == data.len();
        unwrap!(aes.aes_blocking_payload_in_place(ctx, &mut data[i..i + n], last));
        i += n;
        k += 1;
    }
}

fn aad_chunked<C: AuthenticatedCipher>(
    aes: &mut S,
    ctx: &mut AesContext<C>,
    aad: &[u8],
    split: &[usize],
    mark_last: bool,
) {
    let mut i = 0;
    let mut k = 0;
    while i < aad.len() {
        let n = split[k % split.len()].min(aad.len() - i);
        let last = mark_last && i + n == aad.len();
        unwrap!(aes.aes_blocking_aad(ctx, &aad[i..i + n], last));
        i += n;
        k += 1;
    }
}

const BLOCK_SPLITS: &[&[usize]] = &[&[1024], &[16], &[48, 16, 96, 1024]];

pub fn test_ecb(aes: &mut S, suite: &Suite<vectors::Ecb>) {
    info!("{}", suite.name);
    let mut out = [0u8; MAX + 3];
    for v in suite.cases {
        let pt = &MESSAGE[..v.pt_len];
        // Encrypt into a separate, unaligned buffer; decrypt in place.
        let out = &mut out[3..3 + v.pt_len];
        for split in BLOCK_SPLITS {
            let mut ctx = aes.aes_start(unwrap!(AesEcb::new(v.key)), Direction::Encrypt);
            out.fill(0);
            payload_chunked(aes, &mut ctx, pt, out, split);
            assert!(unwrap!(aes.aes_blocking_finish(ctx)).is_none());
            assert_eq!(out[..], v.ct[..], "split {}", split);
            let mut ctx = aes.aes_start(unwrap!(AesEcb::new(v.key)), Direction::Decrypt);
            payload_chunked_in_place(aes, &mut ctx, out, split);
            unwrap!(aes.aes_blocking_finish(ctx));
            assert_eq!(out[..], pt[..], "split {}", split);
        }
    }
}

/// Wycheproof CBC vectors carry PKCS#7 padding; the direct API has none, so only the valid cases
/// are usable, with the padding added here.
pub fn test_cbc(aes: &mut S, suite: &Suite<vectors::Cbc>) {
    info!("{}", suite.name);
    let mut pt = [0u8; MAX + 16];
    let mut out = [0u8; MAX + 3];
    let mut n = 0;
    for v in suite.cases {
        if v.result != Expected::Valid || v.msg.len() > MAX {
            continue;
        }
        let iv = unwrap!(<&[u8; 16]>::try_from(v.iv));
        let pad = 16 - v.msg.len() % 16;
        let len = v.msg.len() + pad;
        assert_eq!(len, v.ct.len());
        let pt = &mut pt[..len];
        pt[..v.msg.len()].copy_from_slice(v.msg);
        pt[v.msg.len()..].fill(pad as u8);
        let out = &mut out[3..3 + len];
        for split in BLOCK_SPLITS {
            let mut ctx = aes.aes_start(unwrap!(AesCbc::new(v.key, iv)), Direction::Encrypt);
            out.fill(0);
            payload_chunked(aes, &mut ctx, pt, out, split);
            unwrap!(aes.aes_blocking_finish(ctx));
            assert_eq!(out[..], v.ct[..], "tc {} split {}", v.tc_id, split);
            let mut ctx = aes.aes_start(unwrap!(AesCbc::new(v.key, iv)), Direction::Decrypt);
            payload_chunked_in_place(aes, &mut ctx, out, split);
            unwrap!(aes.aes_blocking_finish(ctx));
            assert_eq!(out[..], pt[..], "tc {} split {}", v.tc_id, split);
        }
        n += 1;
    }
    info!("{} cases", n);
}

/// Counters crossing the 32-, 64- and 128-bit boundaries, arbitrary chunk sizes.
pub fn test_ctr(aes: &mut S, suite: &Suite<vectors::Ctr>) {
    info!("{}", suite.name);
    let mut out = [0u8; MAX + 3];
    for v in suite.cases {
        let iv = unwrap!(<&[u8; 16]>::try_from(v.iv));
        let pt = &MESSAGE[..v.pt_len];
        let out = &mut out[3..3 + v.pt_len];
        for split in [&[1000][..], &[16], &[1, 15, 16, 17, 100, 3], &[7], &[64, 1]] {
            let mut ctx = aes.aes_start(unwrap!(AesCtr::new(v.key, iv)), Direction::Encrypt);
            out.fill(0);
            payload_chunked(aes, &mut ctx, pt, out, split);
            assert!(unwrap!(aes.aes_blocking_finish(ctx)).is_none());
            assert_eq!(out[..], v.ct[..], "split {}", split);
            let mut ctx = aes.aes_start(unwrap!(AesCtr::new(v.key, iv)), Direction::Decrypt);
            payload_chunked_in_place(aes, &mut ctx, out, split);
            unwrap!(aes.aes_blocking_finish(ctx));
            assert_eq!(out[..], pt[..], "split {}", split);
        }
    }
}

/// Wycheproof CMAC: tags may be truncated.
pub fn test_cmac(aes: &mut S, suite: &Suite<vectors::Mac>) {
    info!("{}", suite.name);
    for v in suite.cases {
        let Ok(cipher) = AesCmac::new(v.key) else {
            assert!(v.result != Expected::Valid, "tc {} key rejected", v.tc_id);
            continue;
        };
        for split in [&[1024][..], &[16], &[1, 15, 16, 17, 100, 3], &[64, 1]] {
            let mut ctx = aes.aes_start(cipher, Direction::Encrypt);
            payload_chunked(aes, &mut ctx, v.msg, &mut [], split);
            let tag = unwrap!(unwrap!(aes.aes_blocking_finish(ctx)));
            let matches = v.tag.len() <= 16 && tag[..v.tag.len()] == *v.tag;
            match v.result {
                Expected::Valid => assert!(matches, "tc {} split {} tag mismatch", v.tc_id, split),
                Expected::Invalid => assert!(!matches, "tc {} invalid tag accepted", v.tc_id),
                Expected::Acceptable => {}
            }
        }
    }
    // Interleaved contexts.
    let valid = |min_len: usize| {
        unwrap!(
            suite
                .cases
                .iter()
                .find(|c| c.result == Expected::Valid && c.msg.len() >= min_len && c.tag.len() == 16)
        )
    };
    let (a, b) = (valid(32), valid(20));
    let mut ctx_a = aes.aes_start(unwrap!(AesCmac::new(a.key)), Direction::Encrypt);
    let mut ctx_b = aes.aes_start(unwrap!(AesCmac::new(b.key)), Direction::Encrypt);
    unwrap!(aes.aes_blocking_payload(&mut ctx_a, &a.msg[..16], &mut [], false));
    unwrap!(aes.aes_blocking_payload(&mut ctx_b, &b.msg[..10], &mut [], false));
    unwrap!(aes.aes_blocking_payload(&mut ctx_a, &a.msg[16..], &mut [], true));
    unwrap!(aes.aes_blocking_payload(&mut ctx_b, &b.msg[10..], &mut [], true));
    assert_eq!(unwrap!(unwrap!(aes.aes_blocking_finish(ctx_a)))[..], a.tag[..]);
    assert_eq!(unwrap!(unwrap!(aes.aes_blocking_finish(ctx_b)))[..], b.tag[..]);
}

/// Wycheproof CCM plus the generated long-AAD cases: every nonce and tag length, and the
/// rejection of the ones the mode does not allow.
pub fn test_ccm(aes: &mut S, suite: &Suite<vectors::Aead>) {
    info!("{}", suite.name);
    let mut out = [0u8; MAX + 1];
    let mut back = [0u8; MAX + 3];
    let mut n = 0;
    for v in suite.cases {
        if v.msg.len() > MAX {
            continue;
        }
        let cipher = match AesCcm::new(v.key, v.nonce, v.aad.len(), v.msg.len(), v.tag.len()) {
            Ok(c) => c,
            Err(e) => {
                assert!(
                    matches!(e, Error::InvalidNonceLength | Error::InvalidTagLength) && v.result != Expected::Valid,
                    "tc {} rejected: {}",
                    v.tc_id,
                    e
                );
                continue;
            }
        };
        let out = &mut out[1..1 + v.ct.len()];
        let back = &mut back[3..3 + v.ct.len()];
        for (aad_split, pt_split, mark_last) in [
            (&[4096][..], &[1024][..], true),
            (&[1, 5, 100][..], &[1, 15, 16, 17, 100, 3][..], false),
            (&[16][..], &[7][..], true),
        ] {
            // Decrypt to a separate buffer; the tag must match exactly for valid cases only.
            let mut ctx = aes.aes_start(cipher, Direction::Decrypt);
            aad_chunked(aes, &mut ctx, v.aad, aad_split, mark_last);
            payload_chunked(aes, &mut ctx, v.ct, back, pt_split);
            let tag = unwrap!(unwrap!(aes.aes_blocking_finish(ctx)));
            let accepted = tag[..v.tag.len()] == v.tag[..] && back[..] == v.msg[..];
            match v.result {
                Expected::Valid => assert!(accepted, "tc {} rejected", v.tc_id),
                Expected::Invalid => assert!(!accepted, "tc {} accepted", v.tc_id),
                Expected::Acceptable => {}
            }
            if v.result != Expected::Valid {
                continue;
            }
            // Encrypt in place.
            let mut ctx = aes.aes_start(cipher, Direction::Encrypt);
            aad_chunked(aes, &mut ctx, v.aad, aad_split, mark_last);
            out.copy_from_slice(v.msg);
            payload_chunked_in_place(aes, &mut ctx, out, pt_split);
            let tag = unwrap!(unwrap!(aes.aes_blocking_finish(ctx)));
            assert_eq!(out[..], v.ct[..], "tc {}", v.tc_id);
            assert_eq!(tag[..v.tag.len()], v.tag[..], "tc {}", v.tc_id);
        }
        n += 1;
    }
    info!("{} cases", n);

    // Parameter validation.
    let key = &MESSAGE[..16];
    let nonce = &MESSAGE[16..29];
    assert_eq!(
        AesCcm::new(key, &nonce[..6], 0, 0, 16).err(),
        Some(Error::InvalidNonceLength)
    );
    assert_eq!(AesCcm::new(key, nonce, 0, 0, 3).err(), Some(Error::InvalidTagLength));
    assert_eq!(AesCcm::new(key, nonce, 0, 0, 5).err(), Some(Error::InvalidTagLength));
    assert_eq!(
        AesCcm::new(key, nonce, 0, 0x1_0000, 8).err(),
        Some(Error::InvalidLength)
    );
    assert!(AesCcm::new(key, nonce, 0, 0xffff, 8).is_ok());
    // Length mismatches and AAD after payload.
    let aad = &MESSAGE[..9];
    let pt = &MESSAGE[..24];
    let out = &mut out[..24];
    let cipher = unwrap!(AesCcm::new(key, nonce, 8, 24, 8));
    let mut ctx = aes.aes_start(cipher, Direction::Encrypt);
    assert_eq!(aes.aes_blocking_aad(&mut ctx, aad, true), Err(Error::InvalidLength));
    let mut ctx = aes.aes_start(cipher, Direction::Encrypt);
    unwrap!(aes.aes_blocking_aad(&mut ctx, &aad[..4], false));
    assert_eq!(
        aes.aes_blocking_payload(&mut ctx, pt, out, true),
        Err(Error::InvalidLength)
    );
    let mut ctx = aes.aes_start(cipher, Direction::Encrypt);
    unwrap!(aes.aes_blocking_aad(&mut ctx, &aad[..8], true));
    unwrap!(aes.aes_blocking_payload(&mut ctx, &pt[..10], &mut out[..10], false));
    assert_eq!(
        aes.aes_blocking_aad(&mut ctx, &aad[..1], true),
        Err(Error::AadAfterPayload)
    );
    assert_eq!(aes.aes_blocking_finish(ctx), Err(Error::InvalidLength));
}

/// Wycheproof GCM: cases with a 12-byte nonce and a full tag.
#[cfg(feature = "aes256")]
pub fn test_gcm(aes: &mut S, suite: &Suite<vectors::Aead>) {
    use embassy_nrf::crypto::symmetric::AesGcm;
    info!("{}", suite.name);
    let mut out = [0u8; MAX + 1];
    let mut back = [0u8; MAX + 3];
    let mut n = 0;
    for v in suite.cases {
        let (Ok(nonce), Ok(tag)) = (<&[u8; 12]>::try_from(v.nonce), <&[u8; 16]>::try_from(v.tag)) else {
            continue;
        };
        if v.msg.len() > MAX {
            continue;
        }
        let cipher = unwrap!(AesGcm::new(v.key, nonce));
        let out = &mut out[1..1 + v.ct.len()];
        let back = &mut back[3..3 + v.ct.len()];
        for (aad_split, pt_split, mark_last) in [
            (&[4096][..], &[1024][..], true),
            (&[1, 5, 100][..], &[16, 48, 1024][..], false),
            (&[16][..], &[32][..], true),
        ] {
            let mut ctx = aes.aes_start(cipher, Direction::Decrypt);
            aad_chunked(aes, &mut ctx, v.aad, aad_split, mark_last);
            payload_chunked(aes, &mut ctx, v.ct, back, pt_split);
            let computed = unwrap!(unwrap!(aes.aes_blocking_finish(ctx)));
            let accepted = computed == *tag && back[..] == v.msg[..];
            match v.result {
                Expected::Valid => assert!(accepted, "tc {} rejected", v.tc_id),
                Expected::Invalid => assert!(!accepted, "tc {} accepted", v.tc_id),
                Expected::Acceptable => {}
            }
            if v.result != Expected::Valid {
                continue;
            }
            let mut ctx = aes.aes_start(cipher, Direction::Encrypt);
            aad_chunked(aes, &mut ctx, v.aad, aad_split, mark_last);
            out.copy_from_slice(v.msg);
            payload_chunked_in_place(aes, &mut ctx, out, pt_split);
            let computed = unwrap!(unwrap!(aes.aes_blocking_finish(ctx)));
            assert_eq!(out[..], v.ct[..], "tc {}", v.tc_id);
            assert_eq!(computed, *tag, "tc {}", v.tc_id);
        }
        n += 1;
    }
    info!("{} cases", n);
}
