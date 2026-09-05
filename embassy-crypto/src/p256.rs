//! NIST P-256 (secp256r1), accelerated.

use elliptic_curve::FieldBytesEncoding;
use elliptic_curve::bigint::U256;
use p256::elliptic_curve;

use crate::ec::{self, Accelerated, Backend};

/// NIST P-256 elliptic curve, accelerated.
pub type NistP256 = Accelerated<p256::NistP256>;

/// Scalar field element modulo the P-256 curve order.
pub type Scalar = ec::Scalar<p256::NistP256>;

/// Point on the P-256 curve in affine coordinates.
pub type AffinePoint = ec::AffinePoint<p256::NistP256>;

/// Point on the P-256 curve in projective coordinates.
pub type ProjectivePoint = ec::ProjectivePoint<p256::NistP256>;

/// Blinded scalar.
pub type BlindedScalar = elliptic_curve::scalar::BlindedScalar<NistP256>;

/// Compressed SEC1-encoded P-256 curve point.
pub type CompressedPoint = elliptic_curve::sec1::CompressedPoint<NistP256>;

/// SEC1-encoded P-256 curve point.
pub type EncodedPoint = elliptic_curve::sec1::EncodedPoint<NistP256>;

/// Byte array containing a serialized field element value (base field or scalar).
pub type FieldBytes = elliptic_curve::FieldBytes<NistP256>;

/// Non-zero P-256 scalar field element.
pub type NonZeroScalar = elliptic_curve::NonZeroScalar<NistP256>;

/// P-256 public key.
pub type PublicKey = elliptic_curve::PublicKey<NistP256>;

/// P-256 secret key.
pub type SecretKey = elliptic_curve::SecretKey<NistP256>;

/// ECDSA over P-256.
#[cfg(feature = "p256-ecdsa")]
pub mod ecdsa {
    use super::NistP256;

    /// ECDSA/P-256 signature (fixed-size).
    pub type Signature = ecdsa::Signature<NistP256>;

    /// ECDSA/P-256 signing key.
    pub type SigningKey = ecdsa::SigningKey<NistP256>;

    /// ECDSA/P-256 verification key (i.e. public key).
    pub type VerifyingKey = ecdsa::VerifyingKey<NistP256>;
}

impl FieldBytesEncoding<NistP256> for U256 {
    fn decode_field_bytes(field_bytes: &FieldBytes) -> Self {
        <U256 as FieldBytesEncoding<p256::NistP256>>::decode_field_bytes(field_bytes)
    }

    fn encode_field_bytes(&self) -> FieldBytes {
        <U256 as FieldBytesEncoding<p256::NistP256>>::encode_field_bytes(self)
    }
}

impl From<Scalar> for U256 {
    fn from(scalar: Scalar) -> Self {
        U256::from(scalar.into_inner())
    }
}

impl From<&Scalar> for U256 {
    fn from(scalar: &Scalar) -> Self {
        U256::from(scalar.into_inner())
    }
}

impl Backend for p256::NistP256 {
    const AFFINE_IDENTITY: p256::AffinePoint = p256::AffinePoint::IDENTITY;
    const AFFINE_GENERATOR: p256::AffinePoint = p256::AffinePoint::GENERATOR;

    const ACCELERATED_MUL: bool = cfg!(not(feature = "driver-p256-scalar-mul"));
    const ACCELERATED_INVERT: bool = cfg!(not(feature = "driver-p256-scalar-invert"));
    const ACCELERATED_LINCOMB: bool = cfg!(not(feature = "driver-p256-lincomb"));

    #[cfg(not(feature = "driver-p256-scalar-mul"))]
    fn mul_base(k: &FieldBytes) -> (FieldBytes, FieldBytes) {
        use embassy_crypto_driver::P256ScalarMulImpl;

        point_from_driver(P256ScalarMulImpl::mul_base(scalar_to_driver(k)))
    }

    #[cfg(not(feature = "driver-p256-scalar-mul"))]
    fn mul_affine(k: &FieldBytes, x: &FieldBytes, y: &FieldBytes) -> (FieldBytes, FieldBytes) {
        use embassy_crypto_driver::P256ScalarMulImpl;

        point_from_driver(P256ScalarMulImpl::mul_affine(
            scalar_to_driver(k),
            point_to_driver(x, y),
        ))
    }

    #[cfg(not(feature = "driver-p256-scalar-invert"))]
    fn invert(k: &FieldBytes) -> FieldBytes {
        use embassy_crypto_driver::P256ScalarInvertImpl;

        P256ScalarInvertImpl::invert(scalar_to_driver(k)).0.into()
    }

    #[cfg(not(feature = "driver-p256-scalar-invert"))]
    fn invert_vartime(k: &FieldBytes) -> FieldBytes {
        use embassy_crypto_driver::P256ScalarInvertImpl;

        P256ScalarInvertImpl::invert_vartime(scalar_to_driver(k)).0.into()
    }

    #[cfg(not(feature = "driver-p256-lincomb"))]
    fn lincomb(
        k1: &FieldBytes,
        x1: &FieldBytes,
        y1: &FieldBytes,
        k2: &FieldBytes,
        x2: &FieldBytes,
        y2: &FieldBytes,
    ) -> Option<(FieldBytes, FieldBytes)> {
        use embassy_crypto_driver::P256LincombImpl;

        P256LincombImpl::lincomb(
            scalar_to_driver(k1),
            point_to_driver(x1, y1),
            scalar_to_driver(k2),
            point_to_driver(x2, y2),
        )
        .map(point_from_driver)
    }
}

#[cfg(not(any(feature = "driver-p256-scalar-mul", feature = "driver-p256-lincomb")))]
fn point_to_driver(x: &FieldBytes, y: &FieldBytes) -> embassy_crypto_driver::P256AffinePoint {
    embassy_crypto_driver::P256AffinePoint {
        x: (*x).into(),
        y: (*y).into(),
    }
}

#[cfg(not(any(feature = "driver-p256-scalar-mul", feature = "driver-p256-lincomb")))]
fn point_from_driver(p: embassy_crypto_driver::P256AffinePoint) -> (FieldBytes, FieldBytes) {
    (p.x.into(), p.y.into())
}

#[cfg(not(any(
    feature = "driver-p256-scalar-mul",
    feature = "driver-p256-scalar-invert",
    feature = "driver-p256-lincomb",
)))]
fn scalar_to_driver(k: &FieldBytes) -> embassy_crypto_driver::P256Scalar {
    embassy_crypto_driver::P256Scalar((*k).into())
}
