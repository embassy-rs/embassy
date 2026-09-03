// required-features: crypto
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
#[cfg(feature = "cracen")]
#[path = "../ba414ep_ucode.rs"]
mod ucode;
#[path = "../pka_vectors.rs"]
mod vectors;

use defmt::{assert_eq, info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::mode::Blocking;
use embassy_nrf::pka::{Curve, Error, Pka, Point, PointMut, Signature, SignatureMut, curve};
use panic_probe as _;
use vectors::*;

type P = Pka<'static, Blocking>;

const MAX: usize = 66;

/// Largest modulus the engine has the memory to handle in CRT form.
#[cfg(feature = "cracen")]
const CRT_MAX: usize = 512;
#[cfg(not(feature = "cracen"))]
const CRT_MAX: usize = 320;

fn test_curve(pka: &mut P, curve: &Curve, v: &EcVector) {
    let n = curve.size();
    let mut x = [0u8; MAX];
    let mut y = [0u8; MAX];

    // Public key derivation.
    unwrap!(pka.blocking_public_key(
        curve,
        v.d,
        PointMut {
            x: &mut x[..n],
            y: &mut y[..n]
        }
    ));
    assert_eq!(&x[..n], v.qx);
    assert_eq!(&y[..n], v.qy);

    // Both sides of a key agreement reach the same point.
    unwrap!(pka.blocking_ecc_mul(
        curve,
        v.d,
        Point { x: v.q2x, y: v.q2y },
        PointMut {
            x: &mut x[..n],
            y: &mut y[..n]
        }
    ));
    assert_eq!(&x[..n], v.sharedx);
    assert_eq!(&y[..n], v.sharedy);

    unwrap!(pka.blocking_ecc_mul(
        curve,
        v.d2,
        Point { x: v.qx, y: v.qy },
        PointMut {
            x: &mut x[..n],
            y: &mut y[..n]
        }
    ));
    assert_eq!(&x[..n], v.sharedx);
    assert_eq!(&y[..n], v.sharedy);

    // Points on and off the curve.
    unwrap!(pka.blocking_point_check(curve, Point { x: v.qx, y: v.qy }));
    x[..n].copy_from_slice(v.qx);
    x[n - 1] ^= 1;
    assert_eq!(
        pka.blocking_point_check(curve, Point { x: &x[..n], y: v.qy }),
        Err(Error::InvalidPoint)
    );
    // A coordinate that is not reduced modulo the field prime is not a valid encoding.
    let all_ones = [0xffu8; MAX];
    assert_eq!(
        pka.blocking_point_check(
            curve,
            Point {
                x: &all_ones[..n],
                y: v.qy
            }
        ),
        Err(Error::InvalidPoint)
    );

    // Signing with the vector's ephemeral key reproduces the vector's signature.
    let mut r = [0u8; MAX];
    let mut s = [0u8; MAX];
    unwrap!(pka.blocking_ecdsa_sign(
        curve,
        v.d,
        v.k,
        v.hash,
        SignatureMut {
            r: &mut r[..n],
            s: &mut s[..n]
        }
    ));
    assert_eq!(&r[..n], v.r);
    assert_eq!(&s[..n], v.s);

    // Verification.
    unwrap!(pka.blocking_ecdsa_verify(
        curve,
        Point { x: v.qx, y: v.qy },
        Signature { r: v.r, s: v.s },
        v.hash
    ));

    // A modified hash, signature or public key must not verify.
    let mut hash = [0u8; 32];
    hash.copy_from_slice(v.hash);
    hash[0] ^= 0x80;
    assert_eq!(
        pka.blocking_ecdsa_verify(curve, Point { x: v.qx, y: v.qy }, Signature { r: v.r, s: v.s }, &hash),
        Err(Error::InvalidSignature)
    );
    r[..n].copy_from_slice(v.r);
    r[n - 1] ^= 1;
    assert_eq!(
        pka.blocking_ecdsa_verify(
            curve,
            Point { x: v.qx, y: v.qy },
            Signature { r: &r[..n], s: v.s },
            v.hash
        ),
        Err(Error::InvalidSignature)
    );
    s[..n].copy_from_slice(v.s);
    s[n - 1] ^= 1;
    assert_eq!(
        pka.blocking_ecdsa_verify(
            curve,
            Point { x: v.qx, y: v.qy },
            Signature { r: v.r, s: &s[..n] },
            v.hash
        ),
        Err(Error::InvalidSignature)
    );
    assert_eq!(
        pka.blocking_ecdsa_verify(
            curve,
            Point {
                x: v.q2x,
                y: v.q2y
            },
            Signature { r: v.r, s: v.s },
            v.hash
        ),
        Err(Error::InvalidSignature)
    );

    // A zero signature component is rejected outright.
    let zero = [0u8; MAX];
    assert_eq!(
        pka.blocking_ecdsa_verify(
            curve,
            Point { x: v.qx, y: v.qy },
            Signature {
                r: &zero[..n],
                s: v.s
            },
            v.hash
        ),
        Err(Error::InvalidSignature)
    );

    // Out-of-range and mis-sized scalars.
    assert_eq!(
        pka.blocking_public_key(
            curve,
            &zero[..n],
            PointMut {
                x: &mut x[..n],
                y: &mut y[..n]
            }
        ),
        Err(Error::InvalidScalar)
    );
    assert_eq!(
        pka.blocking_public_key(
            curve,
            &v.d[..n - 1],
            PointMut {
                x: &mut x[..n],
                y: &mut y[..n]
            }
        ),
        Err(Error::InvalidLength)
    );
}

fn test_rsa(pka: &mut P, v: &RsaVector, crt: bool) {
    let n = v.n.len();
    let mut out = [0u8; 384];
    let out = &mut out[..n];

    // Public operation.
    unwrap!(pka.blocking_mod_exp(v.m, v.e, v.n, out));
    assert_eq!(&out[..], v.c);

    // Private operation, without and with the Chinese remainder theorem.
    unwrap!(pka.blocking_mod_exp(v.c, v.d, v.n, out));
    assert_eq!(&out[..], v.m);

    if crt {
        unwrap!(pka.blocking_rsa_crt(v.c, v.p, v.q, v.dp, v.dq, v.qinv, out));
        assert_eq!(&out[..], v.m);
    } else {
        // The engine does not have the memory for a key this large in CRT form.
        assert_eq!(
            pka.blocking_rsa_crt(v.c, v.p, v.q, v.dp, v.dq, v.qinv, out),
            Err(Error::InvalidModulus)
        );
    }

    // An even modulus has no Barrett tag and is rejected.
    let mut even = [0u8; 384];
    even[..n].copy_from_slice(v.n);
    even[n - 1] &= 0xfe;
    assert_eq!(
        pka.blocking_mod_exp(v.m, v.e, &even[..n], out),
        Err(Error::InvalidModulus)
    );
    assert_eq!(pka.blocking_mod_exp(v.m, v.e, v.n, &mut []), Err(Error::InvalidLength));
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    #[cfg(not(feature = "cracen"))]
    let mut pka = Pka::new_blocking(p.PKA);
    #[cfg(feature = "cracen")]
    let mut pka = Pka::new_blocking(p.PKA, &ucode::BA414EP_UCODE);

    info!("rsa-1024");
    test_rsa(&mut pka, &RSA1024, RSA1024.n.len() <= CRT_MAX);
    info!("rsa-2048");
    test_rsa(&mut pka, &RSA2048, RSA2048.n.len() <= CRT_MAX);
    info!("rsa-3072");
    test_rsa(&mut pka, &RSA3072, RSA3072.n.len() <= CRT_MAX);

    info!("p-192");
    test_curve(&mut pka, &curve::NIST_P192, &NIST_P192);
    info!("p-224");
    test_curve(&mut pka, &curve::NIST_P224, &NIST_P224);
    info!("p-256");
    test_curve(&mut pka, &curve::NIST_P256, &NIST_P256);
    info!("p-384");
    test_curve(&mut pka, &curve::NIST_P384, &NIST_P384);
    info!("p-521");
    test_curve(&mut pka, &curve::NIST_P521, &NIST_P521);
    info!("secp256k1");
    test_curve(&mut pka, &curve::SECP256K1, &SECP256K1);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
