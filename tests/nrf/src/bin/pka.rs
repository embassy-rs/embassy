// required-features: crypto
#![no_std]
#![no_main]

//! The public key engine through the direct API: raw RSA and, on every curve, the shared known
//! answers (key derivation, key agreement, signing with a fixed and with a fresh nonce, verification) and the
//! parameter checks. The Wycheproof ECDSA and ECDH suites run in the `pka_ecdsa_*` and
//! `pka_ecdh_*` binaries.

#[path = "../common.rs"]
mod common;
#[path = "../pka_common.rs"]
mod pka_common;
#[cfg(feature = "cracen")]
#[path = "../ba414ep_ucode.rs"]
mod ucode;

use defmt::{assert, assert_eq, info, unwrap};
use defmt_rtt as _;
use embassy_crypto_test::vectors::{self, Suite};
use embassy_nrf::crypto::pka::{
    Curve, Error, MAX_CURVE_LEN, MAX_MODULUS_LEN, Point, PointMut, Signature, SignatureMut,
};
use embassy_nrf::crypto::rng::Rng;
use embassy_nrf::mode::Blocking;
use panic_probe as _;
use pka_common::{KAT, P, sec1};

teleprobe_meta::timeout!(120);

/// Largest modulus the engine has the memory to handle in CRT form.
#[cfg(feature = "cracen")]
const CRT_MAX: usize = 512;
#[cfg(not(feature = "cracen"))]
const CRT_MAX: usize = 320;

fn test_curve(pka: &mut P, rng: &mut Rng<'_, Blocking>, curve: &Curve, suite: &Suite<vectors::EcKat>) {
    info!("{}", suite.name);
    let n = curve.size();
    let mut x = [0u8; MAX_CURVE_LEN];
    let mut y = [0u8; MAX_CURVE_LEN];
    let mut r = [0u8; MAX_CURVE_LEN];
    let mut s = [0u8; MAX_CURVE_LEN];
    for (i, v) in suite.cases.iter().enumerate() {
        let (qx, qy) = unwrap!(sec1(v.public, n));
        let (q2x, q2y) = unwrap!(sec1(v.public2, n));
        let (sharedx, sharedy) = unwrap!(sec1(v.shared, n));
        let (vr, vs) = v.sig.split_at(n);

        // Public key derivation.
        let out = PointMut {
            x: &mut x[..n],
            y: &mut y[..n],
        };
        unwrap!(pka.blocking_public_key(curve, v.private, out));
        assert_eq!(&x[..n], qx);
        assert_eq!(&y[..n], qy);

        // Both sides of a key agreement reach the same point.
        let out = PointMut {
            x: &mut x[..n],
            y: &mut y[..n],
        };
        unwrap!(pka.blocking_ecc_mul(curve, v.private, Point { x: q2x, y: q2y }, out));
        assert_eq!(&x[..n], sharedx);
        assert_eq!(&y[..n], sharedy);
        let out = PointMut {
            x: &mut x[..n],
            y: &mut y[..n],
        };
        unwrap!(pka.blocking_ecc_mul(curve, v.private2, Point { x: qx, y: qy }, out));
        assert_eq!(&x[..n], sharedx);
        assert_eq!(&y[..n], sharedy);

        // Signing with the vector's nonce reproduces the vector's signature, which
        // verifies.
        let out = SignatureMut {
            r: &mut r[..n],
            s: &mut s[..n],
        };
        unwrap!(pka.blocking_ecdsa_sign_with_nonce(curve, v.private, v.k, v.digest, out));
        assert_eq!(&r[..n], vr);
        assert_eq!(&s[..n], vs);
        unwrap!(pka.blocking_ecdsa_verify(curve, Point { x: qx, y: qy }, Signature { r: vr, s: vs }, v.digest));

        // Signing with a driver-drawn nonce gives a fresh signature that verifies.
        let out = SignatureMut {
            r: &mut r[..n],
            s: &mut s[..n],
        };
        unwrap!(pka.blocking_ecdsa_sign(curve, v.private, v.digest, rng, out));
        assert!(&r[..n] != vr);
        unwrap!(pka.blocking_ecdsa_verify(
            curve,
            Point { x: qx, y: qy },
            Signature { r: &r[..n], s: &s[..n] },
            v.digest
        ));

        if i > 0 {
            continue;
        }
        // Once per curve: the checks of the API.

        // Points on and off the curve.
        unwrap!(pka.blocking_point_check(curve, Point { x: qx, y: qy }));
        x[..n].copy_from_slice(qx);
        x[n - 1] ^= 1;
        assert_eq!(
            pka.blocking_point_check(curve, Point { x: &x[..n], y: qy }),
            Err(Error::InvalidPoint)
        );
        // A coordinate that is not reduced modulo the field prime is not a valid encoding.
        let all_ones = [0xffu8; MAX_CURVE_LEN];
        assert_eq!(
            pka.blocking_point_check(
                curve,
                Point {
                    x: &all_ones[..n],
                    y: qy
                }
            ),
            Err(Error::InvalidPoint)
        );

        // A modified hash, signature or public key must not verify.
        let mut hash = [0u8; 64];
        let hash = &mut hash[..v.digest.len()];
        hash.copy_from_slice(v.digest);
        hash[0] ^= 0x80;
        assert_eq!(
            pka.blocking_ecdsa_verify(curve, Point { x: qx, y: qy }, Signature { r: vr, s: vs }, hash),
            Err(Error::InvalidSignature)
        );
        r[..n].copy_from_slice(vr);
        r[n - 1] ^= 1;
        assert_eq!(
            pka.blocking_ecdsa_verify(curve, Point { x: qx, y: qy }, Signature { r: &r[..n], s: vs }, v.digest),
            Err(Error::InvalidSignature)
        );
        s[..n].copy_from_slice(vs);
        s[n - 1] ^= 1;
        assert_eq!(
            pka.blocking_ecdsa_verify(curve, Point { x: qx, y: qy }, Signature { r: vr, s: &s[..n] }, v.digest),
            Err(Error::InvalidSignature)
        );
        assert_eq!(
            pka.blocking_ecdsa_verify(curve, Point { x: q2x, y: q2y }, Signature { r: vr, s: vs }, v.digest),
            Err(Error::InvalidSignature)
        );
        // A zero signature component is rejected outright.
        let zero = [0u8; MAX_CURVE_LEN];
        assert_eq!(
            pka.blocking_ecdsa_verify(
                curve,
                Point { x: qx, y: qy },
                Signature { r: &zero[..n], s: vs },
                v.digest
            ),
            Err(Error::InvalidSignature)
        );

        // Out-of-range and mis-sized scalars.
        let out = PointMut {
            x: &mut x[..n],
            y: &mut y[..n],
        };
        assert_eq!(
            pka.blocking_public_key(curve, &zero[..n], out),
            Err(Error::InvalidScalar)
        );
        let out = PointMut {
            x: &mut x[..n],
            y: &mut y[..n],
        };
        assert_eq!(
            pka.blocking_public_key(curve, curve.n_bytes(), out),
            Err(Error::InvalidScalar)
        );
        let out = PointMut {
            x: &mut x[..n],
            y: &mut y[..n],
        };
        assert_eq!(
            pka.blocking_public_key(curve, &v.private[..n - 1], out),
            Err(Error::InvalidLength)
        );
    }
}

fn test_rsa(pka: &mut P, suite: &Suite<vectors::Rsa>) {
    info!("{}", suite.name);
    let mut out = [0u8; MAX_MODULUS_LEN];
    for v in suite.cases {
        let n = v.n.len();
        let out = &mut out[..n];

        // Public operation.
        unwrap!(pka.blocking_mod_exp(v.m, v.e, v.n, out));
        assert_eq!(&out[..], v.c);

        // Private operation, without and with the Chinese remainder theorem.
        unwrap!(pka.blocking_mod_exp(v.c, v.d, v.n, out));
        assert_eq!(&out[..], v.m);

        if n <= CRT_MAX {
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
        let mut even = [0u8; MAX_MODULUS_LEN];
        even[..n].copy_from_slice(v.n);
        even[n - 1] &= 0xfe;
        assert_eq!(
            pka.blocking_mod_exp(v.m, v.e, &even[..n], out),
            Err(Error::InvalidModulus)
        );
        assert_eq!(pka.blocking_mod_exp(v.m, v.e, v.n, &mut []), Err(Error::InvalidLength));
    }
}

pka_test!(|p, pka| {
    test_rsa(&mut pka, &vectors::RSA_2048);
    test_rsa(&mut pka, &vectors::RSA_3072);
    test_rsa(&mut pka, &vectors::RSA_4096);
    let mut rng = Rng::new_blocking(p.CRYPTO_RNG);
    for (curve, suite) in KAT {
        test_curve(&mut pka, &mut rng, curve, suite);
    }
});
