//! `embassy_crypto::p384`: pure software backend, cross-checked against the
//! `p384` crate.
//!
//! There are no P-384 driver traits yet, so every operation short-circuits
//! to the `p384` crate inside the `Backend` defaults; the SEC 2 base-point
//! known answer anchors the suite to an independent constant.
#![cfg(all(feature = "p384", feature = "p384-ecdsa"))]

// References are deliberate: every `Mul` impl variant is exercised.
#![allow(clippy::op_ref)]

use embassy_crypto::p384 as hw;
use p384::elliptic_curve::ecdh;
use p384::elliptic_curve::ff::{Field, PrimeField};
use p384::elliptic_curve::group::prime::PrimeCurveAffine;
use p384::elliptic_curve::group::{Curve, Group, GroupEncoding};
use p384::elliptic_curve::ops::{LinearCombination, MulByGenerator};
use p384::elliptic_curve::sec1::ToEncodedPoint;
use rand_core::OsRng;

fn hex48(s: &str) -> p384::FieldBytes {
    let mut out = [0u8; 48];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
    }
    out.into()
}

fn sw_scalar(k: &hw::Scalar) -> p384::Scalar {
    p384::Scalar::from_repr(k.to_repr()).unwrap()
}

fn hw_point_bytes(p: &hw::ProjectivePoint) -> Vec<u8> {
    p.to_affine().to_encoded_point(false).as_bytes().to_vec()
}

fn sw_point_bytes(p: &p384::ProjectivePoint) -> Vec<u8> {
    p.to_affine().to_encoded_point(false).as_bytes().to_vec()
}

#[test]
fn generator_matches_sec2() {
    // SEC 2 (secp384r1) base point: a fixed answer independent of `p384`.
    let g = hw::ProjectivePoint::GENERATOR.to_affine();
    let encoded = g.to_encoded_point(false);

    assert_eq!(
        encoded.x().unwrap(),
        &hex48("AA87CA22BE8B05378EB1C71EF320AD746E1D3B628BA79B9859F741E082542A385502F25DBF55296C3A545E3872760AB7")
    );
    assert_eq!(
        encoded.y().unwrap(),
        &hex48("3617DE4A96262C6F5D9E98BF9292DC29F8F41DBD289A147CE9DA3113B5F0B8C00A60B1CE1D7E819D7A431D7C90EA0E5F")
    );

    // 1 * G = G, through the entry points that route to the backend.
    let one = hw::Scalar::ONE;
    assert_eq!(hw::ProjectivePoint::mul_by_generator(&one).to_affine(), g);
    assert_eq!((hw::ProjectivePoint::GENERATOR * one).to_affine(), g);
    assert_eq!((hw::ProjectivePoint::GENERATOR * &one).to_affine(), g);
    assert_eq!((hw::AffinePoint::GENERATOR * one).to_affine(), g);
}

#[test]
fn scalar_mul_matches_p384() {
    for _ in 0..4 {
        let k = hw::Scalar::random(&mut OsRng);
        let l = hw::Scalar::random(&mut OsRng);
        let (sk, sl) = (sw_scalar(&k), sw_scalar(&l));

        // k * G, through every entry point.
        let expected = sw_point_bytes(&(p384::ProjectivePoint::GENERATOR * sk));
        assert_eq!(hw_point_bytes(&hw::ProjectivePoint::mul_by_generator(&k)), expected);
        assert_eq!(hw_point_bytes(&(hw::ProjectivePoint::GENERATOR * k)), expected);
        assert_eq!(hw_point_bytes(&(hw::ProjectivePoint::GENERATOR * &k)), expected);
        assert_eq!(hw_point_bytes(&(&hw::ProjectivePoint::GENERATOR * &k)), expected);
        assert_eq!(hw_point_bytes(&(hw::AffinePoint::GENERATOR * k)), expected);
        assert_eq!(hw_point_bytes(&(hw::AffinePoint::GENERATOR * &k)), expected);
        let mut acc = hw::ProjectivePoint::GENERATOR;
        acc *= k;
        assert_eq!(hw_point_bytes(&acc), expected);

        // k * P for a non-generator P, and the linear combination.
        let p = hw::ProjectivePoint::GENERATOR * l;
        let sp = p384::ProjectivePoint::GENERATOR * sl;
        assert_eq!(hw_point_bytes(&(p * k)), sw_point_bytes(&(sp * sk)));
        assert_eq!(
            hw_point_bytes(&hw::ProjectivePoint::lincomb(
                &p,
                &k,
                &hw::ProjectivePoint::GENERATOR,
                &l
            )),
            sw_point_bytes(&p384::ProjectivePoint::lincomb(
                &sp,
                &sk,
                &p384::ProjectivePoint::GENERATOR,
                &sl
            ))
        );
    }
}

#[test]
fn degenerate_operands() {
    let k = hw::Scalar::random(&mut OsRng);
    let p = hw::ProjectivePoint::GENERATOR * k;

    // Zero scalars and identity points contribute the identity.
    assert!(bool::from((p * hw::Scalar::ZERO).is_identity()));
    assert!(bool::from(
        hw::ProjectivePoint::mul_by_generator(&hw::Scalar::ZERO).is_identity()
    ));
    assert!(bool::from((hw::ProjectivePoint::IDENTITY * k).is_identity()));
    assert!(bool::from((hw::AffinePoint::IDENTITY * k).is_identity()));
    assert!(bool::from(
        (hw::ProjectivePoint::IDENTITY * hw::Scalar::ZERO).is_identity()
    ));

    // n - 1 (the largest valid scalar) gives -P.
    let n_minus_1 = -hw::Scalar::ONE;
    assert_eq!(p * n_minus_1, -p);
    assert_eq!(
        hw::ProjectivePoint::mul_by_generator(&n_minus_1),
        -hw::ProjectivePoint::GENERATOR
    );
}

#[test]
fn point_arithmetic_and_encoding() {
    let k = hw::Scalar::random(&mut OsRng);
    let p = hw::ProjectivePoint::GENERATOR * k;
    let a = p.to_affine();

    assert_eq!(p + p, p.double());
    assert_eq!(p + a, p.double());
    assert!(bool::from((p - p).is_identity()));
    assert_eq!(hw::ProjectivePoint::from(a), p);
    assert_eq!(hw::AffinePoint::from(p), a);
    assert_eq!(hw::AffinePoint::from_bytes(&a.to_bytes()).unwrap(), a);
    assert_eq!(hw::ProjectivePoint::from_bytes(&p.to_bytes()).unwrap(), p);
    assert_eq!(a.to_curve(), p);
    assert!(!bool::from(a.is_identity()));
    assert_eq!(hw::AffinePoint::generator(), hw::AffinePoint::GENERATOR);
}

#[test]
fn scalar_field_matches_p384() {
    let a = hw::Scalar::random(&mut OsRng);
    let b = hw::Scalar::random(&mut OsRng);
    let (sa, sb) = (sw_scalar(&a), sw_scalar(&b));

    assert_eq!(sw_scalar(&(a + b)), sa + sb);
    assert_eq!(sw_scalar(&(a - b)), sa - sb);
    assert_eq!(sw_scalar(&(a * b)), sa * sb);
    assert_eq!(sw_scalar(&(-a)), -sa);
    assert_eq!(sw_scalar(&a.invert().unwrap()), sa.invert().unwrap());
    assert_eq!(sw_scalar(&a.square()), sa.square());
    assert_eq!(hw::Scalar::from(7u64), hw::Scalar::from_inner(p384::Scalar::from(7u64)));
}

#[test]
fn ecdh_matches_p384() {
    let alice = hw::SecretKey::random(&mut OsRng);
    let bob = hw::SecretKey::random(&mut OsRng);

    let hw_shared = ecdh::diffie_hellman(alice.to_nonzero_scalar(), bob.public_key().as_affine());
    let hw_shared_rev = ecdh::diffie_hellman(bob.to_nonzero_scalar(), alice.public_key().as_affine());
    assert_eq!(hw_shared.raw_secret_bytes(), hw_shared_rev.raw_secret_bytes());

    // Same keys, plain `p384`.
    let sw_alice = p384::SecretKey::from_bytes(&alice.to_bytes()).unwrap();
    let sw_bob = p384::SecretKey::from_bytes(&bob.to_bytes()).unwrap();
    assert_eq!(
        alice.public_key().to_encoded_point(false),
        sw_alice.public_key().to_encoded_point(false)
    );
    let sw_shared = ecdh::diffie_hellman(sw_alice.to_nonzero_scalar(), sw_bob.public_key().as_affine());
    assert_eq!(hw_shared.raw_secret_bytes(), sw_shared.raw_secret_bytes());
}

#[test]
fn ecdsa_interoperates_with_p384() {
    use p384::ecdsa::signature::{Signer, Verifier};

    let msg = b"embassy-crypto p384";

    let hw_signing = hw::ecdsa::SigningKey::random(&mut OsRng);
    let sw_signing = p384::ecdsa::SigningKey::from_bytes(&hw_signing.to_bytes()).unwrap();

    // Deterministic (RFC 6979) signatures, so the two must agree bit for bit.
    let hw_sig: hw::ecdsa::Signature = hw_signing.sign(msg);
    let sw_sig: p384::ecdsa::Signature = sw_signing.sign(msg);
    assert_eq!(hw_sig.to_bytes(), sw_sig.to_bytes());

    // Cross-verification.
    let hw_verifying =
        hw::ecdsa::VerifyingKey::from_sec1_bytes(sw_signing.verifying_key().to_encoded_point(false).as_bytes())
            .unwrap();
    hw_verifying.verify(msg, &hw_sig).unwrap();
    sw_signing.verifying_key().verify(msg, &sw_sig).unwrap();
    assert!(hw_verifying.verify(b"other", &hw_sig).is_err());
}

#[cfg(feature = "pkcs8")]
#[test]
fn associated_oid() {
    use p384::elliptic_curve::pkcs8::AssociatedOid;

    // The accelerated curve reports the wrapped curve's OID, so PKCS#8/SPKI
    // key encoding works unchanged.
    assert_eq!(<hw::NistP384 as AssociatedOid>::OID, <p384::NistP384 as AssociatedOid>::OID);
    assert_eq!(
        <hw::NistP384 as AssociatedOid>::OID,
        p384::elliptic_curve::pkcs8::ObjectIdentifier::new_unwrap("1.3.132.0.34")
    );
}
