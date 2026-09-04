//! Elliptic-curve arithmetic over the RustCrypto traits, with the expensive
//! operations routed to `embassy-crypto-driver` accelerators.
//!
//! [`Accelerated<C>`] implements [`CurveArithmetic`] for any RustCrypto
//! curve `C` that implements [`Backend`], so the generic `elliptic-curve` and
//! `ecdsa` machinery (`PublicKey`, `SecretKey`, `ecdh::diffie_hellman`,
//! `ecdsa::SigningKey`, ...) works with it unchanged. Curve modules such as
//! [`crate::p256`] provide the backend and type aliases.
//!
//! The types wrap `C`'s own types and delegate every trait to them, except
//! for these entry points, which go to the backend's driver:
//!
//! - Scalar multiplication, when [`Backend::ACCELERATED_MUL`] is set.
//! - Scalar inversion, when [`Backend::ACCELERATED_INVERT`] is set.
//! - `k1 * P1 + k2 * P2` (ECDSA verification), when
//!   [`Backend::ACCELERATED_LINCOMB`] is set.
//!
//! Everything else (scalar field, point addition and doubling, encoding,
//! validation) runs in software via `C`.
//!
//! Drivers exchange canonical big-endian bytes and affine points. A
//! [`ProjectivePoint`] that came from affine input or from a driver keeps its
//! affine form, so converting it back is free.

mod affine;
mod projective;
mod scalar;

use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::Shr;

use elliptic_curve::ff::{Field, PrimeField};
#[allow(deprecated)]
use elliptic_curve::generic_array::ArrayLength;
use elliptic_curve::group::prime::PrimeCurveAffine;
use elliptic_curve::group::{self, GroupEncoding};
use elliptic_curve::ops::{LinearCombination, MulByGenerator, ReduceNonZero};
use elliptic_curve::point::{DecompactPoint, DecompressPoint, PointCompaction, PointCompression};
use elliptic_curve::sec1::{CompressedPoint, FromEncodedPoint, ModulusSize, ToCompactEncodedPoint, ToEncodedPoint};
use elliptic_curve::subtle::{ConditionallySelectable, CtOption};
use elliptic_curve::{Curve, CurveArithmetic, FieldBytes, FieldBytesEncoding, PrimeCurve, PrimeCurveArithmetic};

pub use affine::AffinePoint;
pub use projective::ProjectivePoint;
pub use scalar::Scalar;

/// SEC1-encoded point of the wrapped curve (identical to the accelerated curve's).
pub type EncodedPoint<C> = elliptic_curve::sec1::EncodedPoint<C>;

/// A RustCrypto curve whose expensive operations can be delegated to a driver.
///
/// Implemented for the wrapped curve type (for example `p256::NistP256`);
/// [`Accelerated<Self>`] is then the curve type to use with the RustCrypto
/// generics. The supertrait bounds spell out what the wrapped types must
/// provide beyond [`CurveArithmetic`]; every `primeorder`-based curve
/// satisfies them.
///
/// The `ACCELERATED_*` constants say which operations the driver provides;
/// the corresponding methods have software defaults and are only called when
/// the constant is set. The byte-array signatures are the canonical
/// representations: big-endian scalars and affine coordinates.
#[allow(deprecated)]
pub trait Backend:
    PrimeCurve
    + PointCompression
    + PointCompaction
    + Curve<
        FieldBytesSize: ModulusSize<
            CompressedPointSize: ArrayLength<u8, ArrayType: Copy>,
            UncompressedPointSize: ArrayLength<u8, ArrayType: Copy>,
        > + ArrayLength<u8, ArrayType: Copy>,
        Uint: FieldBytesEncoding<Accelerated<Self>> + From<Scalar<Self>> + for<'a> From<&'a Scalar<Self>>,
    > + CurveArithmetic<
        Scalar: Ord + From<u32> + From<u128> + ReduceNonZero<<Self as Curve>::Uint>,
        AffinePoint: PrimeCurveAffine<
            Curve = <Self as CurveArithmetic>::ProjectivePoint,
            Scalar = <Self as CurveArithmetic>::Scalar,
        > + GroupEncoding<Repr = CompressedPoint<Self>>
                         + FromEncodedPoint<Self>
                         + ToEncodedPoint<Self>
                         + ToCompactEncodedPoint<Self>
                         + DecompressPoint<Self>
                         + DecompactPoint<Self>,
    >
{
    /// The point at infinity, in affine form.
    const AFFINE_IDENTITY: Self::AffinePoint;

    /// The base point, in affine form.
    const AFFINE_GENERATOR: Self::AffinePoint;

    /// Whether [`mul_base`](Self::mul_base) and [`mul_affine`](Self::mul_affine)
    /// are provided by a driver.
    const ACCELERATED_MUL: bool = false;

    /// Whether [`invert`](Self::invert) and [`invert_vartime`](Self::invert_vartime)
    /// are provided by a driver.
    const ACCELERATED_INVERT: bool = false;

    /// Whether [`lincomb`](Self::lincomb) is provided by a driver.
    const ACCELERATED_LINCOMB: bool = false;

    /// `k * G`, for `k` in `[1, n-1]`. The result is never the identity.
    fn mul_base(k: &FieldBytes<Self>) -> (FieldBytes<Self>, FieldBytes<Self>) {
        point_to_bytes::<Self>(&group::Curve::to_affine(&Self::ProjectivePoint::mul_by_generator(
            &scalar_from_bytes::<Self>(k),
        )))
    }

    /// `k * P`, for `k` in `[1, n-1]` and a valid affine `P`. The result is
    /// never the identity.
    fn mul_affine(
        k: &FieldBytes<Self>,
        x: &FieldBytes<Self>,
        y: &FieldBytes<Self>,
    ) -> (FieldBytes<Self>, FieldBytes<Self>) {
        point_to_bytes::<Self>(&group::Curve::to_affine(
            &(point_from_bytes::<Self>(x, y) * scalar_from_bytes::<Self>(k)),
        ))
    }

    /// `k^-1 mod n`, for `k` in `[1, n-1]`, in constant time.
    fn invert(k: &FieldBytes<Self>) -> FieldBytes<Self> {
        Field::invert(&scalar_from_bytes::<Self>(k)).unwrap().to_repr()
    }

    /// `k^-1 mod n`, for `k` in `[1, n-1]`, in variable time (public inputs only).
    fn invert_vartime(k: &FieldBytes<Self>) -> FieldBytes<Self> {
        elliptic_curve::ops::Invert::invert_vartime(&scalar_from_bytes::<Self>(k))
            .unwrap()
            .to_repr()
    }

    /// `k1 * P1 + k2 * P2`, for scalars in `[1, n-1]` and valid affine points,
    /// or `None` if the sum is the identity.
    #[allow(clippy::too_many_arguments)]
    fn lincomb(
        k1: &FieldBytes<Self>,
        x1: &FieldBytes<Self>,
        y1: &FieldBytes<Self>,
        k2: &FieldBytes<Self>,
        x2: &FieldBytes<Self>,
        y2: &FieldBytes<Self>,
    ) -> Option<(FieldBytes<Self>, FieldBytes<Self>)> {
        let sum = Self::ProjectivePoint::lincomb(
            &point_from_bytes::<Self>(x1, y1).to_curve(),
            &scalar_from_bytes::<Self>(k1),
            &point_from_bytes::<Self>(x2, y2).to_curve(),
            &scalar_from_bytes::<Self>(k2),
        );

        (!bool::from(group::Group::is_identity(&sum))).then(|| point_to_bytes::<Self>(&group::Curve::to_affine(&sum)))
    }
}

/// The accelerated version of curve `C`: same curve, same wire formats, with
/// [`CurveArithmetic`] types that go through `C`'s driver.
pub struct Accelerated<C>(PhantomData<C>);

impl<C> Clone for Accelerated<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for Accelerated<C> {}

impl<C> fmt::Debug for Accelerated<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Accelerated")
    }
}

impl<C> Default for Accelerated<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<C> PartialEq for Accelerated<C> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<C> Eq for Accelerated<C> {}

impl<C> PartialOrd for Accelerated<C> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<C> Ord for Accelerated<C> {
    fn cmp(&self, _: &Self) -> core::cmp::Ordering {
        core::cmp::Ordering::Equal
    }
}

impl<C> Hash for Accelerated<C> {
    fn hash<H: Hasher>(&self, _: &mut H) {}
}

impl<C: Backend> Curve for Accelerated<C> {
    type FieldBytesSize = C::FieldBytesSize;
    type Uint = C::Uint;

    const ORDER: C::Uint = C::ORDER;
}

impl<C: Backend> PrimeCurve for Accelerated<C> {}

impl<C: Backend> PointCompression for Accelerated<C> {
    const COMPRESS_POINTS: bool = C::COMPRESS_POINTS;
}

impl<C: Backend> PointCompaction for Accelerated<C> {
    const COMPACT_POINTS: bool = C::COMPACT_POINTS;
}

impl<C: Backend> CurveArithmetic for Accelerated<C> {
    type AffinePoint = AffinePoint<C>;
    type ProjectivePoint = ProjectivePoint<C>;
    type Scalar = Scalar<C>;
}

impl<C: Backend> PrimeCurveArithmetic for Accelerated<C> {
    type CurveGroup = ProjectivePoint<C>;
}

/// ECDSA over an accelerated curve.
#[cfg(feature = "ecdsa")]
mod ecdsa_impls {
    use ecdsa::SignatureSize;
    use ecdsa::hazmat::{DigestPrimitive, SignPrimitive, VerifyPrimitive};
    #[allow(deprecated)]
    use elliptic_curve::generic_array::ArrayLength;

    use super::{Accelerated, AffinePoint, Backend, Scalar};

    impl<C: Backend + DigestPrimitive> DigestPrimitive for Accelerated<C> {
        type Digest = C::Digest;
    }

    #[allow(deprecated)]
    impl<C: Backend> SignPrimitive<Accelerated<C>> for Scalar<C> where SignatureSize<Accelerated<C>>: ArrayLength<u8> {}

    #[allow(deprecated)]
    impl<C: Backend> VerifyPrimitive<Accelerated<C>> for AffinePoint<C> where SignatureSize<Accelerated<C>>: ArrayLength<u8> {}
}

// ===========================================================================
// Driver-backed operations
//
// These are the only functions in this module that do anything but delegate.
// Each has a software path (the wrapped curve's own arithmetic) and a driver
// path, chosen by the backend's `ACCELERATED_*` constants. The driver
// contracts exclude the degenerate operands (zero scalar, identity point)
// since neither has a canonical encoding, so they are handled here, without
// branching on the operands.
// ===========================================================================

/// `k * G`.
fn mul_base<C: Backend>(k: &Scalar<C>) -> ProjectivePoint<C> {
    if !C::ACCELERATED_MUL {
        return ProjectivePoint::from_projective(C::ProjectivePoint::mul_by_generator(&k.0));
    }

    let k_is_zero = k.0.is_zero();

    // Feed the driver a scalar that is always in range; the result is then
    // discarded (replaced with the identity) if `k` was zero.
    let k = C::Scalar::conditional_select(&k.0, &C::Scalar::ONE, k_is_zero);

    let (x, y) = C::mul_base(&k.to_repr());
    let result = ProjectivePoint::from_affine(point_from_bytes::<C>(&x, &y));

    ProjectivePoint::conditional_select(&result, &ProjectivePoint::IDENTITY, k_is_zero)
}

/// `k * P`.
fn mul<C: Backend>(k: &Scalar<C>, p: &AffinePoint<C>) -> ProjectivePoint<C> {
    if !C::ACCELERATED_MUL {
        return ProjectivePoint::from_projective(p.0 * k.0);
    }

    let k_is_zero = k.0.is_zero();
    let p_is_identity = p.0.is_identity();
    let degenerate = k_is_zero | p_is_identity;

    // Feed the driver operands that are always within its contract; the
    // result is then discarded (replaced with the identity) if either of the
    // original operands was degenerate.
    let k = C::Scalar::conditional_select(&k.0, &C::Scalar::ONE, k_is_zero);
    let p = C::AffinePoint::conditional_select(&p.0, &C::AFFINE_GENERATOR, p_is_identity);

    let (px, py) = point_to_bytes::<C>(&p);
    let (x, y) = C::mul_affine(&k.to_repr(), &px, &py);
    let result = ProjectivePoint::from_affine(point_from_bytes::<C>(&x, &y));

    ProjectivePoint::conditional_select(&result, &ProjectivePoint::IDENTITY, degenerate)
}

/// `k * P` for a projective `P`.
fn mul_projective<C: Backend>(k: &Scalar<C>, p: &ProjectivePoint<C>) -> ProjectivePoint<C> {
    if !C::ACCELERATED_MUL {
        return ProjectivePoint::from_projective(p.projective() * k.0);
    }

    // Free of an inversion when `P` came from affine input or from the driver.
    mul(k, &AffinePoint(p.affine()))
}

/// `k^-1`.
fn invert<C: Backend>(k: &Scalar<C>) -> CtOption<Scalar<C>> {
    if !C::ACCELERATED_INVERT {
        return Field::invert(&k.0).map(Scalar);
    }

    let k_is_zero = k.0.is_zero();

    // Feed the driver a scalar that is always invertible; the result is
    // reported as absent if `k` was zero.
    let k = C::Scalar::conditional_select(&k.0, &C::Scalar::ONE, k_is_zero);

    let result = scalar_from_bytes::<C>(&C::invert(&k.to_repr()));

    CtOption::new(Scalar(result), !k_is_zero)
}

/// `k^-1`, variable time.
fn invert_vartime<C: Backend>(k: &Scalar<C>) -> CtOption<Scalar<C>> {
    if !C::ACCELERATED_INVERT {
        return elliptic_curve::ops::Invert::invert_vartime(&k.0).map(Scalar);
    }

    let k_is_zero = k.0.is_zero();
    let k = C::Scalar::conditional_select(&k.0, &C::Scalar::ONE, k_is_zero);

    let result = scalar_from_bytes::<C>(&C::invert_vartime(&k.to_repr()));

    CtOption::new(Scalar(result), !k_is_zero)
}

/// `k1 * p1 + k2 * p2`.
fn lincomb<C: Backend>(
    p1: &ProjectivePoint<C>,
    k1: &Scalar<C>,
    p2: &ProjectivePoint<C>,
    k2: &Scalar<C>,
) -> ProjectivePoint<C> {
    if !C::ACCELERATED_LINCOMB {
        return if C::ACCELERATED_MUL {
            (*p1 * k1) + (*p2 * k2)
        } else {
            ProjectivePoint::from_projective(C::ProjectivePoint::lincomb(
                &p1.projective(),
                &k1.0,
                &p2.projective(),
                &k2.0,
            ))
        };
    }

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
    let k1 = C::Scalar::conditional_select(&k1.0, &C::Scalar::ONE, degenerate1);
    let k2 = C::Scalar::conditional_select(&k2.0, &C::Scalar::ONE, degenerate2);
    let p1 = C::AffinePoint::conditional_select(&p1, &C::AFFINE_GENERATOR, p1.is_identity());
    let p2 = C::AffinePoint::conditional_select(&p2, &C::AFFINE_GENERATOR, p2.is_identity());

    let (x1, y1) = point_to_bytes::<C>(&p1);
    let (x2, y2) = point_to_bytes::<C>(&p2);
    let sum = C::lincomb(&k1.to_repr(), &x1, &y1, &k2.to_repr(), &x2, &y2);

    // Whether the sum is the identity is not secret (see the driver contract).
    let sum = match sum {
        Some((x, y)) => ProjectivePoint::from_affine(point_from_bytes::<C>(&x, &y)),
        None => ProjectivePoint::IDENTITY,
    };

    let extra1 = C::AffinePoint::conditional_select(&C::AFFINE_IDENTITY, &p1, degenerate1);
    let extra2 = C::AffinePoint::conditional_select(&C::AFFINE_IDENTITY, &p2, degenerate2);
    let fixed = sum - AffinePoint(extra1) - AffinePoint(extra2);

    ProjectivePoint::conditional_select(&sum, &fixed, degenerate)
}

/// Decode a scalar returned by a driver.
///
/// The driver contract guarantees it is in range, so a failure here is a
/// broken driver.
fn scalar_from_bytes<C: Backend>(k: &FieldBytes<C>) -> C::Scalar {
    C::Scalar::from_repr(*k).expect("driver returned an out-of-range scalar")
}

/// `p` must not be the identity.
fn point_to_bytes<C: Backend>(p: &C::AffinePoint) -> (FieldBytes<C>, FieldBytes<C>) {
    let encoded = p.to_encoded_point(false);

    // Only the identity encodes without coordinates, and the callers
    // never pass the identity.
    let x = encoded.x().expect("not the identity");
    let y = encoded.y().expect("not the identity");

    (*x, *y)
}

/// Decode a point returned by a driver.
///
/// The decode validates that the point is on the curve; the driver contract
/// guarantees it, so a failure here is a broken driver.
fn point_from_bytes<C: Backend>(x: &FieldBytes<C>, y: &FieldBytes<C>) -> C::AffinePoint {
    let encoded = EncodedPoint::<C>::from_affine_coordinates(x, y, false);

    C::AffinePoint::from_encoded_point(&encoded).expect("driver returned an invalid point")
}

#[allow(dead_code)]
fn assert_bounds<C: Backend>() {
    fn is_curve_arithmetic<T: CurveArithmetic>() {}
    fn is_prime_curve_arithmetic<T: PrimeCurveArithmetic>() {}
    fn is_shr<T: Shr<usize>>() {}

    is_curve_arithmetic::<Accelerated<C>>();
    is_prime_curve_arithmetic::<Accelerated<C>>();
    is_shr::<Scalar<C>>();
}
