//! NIST P-384 (secp384r1).
//!
//! Pure software backend: `embassy-crypto-driver` has no P-384 driver traits
//! yet, so every [`Backend`] operation runs through the `p384` crate's
//! arithmetic. The module exists so generic code over [`NistP384`] can be
//! written now, and transparently picks up a driver if one is added later.

use elliptic_curve::FieldBytesEncoding;
use elliptic_curve::bigint::U384;
use p384::elliptic_curve;

use crate::ec::{self, Accelerated, Backend};

/// NIST P-384 elliptic curve.
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

    // No P-384 driver traits exist yet; the default (software) `Backend`
    // methods handle everything.
    const ACCELERATED_MUL: bool = false;
    const ACCELERATED_INVERT: bool = false;
    const ACCELERATED_LINCOMB: bool = false;
}
