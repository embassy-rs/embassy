//! P-256 scalar field element (integer modulo the curve order).
//!
//! A newtype around `p256::Scalar`; every trait is implemented by delegation.

use core::fmt;
use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Shr, ShrAssign, Sub, SubAssign};

use elliptic_curve::ScalarPrimitive;
use elliptic_curve::bigint::U256;
use elliptic_curve::ff::{Field, PrimeField};
use elliptic_curve::ops::{Invert, Reduce, ReduceNonZero};
use elliptic_curve::rand_core::RngCore;
use elliptic_curve::scalar::{FromUintUnchecked, IsHigh};
use elliptic_curve::subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};
use elliptic_curve::zeroize::DefaultIsZeroes;
use p256::elliptic_curve;

use super::{FieldBytes, NistP256, SecretKey};

/// Scalar field element modulo the P-256 curve order.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Scalar(pub(crate) p256::Scalar);

impl Scalar {
    /// Zero scalar.
    pub const ZERO: Self = Self(p256::Scalar::ZERO);

    /// Multiplicative identity.
    pub const ONE: Self = Self(p256::Scalar::ONE);
}

impl From<p256::Scalar> for Scalar {
    fn from(scalar: p256::Scalar) -> Self {
        Self(scalar)
    }
}

impl From<Scalar> for p256::Scalar {
    fn from(scalar: Scalar) -> Self {
        scalar.0
    }
}

impl fmt::Debug for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<Scalar> for Scalar {
    fn as_ref(&self) -> &Scalar {
        self
    }
}

impl Field for Scalar {
    const ZERO: Self = Self(<p256::Scalar as Field>::ZERO);
    const ONE: Self = Self(<p256::Scalar as Field>::ONE);

    fn random(rng: impl RngCore) -> Self {
        Self(<p256::Scalar as Field>::random(rng))
    }

    fn is_zero(&self) -> Choice {
        Field::is_zero(&self.0)
    }

    fn is_zero_vartime(&self) -> bool {
        Field::is_zero_vartime(&self.0)
    }

    fn square(&self) -> Self {
        Self(Field::square(&self.0))
    }

    fn cube(&self) -> Self {
        Self(Field::cube(&self.0))
    }

    fn double(&self) -> Self {
        Self(Field::double(&self.0))
    }

    fn invert(&self) -> CtOption<Self> {
        super::invert(self)
    }

    fn sqrt_ratio(num: &Self, div: &Self) -> (Choice, Self) {
        let (choice, value) = <p256::Scalar as Field>::sqrt_ratio(&num.0, &div.0);
        (choice, Self(value))
    }

    fn sqrt_alt(&self) -> (Choice, Self) {
        let (choice, value) = Field::sqrt_alt(&self.0);
        (choice, Self(value))
    }

    fn sqrt(&self) -> CtOption<Self> {
        Field::sqrt(&self.0).map(Self)
    }

    fn pow<S: AsRef<[u64]>>(&self, exp: S) -> Self {
        Self(Field::pow(&self.0, exp))
    }

    fn pow_vartime<S: AsRef<[u64]>>(&self, exp: S) -> Self {
        Self(Field::pow_vartime(&self.0, exp))
    }
}

impl PrimeField for Scalar {
    type Repr = FieldBytes;

    const MODULUS: &'static str = <p256::Scalar as PrimeField>::MODULUS;
    const NUM_BITS: u32 = <p256::Scalar as PrimeField>::NUM_BITS;
    const CAPACITY: u32 = <p256::Scalar as PrimeField>::CAPACITY;
    const TWO_INV: Self = Self(<p256::Scalar as PrimeField>::TWO_INV);
    const MULTIPLICATIVE_GENERATOR: Self = Self(<p256::Scalar as PrimeField>::MULTIPLICATIVE_GENERATOR);
    const S: u32 = <p256::Scalar as PrimeField>::S;
    const ROOT_OF_UNITY: Self = Self(<p256::Scalar as PrimeField>::ROOT_OF_UNITY);
    const ROOT_OF_UNITY_INV: Self = Self(<p256::Scalar as PrimeField>::ROOT_OF_UNITY_INV);
    const DELTA: Self = Self(<p256::Scalar as PrimeField>::DELTA);

    fn from_str_vartime(s: &str) -> Option<Self> {
        <p256::Scalar as PrimeField>::from_str_vartime(s).map(Self)
    }

    fn from_u128(v: u128) -> Self {
        Self(<p256::Scalar as PrimeField>::from_u128(v))
    }

    fn from_repr(repr: Self::Repr) -> CtOption<Self> {
        <p256::Scalar as PrimeField>::from_repr(repr).map(Self)
    }

    fn from_repr_vartime(repr: Self::Repr) -> Option<Self> {
        <p256::Scalar as PrimeField>::from_repr_vartime(repr).map(Self)
    }

    fn to_repr(&self) -> Self::Repr {
        PrimeField::to_repr(&self.0)
    }

    fn is_odd(&self) -> Choice {
        PrimeField::is_odd(&self.0)
    }

    fn is_even(&self) -> Choice {
        PrimeField::is_even(&self.0)
    }
}

impl DefaultIsZeroes for Scalar {}

impl FromUintUnchecked for Scalar {
    type Uint = U256;

    fn from_uint_unchecked(uint: U256) -> Self {
        Self(<p256::Scalar as FromUintUnchecked>::from_uint_unchecked(uint))
    }
}

impl Invert for Scalar {
    type Output = CtOption<Self>;

    fn invert(&self) -> CtOption<Self> {
        super::invert(self)
    }

    fn invert_vartime(&self) -> CtOption<Self> {
        super::invert_vartime(self)
    }
}

impl IsHigh for Scalar {
    fn is_high(&self) -> Choice {
        self.0.is_high()
    }
}

impl Shr<usize> for Scalar {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self {
        Self(self.0 >> rhs)
    }
}

impl Shr<usize> for &Scalar {
    type Output = Scalar;

    fn shr(self, rhs: usize) -> Scalar {
        Scalar(self.0 >> rhs)
    }
}

impl ShrAssign<usize> for Scalar {
    fn shr_assign(&mut self, rhs: usize) {
        self.0 >>= rhs;
    }
}

impl From<u32> for Scalar {
    fn from(n: u32) -> Self {
        Self(p256::Scalar::from(n))
    }
}

impl From<u64> for Scalar {
    fn from(n: u64) -> Self {
        Self(p256::Scalar::from(n))
    }
}

impl From<u128> for Scalar {
    fn from(n: u128) -> Self {
        Self(p256::Scalar::from(n))
    }
}

impl From<Scalar> for FieldBytes {
    fn from(scalar: Scalar) -> Self {
        FieldBytes::from(scalar.0)
    }
}

impl From<&Scalar> for FieldBytes {
    fn from(scalar: &Scalar) -> Self {
        FieldBytes::from(&scalar.0)
    }
}

impl From<ScalarPrimitive<NistP256>> for Scalar {
    fn from(w: ScalarPrimitive<NistP256>) -> Self {
        Self::from(&w)
    }
}

impl From<&ScalarPrimitive<NistP256>> for Scalar {
    fn from(w: &ScalarPrimitive<NistP256>) -> Self {
        Self::from_uint_unchecked(*w.as_uint())
    }
}

impl From<Scalar> for ScalarPrimitive<NistP256> {
    fn from(scalar: Scalar) -> Self {
        Self::from(&scalar)
    }
}

impl From<&Scalar> for ScalarPrimitive<NistP256> {
    fn from(scalar: &Scalar) -> Self {
        ScalarPrimitive::from_uint_unchecked(U256::from(scalar))
    }
}

impl From<&SecretKey> for Scalar {
    fn from(secret_key: &SecretKey) -> Self {
        *secret_key.to_nonzero_scalar()
    }
}

impl From<Scalar> for U256 {
    fn from(scalar: Scalar) -> Self {
        U256::from(scalar.0)
    }
}

impl From<&Scalar> for U256 {
    fn from(scalar: &Scalar) -> Self {
        U256::from(&scalar.0)
    }
}

impl Add<Scalar> for Scalar {
    type Output = Scalar;

    fn add(self, other: Scalar) -> Scalar {
        Scalar(self.0 + other.0)
    }
}

impl Add<&Scalar> for &Scalar {
    type Output = Scalar;

    fn add(self, other: &Scalar) -> Scalar {
        Scalar(self.0 + other.0)
    }
}

impl Add<&Scalar> for Scalar {
    type Output = Scalar;

    fn add(self, other: &Scalar) -> Scalar {
        Scalar(self.0 + other.0)
    }
}

impl AddAssign<Scalar> for Scalar {
    fn add_assign(&mut self, rhs: Scalar) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&Scalar> for Scalar {
    fn add_assign(&mut self, rhs: &Scalar) {
        self.0 += rhs.0;
    }
}

impl Sub<Scalar> for Scalar {
    type Output = Scalar;

    fn sub(self, other: Scalar) -> Scalar {
        Scalar(self.0 - other.0)
    }
}

impl Sub<&Scalar> for &Scalar {
    type Output = Scalar;

    fn sub(self, other: &Scalar) -> Scalar {
        Scalar(self.0 - other.0)
    }
}

impl Sub<&Scalar> for Scalar {
    type Output = Scalar;

    fn sub(self, other: &Scalar) -> Scalar {
        Scalar(self.0 - other.0)
    }
}

impl SubAssign<Scalar> for Scalar {
    fn sub_assign(&mut self, rhs: Scalar) {
        self.0 -= rhs.0;
    }
}

impl SubAssign<&Scalar> for Scalar {
    fn sub_assign(&mut self, rhs: &Scalar) {
        self.0 -= rhs.0;
    }
}

impl Mul<Scalar> for Scalar {
    type Output = Scalar;

    fn mul(self, other: Scalar) -> Scalar {
        Scalar(self.0 * other.0)
    }
}

impl Mul<&Scalar> for &Scalar {
    type Output = Scalar;

    fn mul(self, other: &Scalar) -> Scalar {
        Scalar(self.0 * other.0)
    }
}

impl Mul<&Scalar> for Scalar {
    type Output = Scalar;

    fn mul(self, other: &Scalar) -> Scalar {
        Scalar(self.0 * other.0)
    }
}

impl MulAssign<Scalar> for Scalar {
    fn mul_assign(&mut self, rhs: Scalar) {
        self.0 *= rhs.0;
    }
}

impl MulAssign<&Scalar> for Scalar {
    fn mul_assign(&mut self, rhs: &Scalar) {
        self.0 *= rhs.0;
    }
}

impl Neg for Scalar {
    type Output = Scalar;

    fn neg(self) -> Scalar {
        Scalar(-self.0)
    }
}

impl Neg for &Scalar {
    type Output = Scalar;

    fn neg(self) -> Scalar {
        Scalar(-self.0)
    }
}

impl Reduce<U256> for Scalar {
    type Bytes = FieldBytes;

    fn reduce(n: U256) -> Self {
        Self(<p256::Scalar as Reduce<U256>>::reduce(n))
    }

    fn reduce_bytes(bytes: &FieldBytes) -> Self {
        Self(<p256::Scalar as Reduce<U256>>::reduce_bytes(bytes))
    }
}

impl ReduceNonZero<U256> for Scalar {
    fn reduce_nonzero(n: U256) -> Self {
        Self(<p256::Scalar as ReduceNonZero<U256>>::reduce_nonzero(n))
    }

    fn reduce_nonzero_bytes(bytes: &FieldBytes) -> Self {
        Self(<p256::Scalar as ReduceNonZero<U256>>::reduce_nonzero_bytes(bytes))
    }
}

impl Sum for Scalar {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|s| s.0).sum())
    }
}

impl<'a> Sum<&'a Scalar> for Scalar {
    fn sum<I: Iterator<Item = &'a Scalar>>(iter: I) -> Self {
        Self(iter.map(|s| &s.0).sum())
    }
}

impl Product for Scalar {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|s| s.0).product())
    }
}

impl<'a> Product<&'a Scalar> for Scalar {
    fn product<I: Iterator<Item = &'a Scalar>>(iter: I) -> Self {
        Self(iter.map(|s| &s.0).product())
    }
}

impl ConditionallySelectable for Scalar {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(p256::Scalar::conditional_select(&a.0, &b.0, choice))
    }
}

impl ConstantTimeEq for Scalar {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}
