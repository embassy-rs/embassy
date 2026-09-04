//! NIST P-256 (secp256r1) with hardware-accelerated scalar multiplication.
//!
//! This module provides the RustCrypto [`elliptic_curve`] trait stack
//! ([`CurveArithmetic`], [`group::Group`], [`ff::Field`], ...) for P-256 in a way
//! that lets a hardware backend take over the one operation that matters for
//! performance: scalar multiplication.
//!
//! The types here are thin newtype wrappers around the `p256` crate's types.
//! Every trait the wrapped type implements is implemented on the wrapper by
//! delegation, except for the handful of scalar-multiplication entry points,
//! which are routed to the [`embassy_crypto_driver::P256ScalarMul`] unitrait
//! instead (see [`hw`](self)). All other arithmetic (scalar field, point
//! addition/doubling, encoding) runs in software via the `p256` crate.
//!
//! Because the wrappers implement [`CurveArithmetic`], the generic
//! `elliptic-curve` / `ecdsa` machinery (`PublicKey`, `SecretKey`,
//! `NonZeroScalar`, `ecdh::diffie_hellman`, `ecdsa::SigningKey`, ...) works
//! with [`NistP256`] unchanged, and transparently benefits from the accelerated
//! scalar multiplication.
//!
//! [`ff::Field`]: elliptic_curve::ff::Field
//! [`group::Group`]: elliptic_curve::group::Group

mod affine;
mod projective;
mod scalar;

use elliptic_curve::bigint::U256;
use elliptic_curve::consts::U32;
#[cfg(not(feature = "driver-p256-scalar-mul"))]
use elliptic_curve::ff::Field as _;
use elliptic_curve::point::{PointCompaction, PointCompression};
#[cfg(not(feature = "driver-p256-scalar-mul"))]
use elliptic_curve::sec1::{FromEncodedPoint as _, ToEncodedPoint as _};
#[cfg(not(feature = "driver-p256-scalar-mul"))]
use elliptic_curve::subtle::ConditionallySelectable;
use elliptic_curve::{Curve, CurveArithmetic, FieldBytesEncoding, PrimeCurve, PrimeCurveArithmetic};
#[cfg(not(feature = "driver-p256-scalar-mul"))]
use embassy_crypto_driver::{P256AffinePoint, P256Scalar};
use p256::elliptic_curve;

pub use affine::AffinePoint;
pub use projective::ProjectivePoint;
pub use scalar::Scalar;

/// NIST P-256 elliptic curve.
///
/// Same curve as `p256::NistP256`, but with [`CurveArithmetic`] types whose
/// scalar multiplication goes through the `P256ScalarMul` driver.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NistP256;

impl Curve for NistP256 {
    type FieldBytesSize = U32;
    type Uint = U256;

    const ORDER: U256 = <p256::NistP256 as Curve>::ORDER;
}

impl PrimeCurve for NistP256 {}

impl PointCompression for NistP256 {
    const COMPRESS_POINTS: bool = <p256::NistP256 as PointCompression>::COMPRESS_POINTS;
}

impl PointCompaction for NistP256 {
    const COMPACT_POINTS: bool = <p256::NistP256 as PointCompaction>::COMPACT_POINTS;
}

impl FieldBytesEncoding<NistP256> for U256 {
    fn decode_field_bytes(field_bytes: &FieldBytes) -> Self {
        <U256 as FieldBytesEncoding<p256::NistP256>>::decode_field_bytes(field_bytes)
    }

    fn encode_field_bytes(&self) -> FieldBytes {
        <U256 as FieldBytesEncoding<p256::NistP256>>::encode_field_bytes(self)
    }
}

impl CurveArithmetic for NistP256 {
    type AffinePoint = AffinePoint;
    type ProjectivePoint = ProjectivePoint;
    type Scalar = Scalar;
}

impl PrimeCurveArithmetic for NistP256 {
    type CurveGroup = ProjectivePoint;
}

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
    use ecdsa::hazmat::{DigestPrimitive, SignPrimitive, VerifyPrimitive};

    use super::{AffinePoint, NistP256, Scalar};

    /// ECDSA/P-256 signature (fixed-size).
    pub type Signature = ecdsa::Signature<NistP256>;

    /// ECDSA/P-256 signing key.
    pub type SigningKey = ecdsa::SigningKey<NistP256>;

    /// ECDSA/P-256 verification key (i.e. public key).
    pub type VerifyingKey = ecdsa::VerifyingKey<NistP256>;

    impl DigestPrimitive for NistP256 {
        type Digest = <p256::NistP256 as DigestPrimitive>::Digest;
    }

    impl SignPrimitive<NistP256> for Scalar {}

    impl VerifyPrimitive<NistP256> for AffinePoint {}
}

// ===========================================================================
// Scalar multiplication
//
// These are the only functions in this module that do anything but delegate.
//
// With the `driver-p256-scalar-mul` feature (software fallback, no hardware),
// they call the `p256` crate directly, so the software path costs exactly what
// plain `p256` costs: no canonical-encoding round trip and no extra inversion.
//
// Otherwise they go through the `P256ScalarMul` driver. The driver contract
// excludes the two degenerate cases (zero scalar, identity point) since neither
// has a canonical encoding, so they are handled here, without branching on the
// operands.
// ===========================================================================

/// `k * G`, in software.
#[cfg(feature = "driver-p256-scalar-mul")]
fn mul_base(k: &Scalar) -> ProjectivePoint {
    use elliptic_curve::ops::MulByGenerator;

    ProjectivePoint(p256::ProjectivePoint::mul_by_generator(&k.0))
}

/// `k * P`, in software.
#[cfg(feature = "driver-p256-scalar-mul")]
fn mul(k: &Scalar, p: &AffinePoint) -> ProjectivePoint {
    ProjectivePoint(p.0 * k.0)
}

/// `k * P` for a projective `P`, in software.
#[cfg(feature = "driver-p256-scalar-mul")]
fn mul_projective(k: &Scalar, p: &ProjectivePoint) -> ProjectivePoint {
    ProjectivePoint(p.0 * k.0)
}

/// `k * G` via the driver.
#[cfg(not(feature = "driver-p256-scalar-mul"))]
fn mul_base(k: &Scalar) -> ProjectivePoint {
    let k_is_zero = k.0.is_zero();

    // Feed the driver a scalar that is always in range; the result is then
    // discarded (replaced with the identity) if `k` was zero.
    let k = p256::Scalar::conditional_select(&k.0, &p256::Scalar::ONE, k_is_zero);

    let result = from_canonical_point(&embassy_crypto_driver::P256ScalarMulImpl::mul_base(
        &to_canonical_scalar(&k),
    ));

    ProjectivePoint(p256::ProjectivePoint::conditional_select(
        &result,
        &p256::ProjectivePoint::IDENTITY,
        k_is_zero,
    ))
}

/// `k * P` via the driver.
#[cfg(not(feature = "driver-p256-scalar-mul"))]
fn mul(k: &Scalar, p: &AffinePoint) -> ProjectivePoint {
    let k_is_zero = k.0.is_zero();
    let p_is_identity = p.0.is_identity();
    let degenerate = k_is_zero | p_is_identity;

    // Feed the driver operands that are always within its contract; the
    // result is then discarded (replaced with the identity) if either of the
    // original operands was degenerate.
    let k = p256::Scalar::conditional_select(&k.0, &p256::Scalar::ONE, k_is_zero);
    let p = p256::AffinePoint::conditional_select(&p.0, &p256::AffinePoint::GENERATOR, p_is_identity);

    let result = from_canonical_point(&embassy_crypto_driver::P256ScalarMulImpl::mul_affine(
        &to_canonical_scalar(&k),
        &to_canonical_point(&p),
    ));

    ProjectivePoint(p256::ProjectivePoint::conditional_select(
        &result,
        &p256::ProjectivePoint::IDENTITY,
        degenerate,
    ))
}

/// `k * P` via the driver, for a projective `P`.
#[cfg(not(feature = "driver-p256-scalar-mul"))]
fn mul_projective(k: &Scalar, p: &ProjectivePoint) -> ProjectivePoint {
    mul(k, &AffinePoint(p.0.to_affine()))
}

#[cfg(not(feature = "driver-p256-scalar-mul"))]
fn to_canonical_scalar(k: &p256::Scalar) -> P256Scalar {
    P256Scalar(k.to_bytes().into())
}

/// `p` must not be the identity.
#[cfg(not(feature = "driver-p256-scalar-mul"))]
fn to_canonical_point(p: &p256::AffinePoint) -> P256AffinePoint {
    let encoded = p.to_encoded_point(false);

    // Only the identity encodes without coordinates, and the callers
    // never pass the identity.
    let x = encoded.x().expect("not the identity");
    let y = encoded.y().expect("not the identity");

    P256AffinePoint {
        x: (*x).into(),
        y: (*y).into(),
    }
}

/// Decode a point returned by the driver.
///
/// The decode validates that the point is on the curve; the driver contract
/// guarantees it, so a failure here is a broken driver.
#[cfg(not(feature = "driver-p256-scalar-mul"))]
fn from_canonical_point(p: &P256AffinePoint) -> p256::ProjectivePoint {
    let encoded = p256::EncodedPoint::from_affine_coordinates(&p.x.into(), &p.y.into(), false);

    let affine =
        p256::AffinePoint::from_encoded_point(&encoded).expect("P256ScalarMul driver returned an invalid point");

    p256::ProjectivePoint::from(affine)
}

// Prove at compile time that the wrappers satisfy every bound the generic
// `elliptic-curve` machinery needs.
#[allow(dead_code)]
fn assert_bounds() {
    fn is_curve_arithmetic<C: CurveArithmetic>() {}
    fn is_prime_curve_arithmetic<C: PrimeCurveArithmetic>() {}
    fn is_prime_curve_affine<T: elliptic_curve::group::prime::PrimeCurveAffine>() {}
    fn is_group_prime_curve<T: elliptic_curve::group::prime::PrimeCurve>() {}
    fn is_cofactor_group<T: elliptic_curve::group::cofactor::CofactorGroup>() {}
    fn is_field<T: elliptic_curve::ff::Field>() {}
    fn is_prime_field<T: elliptic_curve::ff::PrimeField>() {}

    is_curve_arithmetic::<NistP256>();
    is_prime_curve_arithmetic::<NistP256>();
    is_prime_curve_affine::<AffinePoint>();
    is_group_prime_curve::<ProjectivePoint>();
    is_cofactor_group::<ProjectivePoint>();
    is_field::<Scalar>();
    is_prime_field::<Scalar>();
}
