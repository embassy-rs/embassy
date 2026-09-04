//! `embassy_crypto::p256` without the `driver-p256-scalar-mul` feature: the
//! wrappers go through the `P256ScalarMul` unitrait, which this test binary
//! implements itself (with `p256`) and instruments with a call counter.
#![cfg(all(feature = "p256", feature = "p256-ecdsa", not(feature = "driver-p256-scalar-mul")))]

use core::cell::Cell;

use embassy_crypto_driver::{P256AffinePoint, P256Scalar, P256ScalarMul};
use p256::elliptic_curve::ff::PrimeField;
use p256::elliptic_curve::ops::MulByGenerator;
use p256::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};

#[path = "common/p256.rs"]
mod suite;

// Thread-local: the driver runs on the caller's thread, and the harness runs
// the suite tests in parallel.
thread_local! {
    static CALLS: Cell<usize> = const { Cell::new(0) };
}

struct TestDriver;

impl P256ScalarMul for TestDriver {
    fn mul_base(k: &P256Scalar) -> P256AffinePoint {
        CALLS.with(|calls| calls.set(calls.get() + 1));
        to_canonical(&p256::ProjectivePoint::mul_by_generator(&scalar(k)).to_affine())
    }

    fn mul_affine(k: &P256Scalar, p: &P256AffinePoint) -> P256AffinePoint {
        CALLS.with(|calls| calls.set(calls.get() + 1));
        to_canonical(&(point(p) * scalar(k)).to_affine())
    }
}

embassy_crypto_driver::p256_scalar_mul_impl!(TestDriver);

fn scalar(k: &P256Scalar) -> p256::Scalar {
    let k = p256::Scalar::from_repr(k.0.into()).expect("scalar in range");
    assert!(
        !bool::from(p256::elliptic_curve::ff::Field::is_zero(&k)),
        "driver got a zero scalar"
    );
    k
}

fn point(p: &P256AffinePoint) -> p256::AffinePoint {
    p256::AffinePoint::from_encoded_point(&p256::EncodedPoint::from_affine_coordinates(
        &p.x.into(),
        &p.y.into(),
        false,
    ))
    .expect("driver got an off-curve point")
}

fn to_canonical(p: &p256::AffinePoint) -> P256AffinePoint {
    let encoded = p.to_encoded_point(false);
    P256AffinePoint {
        x: (*encoded.x().unwrap()).into(),
        y: (*encoded.y().unwrap()).into(),
    }
}

#[test]
fn wrappers_call_the_driver() {
    use embassy_crypto::p256 as hw;
    use p256::elliptic_curve::ops::LinearCombination;

    let before = CALLS.with(|calls| calls.get());
    let k = hw::Scalar::from(12345u64);

    let p = hw::ProjectivePoint::mul_by_generator(&k);
    let _ = p * k;
    let _ = hw::ProjectivePoint::lincomb(&p, &k, &hw::ProjectivePoint::GENERATOR, &k);
    // Degenerate operands are still routed through the driver (constant-time
    // substitution), so they count too.
    let _ = p * hw::Scalar::ZERO;

    assert_eq!(CALLS.with(|calls| calls.get()) - before, 5);
}
