//! Shared test body for the `p256` wrappers. Included by both `p256.rs`
//! (software short-circuit) and `p256_driver.rs` (through the unitrait).

// References are deliberate: every `Mul` impl variant is exercised.
#![allow(clippy::op_ref)]

use embassy_crypto::p256 as hw;
use p256::elliptic_curve::ecdh;
use p256::elliptic_curve::ff::{Field, PrimeField};
use p256::elliptic_curve::group::prime::PrimeCurveAffine;
use p256::elliptic_curve::group::{Curve, Group, GroupEncoding};
use p256::elliptic_curve::ops::{LinearCombination, MulByGenerator};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use rand_core::OsRng;

fn hex32(s: &str) -> p256::FieldBytes {
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
    }
    out.into()
}

fn sw_scalar(k: &hw::Scalar) -> p256::Scalar {
    p256::Scalar::from_repr(k.to_repr()).unwrap()
}

fn hw_point_bytes(p: &hw::ProjectivePoint) -> Vec<u8> {
    p.to_affine().to_encoded_point(false).as_bytes().to_vec()
}

fn sw_point_bytes(p: &p256::ProjectivePoint) -> Vec<u8> {
    p.to_affine().to_encoded_point(false).as_bytes().to_vec()
}

#[test]
fn known_answer() {
    // k * G for a fixed k; expected value computed independently of `p256`.
    let k = hw::Scalar::from_repr(hex32(
        "C9AFA9D845BA75166B5C215767B1D6934E50C3DB36E89B127B8A622B120F6721",
    ))
    .unwrap();

    let kg = hw::ProjectivePoint::mul_by_generator(&k).to_affine();
    let encoded = kg.to_encoded_point(false);

    assert_eq!(
        encoded.x().unwrap(),
        &hex32("60FED4BA255A9D31C961EB74C6356D68C049B8923B61FA6CE669622E60F29FB6")
    );
    assert_eq!(
        encoded.y().unwrap(),
        &hex32("7903FE1008B8BC99A41AE9E95628BC64F2F1B20C2D7E9F5177A3C294D4462299")
    );
}

#[test]
fn scalar_mul_matches_p256() {
    for _ in 0..16 {
        let k = hw::Scalar::random(&mut OsRng);
        let l = hw::Scalar::random(&mut OsRng);
        let (sk, sl) = (sw_scalar(&k), sw_scalar(&l));

        // k * G, through every entry point that routes to the driver.
        let expected = sw_point_bytes(&(p256::ProjectivePoint::GENERATOR * sk));
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
        let sp = p256::ProjectivePoint::GENERATOR * sl;
        assert_eq!(hw_point_bytes(&(p * k)), sw_point_bytes(&(sp * sk)));
        assert_eq!(
            hw_point_bytes(&hw::ProjectivePoint::lincomb(
                &p,
                &k,
                &hw::ProjectivePoint::GENERATOR,
                &l
            )),
            sw_point_bytes(&p256::ProjectivePoint::lincomb(
                &sp,
                &sk,
                &p256::ProjectivePoint::GENERATOR,
                &sl
            ))
        );
    }
}

#[test]
fn degenerate_operands() {
    let k = hw::Scalar::random(&mut OsRng);
    let p = hw::ProjectivePoint::GENERATOR * k;

    // Zero scalar and the identity point never reach the driver, and yield the
    // identity, like the software implementation does.
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
fn point_size_stays_close_to_p256() {
    // Holding either of `p256`'s point types costs a tag, not a second copy.
    let ours = core::mem::size_of::<hw::ProjectivePoint>();
    let theirs = core::mem::size_of::<p256::ProjectivePoint>();
    assert!(ours <= theirs + 16, "{ours} > {theirs} + 16");
}

#[test]
fn affine_form_matches_p256() {
    use p256::elliptic_curve::subtle::{Choice, ConditionallySelectable};

    let k = hw::Scalar::random(&mut OsRng);
    let sk = sw_scalar(&k);

    // Points whose affine form is cached (driver results, affine input,
    // constants) and points whose form must be computed (sums, doubles,
    // software products) all agree with `p256`.
    let p = hw::ProjectivePoint::GENERATOR * k;
    let sp = p256::ProjectivePoint::GENERATOR * sk;
    let cases: [(hw::ProjectivePoint, p256::ProjectivePoint); 8] = [
        (p, sp),
        (-p, -sp),
        (hw::ProjectivePoint::from(p.to_affine()), sp),
        (p + p, sp + sp),
        (p.double(), sp.double()),
        (
            p - hw::ProjectivePoint::GENERATOR,
            sp - p256::ProjectivePoint::GENERATOR,
        ),
        (
            hw::ProjectivePoint::conditional_select(&p, &(p + p), Choice::from(1)),
            sp + sp,
        ),
        (
            hw::ProjectivePoint::conditional_select(&(p + p), &p, Choice::from(1)),
            sp,
        ),
    ];

    for (hw_point, sw_point) in cases {
        assert_eq!(hw_point_bytes(&hw_point), sw_point_bytes(&sw_point));
        assert_eq!(hw_point.to_bytes(), sw_point.to_bytes());
        assert_eq!(
            hw::AffinePoint::from(hw_point).to_encoded_point(false),
            sw_point.to_encoded_point(false)
        );
    }

    assert!(bool::from(hw::ProjectivePoint::IDENTITY.to_affine().is_identity()));
    assert!(bool::from((-hw::ProjectivePoint::IDENTITY).to_affine().is_identity()));
    assert_eq!(hw::ProjectivePoint::GENERATOR.to_affine(), hw::AffinePoint::GENERATOR);
}

#[test]
fn scalar_field_matches_p256() {
    let a = hw::Scalar::random(&mut OsRng);
    let b = hw::Scalar::random(&mut OsRng);
    let (sa, sb) = (sw_scalar(&a), sw_scalar(&b));

    assert_eq!(sw_scalar(&(a + b)), sa + sb);
    assert_eq!(sw_scalar(&(a - b)), sa - sb);
    assert_eq!(sw_scalar(&(a * b)), sa * sb);
    assert_eq!(sw_scalar(&(-a)), -sa);
    assert_eq!(sw_scalar(&a.invert().unwrap()), sa.invert().unwrap());
    assert_eq!(sw_scalar(&a.square()), sa.square());
    assert_eq!(hw::Scalar::from(7u64), hw::Scalar::from_inner(p256::Scalar::from(7u64)));
}

#[test]
fn ecdh_matches_p256() {
    let alice = hw::SecretKey::random(&mut OsRng);
    let bob = hw::SecretKey::random(&mut OsRng);

    let hw_shared = ecdh::diffie_hellman(alice.to_nonzero_scalar(), bob.public_key().as_affine());
    let hw_shared_rev = ecdh::diffie_hellman(bob.to_nonzero_scalar(), alice.public_key().as_affine());
    assert_eq!(hw_shared.raw_secret_bytes(), hw_shared_rev.raw_secret_bytes());

    // Same keys, plain `p256`.
    let sw_alice = p256::SecretKey::from_bytes(&alice.to_bytes()).unwrap();
    let sw_bob = p256::SecretKey::from_bytes(&bob.to_bytes()).unwrap();
    assert_eq!(
        alice.public_key().to_encoded_point(false),
        sw_alice.public_key().to_encoded_point(false)
    );
    let sw_shared = ecdh::diffie_hellman(sw_alice.to_nonzero_scalar(), sw_bob.public_key().as_affine());
    assert_eq!(hw_shared.raw_secret_bytes(), sw_shared.raw_secret_bytes());
}

#[test]
fn ecdsa_interoperates_with_p256() {
    use p256::ecdsa::signature::{Signer, Verifier};

    let msg = b"embassy-crypto p256";

    let hw_signing = hw::ecdsa::SigningKey::random(&mut OsRng);
    let sw_signing = p256::ecdsa::SigningKey::from_bytes(&hw_signing.to_bytes()).unwrap();

    // Deterministic (RFC 6979) signatures, so the two must agree bit for bit.
    let hw_sig: hw::ecdsa::Signature = hw_signing.sign(msg);
    let sw_sig: p256::ecdsa::Signature = sw_signing.sign(msg);
    assert_eq!(hw_sig.to_bytes(), sw_sig.to_bytes());

    // Cross-verification.
    let hw_verifying =
        hw::ecdsa::VerifyingKey::from_sec1_bytes(sw_signing.verifying_key().to_encoded_point(false).as_bytes())
            .unwrap();
    hw_verifying.verify(msg, &hw_sig).unwrap();
    sw_signing.verifying_key().verify(msg, &sw_sig).unwrap();
    assert!(hw_verifying.verify(b"other", &hw_sig).is_err());
}
