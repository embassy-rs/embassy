// required-features: crypto
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
#[path = "../crypto_vectors.rs"]
mod vectors;

use defmt::{assert_eq, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::hash::{Algorithm, Hash, HashContext, Sha1, Sha224, Sha256};
#[cfg(feature = "sha512")]
use embassy_nrf::hash::{Sha384, Sha512, Sha512_224, Sha512_256};
use embassy_nrf::mode::Blocking;
use panic_probe as _;
use vectors::*;

type H = Hash<'static, Blocking>;

const SPLITS: &[&[usize]] = &[&[4096], &[1], &[63, 64, 65], &[1, 100, 3], &[64], &[128, 7]];

fn update_chunked<C: HashContext>(hash: &mut H, ctx: &mut C, data: &[u8], split: &[usize]) {
    let mut i = 0;
    let mut k = 0;
    while i < data.len() {
        let n = split[k % split.len()].min(data.len() - i);
        hash.blocking_update(ctx, &data[i..i + n]);
        i += n;
        k += 1;
    }
}

fn test_hash<A: Algorithm>(hash: &mut H, vectors: &[(usize, A::Digest)])
where
    A::Digest: defmt::Format + PartialEq,
{
    let mut msg = [0u8; 2049 + 8];
    pattern(&mut msg[1..], 1);
    let msg = &msg[1..];
    for &(len, ref digest) in vectors {
        for split in SPLITS {
            info!("len {} split {}", len, split);
            let mut ctx = hash.start::<A>();
            update_chunked(hash, &mut ctx, &msg[..len], split);
            let out = hash.blocking_finish(ctx);
            assert_eq!(&out, digest);
        }
    }
    // Interleaved and cloned contexts.
    let mut a = hash.start::<A>();
    hash.blocking_update(&mut a, &msg[..500]);
    let mut b = hash.start::<A>();
    hash.blocking_update(&mut b, &msg[..30]);
    hash.blocking_update(&mut a, &msg[500..1000]);
    let mut c = a.clone();
    hash.blocking_update(&mut b, &msg[30..100]);
    hash.blocking_update(&mut c, &msg[1000..1024]);
    let expect = |len: usize| &vectors.iter().find(|v| v.0 == len).unwrap().1;
    assert_eq!(&hash.blocking_finish(a), expect(1000));
    assert_eq!(&hash.blocking_finish(b), expect(100));
    assert_eq!(&hash.blocking_finish(c), expect(1024));
}

fn test_hmac<A: Algorithm>(hash: &mut H, vectors: &[(usize, usize, A::Digest)])
where
    A::Digest: defmt::Format + PartialEq,
{
    let mut key = [0u8; 200];
    pattern(&mut key, 0x55);
    let mut msg = [0u8; 1000];
    pattern(&mut msg, 2);
    for &(key_len, len, ref digest) in vectors {
        for split in [&[4096][..], &[1, 63, 64, 65, 100]] {
            info!("key {} len {} split {}", key_len, len, split);
            let mut ctx = hash.start_hmac::<A>(&key[..key_len]);
            update_chunked(hash, &mut ctx, &msg[..len], split);
            let out = hash.blocking_finish(ctx);
            assert_eq!(&out, digest);
        }
        // Reset restores the keyed state.
        let mut ctx = hash.start_hmac::<A>(&key[..key_len]);
        hash.blocking_update(&mut ctx, &msg[..77]);
        ctx.reset();
        hash.blocking_update(&mut ctx, &msg[..len]);
        assert_eq!(&hash.blocking_finish(ctx), digest);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut hash = Hash::new_blocking(p.HASH);

    info!("sha1");
    test_hash::<Sha1>(&mut hash, SHA1);
    info!("sha224");
    test_hash::<Sha224>(&mut hash, SHA224);
    info!("sha256");
    test_hash::<Sha256>(&mut hash, SHA256);
    info!("hmac-sha1");
    test_hmac::<Sha1>(&mut hash, HMAC_SHA1);
    info!("hmac-sha224");
    test_hmac::<Sha224>(&mut hash, HMAC_SHA224);
    info!("hmac-sha256");
    test_hmac::<Sha256>(&mut hash, HMAC_SHA256);

    #[cfg(feature = "sha512")]
    {
        info!("sha384");
        test_hash::<Sha384>(&mut hash, SHA384);
        info!("sha512");
        test_hash::<Sha512>(&mut hash, SHA512);
        info!("sha512-224");
        test_hash::<Sha512_224>(&mut hash, SHA512_224);
        info!("sha512-256");
        test_hash::<Sha512_256>(&mut hash, SHA512_256);
        info!("hmac-sha384");
        test_hmac::<Sha384>(&mut hash, HMAC_SHA384);
        info!("hmac-sha512");
        test_hmac::<Sha512>(&mut hash, HMAC_SHA512);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
