//! NIST P-256 (secp256r1) over the RustCrypto traits, with the expensive
//! operations routed to `embassy-crypto-driver` accelerators.
//!
//! [`NistP256`] implements [`CurveArithmetic`], so the generic
//! `elliptic-curve` and `ecdsa` machinery (`PublicKey`, `SecretKey`,
//! `ecdh::diffie_hellman`, `ecdsa::SigningKey`, ...) works with it unchanged.
//!
//! The types wrap the `p256` crate's types and delegate every trait to them,
//! except for these entry points, which go to a driver:
//!
//! - Scalar multiplication: [`embassy_crypto_driver::P256ScalarMul`], always.
//!   With the `driver-p256-scalar-mul` feature (software fallback) the
//!   wrappers call `p256` directly instead.
//! - Scalar inversion: [`embassy_crypto_driver::P256ScalarInvert`], with the
//!   `p256-scalar-invert` feature.
//! - `k1 * P1 + k2 * P2` (ECDSA verification): [`embassy_crypto_driver::P256Lincomb`],
//!   with the `p256-lincomb` feature.
//!
//! The optional features require an implementation at link time. Everything
//! else (scalar field, point addition and doubling, encoding, validation) runs
//! in software via `p256`.
//!
//! Drivers exchange canonical big-endian bytes and affine points. A
//! [`ProjectivePoint`] that came from affine input or from a driver keeps its
//! affine form, so converting it back is free.

mod affine;
mod projective;
mod scalar;

use elliptic_curve::bigint::U256;
use elliptic_curve::consts::U32;
#[cfg(any(
    not(feature = "driver-p256-scalar-mul"),
    feature = "p256-scalar-invert",
    feature = "p256-lincomb"
))]
use elliptic_curve::ff::Field as _;
use elliptic_curve::point::{PointCompaction, PointCompression};
#[cfg(any(not(feature = "driver-p256-scalar-mul"), feature = "p256-lincomb"))]
use elliptic_curve::sec1::{FromEncodedPoint as _, ToEncodedPoint as _};
#[cfg(any(
    not(feature = "driver-p256-scalar-mul"),
    feature = "p256-scalar-invert",
    feature = "p256-lincomb"
))]
use elliptic_curve::subtle::ConditionallySelectable;
use elliptic_curve::subtle::CtOption;
use elliptic_curve::{Curve, CurveArithmetic, FieldBytesEncoding, PrimeCurve, PrimeCurveArithmetic};
#[cfg(any(not(feature = "driver-p256-scalar-mul"), feature = "p256-lincomb"))]
use embassy_crypto_driver::P256AffinePoint;
#[cfg(any(
    not(feature = "driver-p256-scalar-mul"),
    feature = "p256-scalar-invert",
    feature = "p256-lincomb"
))]
use embassy_crypto_driver::P256Scalar;
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
// Driver-backed operations
//
// These are the only functions in this module that do anything but delegate.
//
// Scalar multiplication always goes through the `P256ScalarMul` driver,
// except with the `driver-p256-scalar-mul` feature (software fallback, no
// hardware), where it calls the `p256` crate directly so the software path
// costs exactly what plain `p256` costs: no canonical-encoding round trip
// and no extra inversion.
//
// Scalar inversion and two-term linear combinations go through the
// `P256ScalarInvert` and `P256Lincomb` drivers only when the corresponding
// `p256-scalar-invert` / `p256-lincomb` feature is enabled.
//
// The driver contracts exclude the degenerate operands (zero scalar, identity
// point) since neither has a canonical encoding, so they are handled here,
// without branching on the operands.
// ===========================================================================

/// `k * G`, in software.
#[cfg(feature = "driver-p256-scalar-mul")]
fn mul_base(k: &Scalar) -> ProjectivePoint {
    use elliptic_curve::ops::MulByGenerator;

    ProjectivePoint::from_projective(p256::ProjectivePoint::mul_by_generator(&k.0))
}

/// `k * P`, in software.
#[cfg(feature = "driver-p256-scalar-mul")]
fn mul(k: &Scalar, p: &AffinePoint) -> ProjectivePoint {
    ProjectivePoint::from_projective(p.0 * k.0)
}

/// `k * P` for a projective `P`, in software.
#[cfg(feature = "driver-p256-scalar-mul")]
fn mul_projective(k: &Scalar, p: &ProjectivePoint) -> ProjectivePoint {
    ProjectivePoint::from_projective(p256::ProjectivePoint::from(*p) * k.0)
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

    ProjectivePoint::conditional_select(&result, &ProjectivePoint::IDENTITY, k_is_zero)
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

    ProjectivePoint::conditional_select(&result, &ProjectivePoint::IDENTITY, degenerate)
}

/// `k * P` via the driver, for a projective `P`.
///
/// Free of an inversion when `P` came from affine input or from the driver.
#[cfg(not(feature = "driver-p256-scalar-mul"))]
fn mul_projective(k: &Scalar, p: &ProjectivePoint) -> ProjectivePoint {
    mul(k, &AffinePoint(p.affine()))
}

/// `k^-1`, in software.
#[cfg(not(feature = "p256-scalar-invert"))]
fn invert(k: &Scalar) -> CtOption<Scalar> {
    <p256::Scalar as elliptic_curve::ff::Field>::invert(&k.0).map(Scalar)
}

/// `k^-1`, variable time, in software.
#[cfg(not(feature = "p256-scalar-invert"))]
fn invert_vartime(k: &Scalar) -> CtOption<Scalar> {
    <p256::Scalar as elliptic_curve::ops::Invert>::invert_vartime(&k.0).map(Scalar)
}

/// `k^-1` via the driver.
#[cfg(feature = "p256-scalar-invert")]
fn invert(k: &Scalar) -> CtOption<Scalar> {
    let k_is_zero = k.0.is_zero();

    // Feed the driver a scalar that is always invertible; the result is
    // reported as absent if `k` was zero.
    let k = p256::Scalar::conditional_select(&k.0, &p256::Scalar::ONE, k_is_zero);

    let result = from_canonical_scalar(&embassy_crypto_driver::P256ScalarInvertImpl::invert(
        &to_canonical_scalar(&k),
    ));

    CtOption::new(Scalar(result), !k_is_zero)
}

/// `k^-1` via the driver, variable time.
#[cfg(feature = "p256-scalar-invert")]
fn invert_vartime(k: &Scalar) -> CtOption<Scalar> {
    let k_is_zero = k.0.is_zero();
    let k = p256::Scalar::conditional_select(&k.0, &p256::Scalar::ONE, k_is_zero);

    let result = from_canonical_scalar(&embassy_crypto_driver::P256ScalarInvertImpl::invert_vartime(
        &to_canonical_scalar(&k),
    ));

    CtOption::new(Scalar(result), !k_is_zero)
}

/// `k1 * p1 + k2 * p2`: two driver multiplications and a software addition.
#[cfg(all(not(feature = "p256-lincomb"), not(feature = "driver-p256-scalar-mul")))]
fn lincomb(p1: &ProjectivePoint, k1: &Scalar, p2: &ProjectivePoint, k2: &Scalar) -> ProjectivePoint {
    (*p1 * k1) + (*p2 * k2)
}

/// `k1 * p1 + k2 * p2`, in software.
#[cfg(all(not(feature = "p256-lincomb"), feature = "driver-p256-scalar-mul"))]
fn lincomb(p1: &ProjectivePoint, k1: &Scalar, p2: &ProjectivePoint, k2: &Scalar) -> ProjectivePoint {
    use elliptic_curve::ops::LinearCombination;

    ProjectivePoint::from_projective(p256::ProjectivePoint::lincomb(
        &p256::ProjectivePoint::from(*p1),
        &k1.0,
        &p256::ProjectivePoint::from(*p2),
        &k2.0,
    ))
}

/// `k1 * p1 + k2 * p2` via the driver.
#[cfg(feature = "p256-lincomb")]
fn lincomb(p1: &ProjectivePoint, k1: &Scalar, p2: &ProjectivePoint, k2: &Scalar) -> ProjectivePoint {
    let p1 = p1.affine();
    let p2 = p2.affine();

    let degenerate1 = k1.0.is_zero() | p1.is_identity();
    let degenerate2 = k2.0.is_zero() | p2.is_identity();
    let degenerate = degenerate1 | degenerate2;

    // A degenerate term contributes nothing. Feed the driver `1 * P'` in its
    // place, where `P'` is the point itself or `G` if that was the identity,
    // then subtract `P'` again from the sum. The subtraction is always
    // computed and only selected when needed, so nothing depends on the
    // operands.
    let k1 = p256::Scalar::conditional_select(&k1.0, &p256::Scalar::ONE, degenerate1);
    let k2 = p256::Scalar::conditional_select(&k2.0, &p256::Scalar::ONE, degenerate2);
    let p1 = p256::AffinePoint::conditional_select(&p1, &p256::AffinePoint::GENERATOR, p1.is_identity());
    let p2 = p256::AffinePoint::conditional_select(&p2, &p256::AffinePoint::GENERATOR, p2.is_identity());

    let sum = embassy_crypto_driver::P256LincombImpl::lincomb(
        &to_canonical_scalar(&k1),
        &to_canonical_point(&p1),
        &to_canonical_scalar(&k2),
        &to_canonical_point(&p2),
    );

    // Whether the sum is the identity is not secret (see the driver contract).
    let sum = match sum {
        Some(sum) => from_canonical_point(&sum),
        None => ProjectivePoint::IDENTITY,
    };

    let extra1 = p256::AffinePoint::conditional_select(&p256::AffinePoint::IDENTITY, &p1, degenerate1);
    let extra2 = p256::AffinePoint::conditional_select(&p256::AffinePoint::IDENTITY, &p2, degenerate2);
    let fixed = sum - AffinePoint(extra1) - AffinePoint(extra2);

    ProjectivePoint::conditional_select(&sum, &fixed, degenerate)
}

#[cfg(any(
    not(feature = "driver-p256-scalar-mul"),
    feature = "p256-scalar-invert",
    feature = "p256-lincomb"
))]
fn to_canonical_scalar(k: &p256::Scalar) -> P256Scalar {
    P256Scalar(k.to_bytes().into())
}

/// Decode a scalar returned by the driver.
///
/// The driver contract guarantees it is in range, so a failure here is a
/// broken driver.
#[cfg(feature = "p256-scalar-invert")]
fn from_canonical_scalar(k: &P256Scalar) -> p256::Scalar {
    <p256::Scalar as elliptic_curve::ff::PrimeField>::from_repr(k.0.into())
        .expect("P256ScalarInvert driver returned an out-of-range scalar")
}

/// `p` must not be the identity.
#[cfg(any(not(feature = "driver-p256-scalar-mul"), feature = "p256-lincomb"))]
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

/// Decode a point returned by the driver; the result remembers its affine form.
///
/// The decode validates that the point is on the curve; the driver contract
/// guarantees it, so a failure here is a broken driver.
#[cfg(any(not(feature = "driver-p256-scalar-mul"), feature = "p256-lincomb"))]
fn from_canonical_point(p: &P256AffinePoint) -> ProjectivePoint {
    let encoded = p256::EncodedPoint::from_affine_coordinates(&p.x.into(), &p.y.into(), false);

    let affine = p256::AffinePoint::from_encoded_point(&encoded).expect("driver returned an invalid point");

    ProjectivePoint::from_affine(affine)
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
