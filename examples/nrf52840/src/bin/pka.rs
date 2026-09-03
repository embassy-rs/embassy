//! Public key crypto with the CryptoCell PKA: ECDSA, ECDH and RSA.

#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::cryptocell::rng::CcRng;
use embassy_nrf::hash::{Hash, Sha256};
use embassy_nrf::pka::{Error, Pka, Point, PointMut, Signature, SignatureMut, curve};
use {defmt_rtt as _, panic_probe as _};

/// Draws a scalar uniformly from `1..n`, by rejecting anything outside the range.
fn random_scalar(rng: &mut CcRng<'_, embassy_nrf::mode::Blocking>, order: &[u8], out: &mut [u8]) {
    loop {
        rng.blocking_fill_bytes(out);
        // Rejection sampling keeps the distribution flat; a plain reduction would not.
        let zero = out.iter().all(|&b| b == 0);
        let less = out.iter().zip(order).find(|(a, b)| a != b).map(|(a, b)| a < b);
        if !zero && less == Some(true) {
            return;
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut rng = CcRng::new_blocking(p.CC_RNG);
    let mut hash = Hash::new_blocking(p.HASH);
    let mut pka = Pka::new_blocking(p.PKA);

    let curve = &curve::NIST_P256;
    let n = curve.size();

    // A key pair: a private scalar and the generator multiplied by it.
    let mut private_key = [0u8; 32];
    random_scalar(&mut rng, curve.n_bytes(), &mut private_key);
    let mut qx = [0u8; 32];
    let mut qy = [0u8; 32];
    unwrap!(pka.blocking_public_key(
        curve,
        &private_key,
        PointMut {
            x: &mut qx[..n],
            y: &mut qy[..n]
        }
    ));
    info!("public key x = {:02x}", qx);

    // Sign the hash of a message. The ephemeral key must be fresh for every signature and
    // must never leak: recovering it, or reusing it, reveals the private key.
    let mut ctx = hash.start::<Sha256>();
    hash.blocking_update(&mut ctx, b"hello world");
    let digest = hash.blocking_finish(ctx);

    let mut r = [0u8; 32];
    let mut s = [0u8; 32];
    loop {
        let mut k = [0u8; 32];
        random_scalar(&mut rng, curve.n_bytes(), &mut k);
        match pka.blocking_ecdsa_sign(
            curve,
            &private_key,
            &k,
            &digest,
            SignatureMut {
                r: &mut r[..n],
                s: &mut s[..n],
            },
        ) {
            Ok(()) => break,
            // Vanishingly unlikely, but the answer is simply another ephemeral key.
            Err(Error::RetryWithNewK) => continue,
            Err(e) => defmt::panic!("signing failed: {}", e),
        }
    }
    info!("signature r = {:02x}", r);
    info!("signature s = {:02x}", s);

    // Verification, with the right message and with a wrong one.
    let public_key = Point { x: &qx, y: &qy };
    let signature = Signature { r: &r, s: &s };
    unwrap!(pka.blocking_ecdsa_verify(curve, public_key, signature, &digest));
    info!("signature verifies");

    let mut ctx = hash.start::<Sha256>();
    hash.blocking_update(&mut ctx, b"goodbye world");
    let other = hash.blocking_finish(ctx);
    defmt::assert!(pka.blocking_ecdsa_verify(curve, public_key, signature, &other).is_err());
    info!("signature over a different message does not verify");

    // ECDH: both sides multiply the peer's public key by their own private key and end up at
    // the same point. Its X coordinate is the shared secret, which should be hashed before it
    // is used as a key.
    let mut peer_key = [0u8; 32];
    random_scalar(&mut rng, curve.n_bytes(), &mut peer_key);
    let mut px = [0u8; 32];
    let mut py = [0u8; 32];
    unwrap!(pka.blocking_public_key(
        curve,
        &peer_key,
        PointMut {
            x: &mut px[..n],
            y: &mut py[..n]
        }
    ));

    // Never use a public key from the outside world without checking it first.
    unwrap!(pka.blocking_point_check(curve, Point { x: &px, y: &py }));

    let mut sx = [0u8; 32];
    let mut sy = [0u8; 32];
    unwrap!(pka.blocking_ecc_mul(
        curve,
        &private_key,
        Point { x: &px, y: &py },
        PointMut {
            x: &mut sx[..n],
            y: &mut sy[..n]
        }
    ));
    let mut ctx = hash.start::<Sha256>();
    hash.blocking_update(&mut ctx, &sx);
    info!("ecdh shared key = {:02x}", hash.blocking_finish(ctx));

    // The RSA primitive, here with a stand-in modulus. Verifying a real signature also means
    // checking its PKCS#1 padding, which is not the hardware's job.
    let modulus: [u8; 64] = [
        0xc9, 0x1c, 0x9b, 0x27, 0xd1, 0xd7, 0x6b, 0x0f, 0xef, 0x1a, 0x93, 0x1a, 0x1a, 0xd7, 0x6b, 0x0f, 0xa5, 0x93,
        0x8b, 0x4d, 0xb3, 0x2b, 0xe0, 0x0d, 0xdb, 0x4c, 0xd0, 0x2f, 0xe7, 0xf5, 0x3d, 0xe5, 0x21, 0x7a, 0x6f, 0x03,
        0x92, 0x1b, 0x2b, 0x2c, 0xe6, 0x69, 0x2a, 0x7e, 0x03, 0x4d, 0x4a, 0xa7, 0x1b, 0x3d, 0x2a, 0x35, 0x51, 0x93,
        0x6f, 0x1b, 0x9c, 0x0d, 0xf4, 0x3f, 0x25, 0x18, 0xd3, 0x8b,
    ];
    let mut message = [0u8; 64];
    message[63] = 42;
    let mut out = [0u8; 64];
    unwrap!(pka.blocking_mod_exp(&message, &[0x01, 0x00, 0x01], &modulus, &mut out));
    info!("42 ^ 65537 mod n = {:02x}", out);

    info!("done");
}
