// required-features: crypto
#![no_std]
#![no_main]

//! The hash engine through the direct API, against the shared vectors: every digest fed in many
//! chunk patterns, interleaved and cloned contexts, and HMAC over the Wycheproof cases.

#[path = "../common.rs"]
mod common;

use defmt::{assert, assert_eq, info, unwrap};
use defmt_rtt as _;
use embassy_crypto_test::vectors::{self, Expected, MESSAGE, Suite};
use embassy_executor::Spawner;
use embassy_nrf::crypto::symmetric::{DigestContext, HashAlgorithm, Sha1, Sha224, Sha256, Symmetric};
#[cfg(feature = "sha512")]
use embassy_nrf::crypto::symmetric::{Sha384, Sha512, Sha512_224, Sha512_256};
use embassy_nrf::mode::Blocking;
use panic_probe as _;

teleprobe_meta::timeout!(120);

type S = Symmetric<'static, Blocking>;

const SPLITS: &[&[usize]] = &[&[4096], &[1], &[63, 64, 65], &[1, 100, 3], &[64], &[128, 7]];

fn update_chunked<C: DigestContext>(hash: &mut S, ctx: &mut C, data: &[u8], split: &[usize]) {
    let mut i = 0;
    let mut k = 0;
    while i < data.len() {
        let n = split[k % split.len()].min(data.len() - i);
        hash.hash_blocking_update(ctx, &data[i..i + n]);
        i += n;
        k += 1;
    }
}

fn test_hash<A: HashAlgorithm>(hash: &mut S, suite: &Suite<vectors::Digest>)
where
    A::Digest: AsRef<[u8]>,
{
    info!("{}", suite.name);
    for case in suite.cases {
        for split in SPLITS {
            let mut ctx = hash.hash_start::<A>();
            update_chunked(hash, &mut ctx, &MESSAGE[..case.len], split);
            let out = hash.hash_blocking_finish(ctx);
            assert_eq!(out.as_ref(), case.digest, "len {} split {}", case.len, split);
        }
    }
    // Interleaved and cloned contexts.
    let expect = |len: usize| unwrap!(suite.cases.iter().find(|c| c.len == len)).digest;
    let mut a = hash.hash_start::<A>();
    hash.hash_blocking_update(&mut a, &MESSAGE[..500]);
    let mut b = hash.hash_start::<A>();
    hash.hash_blocking_update(&mut b, &MESSAGE[..30]);
    hash.hash_blocking_update(&mut a, &MESSAGE[500..1000]);
    let mut c = a.clone();
    hash.hash_blocking_update(&mut b, &MESSAGE[30..100]);
    hash.hash_blocking_update(&mut c, &MESSAGE[1000..1024]);
    assert_eq!(hash.hash_blocking_finish(a).as_ref(), expect(1000));
    assert_eq!(hash.hash_blocking_finish(b).as_ref(), expect(100));
    assert_eq!(hash.hash_blocking_finish(c).as_ref(), expect(1024));
}

/// Wycheproof HMAC: tags may be truncated, keys have any length.
fn test_hmac<A: HashAlgorithm>(hash: &mut S, suite: &Suite<vectors::Mac>)
where
    A::Digest: AsRef<[u8]>,
{
    info!("{}", suite.name);
    for case in suite.cases {
        for split in [&[4096][..], &[1, 63, 64, 65, 100]] {
            let mut ctx = hash.hmac_start::<A>(case.key);
            update_chunked(hash, &mut ctx, case.msg, split);
            let out = hash.hash_blocking_finish(ctx);
            let matches = case.tag.len() <= out.as_ref().len() && out.as_ref()[..case.tag.len()] == *case.tag;
            match case.result {
                Expected::Valid => assert!(matches, "tc {} tag mismatch", case.tc_id),
                Expected::Invalid => assert!(!matches, "tc {} invalid tag accepted", case.tc_id),
                Expected::Acceptable => {}
            }
        }
    }
    // Reset restores the keyed state.
    let case = unwrap!(
        suite
            .cases
            .iter()
            .find(|c| c.result == Expected::Valid && c.msg.len() > 64)
    );
    let mut ctx = hash.hmac_start::<A>(case.key);
    hash.hash_blocking_update(&mut ctx, &MESSAGE[..77]);
    ctx.reset();
    hash.hash_blocking_update(&mut ctx, case.msg);
    let out = hash.hash_blocking_finish(ctx);
    assert_eq!(&out.as_ref()[..case.tag.len()], case.tag);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut hash = Symmetric::new_blocking(p.CRYPTO_SYMMETRIC);

    test_hash::<Sha1>(&mut hash, &vectors::SHA1);
    test_hash::<Sha224>(&mut hash, &vectors::SHA224);
    test_hash::<Sha256>(&mut hash, &vectors::SHA256);
    test_hmac::<Sha1>(&mut hash, &vectors::HMAC_SHA1);
    test_hmac::<Sha224>(&mut hash, &vectors::HMAC_SHA224);
    test_hmac::<Sha256>(&mut hash, &vectors::HMAC_SHA256);

    #[cfg(feature = "sha512")]
    {
        test_hash::<Sha384>(&mut hash, &vectors::SHA384);
        test_hash::<Sha512>(&mut hash, &vectors::SHA512);
        test_hash::<Sha512_224>(&mut hash, &vectors::SHA512_224);
        test_hash::<Sha512_256>(&mut hash, &vectors::SHA512_256);
        test_hmac::<Sha384>(&mut hash, &vectors::HMAC_SHA384);
        test_hmac::<Sha512>(&mut hash, &vectors::HMAC_SHA512);
        test_hmac::<Sha512_224>(&mut hash, &vectors::HMAC_SHA512_224);
        test_hmac::<Sha512_256>(&mut hash, &vectors::HMAC_SHA512_256);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
