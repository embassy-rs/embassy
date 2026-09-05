//! Point in affine coordinates.
//!
//! A newtype around the wrapped curve's affine point; every trait is
//! implemented by delegation, except scalar multiplication which goes through
//! the driver.

use core::borrow::Borrow;
use core::fmt;
use core::ops::{Mul, Neg};

use elliptic_curve::group::GroupEncoding;
use elliptic_curve::group::prime::PrimeCurveAffine;
use elliptic_curve::point::{AffineCoordinates, DecompactPoint, DecompressPoint};
use elliptic_curve::sec1::{CompressedPoint, FromEncodedPoint, ToCompactEncodedPoint, ToEncodedPoint};
use elliptic_curve::subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};
use elliptic_curve::zeroize::DefaultIsZeroes;
use elliptic_curve::{Error, FieldBytes, PublicKey};

use super::{Accelerated, Backend, EncodedPoint, ProjectivePoint, Scalar};

/// Point on the curve in affine coordinates.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct AffinePoint<C: Backend>(pub(crate) C::AffinePoint);

impl<C: Backend> AffinePoint<C> {
    /// Additive identity of the group a.k.a. the point at infinity.
    pub const IDENTITY: Self = Self(C::AFFINE_IDENTITY);

    /// Base point of the curve.
    pub const GENERATOR: Self = Self(C::AFFINE_GENERATOR);

    /// The wrapped curve's point.
    pub fn into_inner(self) -> C::AffinePoint {
        self.0
    }

    /// Wrap the wrapped curve's point.
    pub fn from_inner(point: C::AffinePoint) -> Self {
        Self(point)
    }
}

impl<C: Backend> fmt::Debug for AffinePoint<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<C: Backend> AffineCoordinates for AffinePoint<C> {
    type FieldRepr = FieldBytes<C>;

    fn x(&self) -> FieldBytes<C> {
        self.0.x()
    }

    fn y_is_odd(&self) -> Choice {
        self.0.y_is_odd()
    }
}

impl<C: Backend> ConditionallySelectable for AffinePoint<C> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(C::AffinePoint::conditional_select(&a.0, &b.0, choice))
    }
}

impl<C: Backend> ConstantTimeEq for AffinePoint<C> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl<C: Backend> DefaultIsZeroes for AffinePoint<C> {}

impl<C: Backend> DecompressPoint<Accelerated<C>> for AffinePoint<C> {
    fn decompress(x: &FieldBytes<C>, y_is_odd: Choice) -> CtOption<Self> {
        <C::AffinePoint as DecompressPoint<C>>::decompress(x, y_is_odd).map(Self)
    }
}

impl<C: Backend> DecompactPoint<Accelerated<C>> for AffinePoint<C> {
    fn decompact(x: &FieldBytes<C>) -> CtOption<Self> {
        <C::AffinePoint as DecompactPoint<C>>::decompact(x).map(Self)
    }
}

impl<C: Backend> FromEncodedPoint<Accelerated<C>> for AffinePoint<C> {
    fn from_encoded_point(point: &EncodedPoint<C>) -> CtOption<Self> {
        <C::AffinePoint as FromEncodedPoint<C>>::from_encoded_point(point).map(Self)
    }
}

impl<C: Backend> ToEncodedPoint<Accelerated<C>> for AffinePoint<C> {
    fn to_encoded_point(&self, compress: bool) -> EncodedPoint<C> {
        <C::AffinePoint as ToEncodedPoint<C>>::to_encoded_point(&self.0, compress)
    }
}

impl<C: Backend> ToCompactEncodedPoint<Accelerated<C>> for AffinePoint<C> {
    fn to_compact_encoded_point(&self) -> CtOption<EncodedPoint<C>> {
        <C::AffinePoint as ToCompactEncodedPoint<C>>::to_compact_encoded_point(&self.0)
    }
}

impl<C: Backend> From<ProjectivePoint<C>> for AffinePoint<C> {
    fn from(point: ProjectivePoint<C>) -> Self {
        Self(point.affine())
    }
}

impl<C: Backend> From<&ProjectivePoint<C>> for AffinePoint<C> {
    fn from(point: &ProjectivePoint<C>) -> Self {
        Self(point.affine())
    }
}

impl<C: Backend> From<PublicKey<Accelerated<C>>> for AffinePoint<C> {
    fn from(public_key: PublicKey<Accelerated<C>>) -> Self {
        *public_key.as_affine()
    }
}

impl<C: Backend> From<&PublicKey<Accelerated<C>>> for AffinePoint<C> {
    fn from(public_key: &PublicKey<Accelerated<C>>) -> Self {
        *public_key.as_affine()
    }
}

impl<C: Backend> From<AffinePoint<C>> for EncodedPoint<C> {
    fn from(point: AffinePoint<C>) -> Self {
        point.to_encoded_point(C::COMPRESS_POINTS)
    }
}

impl<C: Backend> GroupEncoding for AffinePoint<C> {
    type Repr = CompressedPoint<C>;

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        <C::AffinePoint as GroupEncoding>::from_bytes(bytes).map(Self)
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        <C::AffinePoint as GroupEncoding>::from_bytes_unchecked(bytes).map(Self)
    }

    fn to_bytes(&self) -> Self::Repr {
        <C::AffinePoint as GroupEncoding>::to_bytes(&self.0)
    }
}

impl<C: Backend> PrimeCurveAffine for AffinePoint<C> {
    type Curve = ProjectivePoint<C>;
    type Scalar = Scalar<C>;

    fn identity() -> Self {
        Self::IDENTITY
    }

    fn generator() -> Self {
        Self::GENERATOR
    }

    fn is_identity(&self) -> Choice {
        PrimeCurveAffine::is_identity(&self.0)
    }

    fn to_curve(&self) -> ProjectivePoint<C> {
        ProjectivePoint::from_affine(self.0)
    }
}

impl<C: Backend> TryFrom<EncodedPoint<C>> for AffinePoint<C> {
    type Error = Error;

    fn try_from(point: EncodedPoint<C>) -> Result<Self, Error> {
        Self::try_from(&point)
    }
}

impl<C: Backend> TryFrom<&EncodedPoint<C>> for AffinePoint<C> {
    type Error = Error;

    fn try_from(point: &EncodedPoint<C>) -> Result<Self, Error> {
        Option::from(Self::from_encoded_point(point)).ok_or(Error)
    }
}

impl<C: Backend> TryFrom<AffinePoint<C>> for PublicKey<Accelerated<C>> {
    type Error = Error;

    fn try_from(point: AffinePoint<C>) -> Result<Self, Error> {
        PublicKey::from_affine(point)
    }
}

impl<C: Backend> TryFrom<&AffinePoint<C>> for PublicKey<Accelerated<C>> {
    type Error = Error;

    fn try_from(point: &AffinePoint<C>) -> Result<Self, Error> {
        PublicKey::from_affine(*point)
    }
}

/// Scalar multiplication, via the driver.
impl<C: Backend, S> Mul<S> for AffinePoint<C>
where
    S: Borrow<Scalar<C>>,
{
    type Output = ProjectivePoint<C>;

    fn mul(self, scalar: S) -> ProjectivePoint<C> {
        super::mul(scalar.borrow(), &self)
    }
}

impl<C: Backend> Neg for AffinePoint<C> {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl<C: Backend> Neg for &AffinePoint<C> {
    type Output = AffinePoint<C>;

    fn neg(self) -> AffinePoint<C> {
        AffinePoint(-self.0)
    }
}
