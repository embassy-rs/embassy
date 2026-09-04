//! The P-256 scalar type: [`Scalar`].

use core::fmt;
use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, ShrAssign, Sub, SubAssign};

use crypto_bigint::U256;
use elliptic_curve::ops::{Invert, Reduce, ReduceNonZero};
use elliptic_curve::scalar::{FromUintUnchecked, IsHigh};
use elliptic_curve::{FieldBytes, FieldBytesEncoding, ScalarPrimitive};
use embassy_crypto_driver as drv;
use ff::{Field, PrimeField};
use rand_core::RngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess, CtOption};
use zeroize::DefaultIsZeroes;

use crate::p256::constants::{
    DELTA, MODULUS_DEC, MULT_GENERATOR, N, N_HALF, ROOT_OF_UNITY, ROOT_OF_UNITY_INV, SQRT_Q, SQRT_Q1_2, TWO_INV,
};
use crate::p256::ct::{ct_eq_bytes, ct_gt_bytes, ct_lt_bytes, ct_select_bytes};

/// Number of Tonelli–Shanks rounds: `S - 1` where `v2(n - 1) = S = 4`.
const TS_ROUNDS: usize = 3;

// ===========================================================================
// Representation
// ===========================================================================

/// Tagged storage for a scalar: either the driver's opaque backend handle
/// (the zero-conversion representation intended by the unitrait design) or a
/// canonical big-endian encoding (used for associated `const`s, `from_repr`,
/// `random`, and freshly decoded values; converted to `Native` lazily on the
/// first operation).
///
/// An enum rather than a union+flag because the opaque handle is `Copy`
/// (declared `pub type Scalar: Copy` on the driver trait): both payloads are
/// plain `Copy` data, the enum derives `Copy`/`Clone`, and every access is
/// safe — no `unsafe`, no `ManuallyDrop`, no byte transmutes.
#[derive(Copy, Clone)]
enum Repr {
    Native(drv::P256OpsImplScalar),
    Canon([u8; 32]),
}

/// A P-256 scalar: `GF(n)` element, the field of `elliptic_curve::CurveArithmetic::Scalar`.
///
/// See the crate-level docs for the representation and the backend contract.
#[derive(Copy, Clone)]
pub struct Scalar {
    repr: Repr,
}

impl Scalar {
    /// Wrap a canonical encoding without touching the backend.
    #[inline]
    pub(crate) const fn from_canon(c: [u8; 32]) -> Self {
        Self { repr: Repr::Canon(c) }
    }

    /// Wrap an opaque backend handle.
    #[inline]
    fn from_native(n: drv::P256OpsImplScalar) -> Self {
        Self { repr: Repr::Native(n) }
    }

    /// Obtain an owned opaque backend handle, decoding canonical storage first
    /// if necessary (one `scalar_from_canonical` call).
    #[inline]
    pub(crate) fn to_opaque(&self) -> drv::P256OpsImplScalar {
        match &self.repr {
            // The handle is `Copy`: a plain bitwise copy, no drop glue and no
            // backend round-trip.
            Repr::Native(n) => *n,
            Repr::Canon(c) => drv::P256OpsImpl::scalar_from_canonical(&drv::P256Scalar(*c)),
        }
    }

    /// Canonical big-endian encoding (one `scalar_to_canonical` call for native
    /// storage; free for canonical storage).
    #[inline]
    pub fn to_canonical_bytes(&self) -> [u8; 32] {
        match &self.repr {
            Repr::Native(n) => drv::P256OpsImpl::scalar_to_canonical(n).0,
            Repr::Canon(c) => *c,
        }
    }

    /// Raise to a fixed public exponent (256-bit big-endian) by
    /// square-and-multiply. Constant-time with respect to `self`.
    fn pow_ct(&self, exp_be: &[u8; 32]) -> Scalar {
        let mut acc = Scalar::ONE;
        for byte in 0..32 {
            for bit in 0..8 {
                acc = acc.square();
                let b = Choice::from((exp_be[byte] >> (7 - bit)) & 1);
                let acc2 = acc * self;
                acc = Scalar::conditional_select(&acc, &acc2, b);
            }
        }
        acc
    }

    /// Constant-time Tonelli–Shanks square root for `n ≡ 1 (mod 4)`, `S = 4`.
    ///
    /// Returns `(is_square, root)` where `root² == a` whenever `is_square` is
    /// true (including `a == 0`).
    fn ts_sqrt(a: &Scalar) -> (Choice, Scalar) {
        let a_zero = a.ct_eq(&Scalar::ZERO);
        let mut t = a.pow_ct(&SQRT_Q);
        let mut x = a.pow_ct(&SQRT_Q1_2);
        // c = ROOT_OF_UNITY^Q, an element of exact order 2^S = 16.
        let mut c = Scalar::ROOT_OF_UNITY.pow_ct(&SQRT_Q);
        let mut m: usize = 4; // S

        for _ in 0..TS_ROUNDS {
            let done = t.ct_eq(&Scalar::ONE);
            let t2 = t.square();
            let t4 = t2.square();
            let i1 = t2.ct_eq(&Scalar::ONE);
            let i2 = (!i1) & t4.ct_eq(&Scalar::ONE);
            // i = least index in {1,2,3} with t^(2^i) == 1; defaults to 3, which
            // also covers the "no such index" (non-square) case.
            let n1 = (!i1).unwrap_u8() as usize;
            let n2 = ((!i1) & (!i2)).unwrap_u8() as usize;
            let i = 1 + n1 + n2;
            // b = c^(2^(m-i-1)); exponent e = m-i-1 is in {0,1,2}.
            // `saturating_sub` keeps garbage (non-square) inputs from underflowing.
            let e = (m - 1).saturating_sub(i);
            let c2 = c.square();
            let c4 = c2.square();
            let mut b = Scalar::conditional_select(&c2, &c, Choice::from((e == 0) as u8));
            b = Scalar::conditional_select(&b, &c4, Choice::from((e == 2) as u8));
            let b2 = b.square();
            let apply = !done;
            x = Scalar::conditional_select(&x, &(x * b), apply);
            t = Scalar::conditional_select(&t, &(t * b2), apply);
            c = Scalar::conditional_select(&c, &b2, apply);
            let mask = 0usize.wrapping_sub(apply.unwrap_u8() as usize);
            m = (m & !mask) | (i & mask);
        }

        let is_square = t.ct_eq(&Scalar::ONE) | a_zero;
        let root = Scalar::conditional_select(&x, &Scalar::ZERO, a_zero);
        (is_square, root)
    }
}

// ===========================================================================
// Core trait impls that don't touch the backend
// ===========================================================================

impl Default for Scalar {
    #[inline]
    fn default() -> Self {
        Scalar::ZERO
    }
}

impl fmt::Debug for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Scalar(0x")?;
        for b in self.to_canonical_bytes() {
            write!(f, "{b:02x}")?;
        }
        f.write_str(")")
    }
}

impl PartialEq for Scalar {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}
impl Eq for Scalar {}

impl PartialOrd for Scalar {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.to_canonical_bytes().cmp(&other.to_canonical_bytes()))
    }
}

// Note: `Zeroize` comes from zeroize's blanket impl for `DefaultIsZeroes`
// types (`*self = Self::default()` — i.e. the canonical zero, exactly what a
// manual impl would do).
impl DefaultIsZeroes for Scalar {}

impl ConstantTimeEq for Scalar {
    fn ct_eq(&self, other: &Self) -> Choice {
        ct_eq_bytes(&self.to_canonical_bytes(), &other.to_canonical_bytes())
    }
}

impl ConstantTimeGreater for Scalar {
    fn ct_gt(&self, other: &Self) -> Choice {
        ct_gt_bytes(&self.to_canonical_bytes(), &other.to_canonical_bytes())
    }
}

impl ConstantTimeLess for Scalar {
    fn ct_lt(&self, other: &Self) -> Choice {
        ct_lt_bytes(&self.to_canonical_bytes(), &other.to_canonical_bytes())
    }
}

impl ConditionallySelectable for Scalar {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        // Select in the canonical domain so a canonical-tagged result needs no
        // backend round-trip. `scalar_to_canonical` must be constant-time.
        let ab = a.to_canonical_bytes();
        let bb = b.to_canonical_bytes();
        Scalar::from_canon(ct_select_bytes(&ab, &bb, choice))
    }
}

// ===========================================================================
// Arithmetic (delegates to the driver unitrait)
// ===========================================================================

fn binop(
    a: &Scalar,
    b: &Scalar,
    f: fn(&drv::P256OpsImplScalar, &drv::P256OpsImplScalar) -> drv::P256OpsImplScalar,
) -> Scalar {
    let (a, b) = (a.to_opaque(), b.to_opaque());
    Scalar::from_native(f(&a, &b))
}

macro_rules! impl_arith {
    ($trait:ident, $method:ident, $op:expr) => {
        impl $trait<Scalar> for Scalar {
            type Output = Scalar;
            #[inline]
            fn $method(self, rhs: Scalar) -> Scalar {
                binop(&self, &rhs, $op)
            }
        }
        impl<'a> $trait<&'a Scalar> for Scalar {
            type Output = Scalar;
            #[inline]
            fn $method(self, rhs: &'a Scalar) -> Scalar {
                binop(&self, rhs, $op)
            }
        }
        impl $trait<Scalar> for &Scalar {
            type Output = Scalar;
            #[inline]
            fn $method(self, rhs: Scalar) -> Scalar {
                binop(self, &rhs, $op)
            }
        }
        impl<'a> $trait<&'a Scalar> for &Scalar {
            type Output = Scalar;
            #[inline]
            fn $method(self, rhs: &'a Scalar) -> Scalar {
                binop(self, rhs, $op)
            }
        }
    };
}

impl_arith!(Add, add, drv::P256OpsImpl::scalar_add);
impl_arith!(Mul, mul, drv::P256OpsImpl::scalar_mul);

fn sub_impl(a: &Scalar, b: &Scalar) -> Scalar {
    // The driver trait has no `scalar_sub`; subtraction is a + (-b).
    let (a, b) = (a.to_opaque(), b.to_opaque());
    let neg_b = drv::P256OpsImpl::scalar_neg(&b);
    Scalar::from_native(drv::P256OpsImpl::scalar_add(&a, &neg_b))
}

impl Sub<Scalar> for Scalar {
    type Output = Scalar;
    #[inline]
    fn sub(self, rhs: Scalar) -> Scalar {
        sub_impl(&self, &rhs)
    }
}
impl<'a> Sub<&'a Scalar> for Scalar {
    type Output = Scalar;
    #[inline]
    fn sub(self, rhs: &'a Scalar) -> Scalar {
        sub_impl(&self, rhs)
    }
}
impl Sub<Scalar> for &Scalar {
    type Output = Scalar;
    #[inline]
    fn sub(self, rhs: Scalar) -> Scalar {
        sub_impl(self, &rhs)
    }
}
impl<'a> Sub<&'a Scalar> for &Scalar {
    type Output = Scalar;
    #[inline]
    fn sub(self, rhs: &'a Scalar) -> Scalar {
        sub_impl(self, rhs)
    }
}

macro_rules! impl_assign {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait<Scalar> for Scalar {
            #[inline]
            fn $method(&mut self, rhs: Scalar) {
                *self = *self $op rhs;
            }
        }
        impl<'a> $trait<&'a Scalar> for Scalar {
            #[inline]
            fn $method(&mut self, rhs: &'a Scalar) {
                *self = *self $op rhs;
            }
        }
    };
}

impl_assign!(AddAssign, add_assign, +);
impl_assign!(SubAssign, sub_assign, -);
impl_assign!(MulAssign, mul_assign, *);

impl Neg for Scalar {
    type Output = Scalar;
    #[inline]
    fn neg(self) -> Scalar {
        Scalar::from_native(drv::P256OpsImpl::scalar_neg(&self.to_opaque()))
    }
}

impl Sum for Scalar {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Scalar::ZERO, Add::add)
    }
}
impl<'a> Sum<&'a Scalar> for Scalar {
    fn sum<I: Iterator<Item = &'a Scalar>>(iter: I) -> Self {
        iter.fold(Scalar::ZERO, Add::add)
    }
}
impl Product for Scalar {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Scalar::ONE, Mul::mul)
    }
}
impl<'a> Product<&'a Scalar> for Scalar {
    fn product<I: Iterator<Item = &'a Scalar>>(iter: I) -> Self {
        iter.fold(Scalar::ONE, Mul::mul)
    }
}

impl Invert for Scalar {
    type Output = CtOption<Self>;

    fn invert(&self) -> CtOption<Self> {
        // v2 contract: `scalar_inv(0)` returns 0 (defined fallback); validity
        // is exactly `!scalar_is_zero(a)`. NOTE: parenthesize the `!` — `as`
        // binds tighter than unary `!`.
        let a = self.to_opaque();
        let inv = Scalar::from_native(drv::P256OpsImpl::scalar_inv(&a));
        CtOption::new(inv, Choice::from((!drv::P256OpsImpl::scalar_is_zero(&a)) as u8))
    }

    fn invert_vartime(&self) -> Self::Output {
        // v2 adds the variable-time inverse (ECDSA verify speedup); identical
        // fallback + validity contract as `invert`.
        let a = self.to_opaque();
        let inv = Scalar::from_native(drv::P256OpsImpl::scalar_inv_vartime(&a));
        CtOption::new(inv, Choice::from((!drv::P256OpsImpl::scalar_is_zero(&a)) as u8))
    }
}

// ===========================================================================
// `ff` traits
// ===========================================================================

impl Field for Scalar {
    const ZERO: Self = Scalar::from_canon([0u8; 32]);
    const ONE: Self = {
        let mut c = [0u8; 32];
        c[31] = 1;
        Scalar::from_canon(c)
    };

    fn random(mut rng: impl RngCore) -> Self {
        // Rejection sampling in the canonical domain: accepts iff 0 < b < n.
        // (`n > 2^255`, so acceptance probability is `> 1 - 2^-32`.)
        let mut bytes = [0u8; 32];
        loop {
            rng.fill_bytes(&mut bytes);
            if bytes != [0u8; 32] && bool::from(ct_lt_bytes(&bytes, &N)) {
                return Scalar::from_canon(bytes);
            }
        }
    }

    #[inline]
    fn square(&self) -> Self {
        binop(self, self, drv::P256OpsImpl::scalar_mul)
    }

    #[inline]
    fn double(&self) -> Self {
        binop(self, self, drv::P256OpsImpl::scalar_add)
    }

    fn invert(&self) -> CtOption<Self> {
        <Scalar as Invert>::invert(self)
    }

    fn sqrt_ratio(num: &Self, div: &Self) -> (Choice, Self) {
        let num_zero = num.ct_eq(&Scalar::ZERO);
        let div_zero = div.ct_eq(&Scalar::ZERO);
        let inv = Option::<Scalar>::from(<Scalar as Field>::invert(div)).unwrap_or(Scalar::ZERO);
        let r = *num * inv;

        let (sq, root) = Scalar::ts_sqrt(&r);
        // ff contract: on a non-square, return sqrt(G_S * r) with an unspecified
        // fixed non-square G_S; we use -1 (a non-residue since n == 1 mod 4 and
        // S = 4 implies the 2-Sylow structure). `-r` is then a square.
        let (_sq2, neg_root) = Scalar::ts_sqrt(&(-r));
        let root = Scalar::conditional_select(&neg_root, &root, sq);

        let choice = (sq & !div_zero) | num_zero;
        let root = Scalar::conditional_select(&root, &Scalar::ZERO, div_zero & !num_zero);
        (choice, root)
    }
}

impl PrimeField for Scalar {
    type Repr = FieldBytes<crate::p256::NistP256>;

    const MODULUS: &'static str = MODULUS_DEC;
    const NUM_BITS: u32 = 256;
    const CAPACITY: u32 = 255;
    const TWO_INV: Self = Scalar::from_canon(TWO_INV);
    const MULTIPLICATIVE_GENERATOR: Self = Scalar::from_canon(MULT_GENERATOR);
    /// `v2(n - 1) = 4`.
    const S: u32 = 4;
    const ROOT_OF_UNITY: Self = Scalar::from_canon(ROOT_OF_UNITY);
    const ROOT_OF_UNITY_INV: Self = Scalar::from_canon(ROOT_OF_UNITY_INV);
    const DELTA: Self = Scalar::from_canon(DELTA);

    fn from_repr(repr: Self::Repr) -> CtOption<Self> {
        let arr: [u8; 32] = AsRef::<[u8]>::as_ref(&repr)
            .try_into()
            .expect("FieldBytes<NistP256> is 32 bytes");
        CtOption::new(Scalar::from_canon(arr), ct_lt_bytes(&arr, &N))
    }

    fn to_repr(&self) -> Self::Repr {
        let bytes = self.to_canonical_bytes();
        let mut out = FieldBytes::<crate::p256::NistP256>::default();
        out.copy_from_slice(&bytes);
        out
    }

    fn is_odd(&self) -> Choice {
        Choice::from(self.to_canonical_bytes()[31] & 1)
    }
}

// ===========================================================================
// `elliptic-curve` scalar-side traits
// ===========================================================================

impl IsHigh for Scalar {
    fn is_high(&self) -> Choice {
        ct_gt_bytes(&self.to_canonical_bytes(), &N_HALF)
    }
}

impl ShrAssign<usize> for Scalar {
    fn shr_assign(&mut self, rhs: usize) {
        // Operate in the canonical domain; result stays canonical-tagged.
        let mut b = self.to_canonical_bytes();
        if rhs >= 256 {
            b = [0u8; 32];
        } else {
            let byte_shift = rhs / 8;
            let bit_shift = rhs % 8;
            if byte_shift > 0 {
                b.copy_within(byte_shift..32, 0);
                for x in &mut b[32 - byte_shift..] {
                    *x = 0;
                }
            }
            if bit_shift > 0 {
                for i in 0..32 {
                    let hi = if i + 1 < 32 { b[i + 1] } else { 0 };
                    b[i] = (b[i] >> bit_shift) | (hi << (8 - bit_shift));
                }
            }
        }
        *self = Scalar::from_canon(b);
    }
}

impl AsRef<Scalar> for Scalar {
    #[inline]
    fn as_ref(&self) -> &Scalar {
        self
    }
}

impl From<u64> for Scalar {
    fn from(v: u64) -> Self {
        let mut c = [0u8; 32];
        c[24..].copy_from_slice(&v.to_be_bytes());
        // v < 2^64 < n, so this is canonical without a backend call.
        Scalar::from_canon(c)
    }
}

impl From<ScalarPrimitive<crate::p256::NistP256>> for Scalar {
    fn from(prim: ScalarPrimitive<crate::p256::NistP256>) -> Self {
        // `ScalarPrimitive` is `< n` by construction.
        Option::<Self>::from(<Scalar as PrimeField>::from_repr(prim.to_bytes())).expect("ScalarPrimitive < order")
    }
}

impl From<Scalar> for ScalarPrimitive<crate::p256::NistP256> {
    fn from(s: Scalar) -> Self {
        Option::<Self>::from(ScalarPrimitive::<crate::p256::NistP256>::from_bytes(&s.to_repr()))
            .expect("scalar < order")
    }
}

impl From<Scalar> for FieldBytes<crate::p256::NistP256> {
    fn from(s: Scalar) -> Self {
        <Scalar as PrimeField>::to_repr(&s)
    }
}

impl From<Scalar> for U256 {
    fn from(s: Scalar) -> Self {
        <U256 as FieldBytesEncoding<crate::p256::NistP256>>::decode_field_bytes(&s.to_repr())
    }
}

// ===========================================================================
// Modular reduction (driver v2 `scalar_reduce_32bytes`)
//
// These three impls unblock `elliptic_curve::CurveArithmetic`, whose `Scalar`
// bound requires `Reduce<Self::Uint>` + `FromUintUnchecked`.
// ===========================================================================

impl Reduce<U256> for Scalar {
    type Bytes = FieldBytes<crate::p256::NistP256>;

    fn reduce(n: U256) -> Self {
        Self::reduce_bytes(&<U256 as FieldBytesEncoding<crate::p256::NistP256>>::encode_field_bytes(&n))
    }

    fn reduce_bytes(bytes: &Self::Bytes) -> Self {
        let arr: [u8; 32] = AsRef::<[u8]>::as_ref(bytes)
            .try_into()
            .expect("FieldBytes<NistP256> is 32 bytes");
        Scalar::from_native(drv::P256OpsImpl::scalar_reduce_bytes(&arr))
    }
}

impl ReduceNonZero<U256> for Scalar {
    fn reduce_nonzero(n: U256) -> Self {
        Self::reduce_nonzero_bytes(&<U256 as FieldBytesEncoding<crate::p256::NistP256>>::encode_field_bytes(&n))
    }

    fn reduce_nonzero_bytes(bytes: &Self::Bytes) -> Self {
        let s = Self::reduce_bytes(bytes);
        // The integer input was non-zero; if it is a non-zero multiple of n the
        // reduction is 0, which has no inverse — `ReduceNonZero`'s contract is a
        // non-zero output, so map that case to 1 (matches the RustCrypto curve
        // crates).
        Self::conditional_select(&s, &Scalar::ONE, s.ct_eq(&Scalar::ZERO))
    }
}

impl FromUintUnchecked for Scalar {
    type Uint = U256;

    fn from_uint_unchecked(uint: Self::Uint) -> Self {
        Self::reduce(uint)
    }
}
