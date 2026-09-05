//! `embassy_crypto::p256` without the `driver-p256-scalar-mul` feature: the
//! wrappers go through the `P256ScalarMul` unitrait, which this test binary
//! implements itself (with `p256`) and instruments with a call counter.
#![cfg(all(feature = "p256", feature = "p256-ecdsa", not(feature = "driver-p256-scalar-mul")))]

use core::cell::Cell;

use embassy_crypto_driver::{P256AffinePoint, P256Scalar, P256ScalarMul};
use p256::elliptic_curve::ff::PrimeField;
#[cfg(not(feature = "driver-p256-lincomb"))]
use p256::elliptic_curve::group::Curve;
use p256::elliptic_curve::ops::MulByGenerator;
use p256::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};

#[path = "common/p256.rs"]
mod suite;

// Thread-local: the driver runs on the caller's thread, and the harness runs
// the suite tests in parallel.
thread_local! {
    static CALLS: Cell<usize> = const { Cell::new(0) };
    static ZERO_SCALARS: Cell<usize> = const { Cell::new(0) };
}

fn saw_zero(k: &P256Scalar) {
    if k.0 == [0u8; 32] {
        ZERO_SCALARS.with(|zeros| zeros.set(zeros.get() + 1));
    }
}

struct TestDriver;

impl P256ScalarMul for TestDriver {
    fn mul_base(k: &P256Scalar) -> P256AffinePoint {
        CALLS.with(|calls| calls.set(calls.get() + 1));
        saw_zero(k);
        to_canonical_or_identity(&p256::ProjectivePoint::mul_by_generator(&scalar(k)).to_affine())
    }

    fn mul_affine(k: &P256Scalar, p: &P256AffinePoint) -> P256AffinePoint {
        CALLS.with(|calls| calls.set(calls.get() + 1));
        saw_zero(k);
        to_canonical_or_identity(&(point(p) * scalar(k)).to_affine())
    }
}

embassy_crypto_driver::p256_scalar_mul_impl!(TestDriver);

#[cfg(not(feature = "driver-p256-scalar-invert"))]
impl embassy_crypto_driver::P256ScalarInvert for TestDriver {
    fn invert(k: &P256Scalar) -> P256Scalar {
        CALLS.with(|calls| calls.set(calls.get() + 1));
        P256Scalar(
            p256::elliptic_curve::ff::Field::invert(&scalar(k))
                .unwrap()
                .to_bytes()
                .into(),
        )
    }

    fn invert_vartime(k: &P256Scalar) -> P256Scalar {
        CALLS.with(|calls| calls.set(calls.get() + 1));
        P256Scalar(
            p256::elliptic_curve::ops::Invert::invert_vartime(&scalar(k))
                .unwrap()
                .to_bytes()
                .into(),
        )
    }
}

#[cfg(not(feature = "driver-p256-scalar-invert"))]
embassy_crypto_driver::p256_scalar_invert_impl!(TestDriver);

#[cfg(not(feature = "driver-p256-lincomb"))]
impl embassy_crypto_driver::P256Lincomb for TestDriver {
    fn lincomb(
        k1: &P256Scalar,
        p1: &P256AffinePoint,
        k2: &P256Scalar,
        p2: &P256AffinePoint,
    ) -> Option<P256AffinePoint> {
        use p256::elliptic_curve::group::Group;

        CALLS.with(|calls| calls.set(calls.get() + 1));
        saw_zero(k1);
        saw_zero(k2);
        let sum = point(p1) * scalar(k1) + point(p2) * scalar(k2);
        (!bool::from(sum.is_identity())).then(|| to_canonical(&sum.to_affine()))
    }
}

#[cfg(not(feature = "driver-p256-lincomb"))]
embassy_crypto_driver::p256_lincomb_impl!(TestDriver);

fn scalar(k: &P256Scalar) -> p256::Scalar {
    // Zero scalars are within the relaxed driver contract.
    p256::Scalar::from_repr(k.0.into()).expect("scalar in range")
}

fn point(p: &P256AffinePoint) -> p256::AffinePoint {
    p256::AffinePoint::from_encoded_point(&p256::EncodedPoint::from_affine_coordinates(
        &p.x.into(),
        &p.y.into(),
        false,
    ))
    .expect("driver got an off-curve point")
}

/// The identity result of a zero scalar is reported with the all-zero
/// sentinel coordinates.
fn to_canonical_or_identity(p: &p256::AffinePoint) -> P256AffinePoint {
    if bool::from(p256::elliptic_curve::group::prime::PrimeCurveAffine::is_identity(p)) {
        P256AffinePoint::default()
    } else {
        to_canonical(p)
    }
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
    let zero_before = ZERO_SCALARS.with(|zeros| zeros.get());
    let k = hw::Scalar::from(12345u64);

    let p = hw::ProjectivePoint::mul_by_generator(&k);
    let _ = p * k;
    let _ = hw::ProjectivePoint::lincomb(&p, &k, &hw::ProjectivePoint::GENERATOR, &k);
    // Degenerate operands still reach the driver in a single call — a zero
    // scalar is passed as-is under the relaxed contract — so they count too.
    let _ = p * hw::Scalar::ZERO;

    // The last call must have reached the driver with k = 0.
    assert_eq!(ZERO_SCALARS.with(|zeros| zeros.get()) - zero_before, 1);

    // The linear combination is one driver call with the lincomb hook, two without.
    let expected = if cfg!(not(feature = "driver-p256-lincomb")) {
        4
    } else {
        5
    };
    assert_eq!(CALLS.with(|calls| calls.get()) - before, expected);
}

#[cfg(not(feature = "driver-p256-scalar-invert"))]
#[test]
fn inversion_calls_the_driver() {
    use embassy_crypto::p256 as hw;
    use p256::elliptic_curve::ff::Field;
    use p256::elliptic_curve::ops::Invert;

    let before = CALLS.with(|calls| calls.get());
    let k = hw::Scalar::from(987654321u64);
    let sw = p256::Scalar::from(987654321u64);

    assert_eq!(Field::invert(&k).unwrap().into_inner(), Field::invert(&sw).unwrap());
    assert_eq!(Invert::invert(&k).unwrap().into_inner(), Field::invert(&sw).unwrap());
    assert_eq!(
        Invert::invert_vartime(&k).unwrap().into_inner(),
        Field::invert(&sw).unwrap()
    );
    // Zero is never shown to the driver, but still counts as a (masked) call.
    assert!(bool::from(Field::invert(&hw::Scalar::ZERO).is_none()));
    assert!(bool::from(Invert::invert_vartime(&hw::Scalar::ZERO).is_none()));

    assert_eq!(CALLS.with(|calls| calls.get()) - before, 5);
}

#[cfg(not(feature = "driver-p256-lincomb"))]
#[test]
fn lincomb_calls_the_driver() {
    use embassy_crypto::p256 as hw;
    use p256::elliptic_curve::group::Group;
    use p256::elliptic_curve::ops::LinearCombination;

    let k = hw::Scalar::from(1234567u64);
    let l = hw::Scalar::from(7654321u64);
    let g = hw::ProjectivePoint::GENERATOR;
    let p = g * l;
    let sw = |x: &hw::ProjectivePoint| p256::ProjectivePoint::from(x.to_affine().into_inner());

    let before = CALLS.with(|calls| calls.get());
    let zero_before = ZERO_SCALARS.with(|zeros| zeros.get());
    let expected = sw(&g) * k.into_inner() + sw(&p) * l.into_inner();

    // Regular operands: exactly one driver call, and the result is already affine.
    let r = hw::ProjectivePoint::lincomb(&g, &k, &p, &l);
    assert_eq!(sw(&r), expected);
    assert_eq!(CALLS.with(|calls| calls.get()) - before, 1);

    // Degenerate operands are handled in the same single driver call: zero
    // scalars are passed as-is and identity points are replaced by the
    // generator.
    assert_eq!(
        sw(&hw::ProjectivePoint::lincomb(&g, &hw::Scalar::ZERO, &p, &l)),
        sw(&p) * l.into_inner()
    );
    assert_eq!(
        sw(&hw::ProjectivePoint::lincomb(&g, &k, &p, &hw::Scalar::ZERO)),
        sw(&g) * k.into_inner()
    );
    assert_eq!(
        sw(&hw::ProjectivePoint::lincomb(
            &hw::ProjectivePoint::IDENTITY,
            &k,
            &p,
            &l
        )),
        sw(&p) * l.into_inner()
    );
    assert!(bool::from(
        hw::ProjectivePoint::lincomb(&hw::ProjectivePoint::IDENTITY, &hw::Scalar::ZERO, &p, &hw::Scalar::ZERO)
            .is_identity()
    ));

    // Cancellation: the driver reports the identity.
    assert!(bool::from(hw::ProjectivePoint::lincomb(&p, &k, &-p, &k).is_identity()));
    assert_eq!(CALLS.with(|calls| calls.get()) - before, 6);

    // Four degenerate calls above, carrying five zero scalars in total: two
    // explicit `k = 0` terms, one from the identity point, and two in the
    // all-degenerate call.
    assert_eq!(ZERO_SCALARS.with(|zeros| zeros.get()) - zero_before, 5);
}
