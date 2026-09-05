//! NIST P-384 (secp384r1), accelerated.
//!
//! Scalar multiplication and inversion are delegated to the
//! [`P384ScalarMul`], [`P384ScalarInvert`] and [`P384Lincomb`] driver
//! unitraits when the corresponding `driver-*` features are off; with them
//! on, the same operations run in software via the `p384` crate.

use elliptic_curve::FieldBytesEncoding;
use elliptic_curve::bigint::U384;
use p384::elliptic_curve;

use crate::ec::{self, Accelerated, Backend};

/// NIST P-384 elliptic curve, accelerated.
pub type NistP384 = Accelerated<p384::NistP384>;

/// Scalar field element modulo the P-384 curve order.
pub type Scalar = ec::Scalar<p384::NistP384>;

/// Point on the P-384 curve in affine coordinates.
pub type AffinePoint = ec::AffinePoint<p384::NistP384>;

/// Point on the P-384 curve in projective coordinates.
pub type ProjectivePoint = ec::ProjectivePoint<p384::NistP384>;

/// Blinded scalar.
pub type BlindedScalar = elliptic_curve::scalar::BlindedScalar<NistP384>;

/// Compressed SEC1-encoded P-384 curve point.
pub type CompressedPoint = elliptic_curve::sec1::CompressedPoint<NistP384>;

/// SEC1-encoded P-384 curve point.
pub type EncodedPoint = elliptic_curve::sec1::EncodedPoint<NistP384>;

/// Byte array containing a serialized field element value (base field or scalar).
pub type FieldBytes = elliptic_curve::FieldBytes<NistP384>;

/// Non-zero P-384 scalar field element.
pub type NonZeroScalar = elliptic_curve::NonZeroScalar<NistP384>;

/// P-384 public key.
pub type PublicKey = elliptic_curve::PublicKey<NistP384>;

/// P-384 secret key.
pub type SecretKey = elliptic_curve::SecretKey<NistP384>;

/// ECDSA over P-384.
#[cfg(feature = "p384-ecdsa")]
pub mod ecdsa {
    use super::NistP384;

    /// ECDSA/P-384 signature (fixed-size).
    pub type Signature = ecdsa::Signature<NistP384>;

    /// ECDSA/P-384 signing key.
    pub type SigningKey = ecdsa::SigningKey<NistP384>;

    /// ECDSA/P-384 verification key (i.e. public key).
    pub type VerifyingKey = ecdsa::VerifyingKey<NistP384>;
}

impl FieldBytesEncoding<NistP384> for U384 {
    fn decode_field_bytes(field_bytes: &FieldBytes) -> Self {
        <U384 as FieldBytesEncoding<p384::NistP384>>::decode_field_bytes(field_bytes)
    }

    fn encode_field_bytes(&self) -> FieldBytes {
        <U384 as FieldBytesEncoding<p384::NistP384>>::encode_field_bytes(self)
    }
}

impl From<Scalar> for U384 {
    fn from(scalar: Scalar) -> Self {
        U384::from(scalar.into_inner())
    }
}

impl From<&Scalar> for U384 {
    fn from(scalar: &Scalar) -> Self {
        U384::from(scalar.into_inner())
    }
}

impl Backend for p384::NistP384 {
    const AFFINE_IDENTITY: p384::AffinePoint = p384::AffinePoint::IDENTITY;
    const AFFINE_GENERATOR: p384::AffinePoint = p384::AffinePoint::GENERATOR;

    // NOTE: the polarity is intentional, do not "fix" it. With the feature
    // ON, the wrappers run this operation in software directly (converting
    // to the canonical byte form the driver unitraits exchange is only worth
    // paying when an accelerator is behind them), so `ACCELERATED_*` is
    // false and the unitrait is never called; `driver_rustcrypto` still
    // registers an `unreachable!()` impl of it so a HAL that also registers
    // one fails to link. With the feature OFF, the wrappers route through
    // the unitrait and the HAL must provide the impl.
    const ACCELERATED_MUL: bool = cfg!(not(feature = "driver-p384-scalar-mul"));
    const ACCELERATED_INVERT: bool = cfg!(not(feature = "driver-p384-scalar-invert"));
    const ACCELERATED_LINCOMB: bool = cfg!(not(feature = "driver-p384-lincomb"));

    #[cfg(not(feature = "driver-p384-scalar-mul"))]
    fn mul_base(k: &FieldBytes) -> (FieldBytes, FieldBytes) {
        use embassy_crypto_driver::P384ScalarMulImpl;

        point_from_driver(P384ScalarMulImpl::mul_base(scalar_to_driver(k)))
    }

    #[cfg(not(feature = "driver-p384-scalar-mul"))]
    fn mul_affine(k: &FieldBytes, x: &FieldBytes, y: &FieldBytes) -> (FieldBytes, FieldBytes) {
        use embassy_crypto_driver::P384ScalarMulImpl;

        point_from_driver(P384ScalarMulImpl::mul_affine(
            scalar_to_driver(k),
            point_to_driver(x, y),
        ))
    }

    #[cfg(not(feature = "driver-p384-scalar-invert"))]
    fn invert(k: &FieldBytes) -> FieldBytes {
        use embassy_crypto_driver::P384ScalarInvertImpl;

        P384ScalarInvertImpl::invert(scalar_to_driver(k)).0.into()
    }

    #[cfg(not(feature = "driver-p384-scalar-invert"))]
    fn invert_vartime(k: &FieldBytes) -> FieldBytes {
        use embassy_crypto_driver::P384ScalarInvertImpl;

        P384ScalarInvertImpl::invert_vartime(scalar_to_driver(k)).0.into()
    }

    #[cfg(not(feature = "driver-p384-lincomb"))]
    fn lincomb(
        k1: &FieldBytes,
        x1: &FieldBytes,
        y1: &FieldBytes,
        k2: &FieldBytes,
        x2: &FieldBytes,
        y2: &FieldBytes,
    ) -> Option<(FieldBytes, FieldBytes)> {
        use embassy_crypto_driver::P384LincombImpl;

        P384LincombImpl::lincomb(
            scalar_to_driver(k1),
            point_to_driver(x1, y1),
            scalar_to_driver(k2),
            point_to_driver(x2, y2),
        )
        .map(point_from_driver)
    }
}

// NOTE: unlike `p256.rs`, the cfgs below use `not(all(..))` rather than
// `not(any(..))`, so every combination of the three `driver-p384-*` features
// compiles (for example a backend with hardware scalar multiplication but
// software lincomb).
#[cfg(not(all(feature = "driver-p384-scalar-mul", feature = "driver-p384-lincomb")))]
fn point_to_driver(x: &FieldBytes, y: &FieldBytes) -> embassy_crypto_driver::P384AffinePoint {
    embassy_crypto_driver::P384AffinePoint {
        x: (*x).into(),
        y: (*y).into(),
    }
}

#[cfg(not(all(feature = "driver-p384-scalar-mul", feature = "driver-p384-lincomb")))]
fn point_from_driver(p: embassy_crypto_driver::P384AffinePoint) -> (FieldBytes, FieldBytes) {
    (p.x.into(), p.y.into())
}

#[cfg(not(all(
    feature = "driver-p384-scalar-mul",
    feature = "driver-p384-scalar-invert",
    feature = "driver-p384-lincomb",
)))]
fn scalar_to_driver(k: &FieldBytes) -> embassy_crypto_driver::P384Scalar {
    embassy_crypto_driver::P384Scalar((*k).into())
}
