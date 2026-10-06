//! Public key crypto with the CryptoCell PKA: ECDSA, ECDH and RSA.

#![no_std]
#![no_main]

use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::crypto::pka::{Pka, Point, PointMut, Signature, SignatureMut, curve};
use embassy_nrf::crypto::rng::Rng;
use embassy_nrf::crypto::symmetric::{Sha256, Symmetric};
use panic_probe as _;

/// Draws a scalar uniformly from `1..n`, by rejecting anything outside the range.
fn random_scalar(rng: &mut Rng<'_, embassy_nrf::mode::Blocking>, order: &[u8], out: &mut [u8]) {
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

    let mut rng = Rng::new_blocking(p.CRYPTO_RNG);
    let mut crypto = Symmetric::new_blocking(p.CRYPTO_SYMMETRIC);
    let mut pka = Pka::new_blocking(p.CRYPTO_PKA);

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

    // Sign the hash of a message. The driver draws the nonce from the random number generator.
    let mut ctx = crypto.hash_start::<Sha256>();
    crypto.hash_blocking_update(&mut ctx, b"hello world");
    let digest = crypto.hash_blocking_finish(ctx);

    let mut r = [0u8; 32];
    let mut s = [0u8; 32];
    unwrap!(pka.blocking_ecdsa_sign(
        curve,
        &private_key,
        &digest,
        &mut rng,
        SignatureMut {
            r: &mut r[..n],
            s: &mut s[..n],
        },
    ));
    info!("signature r = {:02x}", r);
    info!("signature s = {:02x}", s);

    // Verification, with the right message and with a wrong one.
    let public_key = Point { x: &qx, y: &qy };
    let signature = Signature { r: &r, s: &s };
    unwrap!(pka.blocking_ecdsa_verify(curve, public_key, signature, &digest));
    info!("signature verifies");

    let mut ctx = crypto.hash_start::<Sha256>();
    crypto.hash_blocking_update(&mut ctx, b"goodbye world");
    let other = crypto.hash_blocking_finish(ctx);
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
    let mut ctx = crypto.hash_start::<Sha256>();
    crypto.hash_blocking_update(&mut ctx, &sx);
    info!("ecdh shared key = {:02x}", crypto.hash_blocking_finish(ctx));

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
