//! P-256 point in affine coordinates.
//!
//! A newtype around `p256::AffinePoint`; every trait is implemented by
//! delegation, except scalar multiplication which goes through the driver.

use core::borrow::Borrow;
use core::fmt;
use core::ops::{Mul, Neg};

use elliptic_curve::Error;
use elliptic_curve::group::GroupEncoding;
use elliptic_curve::group::prime::PrimeCurveAffine;
use elliptic_curve::point::{AffineCoordinates, DecompactPoint, DecompressPoint};
use elliptic_curve::sec1::{FromEncodedPoint, ToCompactEncodedPoint, ToEncodedPoint};
use elliptic_curve::subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};
use elliptic_curve::zeroize::DefaultIsZeroes;
use p256::elliptic_curve;

use super::{CompressedPoint, EncodedPoint, FieldBytes, NistP256, ProjectivePoint, PublicKey, Scalar};

/// Point on the P-256 curve in affine coordinates.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct AffinePoint(pub(crate) p256::AffinePoint);

impl AffinePoint {
    /// Additive identity of the group a.k.a. the point at infinity.
    pub const IDENTITY: Self = Self(p256::AffinePoint::IDENTITY);

    /// Base point of the curve.
    pub const GENERATOR: Self = Self(p256::AffinePoint::GENERATOR);
}

impl From<p256::AffinePoint> for AffinePoint {
    fn from(point: p256::AffinePoint) -> Self {
        Self(point)
    }
}

impl From<AffinePoint> for p256::AffinePoint {
    fn from(point: AffinePoint) -> Self {
        point.0
    }
}

impl fmt::Debug for AffinePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AffineCoordinates for AffinePoint {
    type FieldRepr = FieldBytes;

    fn x(&self) -> FieldBytes {
        self.0.x()
    }

    fn y_is_odd(&self) -> Choice {
        self.0.y_is_odd()
    }
}

impl ConditionallySelectable for AffinePoint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(p256::AffinePoint::conditional_select(&a.0, &b.0, choice))
    }
}

impl ConstantTimeEq for AffinePoint {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl DefaultIsZeroes for AffinePoint {}

impl DecompressPoint<NistP256> for AffinePoint {
    fn decompress(x: &FieldBytes, y_is_odd: Choice) -> CtOption<Self> {
        <p256::AffinePoint as DecompressPoint<p256::NistP256>>::decompress(x, y_is_odd).map(Self)
    }
}

impl DecompactPoint<NistP256> for AffinePoint {
    fn decompact(x: &FieldBytes) -> CtOption<Self> {
        <p256::AffinePoint as DecompactPoint<p256::NistP256>>::decompact(x).map(Self)
    }
}

impl FromEncodedPoint<NistP256> for AffinePoint {
    fn from_encoded_point(point: &EncodedPoint) -> CtOption<Self> {
        <p256::AffinePoint as FromEncodedPoint<p256::NistP256>>::from_encoded_point(point).map(Self)
    }
}

impl ToEncodedPoint<NistP256> for AffinePoint {
    fn to_encoded_point(&self, compress: bool) -> EncodedPoint {
        <p256::AffinePoint as ToEncodedPoint<p256::NistP256>>::to_encoded_point(&self.0, compress)
    }
}

impl ToCompactEncodedPoint<NistP256> for AffinePoint {
    fn to_compact_encoded_point(&self) -> CtOption<EncodedPoint> {
        <p256::AffinePoint as ToCompactEncodedPoint<p256::NistP256>>::to_compact_encoded_point(&self.0)
    }
}

impl From<ProjectivePoint> for AffinePoint {
    fn from(point: ProjectivePoint) -> Self {
        Self(point.affine())
    }
}

impl From<&ProjectivePoint> for AffinePoint {
    fn from(point: &ProjectivePoint) -> Self {
        Self(point.affine())
    }
}

impl From<PublicKey> for AffinePoint {
    fn from(public_key: PublicKey) -> Self {
        *public_key.as_affine()
    }
}

impl From<&PublicKey> for AffinePoint {
    fn from(public_key: &PublicKey) -> Self {
        *public_key.as_affine()
    }
}

impl From<AffinePoint> for EncodedPoint {
    fn from(point: AffinePoint) -> Self {
        EncodedPoint::from(point.0)
    }
}

impl GroupEncoding for AffinePoint {
    type Repr = CompressedPoint;

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        <p256::AffinePoint as GroupEncoding>::from_bytes(bytes).map(Self)
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        <p256::AffinePoint as GroupEncoding>::from_bytes_unchecked(bytes).map(Self)
    }

    fn to_bytes(&self) -> Self::Repr {
        <p256::AffinePoint as GroupEncoding>::to_bytes(&self.0)
    }
}

impl PrimeCurveAffine for AffinePoint {
    type Curve = ProjectivePoint;
    type Scalar = Scalar;

    fn identity() -> Self {
        Self(<p256::AffinePoint as PrimeCurveAffine>::identity())
    }

    fn generator() -> Self {
        Self(<p256::AffinePoint as PrimeCurveAffine>::generator())
    }

    fn is_identity(&self) -> Choice {
        PrimeCurveAffine::is_identity(&self.0)
    }

    fn to_curve(&self) -> ProjectivePoint {
        ProjectivePoint::from_affine(self.0)
    }
}

impl TryFrom<EncodedPoint> for AffinePoint {
    type Error = Error;

    fn try_from(point: EncodedPoint) -> Result<Self, Error> {
        p256::AffinePoint::try_from(point).map(Self)
    }
}

impl TryFrom<&EncodedPoint> for AffinePoint {
    type Error = Error;

    fn try_from(point: &EncodedPoint) -> Result<Self, Error> {
        p256::AffinePoint::try_from(point).map(Self)
    }
}

impl TryFrom<AffinePoint> for PublicKey {
    type Error = Error;

    fn try_from(point: AffinePoint) -> Result<Self, Error> {
        PublicKey::from_affine(point)
    }
}

impl TryFrom<&AffinePoint> for PublicKey {
    type Error = Error;

    fn try_from(point: &AffinePoint) -> Result<Self, Error> {
        PublicKey::from_affine(*point)
    }
}

/// Scalar multiplication, via the `P256ScalarMul` driver.
impl<S> Mul<S> for AffinePoint
where
    S: Borrow<Scalar>,
{
    type Output = ProjectivePoint;

    fn mul(self, scalar: S) -> ProjectivePoint {
        super::mul(scalar.borrow(), &self)
    }
}

impl Neg for AffinePoint {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl Neg for &AffinePoint {
    type Output = AffinePoint;

    fn neg(self) -> AffinePoint {
        AffinePoint(-self.0)
    }
}
