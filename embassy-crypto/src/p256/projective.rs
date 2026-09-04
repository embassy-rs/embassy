//! The projective point type: [`ProjectivePoint`].

use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use elliptic_curve::ops::{LinearCombination, MulByGenerator};
use embassy_crypto_driver as drv;
use ff::Field as _;
use group::prime::{PrimeCurve, PrimeCurveAffine as _, PrimeGroup};
use group::{Curve, Group, GroupEncoding};
use rand_core::RngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};
use zeroize::DefaultIsZeroes;

use crate::p256::{AffinePoint, EncodingRepr, Scalar};

// ---------------------------------------------------------------------------
// Representation: the opaque backend handle, stored directly.
//
// The handle is `Copy` (declared `pub type ProjectivePoint: Copy` on the
// driver trait), so the wrapper derives `Copy`/`Clone` over it directly:
// zero per-copy backend calls, no `ManuallyDrop`, no byte-shadow unions.
// ---------------------------------------------------------------------------

/// Byte view of the opaque handle, used only by `ConditionallySelectable`.
/// The handle is `Copy` plain data (any bit pattern of the storage is a valid
/// value per the driver contract) with the exact `opaque(size = 128,
/// align = 16)` layout, so viewing a live handle as its bytes — and rebuilding
/// a handle from bytes obtained by selecting between two live handles — is
/// sound. No `ManuallyDrop` needed: union fields must be `Copy`, which the
/// handle now satisfies.
union NativeBytes {
    native: drv::P256OpsImplProjectivePoint,
    bytes: [u8; 128],
}

/// A P-256 point in the backend's projective representation.
///
/// **Backend contract:** the associated `ProjectivePoint` type is plain data
/// (any bit pattern of the storage is a valid value), and
/// `point_to_canonical(identity)` returns `(0, 0)`.
#[derive(Copy, Clone)]
pub struct ProjectivePoint {
    native: drv::P256OpsImplProjectivePoint,
}

impl ProjectivePoint {
    #[inline]
    pub(crate) fn from_native(p: drv::P256OpsImplProjectivePoint) -> Self {
        Self { native: p }
    }

    #[inline]
    pub(crate) fn to_native(&self) -> drv::P256OpsImplProjectivePoint {
        self.native
    }
}

fn binop(
    a: &ProjectivePoint,
    b: &ProjectivePoint,
    f: fn(&drv::P256OpsImplProjectivePoint, &drv::P256OpsImplProjectivePoint) -> drv::P256OpsImplProjectivePoint,
) -> ProjectivePoint {
    ProjectivePoint::from_native(f(&a.to_native(), &b.to_native()))
}

macro_rules! impl_arith {
    ($trait:ident, $method:ident, $op:expr) => {
        impl $trait<ProjectivePoint> for ProjectivePoint {
            type Output = ProjectivePoint;
            #[inline]
            fn $method(self, rhs: ProjectivePoint) -> ProjectivePoint {
                binop(&self, &rhs, $op)
            }
        }
        impl<'a> $trait<&'a ProjectivePoint> for ProjectivePoint {
            type Output = ProjectivePoint;
            #[inline]
            fn $method(self, rhs: &'a ProjectivePoint) -> ProjectivePoint {
                binop(&self, rhs, $op)
            }
        }
        impl $trait<ProjectivePoint> for &ProjectivePoint {
            type Output = ProjectivePoint;
            #[inline]
            fn $method(self, rhs: ProjectivePoint) -> ProjectivePoint {
                binop(self, &rhs, $op)
            }
        }
        impl<'a> $trait<&'a ProjectivePoint> for &ProjectivePoint {
            type Output = ProjectivePoint;
            #[inline]
            fn $method(self, rhs: &'a ProjectivePoint) -> ProjectivePoint {
                binop(self, rhs, $op)
            }
        }
    };
}

impl_arith!(Add, add, drv::P256OpsImpl::projective_add);
impl_arith!(Sub, sub, drv::P256OpsImpl::projective_sub);

macro_rules! impl_assign {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait<ProjectivePoint> for ProjectivePoint {
            #[inline]
            fn $method(&mut self, rhs: ProjectivePoint) {
                *self = *self $op rhs;
            }
        }
        impl<'a> $trait<&'a ProjectivePoint> for ProjectivePoint {
            #[inline]
            fn $method(&mut self, rhs: &'a ProjectivePoint) {
                *self = *self $op rhs;
            }
        }
    };
}

impl_assign!(AddAssign, add_assign, +);
impl_assign!(SubAssign, sub_assign, -);

// ---------------------------------------------------------------------------
// Mixed projective +/- affine arithmetic.
//
// `group::Curve` requires `GroupOps<AffineRepr> + GroupOpsOwned<AffineRepr>`,
// i.e. these 12 impls. They convert the affine operand to projective and
// delegate (one extra cheap conversion; the driver has no mixed-native ops).
// ---------------------------------------------------------------------------

macro_rules! impl_affine_arith {
    ($trait:ident, $method:ident, $op:expr) => {
        impl $trait<AffinePoint> for ProjectivePoint {
            type Output = ProjectivePoint;
            #[inline]
            fn $method(self, rhs: AffinePoint) -> ProjectivePoint {
                binop(&self, &ProjectivePoint::from_native(rhs.to_native()), $op)
            }
        }
        impl<'a> $trait<&'a AffinePoint> for ProjectivePoint {
            type Output = ProjectivePoint;
            #[inline]
            fn $method(self, rhs: &'a AffinePoint) -> ProjectivePoint {
                binop(&self, &ProjectivePoint::from_native(rhs.to_native()), $op)
            }
        }
    };
}
impl_affine_arith!(Add, add, drv::P256OpsImpl::projective_add);
impl_affine_arith!(Sub, sub, drv::P256OpsImpl::projective_sub);

macro_rules! impl_affine_assign {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait<AffinePoint> for ProjectivePoint {
            #[inline]
            fn $method(&mut self, rhs: AffinePoint) {
                *self = *self $op rhs;
            }
        }
        impl<'a> $trait<&'a AffinePoint> for ProjectivePoint {
            #[inline]
            fn $method(&mut self, rhs: &'a AffinePoint) {
                *self = *self $op rhs;
            }
        }
    };
}
impl_affine_assign!(AddAssign, add_assign, +);
impl_affine_assign!(SubAssign, sub_assign, -);

macro_rules! impl_mul {
    ($lhs:ty, $rhs:ty) => {
        impl Mul<$rhs> for $lhs {
            type Output = ProjectivePoint;
            #[inline]
            fn mul(self, rhs: $rhs) -> ProjectivePoint {
                ProjectivePoint::from_native(drv::P256OpsImpl::scalar_mul_projective(
                    &rhs.to_opaque(),
                    &self.to_native(),
                ))
            }
        }
    };
}
impl_mul!(ProjectivePoint, Scalar);
impl_mul!(ProjectivePoint, &Scalar);
impl_mul!(&ProjectivePoint, Scalar);
impl_mul!(&ProjectivePoint, &Scalar);

impl MulAssign<Scalar> for ProjectivePoint {
    #[inline]
    fn mul_assign(&mut self, rhs: Scalar) {
        *self = *self * rhs;
    }
}
impl<'a> MulAssign<&'a Scalar> for ProjectivePoint {
    #[inline]
    fn mul_assign(&mut self, rhs: &'a Scalar) {
        *self = *self * rhs;
    }
}

/// -p = identity - p (the driver has no `projective_neg`).
impl Neg for ProjectivePoint {
    type Output = ProjectivePoint;
    #[inline]
    fn neg(self) -> ProjectivePoint {
        ProjectivePoint::from_native(drv::P256OpsImpl::projective_sub(
            &drv::P256OpsImpl::projective_identity(),
            &self.to_native(),
        ))
    }
}

impl Sum for ProjectivePoint {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(ProjectivePoint::identity(), Add::add)
    }
}
impl<'a> Sum<&'a ProjectivePoint> for ProjectivePoint {
    fn sum<I: Iterator<Item = &'a ProjectivePoint>>(iter: I) -> Self {
        iter.fold(ProjectivePoint::identity(), Add::add)
    }
}

impl Default for ProjectivePoint {
    fn default() -> Self {
        Self::identity()
    }
}
impl DefaultIsZeroes for ProjectivePoint {}

impl fmt::Debug for ProjectivePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectivePoint(0x")?;
        for b in <Self as GroupEncoding>::to_bytes(self).as_ref() {
            write!(f, "{b:02x}")?;
        }
        f.write_str(")")
    }
}

impl PartialEq for ProjectivePoint {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}
impl Eq for ProjectivePoint {}

impl ConstantTimeEq for ProjectivePoint {
    fn ct_eq(&self, other: &Self) -> Choice {
        // Projective representations differ for equal points, so compare in the
        // canonical affine domain.
        self.to_affine().ct_eq(&other.to_affine())
    }
}

impl ConditionallySelectable for ProjectivePoint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        // Byte-selecting between two valid projective representations is sound:
        // the storage is plain data (any bit pattern is valid), so the result
        // is a representation of the chosen point (identical inputs may have
        // distinct representations; either is a correct answer). This must
        // stay in the native domain: the driver's `point_from_canonical` is
        // explicitly NOT constant-time, so selecting via affine round-trips
        // would leak the secret choice bit through a variable-time function.
        let ab = unsafe { NativeBytes { native: a.native }.bytes };
        let bb = unsafe { NativeBytes { native: b.native }.bytes };
        let mut out = [0u8; 128];
        let mask = 0u8.wrapping_sub(choice.unwrap_u8());
        for i in 0..128 {
            // subtle contract: choice == 1 -> b (was inverted, same bug as
            // ct_select_bytes).
            out[i] = (bb[i] & mask) | (ab[i] & !mask);
        }
        Self {
            native: unsafe { NativeBytes { bytes: out }.native },
        }
    }
}

impl GroupEncoding for ProjectivePoint {
    type Repr = EncodingRepr;

    fn to_bytes(&self) -> Self::Repr {
        self.to_affine().to_bytes()
    }

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        AffinePoint::from_bytes(bytes).map(|a| a.to_curve())
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        AffinePoint::from_bytes_unchecked(bytes).map(|a| a.to_curve())
    }
}

impl Group for ProjectivePoint {
    type Scalar = Scalar;

    fn random(mut rng: impl RngCore) -> Self {
        Self::from_native(drv::P256OpsImpl::scalar_mul_base(&Scalar::random(&mut rng).to_opaque()))
    }

    fn identity() -> Self {
        Self::from_native(drv::P256OpsImpl::projective_identity())
    }

    fn generator() -> Self {
        Self::from_native(drv::P256OpsImpl::projective_generator())
    }

    fn is_identity(&self) -> Choice {
        // v2 predicate: the canonical identity test (Jacobian Z == 0). No
        // canonical round-trip needed.
        Choice::from(drv::P256OpsImpl::projective_is_identity(&self.to_native()) as u8)
    }

    fn double(&self) -> Self {
        Self::from_native(drv::P256OpsImpl::projective_double(&self.to_native()))
    }
}

impl Curve for ProjectivePoint {
    type AffineRepr = AffinePoint;

    fn to_affine(&self) -> AffinePoint {
        // v2 contract: `point_to_canonical(identity)` is `(0, 0)`; distinguish
        // with the `projective_is_identity` predicate — never by sniffing coords.
        let p = self.to_native();
        let c = drv::P256OpsImpl::point_to_canonical(&p);
        let infinity = Choice::from(drv::P256OpsImpl::projective_is_identity(&p) as u8);
        AffinePoint::from_parts(c.x, c.y, infinity)
    }
}

impl PrimeGroup for ProjectivePoint {}

impl PrimeCurve for ProjectivePoint {
    type Affine = AffinePoint;
}

impl From<AffinePoint> for ProjectivePoint {
    #[inline]
    fn from(a: AffinePoint) -> Self {
        Self::from_native(a.to_native())
    }
}

impl From<ProjectivePoint> for AffinePoint {
    #[inline]
    fn from(p: ProjectivePoint) -> Self {
        p.to_affine()
    }
}

impl MulByGenerator for ProjectivePoint {
    #[inline]
    fn mul_by_generator(scalar: &Scalar) -> Self {
        Self::from_native(drv::P256OpsImpl::scalar_mul_base(&scalar.to_opaque()))
    }
}

impl LinearCombination for ProjectivePoint {
    #[inline]
    fn lincomb(x: &Self, k: &Self::Scalar, y: &Self, l: &Self::Scalar) -> Self {
        Self::from_native(drv::P256OpsImpl::projective_lincomb(
            &k.to_opaque(),
            &x.to_native(),
            &l.to_opaque(),
            &y.to_native(),
        ))
    }
}
