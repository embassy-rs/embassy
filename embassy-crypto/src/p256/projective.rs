//! P-256 point in projective coordinates.
//!
//! Holds whichever of `p256`'s two point types it was last given: the
//! projective form after arithmetic, the affine form when built from affine
//! input or returned by the driver, so `to_affine` is free in that case.
//! Which form is held depends only on how the point was constructed, never
//! on its value. Every trait is implemented by delegation, except scalar
//! multiplication which goes through the driver.

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

#[derive(Clone, Copy)]
enum Repr {
    /// The general case, as produced by arithmetic.
    Projective(p256::ProjectivePoint),
    /// A point whose affine form is known without a field inversion.
    Affine(p256::AffinePoint),
}

/// Point on the P-256 curve in projective coordinates.
#[derive(Clone, Copy)]
pub struct ProjectivePoint(Repr);

impl ProjectivePoint {
    /// Additive identity of the group a.k.a. the point at infinity.
    pub const IDENTITY: Self = Self(Repr::Affine(p256::AffinePoint::IDENTITY));

    /// Base point of the curve.
    pub const GENERATOR: Self = Self(Repr::Affine(p256::AffinePoint::GENERATOR));

    pub(crate) fn from_projective(point: p256::ProjectivePoint) -> Self {
        Self(Repr::Projective(point))
    }

    pub(crate) fn from_affine(point: p256::AffinePoint) -> Self {
        Self(Repr::Affine(point))
    }

    /// The projective form; free in both cases (an affine point embeds with `Z = 1`).
    pub(crate) fn projective(&self) -> p256::ProjectivePoint {
        match self.0 {
            Repr::Projective(point) => point,
            Repr::Affine(point) => p256::ProjectivePoint::from(point),
        }
    }

    /// The affine form; free when it is the form held, else one field inversion.
    pub(crate) fn affine(&self) -> p256::AffinePoint {
        match self.0 {
            Repr::Projective(point) => group::Curve::to_affine(&point),
            Repr::Affine(point) => point,
        }
    }
}

impl Default for ProjectivePoint {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl PartialEq for ProjectivePoint {
    fn eq(&self, other: &Self) -> bool {
        self.projective() == other.projective()
    }
}

impl Eq for ProjectivePoint {}

impl From<p256::ProjectivePoint> for ProjectivePoint {
    fn from(point: p256::ProjectivePoint) -> Self {
        Self::from_projective(point)
    }
}

impl From<ProjectivePoint> for p256::ProjectivePoint {
    fn from(point: ProjectivePoint) -> Self {
        point.projective()
    }
}

impl fmt::Debug for ProjectivePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.projective().fmt(f)
    }
}

impl CofactorGroup for ProjectivePoint {
    type Subgroup = Self;

    /// The group has prime order, so this is the identity map and keeps the form held.
    fn clear_cofactor(&self) -> Self::Subgroup {
        *self
    }

    fn into_subgroup(self) -> CtOption<Self> {
        self.projective().into_subgroup().map(|_| self)
    }

    fn is_small_order(&self) -> Choice {
        self.projective().is_small_order()
    }

    fn is_torsion_free(&self) -> Choice {
        self.projective().is_torsion_free()
    }
}

impl ConditionallySelectable for ProjectivePoint {
    /// The match is on the forms held, which depend on how the operands were
    /// constructed, not on `choice` or on coordinates; the form of the result
    /// is the same whichever operand is selected.
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        match (&a.0, &b.0) {
            (Repr::Affine(a), Repr::Affine(b)) => {
                Self::from_affine(p256::AffinePoint::conditional_select(a, b, choice))
            }
            _ => Self::from_projective(p256::ProjectivePoint::conditional_select(
                &a.projective(),
                &b.projective(),
                choice,
            )),
        }
    }
}

impl ConstantTimeEq for ProjectivePoint {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.projective().ct_eq(&other.projective())
    }
}

impl DefaultIsZeroes for ProjectivePoint {}

impl Double for ProjectivePoint {
    fn double(&self) -> Self {
        Self::from_projective(Double::double(&self.projective()))
    }
}

impl From<AffinePoint> for ProjectivePoint {
    fn from(point: AffinePoint) -> Self {
        Self::from_affine(point.0)
    }
}

impl From<&AffinePoint> for ProjectivePoint {
    fn from(point: &AffinePoint) -> Self {
        Self::from_affine(point.0)
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
        <p256::AffinePoint as FromEncodedPoint<p256::NistP256>>::from_encoded_point(point).map(Self::from_affine)
    }
}

impl ToEncodedPoint<NistP256> for ProjectivePoint {
    fn to_encoded_point(&self, compress: bool) -> EncodedPoint {
        <p256::AffinePoint as ToEncodedPoint<p256::NistP256>>::to_encoded_point(&self.affine(), compress)
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
        Self::IDENTITY
    }

    fn generator() -> Self {
        Self::GENERATOR
    }

    fn is_identity(&self) -> Choice {
        Group::is_identity(&self.projective())
    }

    fn double(&self) -> Self {
        Self::from_projective(Group::double(&self.projective()))
    }
}

impl GroupEncoding for ProjectivePoint {
    type Repr = CompressedPoint;

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        <p256::AffinePoint as GroupEncoding>::from_bytes(bytes).map(Self::from_affine)
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        <p256::AffinePoint as GroupEncoding>::from_bytes_unchecked(bytes).map(Self::from_affine)
    }

    fn to_bytes(&self) -> Self::Repr {
        <p256::AffinePoint as GroupEncoding>::to_bytes(&self.affine())
    }
}

impl group::Curve for ProjectivePoint {
    type AffineRepr = AffinePoint;

    fn to_affine(&self) -> AffinePoint {
        AffinePoint(self.affine())
    }
}

/// Two-term linear combination `x * k + y * l`, via the `P256Lincomb`
/// driver when enabled, else the software path (see `super::lincomb`).
impl LinearCombination for ProjectivePoint {
    fn lincomb(x: &Self, k: &Scalar, y: &Self, l: &Scalar) -> Self {
        super::lincomb(x, k, y, l)
    }
}

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
        Self::from_projective(self.projective() + other.projective())
    }
}

impl Add<&ProjectivePoint> for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, other: &ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint::from_projective(self.projective() + other.projective())
    }
}

impl Add<&ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, other: &ProjectivePoint) -> ProjectivePoint {
        Self::from_projective(self.projective() + other.projective())
    }
}

impl AddAssign<ProjectivePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: ProjectivePoint) {
        *self = Self::from_projective(self.projective() + rhs.projective());
    }
}

impl AddAssign<&ProjectivePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: &ProjectivePoint) {
        *self = Self::from_projective(self.projective() + rhs.projective());
    }
}

impl Add<AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, other: AffinePoint) -> ProjectivePoint {
        Self::from_projective(self.projective() + other.0)
    }
}

impl Add<&AffinePoint> for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, other: &AffinePoint) -> ProjectivePoint {
        ProjectivePoint::from_projective(self.projective() + other.0)
    }
}

impl Add<&AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, other: &AffinePoint) -> ProjectivePoint {
        Self::from_projective(self.projective() + other.0)
    }
}

impl AddAssign<AffinePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: AffinePoint) {
        *self = Self::from_projective(self.projective() + rhs.0);
    }
}

impl AddAssign<&AffinePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: &AffinePoint) {
        *self = Self::from_projective(self.projective() + rhs.0);
    }
}

impl Sum for ProjectivePoint {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self::from_projective(iter.map(|point| point.projective()).sum())
    }
}

impl<'a> Sum<&'a ProjectivePoint> for ProjectivePoint {
    fn sum<I: Iterator<Item = &'a ProjectivePoint>>(iter: I) -> Self {
        Self::from_projective(iter.map(|point| point.projective()).sum())
    }
}

impl Sub<ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: ProjectivePoint) -> ProjectivePoint {
        Self::from_projective(self.projective() - other.projective())
    }
}

impl Sub<&ProjectivePoint> for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: &ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint::from_projective(self.projective() - other.projective())
    }
}

impl Sub<&ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: &ProjectivePoint) -> ProjectivePoint {
        Self::from_projective(self.projective() - other.projective())
    }
}

impl SubAssign<ProjectivePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: ProjectivePoint) {
        *self = Self::from_projective(self.projective() - rhs.projective());
    }
}

impl SubAssign<&ProjectivePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: &ProjectivePoint) {
        *self = Self::from_projective(self.projective() - rhs.projective());
    }
}

impl Sub<AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: AffinePoint) -> ProjectivePoint {
        Self::from_projective(self.projective() - other.0)
    }
}

impl Sub<&AffinePoint> for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: &AffinePoint) -> ProjectivePoint {
        ProjectivePoint::from_projective(self.projective() - other.0)
    }
}

impl Sub<&AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, other: &AffinePoint) -> ProjectivePoint {
        Self::from_projective(self.projective() - other.0)
    }
}

impl SubAssign<AffinePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: AffinePoint) {
        *self = Self::from_projective(self.projective() - rhs.0);
    }
}

impl SubAssign<&AffinePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: &AffinePoint) {
        *self = Self::from_projective(self.projective() - rhs.0);
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

/// Negation keeps the form held: negating an affine point is a field negation.
impl Neg for ProjectivePoint {
    type Output = ProjectivePoint;

    fn neg(self) -> ProjectivePoint {
        match self.0 {
            Repr::Projective(point) => Self::from_projective(-point),
            Repr::Affine(point) => Self::from_affine(-point),
        }
    }
}

impl Neg for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn neg(self) -> ProjectivePoint {
        -*self
    }
}
