//! Exercises the `P384Scalar*` driver unitraits: that `embassy_crypto::p384`
//! routes accelerated operations through them when the `driver-p384-*`
//! features are off, that the wrappers enforce the driver contract, and that
//! the accelerated results stay correct.
//!
//! The `p384` crate provides the software implementation of the unitraits
//! under test, and the suite in `common/p384.rs` additionally cross-checks
//! every accelerated result against it.
//!
//! With any of the `driver-p384-scalar-mul`/`driver-p384-scalar-invert`/
//! `driver-p384-lincomb` features on, `embassy_crypto::p384` stops using the
//! corresponding unitraits (running the same operations in software
//! directly), and the affected assertions below do not apply; the impls in
//! this file are registered only for the unitraits whose features are off,
//! so this binary never defines a unitrait global that the crate also
//! defines.
#![cfg(all(
    feature = "p384",
    feature = "p384-ecdsa",
    not(feature = "driver-p384-scalar-mul")
))]

use core::cell::Cell;

use embassy_crypto::p384 as hw;
use embassy_crypto_driver::{P384AffinePoint, P384Scalar, P384ScalarMul};
use p384::elliptic_curve::ff::PrimeField;
use p384::elliptic_curve::group::Group;
#[cfg(not(feature = "driver-p384-lincomb"))]
use p384::elliptic_curve::group::Curve;
use p384::elliptic_curve::ops::MulByGenerator;
use p384::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};

thread_local! {
    static CALLS: Cell<usize> = const { Cell::new(0) };
    static ZERO_SCALARS: Cell<usize> = const { Cell::new(0) };
}

struct TestDriver;

impl P384ScalarMul for TestDriver {
    fn mul_base(k: P384Scalar) -> P384AffinePoint {
        CALLS.with(|c| c.set(c.get() + 1));
        if k == P384Scalar::default() {
            ZERO_SCALARS.with(|c| c.set(c.get() + 1));
        }

        to_canonical_or_identity(p384::ProjectivePoint::mul_by_generator(&scalar(&k)))
    }

    fn mul_affine(k: P384Scalar, p: P384AffinePoint) -> P384AffinePoint {
        CALLS.with(|c| c.set(c.get() + 1));
        if k == P384Scalar::default() {
            ZERO_SCALARS.with(|c| c.set(c.get() + 1));
        }

        to_canonical_or_identity(point(&p) * scalar(&k))
    }
}

embassy_crypto_driver::p384_scalar_mul_impl!(TestDriver);

#[cfg(not(feature = "driver-p384-scalar-invert"))]
mod invert {
    use super::*;
    use p384::elliptic_curve::ff::Field;
    use p384::elliptic_curve::ops::Invert;

    impl embassy_crypto_driver::P384ScalarInvert for TestDriver {
        fn invert(k: P384Scalar) -> P384Scalar {
            CALLS.with(|c| c.set(c.get() + 1));
            P384Scalar(Field::invert(&scalar(&k)).unwrap().to_repr().into())
        }

        fn invert_vartime(k: P384Scalar) -> P384Scalar {
            CALLS.with(|c| c.set(c.get() + 1));
            P384Scalar(Invert::invert_vartime(&scalar(&k)).unwrap().to_repr().into())
        }
    }

    embassy_crypto_driver::p384_scalar_invert_impl!(TestDriver);
}

#[cfg(not(feature = "driver-p384-lincomb"))]
mod lincomb {
    use super::*;

    impl embassy_crypto_driver::P384Lincomb for TestDriver {
        fn lincomb(
            k1: P384Scalar,
            p1: P384AffinePoint,
            k2: P384Scalar,
            p2: P384AffinePoint,
        ) -> Option<P384AffinePoint> {
            CALLS.with(|c| c.set(c.get() + 1));
            if k1 == P384Scalar::default() {
                ZERO_SCALARS.with(|c| c.set(c.get() + 1));
            }
            if k2 == P384Scalar::default() {
                ZERO_SCALARS.with(|c| c.set(c.get() + 1));
            }

            let sum = point(&p1) * scalar(&k1) + point(&p2) * scalar(&k2);
            (!bool::from(sum.is_identity())).then(|| to_canonical(sum.to_affine()))
        }
    }

    embassy_crypto_driver::p384_lincomb_impl!(TestDriver);
}

fn scalar(k: &P384Scalar) -> p384::Scalar {
    p384::Scalar::from_repr(k.0.into()).expect("scalar in range")
}

fn point(p: &P384AffinePoint) -> p384::AffinePoint {
    p384::AffinePoint::from_encoded_point(&p384::EncodedPoint::from_affine_coordinates(
        &p.x.into(),
        &p.y.into(),
        false,
    ))
    .expect("driver got an off-curve point")
}

fn to_canonical_or_identity(p: p384::ProjectivePoint) -> P384AffinePoint {
    use p384::elliptic_curve::group::prime::PrimeCurveAffine;

    if bool::from(p.to_affine().is_identity()) {
        P384AffinePoint::default()
    } else {
        to_canonical(p.to_affine())
    }
}

fn to_canonical(x: p384::AffinePoint) -> P384AffinePoint {
    let encoded = x.to_encoded_point(false);
    P384AffinePoint {
        x: (*encoded.x().unwrap()).into(),
        y: (*encoded.y().unwrap()).into(),
    }
}

fn assert_eq_points(hw: &hw::ProjectivePoint, sw: &p384::ProjectivePoint) {
    assert_eq!(
        hw.to_affine().to_encoded_point(false).as_bytes(),
        sw.to_affine().to_encoded_point(false).as_bytes()
    );
}

#[test]
fn wrappers_call_the_driver() {
    use p384::elliptic_curve::ops::LinearCombination;

    let k = hw::Scalar::from(12345u64);
    let l = hw::Scalar::from(67890u64);

    let (zero_product, calls, zero_scalars) = {
        CALLS.with(|c| c.set(0));
        ZERO_SCALARS.with(|c| c.set(0));

        // mul_by_generator + Mul<Scalar> (projective and affine) + MulAssign:
        // 4 scalar-mul driver calls.
        let p = hw::ProjectivePoint::mul_by_generator(&k);
        let p = p * k;
        let p = hw::AffinePoint::from(p) * k;
        let mut p = hw::ProjectivePoint::from(p);
        p *= k;

        // LinearCombination: 1 lincomb driver call when the lincomb hook is
        // in use, 2 mul_affine calls otherwise.
        let _ = hw::ProjectivePoint::lincomb(&p, &k, &hw::ProjectivePoint::GENERATOR, &l);

        // A zero scalar reaches the driver as a zero scalar, and the driver
        // reports the identity result with the all-zero sentinel: 1 call.
        let zero_product = p * hw::Scalar::ZERO;

        (
            zero_product,
            CALLS.with(|c| c.get()),
            ZERO_SCALARS.with(|c| c.get()),
        )
    };

    // 4 mul calls + (1 lincomb | 2 mul) + 1 zero-mul.
    let expected_calls = if cfg!(not(feature = "driver-p384-lincomb")) {
        6
    } else {
        7
    };
    assert_eq!(calls, expected_calls, "wrong number of driver calls");
    assert_eq!(zero_scalars, 1, "the zero-scalar call must reach the driver");
    assert!(bool::from(zero_product.is_identity()));
}

#[cfg(not(feature = "driver-p384-scalar-invert"))]
#[test]
fn inversion_calls_the_driver() {
    use p384::elliptic_curve::ff::Field;
    use p384::elliptic_curve::ops::Invert;

    let k = hw::Scalar::from_inner(p384::Scalar::from(987654321u64));

    let (calls, result) = {
        CALLS.with(|c| c.set(0));

        // Scalar::invert (Invert's method) + ff::Field::invert +
        // Invert::invert_vartime: 3 calls.
        let a = k.invert().unwrap();
        let b = Field::invert(&k).unwrap();
        let c = Invert::invert_vartime(&k).unwrap();

        // Zero has no inverse: both forms report absence without dividing by
        // zero in the driver: 2 more calls.
        assert!(bool::from(hw::Scalar::ZERO.invert().is_none()));
        assert!(bool::from(Invert::invert_vartime(&hw::Scalar::ZERO).is_none()));

        // Sanity: the inverses are right.
        assert_eq!(k * a, hw::Scalar::ONE);
        assert_eq!(k * b, hw::Scalar::ONE);
        assert_eq!(k * c, hw::Scalar::ONE);

        (CALLS.with(|c| c.get()), a)
    };

    assert_eq!(calls, 5, "wrong number of driver calls");
    assert_eq!(
        result,
        hw::Scalar::from_inner(p384::Scalar::from(987654321u64).invert().unwrap())
    );
}

#[cfg(not(feature = "driver-p384-lincomb"))]
#[test]
fn lincomb_calls_the_driver() {
    let k1 = hw::Scalar::from(424242u64);
    let k2 = hw::Scalar::from(987654321u64);

    let p1 = hw::ProjectivePoint::mul_by_generator(&hw::Scalar::from(123456789u64));
    let p2 = hw::ProjectivePoint::GENERATOR;

    let sw_p1 = p384::ProjectivePoint::from_encoded_point(
        &p384::EncodedPoint::from_bytes(p1.to_affine().to_encoded_point(false).as_bytes()).unwrap(),
    )
    .unwrap();
    let sw_p2 = p384::ProjectivePoint::from_encoded_point(
        &p384::EncodedPoint::from_bytes(p2.to_affine().to_encoded_point(false).as_bytes()).unwrap(),
    )
    .unwrap();
    let sw_k1 = sw_scalar(&k1);
    let sw_k2 = sw_scalar(&k2);

    let (calls, zero_scalars) = {
        CALLS.with(|c| c.set(0));
        ZERO_SCALARS.with(|c| c.set(0));

        // k1 * P1 + k2 * G: one lincomb driver call.
        let sum = hw::ProjectivePoint::lincomb(&p1, &k1, &p2, &k2);
        assert_eq_points(&sum, &(sw_p1 * sw_k1 + sw_p2 * sw_k2));

        // A zero scalar is passed through as-is (it contributes the identity).
        let sum = hw::ProjectivePoint::lincomb(&p1, &hw::Scalar::ZERO, &p2, &k2);
        assert_eq_points(&sum, &(sw_p2 * sw_k2));

        // A sum of the identity has no affine encoding: the driver reports
        // it as None and the wrapper maps it to the identity point.
        let sum = hw::ProjectivePoint::lincomb(&p1, &k1, &p2, &-k1);
        assert!(bool::from(sum.is_identity()));

        (
            CALLS.with(|c| c.get()),
            ZERO_SCALARS.with(|c| c.get()),
        )
    };

    // 3 lincomb calls; the zero scalar of the second reaches the driver
    // as a zero scalar.
    assert_eq!((calls, zero_scalars), (3, 1));
}

fn sw_scalar(k: &hw::Scalar) -> p384::Scalar {
    p384::Scalar::from_repr(k.to_repr()).unwrap()
}

#[path = "common/p384.rs"]
mod suite;
