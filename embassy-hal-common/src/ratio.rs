use core::ops::{Add, Div, Mul};

use num_traits::{CheckedAdd, CheckedDiv, CheckedMul};

/// Represents the ratio between two numbers.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ratio<T> {
    /// Numerator.
    numer: T,
    /// Denominator.
    denom: T,
}

impl<T> Ratio<T> {
    /// Creates a new `Ratio`.
    #[inline(always)]
    pub const fn new_raw(numer: T, denom: T) -> Ratio<T> {
        Ratio { numer, denom }
    }

    /// Gets an immutable reference to the numerator.
    #[inline(always)]
    pub const fn numer(&self) -> &T {
        &self.numer
    }

    /// Gets an immutable reference to the denominator.
    #[inline(always)]
    pub const fn denom(&self) -> &T {
        &self.denom
    }
}

impl<T: CheckedDiv> Ratio<T> {
    /// Converts to an integer, rounding towards zero.
    #[inline(always)]
    pub fn to_integer(&self) -> T {
        unwrap!(self.numer().checked_div(self.denom()))
    }
}

impl<T: CheckedMul> Div<T> for Ratio<T> {
    type Output = Self;

    #[inline(always)]
    fn div(mut self, rhs: T) -> Self::Output {
        self.denom = unwrap!(self.denom().checked_mul(&rhs));
        self
    }
}

impl<T: CheckedMul> Mul<T> for Ratio<T> {
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, rhs: T) -> Self::Output {
        self.numer = unwrap!(self.numer().checked_mul(&rhs));
        self
    }
}

impl<T: CheckedMul + CheckedAdd> Add<T> for Ratio<T> {
    type Output = Self;

    #[inline(always)]
    fn add(mut self, rhs: T) -> Self::Output {
        self.numer = unwrap!(unwrap!(self.denom().checked_mul(&rhs)).checked_add(self.numer()));
        self
    }
}

macro_rules! impl_from_for_float {
    ($from:ident) => {
        impl From<Ratio<$from>> for f32 {
            #[inline(always)]
            fn from(r: Ratio<$from>) -> Self {
                (r.numer as f32) / (r.denom as f32)
            }
        }

        impl From<Ratio<$from>> for f64 {
            #[inline(always)]
            fn from(r: Ratio<$from>) -> Self {
                (r.numer as f64) / (r.denom as f64)
            }
        }
    };
}

impl_from_for_float!(u8);
impl_from_for_float!(u16);
impl_from_for_float!(u32);
impl_from_for_float!(u64);
impl_from_for_float!(u128);
impl_from_for_float!(i8);
impl_from_for_float!(i16);
impl_from_for_float!(i32);
impl_from_for_float!(i64);
impl_from_for_float!(i128);

impl<T: core::fmt::Display> core::fmt::Display for Ratio<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "{} / {}", self.numer(), self.denom())
    }
}

#[cfg(test)]
mod tests {
    use super::Ratio;

    #[test]
    fn basics() {
        let mut r = Ratio::new_raw(1, 2) + 2;
        assert_eq!(*r.numer(), 5);
        assert_eq!(*r.denom(), 2);
        assert_eq!(r.to_integer(), 2);

        r = r * 2;
        assert_eq!(*r.numer(), 10);
        assert_eq!(*r.denom(), 2);
        assert_eq!(r.to_integer(), 5);

        r = r / 2;
        assert_eq!(*r.numer(), 10);
        assert_eq!(*r.denom(), 4);
        assert_eq!(r.to_integer(), 2);
    }
}
