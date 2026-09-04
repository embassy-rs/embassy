//! Software fallback for the `P256ScalarMul` driver, implemented with the
//! `p256` crate.

struct P256ScalarMulDriver;

impl embassy_crypto_driver::P256ScalarMul for P256ScalarMulDriver {
    fn mul_base(k: &embassy_crypto_driver::P256Scalar) -> embassy_crypto_driver::P256AffinePoint {
        use p256::elliptic_curve::ops::MulByGenerator;

        p256_affine_to_canonical(&p256::ProjectivePoint::mul_by_generator(&p256_scalar_from_canonical(k)).to_affine())
    }

    fn mul_affine(
        k: &embassy_crypto_driver::P256Scalar,
        p: &embassy_crypto_driver::P256AffinePoint,
    ) -> embassy_crypto_driver::P256AffinePoint {
        p256_affine_to_canonical(&(p256_affine_from_canonical(p) * p256_scalar_from_canonical(k)).to_affine())
    }
}

embassy_crypto_driver::p256_scalar_mul_impl!(P256ScalarMulDriver);

/// Contract: `k` is in `[1, n-1]`.
fn p256_scalar_from_canonical(k: &embassy_crypto_driver::P256Scalar) -> p256::Scalar {
    use p256::elliptic_curve::ff::PrimeField;

    p256::Scalar::from_repr(k.0.into()).expect("scalar not in range")
}

/// Contract: `p` is a valid on-curve point.
fn p256_affine_from_canonical(p: &embassy_crypto_driver::P256AffinePoint) -> p256::AffinePoint {
    use p256::elliptic_curve::sec1::FromEncodedPoint;

    p256::AffinePoint::from_encoded_point(&p256::EncodedPoint::from_affine_coordinates(
        &p.x.into(),
        &p.y.into(),
        false,
    ))
    .expect("point not on curve")
}

/// Contract: `p` is not the identity.
fn p256_affine_to_canonical(p: &p256::AffinePoint) -> embassy_crypto_driver::P256AffinePoint {
    use p256::elliptic_curve::sec1::ToEncodedPoint;

    let encoded = p.to_encoded_point(false);

    embassy_crypto_driver::P256AffinePoint {
        x: (*encoded.x().expect("not the identity")).into(),
        y: (*encoded.y().expect("not the identity")).into(),
    }
}
