//! Point in projective coordinates.
//!
//! Holds whichever of the wrapped curve's two point types it was last given:
//! the projective form after arithmetic, the affine form when built from
//! affine input or returned by the driver, so `to_affine` is free in that
//! case. Which form is held depends only on how the point was constructed,
//! never on its value. Every trait is implemented by delegation, except scalar
//! multiplication which goes through the driver.

use core::borrow::Borrow;
use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use elliptic_curve::group::cofactor::CofactorGroup;
use elliptic_curve::group::prime::{PrimeCurve, PrimeCurveAffine, PrimeGroup};
use elliptic_curve::group::{self, Group, GroupEncoding};
use elliptic_curve::ops::{LinearCombination, MulByGenerator};
use elliptic_curve::point::Double;
use elliptic_curve::rand_core::RngCore;
use elliptic_curve::sec1::{CompressedPoint, FromEncodedPoint, ToEncodedPoint};
use elliptic_curve::subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};
use elliptic_curve::zeroize::DefaultIsZeroes;
use elliptic_curve::{Error, PublicKey};

use super::{Accelerated, AffinePoint, Backend, EncodedPoint, Scalar};

#[derive(Clone, Copy)]
enum Repr<C: Backend> {
    /// The general case, as produced by arithmetic.
    Projective(C::ProjectivePoint),
    /// A point whose affine form is known without a field inversion.
    Affine(C::AffinePoint),
}

/// Point on the curve in projective coordinates.
#[derive(Clone, Copy)]
pub struct ProjectivePoint<C: Backend>(Repr<C>);

impl<C: Backend> ProjectivePoint<C> {
    /// Additive identity of the group a.k.a. the point at infinity.
    pub const IDENTITY: Self = Self(Repr::Affine(C::AFFINE_IDENTITY));

    /// Base point of the curve.
    pub const GENERATOR: Self = Self(Repr::Affine(C::AFFINE_GENERATOR));

    /// Wrap the wrapped curve's projective point.
    pub fn from_inner(point: C::ProjectivePoint) -> Self {
        Self::from_projective(point)
    }

    /// The wrapped curve's projective point.
    pub fn into_inner(self) -> C::ProjectivePoint {
        self.projective()
    }

    pub(crate) fn from_projective(point: C::ProjectivePoint) -> Self {
        Self(Repr::Projective(point))
    }

    pub(crate) fn from_affine(point: C::AffinePoint) -> Self {
        Self(Repr::Affine(point))
    }

    /// The projective form; free in both cases (an affine point embeds with `Z = 1`).
    pub(crate) fn projective(&self) -> C::ProjectivePoint {
        match self.0 {
            Repr::Projective(point) => point,
            Repr::Affine(point) => point.to_curve(),
        }
    }

    /// The affine form; free when it is the form held, else one field inversion.
    pub(crate) fn affine(&self) -> C::AffinePoint {
        match self.0 {
            Repr::Projective(point) => group::Curve::to_affine(&point),
            Repr::Affine(point) => point,
        }
    }
}

impl<C: Backend> Default for ProjectivePoint<C> {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl<C: Backend> PartialEq for ProjectivePoint<C> {
    fn eq(&self, other: &Self) -> bool {
        self.projective() == other.projective()
    }
}

impl<C: Backend> Eq for ProjectivePoint<C> {}

impl<C: Backend> fmt::Debug for ProjectivePoint<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.projective().fmt(f)
    }
}

/// The group has prime order, so the cofactor is 1 and every point is in the subgroup.
impl<C: Backend> CofactorGroup for ProjectivePoint<C> {
    type Subgroup = Self;

    fn clear_cofactor(&self) -> Self::Subgroup {
        *self
    }

    fn into_subgroup(self) -> CtOption<Self> {
        CtOption::new(self, 1.into())
    }

    fn is_torsion_free(&self) -> Choice {
        1.into()
    }
}

impl<C: Backend> ConditionallySelectable for ProjectivePoint<C> {
    /// The match is on the forms held, which depend on how the operands were
    /// constructed, not on `choice` or on coordinates; the form of the result
    /// is the same whichever operand is selected.
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        match (&a.0, &b.0) {
            (Repr::Affine(a), Repr::Affine(b)) => Self::from_affine(C::AffinePoint::conditional_select(a, b, choice)),
            _ => Self::from_projective(C::ProjectivePoint::conditional_select(
                &a.projective(),
                &b.projective(),
                choice,
            )),
        }
    }
}

impl<C: Backend> ConstantTimeEq for ProjectivePoint<C> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.projective().ct_eq(&other.projective())
    }
}

impl<C: Backend> DefaultIsZeroes for ProjectivePoint<C> {}

impl<C: Backend> Double for ProjectivePoint<C> {
    fn double(&self) -> Self {
        Self::from_projective(Group::double(&self.projective()))
    }
}

impl<C: Backend> From<AffinePoint<C>> for ProjectivePoint<C> {
    fn from(point: AffinePoint<C>) -> Self {
        Self::from_affine(point.0)
    }
}

impl<C: Backend> From<&AffinePoint<C>> for ProjectivePoint<C> {
    fn from(point: &AffinePoint<C>) -> Self {
        Self::from_affine(point.0)
    }
}

impl<C: Backend> From<PublicKey<Accelerated<C>>> for ProjectivePoint<C> {
    fn from(public_key: PublicKey<Accelerated<C>>) -> Self {
        AffinePoint::from(public_key).into()
    }
}

impl<C: Backend> From<&PublicKey<Accelerated<C>>> for ProjectivePoint<C> {
    fn from(public_key: &PublicKey<Accelerated<C>>) -> Self {
        AffinePoint::from(public_key).into()
    }
}

impl<C: Backend> FromEncodedPoint<Accelerated<C>> for ProjectivePoint<C> {
    fn from_encoded_point(point: &EncodedPoint<C>) -> CtOption<Self> {
        <C::AffinePoint as FromEncodedPoint<C>>::from_encoded_point(point).map(Self::from_affine)
    }
}

impl<C: Backend> ToEncodedPoint<Accelerated<C>> for ProjectivePoint<C> {
    fn to_encoded_point(&self, compress: bool) -> EncodedPoint<C> {
        <C::AffinePoint as ToEncodedPoint<C>>::to_encoded_point(&self.affine(), compress)
    }
}

impl<C: Backend> Group for ProjectivePoint<C> {
    type Scalar = Scalar<C>;

    /// A random point, computed as `k * G` for a random scalar `k`, via the driver.
    fn random(rng: impl RngCore) -> Self {
        Self::mul_by_generator(&<Scalar<C> as elliptic_curve::ff::Field>::random(rng))
    }

    fn identity() -> Self {
        Self::IDENTITY
    }

    fn generator() -> Self {
        Self::GENERATOR
    }

    fn is_identity(&self) -> Choice {
        match self.0 {
            Repr::Projective(point) => Group::is_identity(&point),
            Repr::Affine(point) => PrimeCurveAffine::is_identity(&point),
        }
    }

    fn double(&self) -> Self {
        Self::from_projective(Group::double(&self.projective()))
    }
}

impl<C: Backend> GroupEncoding for ProjectivePoint<C> {
    type Repr = CompressedPoint<C>;

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        <C::AffinePoint as GroupEncoding>::from_bytes(bytes).map(Self::from_affine)
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        <C::AffinePoint as GroupEncoding>::from_bytes_unchecked(bytes).map(Self::from_affine)
    }

    fn to_bytes(&self) -> Self::Repr {
        <C::AffinePoint as GroupEncoding>::to_bytes(&self.affine())
    }
}

impl<C: Backend> group::Curve for ProjectivePoint<C> {
    type AffineRepr = AffinePoint<C>;

    fn to_affine(&self) -> AffinePoint<C> {
        AffinePoint(self.affine())
    }
}

/// Two-term linear combination `x * k + y * l`, via the driver when it
/// provides one.
impl<C: Backend> LinearCombination for ProjectivePoint<C> {
    fn lincomb(x: &Self, k: &Scalar<C>, y: &Self, l: &Scalar<C>) -> Self {
        super::lincomb(x, k, y, l)
    }
}

/// Scalar multiplication by the generator, via the driver.
impl<C: Backend> MulByGenerator for ProjectivePoint<C> {
    fn mul_by_generator(scalar: &Scalar<C>) -> Self {
        super::mul_base(scalar)
    }
}

impl<C: Backend> PrimeGroup for ProjectivePoint<C> {}

impl<C: Backend> PrimeCurve for ProjectivePoint<C> {
    type Affine = AffinePoint<C>;
}

impl<C: Backend> TryFrom<ProjectivePoint<C>> for PublicKey<Accelerated<C>> {
    type Error = Error;

    fn try_from(point: ProjectivePoint<C>) -> Result<Self, Error> {
        AffinePoint::from(point).try_into()
    }
}

impl<C: Backend> TryFrom<&ProjectivePoint<C>> for PublicKey<Accelerated<C>> {
    type Error = Error;

    fn try_from(point: &ProjectivePoint<C>) -> Result<Self, Error> {
        AffinePoint::from(point).try_into()
    }
}

impl<C: Backend> Add<ProjectivePoint<C>> for ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn add(self, other: ProjectivePoint<C>) -> ProjectivePoint<C> {
        Self::from_projective(self.projective() + other.projective())
    }
}

impl<C: Backend> Add<&ProjectivePoint<C>> for &ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn add(self, other: &ProjectivePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::from_projective(self.projective() + other.projective())
    }
}

impl<C: Backend> Add<&ProjectivePoint<C>> for ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn add(self, other: &ProjectivePoint<C>) -> ProjectivePoint<C> {
        Self::from_projective(self.projective() + other.projective())
    }
}

impl<C: Backend> AddAssign<ProjectivePoint<C>> for ProjectivePoint<C> {
    fn add_assign(&mut self, rhs: ProjectivePoint<C>) {
        *self = Self::from_projective(self.projective() + rhs.projective());
    }
}

impl<C: Backend> AddAssign<&ProjectivePoint<C>> for ProjectivePoint<C> {
    fn add_assign(&mut self, rhs: &ProjectivePoint<C>) {
        *self = Self::from_projective(self.projective() + rhs.projective());
    }
}

impl<C: Backend> Add<AffinePoint<C>> for ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn add(self, other: AffinePoint<C>) -> ProjectivePoint<C> {
        Self::from_projective(self.projective() + other.0)
    }
}

impl<C: Backend> Add<&AffinePoint<C>> for &ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn add(self, other: &AffinePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::from_projective(self.projective() + other.0)
    }
}

impl<C: Backend> Add<&AffinePoint<C>> for ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn add(self, other: &AffinePoint<C>) -> ProjectivePoint<C> {
        Self::from_projective(self.projective() + other.0)
    }
}

impl<C: Backend> AddAssign<AffinePoint<C>> for ProjectivePoint<C> {
    fn add_assign(&mut self, rhs: AffinePoint<C>) {
        *self = Self::from_projective(self.projective() + rhs.0);
    }
}

impl<C: Backend> AddAssign<&AffinePoint<C>> for ProjectivePoint<C> {
    fn add_assign(&mut self, rhs: &AffinePoint<C>) {
        *self = Self::from_projective(self.projective() + rhs.0);
    }
}

impl<C: Backend> Sum for ProjectivePoint<C> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self::from_projective(iter.map(|point| point.projective()).sum())
    }
}

impl<'a, C: Backend> Sum<&'a ProjectivePoint<C>> for ProjectivePoint<C> {
    fn sum<I: Iterator<Item = &'a ProjectivePoint<C>>>(iter: I) -> Self {
        Self::from_projective(iter.map(|point| point.projective()).sum())
    }
}

impl<C: Backend> Sub<ProjectivePoint<C>> for ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn sub(self, other: ProjectivePoint<C>) -> ProjectivePoint<C> {
        Self::from_projective(self.projective() - other.projective())
    }
}

impl<C: Backend> Sub<&ProjectivePoint<C>> for &ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn sub(self, other: &ProjectivePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::from_projective(self.projective() - other.projective())
    }
}

impl<C: Backend> Sub<&ProjectivePoint<C>> for ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn sub(self, other: &ProjectivePoint<C>) -> ProjectivePoint<C> {
        Self::from_projective(self.projective() - other.projective())
    }
}

impl<C: Backend> SubAssign<ProjectivePoint<C>> for ProjectivePoint<C> {
    fn sub_assign(&mut self, rhs: ProjectivePoint<C>) {
        *self = Self::from_projective(self.projective() - rhs.projective());
    }
}

impl<C: Backend> SubAssign<&ProjectivePoint<C>> for ProjectivePoint<C> {
    fn sub_assign(&mut self, rhs: &ProjectivePoint<C>) {
        *self = Self::from_projective(self.projective() - rhs.projective());
    }
}

impl<C: Backend> Sub<AffinePoint<C>> for ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn sub(self, other: AffinePoint<C>) -> ProjectivePoint<C> {
        Self::from_projective(self.projective() - other.0)
    }
}

impl<C: Backend> Sub<&AffinePoint<C>> for &ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn sub(self, other: &AffinePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::from_projective(self.projective() - other.0)
    }
}

impl<C: Backend> Sub<&AffinePoint<C>> for ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn sub(self, other: &AffinePoint<C>) -> ProjectivePoint<C> {
        Self::from_projective(self.projective() - other.0)
    }
}

impl<C: Backend> SubAssign<AffinePoint<C>> for ProjectivePoint<C> {
    fn sub_assign(&mut self, rhs: AffinePoint<C>) {
        *self = Self::from_projective(self.projective() - rhs.0);
    }
}

impl<C: Backend> SubAssign<&AffinePoint<C>> for ProjectivePoint<C> {
    fn sub_assign(&mut self, rhs: &AffinePoint<C>) {
        *self = Self::from_projective(self.projective() - rhs.0);
    }
}

/// Scalar multiplication, via the driver.
impl<C: Backend, S> Mul<S> for ProjectivePoint<C>
where
    S: Borrow<Scalar<C>>,
{
    type Output = Self;

    fn mul(self, scalar: S) -> Self {
        super::mul_projective(scalar.borrow(), &self)
    }
}

/// Scalar multiplication, via the driver.
impl<C: Backend> Mul<&Scalar<C>> for &ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn mul(self, scalar: &Scalar<C>) -> ProjectivePoint<C> {
        super::mul_projective(scalar, self)
    }
}

/// Scalar multiplication, via the driver.
impl<C: Backend, S> MulAssign<S> for ProjectivePoint<C>
where
    S: Borrow<Scalar<C>>,
{
    fn mul_assign(&mut self, scalar: S) {
        *self = super::mul_projective(scalar.borrow(), self);
    }
}

/// Negation keeps the form held: negating an affine point is a field negation.
impl<C: Backend> Neg for ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn neg(self) -> ProjectivePoint<C> {
        match self.0 {
            Repr::Projective(point) => Self::from_projective(-point),
            Repr::Affine(point) => Self::from_affine(-point),
        }
    }
}

impl<C: Backend> Neg for &ProjectivePoint<C> {
    type Output = ProjectivePoint<C>;

    fn neg(self) -> ProjectivePoint<C> {
        -*self
    }
}
