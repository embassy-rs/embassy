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

pub use affine::AffinePoint;
use elliptic_curve::ff::{Field, PrimeField};
#[allow(deprecated)]
use elliptic_curve::generic_array::ArrayLength;
use elliptic_curve::group::prime::PrimeCurveAffine;
use elliptic_curve::group::{self, GroupEncoding};
use elliptic_curve::ops::{LinearCombination, MulByGenerator, Reduce};
#[cfg(feature = "pkcs8")]
use elliptic_curve::pkcs8::{AssociatedOid, ObjectIdentifier};
use elliptic_curve::point::{DecompactPoint, DecompressPoint, PointCompaction, PointCompression};
use elliptic_curve::sec1::{CompressedPoint, FromEncodedPoint, ModulusSize, ToCompactEncodedPoint, ToEncodedPoint};
use elliptic_curve::subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};
use elliptic_curve::{Curve, CurveArithmetic, FieldBytes, FieldBytesEncoding, PrimeCurve, PrimeCurveArithmetic};
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
/// satisfies them, plus `Reduce`. `ReduceNonZero` is provided through
/// [`Backend::reduce_nonzero`] instead of being required of the wrapped
/// scalar, since some curves (e.g. P-384) do not implement it.
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
        Scalar: Ord + From<u32> + From<u128> + Reduce<<Self as Curve>::Uint>,
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

    /// `k * G`, for `k` in `[0, n-1]`. A zero `k` makes the result the
    /// identity, reported as the all-zero sentinel coordinates (see
    /// `point_from_driver`).
    fn mul_base(k: &FieldBytes<Self>) -> (FieldBytes<Self>, FieldBytes<Self>) {
        let k = scalar_from_bytes::<Self>(k);
        let result = Self::ProjectivePoint::mul_by_generator(&k);

        point_to_bytes_or_identity::<Self>(&result)
    }

    /// `k * P`, for `k` in `[0, n-1]` and a valid affine `P`. A zero `k`
    /// makes the result the identity, reported as the all-zero sentinel
    /// coordinates (see `point_from_driver`).
    fn mul_affine(
        k: &FieldBytes<Self>,
        x: &FieldBytes<Self>,
        y: &FieldBytes<Self>,
    ) -> (FieldBytes<Self>, FieldBytes<Self>) {
        let result = point_from_bytes::<Self>(x, y) * scalar_from_bytes::<Self>(k);

        point_to_bytes_or_identity::<Self>(&result)
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

    /// Reduce a bigint into a nonzero scalar, per the `ReduceNonZero`
    /// contract.
    ///
    /// The default composes this from [`Reduce`], remapping the zero residue
    /// to one so the result is never zero. Curves whose scalar implements
    /// `ReduceNonZero` natively (e.g. P-256, where it is the
    /// `(w mod (n-1)) + 1` bijection onto the nonzero residues) override this
    /// to use their native implementation unchanged.
    fn reduce_nonzero(n: Self::Uint) -> Self::Scalar {
        let reduced = <Self::Scalar as Reduce<Self::Uint>>::reduce(n);
        Self::Scalar::conditional_select(&reduced, &<Self::Scalar as Field>::ONE, reduced.is_zero())
    }

    /// Byte-array form of [`reduce_nonzero`](Self::reduce_nonzero).
    fn reduce_nonzero_bytes(bytes: &FieldBytes<Self>) -> Self::Scalar {
        let reduced = <Self::Scalar as Reduce<Self::Uint>>::reduce_bytes(bytes);
        Self::Scalar::conditional_select(&reduced, &<Self::Scalar as Field>::ONE, reduced.is_zero())
    }

    /// `k1 * P1 + k2 * P2`, for scalars in `[0, n-1]` and valid affine
    /// points, or `None` if the sum is the identity. A zero scalar
    /// contributes the identity.
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

/// The accelerated curve *is* the wrapped curve, so it carries the same
/// object identifier. This makes PKCS#8/SPKI key encoding (which requires
/// [`AssociatedOid`]) work for the accelerated types unchanged.
#[cfg(feature = "pkcs8")]
impl<C: Backend + AssociatedOid> AssociatedOid for Accelerated<C> {
    const OID: ObjectIdentifier = C::OID;
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
// contracts accept a zero scalar (the term then contributes the identity) but
// not the identity point, which has no affine encoding; identity points are
// substituted away here, without branching on the operands.
// ===========================================================================

/// `k * G`.
fn mul_base<C: Backend>(k: &Scalar<C>) -> ProjectivePoint<C> {
    if !C::ACCELERATED_MUL {
        return ProjectivePoint::from_projective(C::ProjectivePoint::mul_by_generator(&k.0));
    }

    // Zero scalars are within the driver contract: the driver reports the
    // identity result with the all-zero sentinel coordinates.
    let (x, y) = C::mul_base(&k.to_repr());
    point_from_driver::<C>(x, y)
}

/// `k * P`.
fn mul<C: Backend>(k: &Scalar<C>, p: &AffinePoint<C>) -> ProjectivePoint<C> {
    if !C::ACCELERATED_MUL {
        return ProjectivePoint::from_projective(p.0 * k.0);
    }

    // A zero scalar, or the identity point, contributes the identity.
    // Zeroing the scalar is a constant-time select: it leaks nothing about a
    // secret scalar ("k was zero" is mathematically forced, not leaked) and
    // leaks only point-identity, which the contract already treats as
    // public. The identity point has no affine encoding, so the generator
    // stands in for it; the zeroed scalar makes the driver ignore the
    // substitution.
    let k = C::Scalar::conditional_select(&k.0, &C::Scalar::ZERO, k.0.is_zero() | p.0.is_identity());
    let p = C::AffinePoint::conditional_select(&p.0, &C::AFFINE_GENERATOR, p.0.is_identity());

    let (px, py) = point_to_bytes::<C>(&p);
    let (x, y) = C::mul_affine(&k.to_repr(), &px, &py);
    point_from_driver::<C>(x, y)
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

    // A zero scalar, or an identity point, contributes the identity (see
    // `mul` for the constant-time reasoning). Zero scalars are passed to the
    // driver as-is; identity points are replaced by the generator so that
    // every point has an affine encoding.
    let k1 = C::Scalar::conditional_select(&k1.0, &C::Scalar::ZERO, k1.0.is_zero() | p1.is_identity());
    let k2 = C::Scalar::conditional_select(&k2.0, &C::Scalar::ZERO, k2.0.is_zero() | p2.is_identity());
    let p1 = C::AffinePoint::conditional_select(&p1, &C::AFFINE_GENERATOR, p1.is_identity());
    let p2 = C::AffinePoint::conditional_select(&p2, &C::AFFINE_GENERATOR, p2.is_identity());

    let (x1, y1) = point_to_bytes::<C>(&p1);
    let (x2, y2) = point_to_bytes::<C>(&p2);

    // Whether the sum is the identity is not secret (see the driver contract).
    match C::lincomb(&k1.to_repr(), &x1, &y1, &k2.to_repr(), &x2, &y2) {
        Some((x, y)) => ProjectivePoint::from_affine(point_from_driver_bytes::<C>(&x, &y)),
        None => ProjectivePoint::IDENTITY,
    }
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

/// Decode a point returned by a driver, mapping the identity sentinel
/// (all-zero coordinates) back to the identity.
///
/// `mul_base`/`mul_affine` report the identity result of a zero scalar with
/// the all-zero sentinel coordinates, which are not on the curve, so the
/// decode is skipped for them.
fn point_from_driver<C: Backend>(x: FieldBytes<C>, y: FieldBytes<C>) -> ProjectivePoint<C> {
    let is_identity = x
        .iter()
        .chain(&y)
        .fold(Choice::from(1), |acc, byte| acc & byte.ct_eq(&0));

    // Whether the result is the identity is not secret (see the driver
    // contract), so this branch is fine.
    if bool::from(is_identity) {
        ProjectivePoint::IDENTITY
    } else {
        ProjectivePoint::from_affine(point_from_driver_bytes::<C>(&x, &y))
    }
}

/// Encode a software point result for the driver boundary: the identity
/// becomes the all-zero sentinel that `point_from_driver` maps back.
fn point_to_bytes_or_identity<C: Backend>(point: &C::ProjectivePoint) -> (FieldBytes<C>, FieldBytes<C>) {
    let affine = group::Curve::to_affine(point);

    if bool::from(PrimeCurveAffine::is_identity(&affine)) {
        (FieldBytes::<C>::default(), FieldBytes::<C>::default())
    } else {
        point_to_bytes::<C>(&affine)
    }
}

/// Decode a point returned by a driver.
///
/// The decode validates that the point is on the curve; the driver contract
/// guarantees it, so a failure here is a broken driver.
///
/// NOTE: the on-curve check cannot be dropped with p256 0.13 / primeorder
/// 0.13 as dependencies: `AffinePoint`'s coordinate fields are crate-private
/// and `GroupEncoding::from_bytes_unchecked` delegates to the validating
/// `from_bytes` ("no unchecked conversion possible for compressed points").
/// This function is kept separate from the software decode path so the check
/// can be relaxed the moment the wrapped curve offers an unchecked
/// constructor (or a per-curve `Backend` method is added).
fn point_from_driver_bytes<C: Backend>(x: &FieldBytes<C>, y: &FieldBytes<C>) -> C::AffinePoint {
    let encoded = EncodedPoint::<C>::from_affine_coordinates(x, y, false);

    C::AffinePoint::from_encoded_point(&encoded).expect("driver returned an invalid point")
}

/// Decode point coordinates for a software operation. Callers never pass the
/// identity.
fn point_from_bytes<C: Backend>(x: &FieldBytes<C>, y: &FieldBytes<C>) -> C::AffinePoint {
    let encoded = EncodedPoint::<C>::from_affine_coordinates(x, y, false);

    C::AffinePoint::from_encoded_point(&encoded).expect("invalid point")
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
