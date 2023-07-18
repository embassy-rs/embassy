// Credit: taken from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp2040-hal/src/float/mod.rs

use core::ops;

// Borrowed and simplified from compiler-builtins so we can use bit ops
// on floating point without macro soup.
pub(crate) trait Int:
    Copy
    + core::fmt::Debug
    + PartialEq
    + PartialOrd
    + ops::AddAssign
    + ops::SubAssign
    + ops::BitAndAssign
    + ops::BitOrAssign
    + ops::BitXorAssign
    + ops::ShlAssign<i32>
    + ops::ShrAssign<u32>
    + ops::Add<Output = Self>
    + ops::Sub<Output = Self>
    + ops::Div<Output = Self>
    + ops::Shl<u32, Output = Self>
    + ops::Shr<u32, Output = Self>
    + ops::BitOr<Output = Self>
    + ops::BitXor<Output = Self>
    + ops::BitAnd<Output = Self>
    + ops::Not<Output = Self>
{
    const ZERO: Self;
}

macro_rules! int_impl {
    ($ty:ty) => {
        impl Int for $ty {
            const ZERO: Self = 0;
        }
    };
}

int_impl!(u32);
int_impl!(u64);

pub(crate) trait Float:
    Copy
    + core::fmt::Debug
    + PartialEq
    + PartialOrd
    + ops::AddAssign
    + ops::MulAssign
    + ops::Add<Output = Self>
    + ops::Sub<Output = Self>
    + ops::Div<Output = Self>
    + ops::Rem<Output = Self>
{
    /// A uint of the same with as the float
    type Int: Int;

    /// NaN representation for the float
    const NAN: Self;

    /// The bitwidth of the float type
    const BITS: u32;

    /// The bitwidth of the significand
    const SIGNIFICAND_BITS: u32;

    /// A mask for the sign bit
    const SIGN_MASK: Self::Int;

    /// A mask for the significand
    const SIGNIFICAND_MASK: Self::Int;

    /// A mask for the exponent
    const EXPONENT_MASK: Self::Int;

    /// Returns `self` transmuted to `Self::Int`
    fn repr(self) -> Self::Int;

    /// Returns a `Self::Int` transmuted back to `Self`
    fn from_repr(a: Self::Int) -> Self;

    /// Return a sign swapped `self`
    fn negate(self) -> Self;

    /// Returns true if `self` is either NaN or infinity
    fn is_not_finite(self) -> bool {
        (self.repr() & Self::EXPONENT_MASK) == Self::EXPONENT_MASK
    }

    /// Returns true if `self` is infinity
    fn is_infinity(self) -> bool {
        (self.repr() & (Self::EXPONENT_MASK | Self::SIGNIFICAND_MASK)) == Self::EXPONENT_MASK
    }

    /// Returns true if `self is NaN
    fn is_nan(self) -> bool {
        (self.repr() & (Self::EXPONENT_MASK | Self::SIGNIFICAND_MASK)) > Self::EXPONENT_MASK
    }

    /// Returns true if `self` is negative
    fn is_sign_negative(self) -> bool {
        (self.repr() & Self::SIGN_MASK) != Self::Int::ZERO
    }

    /// Returns true if `self` is zero (either sign)
    fn is_zero(self) -> bool {
        (self.repr() & (Self::SIGNIFICAND_MASK | Self::EXPONENT_MASK)) == Self::Int::ZERO
    }
}

macro_rules! float_impl {
    ($ty:ident, $ity:ident, $bits:expr, $significand_bits:expr) => {
        impl Float for $ty {
            type Int = $ity;

            const NAN: Self = <$ty>::NAN;

            const BITS: u32 = $bits;
            const SIGNIFICAND_BITS: u32 = $significand_bits;

            const SIGN_MASK: Self::Int = 1 << (Self::BITS - 1);
            const SIGNIFICAND_MASK: Self::Int = (1 << Self::SIGNIFICAND_BITS) - 1;
            const EXPONENT_MASK: Self::Int = !(Self::SIGN_MASK | Self::SIGNIFICAND_MASK);

            fn repr(self) -> Self::Int {
                self.to_bits()
            }

            fn from_repr(a: Self::Int) -> Self {
                Self::from_bits(a)
            }

            fn negate(self) -> Self {
                -self
            }
        }
    };
}

float_impl!(f32, u32, 32, 23);
float_impl!(f64, u64, 64, 52);

mod add_sub;
mod cmp;
mod conv;
mod div;
mod functions;
mod mul;
