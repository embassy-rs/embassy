//! P-256 point in projective coordinates.
//!
//! A newtype around `p256::ProjectivePoint`; every trait is implemented by
//! delegation, except scalar multiplication which goes through the driver.

use core::borrow::Borrow;
use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use elliptic_curve::Error;
use elliptic_curve::group::cofactor::CofactorGroup;
use elliptic_curve::group::prime::{PrimeCurve, PrimeGroup};
use elliptic_curve::group::{self, Group, GroupEncoding};
use elliptic_curve::ops::{LinearCombination, MulByGenerator};
use elliptic_curve::point::Double;
use elliptic_curve::rand_core::RngCore;
use elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use elliptic_curve::subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};
use elliptic_curve::zeroize::DefaultIsZeroes;
use p256::elliptic_curve;

use super::{AffinePoint, CompressedPoint, EncodedPoint, NistP256, PublicKey, Scalar};

/// Point on the P-256 curve in projective coordinates.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct ProjectivePoint(pub(crate) p256::ProjectivePoint);

impl ProjectivePoint {
    /// Additive identity of the group a.k.a. the point at infinity.
    pub const IDENTITY: Self = Self(p256::ProjectivePoint::IDENTITY);

    /// Base point of the curve.
    pub const GENERATOR: Self = Self(p256::ProjectivePoint::GENERATOR);
}

impl From<p256::ProjectivePoint> for ProjectivePoint {
    fn from(point: p256::ProjectivePoint) -> Self {
        Self(point)
    }
}

impl From<ProjectivePoint> for p256::ProjectivePoint {
    fn from(point: ProjectivePoint) -> Self {
        point.0
    }
}

impl fmt::Debug for ProjectivePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl CofactorGroup for ProjectivePoint {
    type Subgroup = Self;

    fn clear_cofactor(&self) -> Self::Subgroup {
        Self(self.0.clear_cofactor())
    }

    fn into_subgroup(self) -> CtOption<Self> {
        self.0.into_subgroup().map(Self)
    }

    fn is_small_order(&self) -> Choice {
        self.0.is_small_order()
    }

    fn is_torsion_free(&self) -> Choice {
        self.0.is_torsion_free()
    }
}

impl ConditionallySelectable for ProjectivePoint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(p256::ProjectivePoint::conditional_select(&a.0, &b.0, choice))
    }
}

impl ConstantTimeEq for ProjectivePoint {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl DefaultIsZeroes for ProjectivePoint {}

impl Double for ProjectivePoint {
    fn double(&self) -> Self {
        Self(Double::double(&self.0))
    }
}

impl From<AffinePoint> for ProjectivePoint {
    fn from(point: AffinePoint) -> Self {
        Self(p256::ProjectivePoint::from(point.0))
    }
}

impl From<&AffinePoint> for ProjectivePoint {
    fn from(point: &AffinePoint) -> Self {
        Self(p256::ProjectivePoint::from(&point.0))
    }
}

impl From<PublicKey> for ProjectivePoint {
    fn from(public_key: PublicKey) -> Self {
        AffinePoint::from(public_key).into()
    }
}

impl From<&PublicKey> for ProjectivePoint {
    fn from(public_key: &PublicKey) -> Self {
        AffinePoint::from(public_key).into()
    }
}

impl FromEncodedPoint<NistP256> for ProjectivePoint {
    fn from_encoded_point(point: &EncodedPoint) -> CtOption<Self> {
        <p256::ProjectivePoint as FromEncodedPoint<p256::NistP256>>::from_encoded_point(point).map(Self)
    }
}

impl ToEncodedPoint<NistP256> for ProjectivePoint {
    fn to_encoded_point(&self, compress: bool) -> EncodedPoint {
        <p256::ProjectivePoint as ToEncodedPoint<p256::NistP256>>::to_encoded_point(&self.0, compress)
    }
}

impl Group for ProjectivePoint {
    type Scalar = Scalar;

    /// A random point, computed as `k * G` for a random scalar `k`, via the
    /// `P256ScalarMul` driver.
    fn random(rng: impl RngCore) -> Self {
        Self::mul_by_generator(&<Scalar as elliptic_curve::ff::Field>::random(rng))
    }

    fn identity() -> Self {
        Self(<p256::ProjectivePoint as Group>::identity())
    }

    fn generator() -> Self {
        Self(<p256::ProjectivePoint as Group>::generator())
    }

    fn is_identity(&self) -> Choice {
        Group::is_identity(&self.0)
    }

    fn double(&self) -> Self {
        Self(Group::double(&self.0))
    }
}

impl GroupEncoding for ProjectivePoint {
    type Repr = CompressedPoint;

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        <p256::ProjectivePoint as GroupEncoding>::from_bytes(bytes).map(Self)
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        <p256::ProjectivePoint as GroupEncoding>::from_bytes_unchecked(bytes).map(Self)
    }

    fn to_bytes(&self) -> Self::Repr {
        <p256::ProjectivePoint as GroupEncoding>::to_bytes(&self.0)
    }
}

impl group::Curve for ProjectivePoint {
    type AffineRepr = AffinePoint;

    fn to_affine(&self) -> AffinePoint {
        AffinePoint(group::Curve::to_affine(&self.0))
    }
}

/// Uses the default `x * k + y * l` computation, so that both scalar
/// multiplications go through the `P256ScalarMul` driver.
impl LinearCombination for ProjectivePoint {}

/// Scalar multiplication by the generator, via the `P256ScalarMul` driver.
impl MulByGenerator for ProjectivePoint {
    fn mul_by_generator(scalar: &Scalar) -> Self {
        super::mul_base(scalar)
    }
}

impl PrimeGroup for ProjectivePoint {}

impl PrimeCurve for ProjectivePoint {
    type Affine = AffinePoint;
}

impl TryFrom<ProjectivePoint> for PublicKey {
    type Error = Error;

    fn try_from(point: ProjectivePoint) -> Result<Self, Error> {
        AffinePoint::from(point).try_into()
    }
}

impl TryFrom<&ProjectivePoint> for PublicKey {
    type Error = Error;

    fn try_from(point: &ProjectivePoint) -> Result<Self, Error> {
        AffinePoint::from(point).try_into()
    }
}

impl Add<ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, other: ProjectivePoint) -> ProjectivePoint {
        Self(self.0 + other.0)
    }
}

impl Add<&ProjectivePoint> for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, other: &ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0 + other.0)
    }
}

impl Add<&ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, other: &ProjectivePoint) -> ProjectivePoint {
        Self(self.0 + other.0)
    }
}

impl AddAssign<ProjectivePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: ProjectivePoint) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&ProjectivePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: &ProjectivePoint) {
        self.0 += rhs.0;
    }
}

impl Add<AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, other: AffinePoint) -> ProjectivePoint {
        Self(self.0 + other.0)
    }
}

impl Add<&AffinePoint> for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, other: &AffinePoint) -> ProjectivePoint {
        ProjectivePoint(self.0 + other.0)
    }
}

impl Add<&AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, other: &AffinePoint) -> ProjectivePoint {
        Self(self.0 + other.0)
    }
}

impl AddAssign<AffinePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: AffinePoint) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&AffinePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: &AffinePoint) {
        self.0 += rhs.0;
    }
}

impl Sum for ProjectivePoint {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|point| point.0).sum())
    }
}

impl<'a> Sum<&'a ProjectivePoint> for ProjectivePoint {
    fn sum<I: Iterator<Item = &'a ProjectivePoint>>(iter: I) -> Self {
        Self(iter.map(|point| &point.0).sum())
    }
}

impl Sub<ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: ProjectivePoint) -> ProjectivePoint {
        Self(self.0 - other.0)
    }
}

impl Sub<&ProjectivePoint> for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: &ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0 - other.0)
    }
}

impl Sub<&ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: &ProjectivePoint) -> ProjectivePoint {
        Self(self.0 - other.0)
    }
}

impl SubAssign<ProjectivePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: ProjectivePoint) {
        self.0 -= rhs.0;
    }
}

impl SubAssign<&ProjectivePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: &ProjectivePoint) {
        self.0 -= rhs.0;
    }
}

impl Sub<AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: AffinePoint) -> ProjectivePoint {
        Self(self.0 - other.0)
    }
}

impl Sub<&AffinePoint> for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: &AffinePoint) -> ProjectivePoint {
        ProjectivePoint(self.0 - other.0)
    }
}

impl Sub<&AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: &AffinePoint) -> ProjectivePoint {
        Self(self.0 - other.0)
    }
}

impl SubAssign<AffinePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: AffinePoint) {
        self.0 -= rhs.0;
    }
}

impl SubAssign<&AffinePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: &AffinePoint) {
        self.0 -= rhs.0;
    }
}

/// Scalar multiplication, via the `P256ScalarMul` driver.
impl<S> Mul<S> for ProjectivePoint
where
    S: Borrow<Scalar>,
{
    type Output = Self;

    fn mul(self, scalar: S) -> Self {
        super::mul_projective(scalar.borrow(), &self)
    }
}

/// Scalar multiplication, via the `P256ScalarMul` driver.
impl Mul<&Scalar> for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn mul(self, scalar: &Scalar) -> ProjectivePoint {
        super::mul_projective(scalar, self)
    }
}

/// Scalar multiplication, via the `P256ScalarMul` driver.
impl<S> MulAssign<S> for ProjectivePoint
where
    S: Borrow<Scalar>,
{
    fn mul_assign(&mut self, scalar: S) {
        *self = super::mul_projective(scalar.borrow(), self);
    }
}

impl Neg for ProjectivePoint {
    type Output = ProjectivePoint;

    fn neg(self) -> ProjectivePoint {
        Self(-self.0)
    }
}

impl Neg for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn neg(self) -> ProjectivePoint {
        ProjectivePoint(-self.0)
    }
}
