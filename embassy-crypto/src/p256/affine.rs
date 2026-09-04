//! The affine point type: [`AffinePoint`].

use core::fmt;
use core::ops::{Mul, Neg};

use elliptic_curve::FieldBytes;
use elliptic_curve::point::AffineCoordinates;
use elliptic_curve::sec1::{Coordinates, EncodedPoint as Sec1EncodedPoint, FromEncodedPoint, ToEncodedPoint};
use embassy_crypto_driver as drv;
use group::prime::PrimeCurveAffine;
use group::{Curve as _, GroupEncoding};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};
use zeroize::DefaultIsZeroes;

use crate::p256::constants::{GENERATOR_X, GENERATOR_Y, N_MINUS_ONE};
use crate::p256::ct::{ct_eq_bytes, ct_select_bytes};
use crate::p256::{NistP256, ProjectivePoint, Scalar};

/// SEC1 uncompressed point encoding: `0x04 || x || y` (65 bytes), as a newtype.
///
/// Neither `GenericArray` (deprecated by generic-array 0.14) nor a bare array
/// (`Default` is only implemented for arrays up to 32 elements, and
/// `GroupEncoding::Repr` requires it) works, so this is a manual newtype.
#[derive(Copy, Clone)]
pub struct EncodedPoint(pub(crate) [u8; 65]);

impl Default for EncodedPoint {
    fn default() -> Self {
        Self([0u8; 65])
    }
}

impl AsRef<[u8]> for EncodedPoint {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for EncodedPoint {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// The [`GroupEncoding`] representation of both point types (re-exported).
pub type EncodingRepr = EncodedPoint;

/// A P-256 point in affine coordinates, plus an explicit infinity flag.
///
/// Affine points are *canonical*: `(x, y)` big-endian plus `infinity`. The
/// infinity flag exists because the driver's canonical [`drv::P256AffinePoint`]
/// cannot encode the identity; the adapter tracks it, with the backend contract
/// that `point_to_canonical(identity)` returns `(0, 0)`.
#[derive(Copy, Clone)]
pub struct AffinePoint {
    x: [u8; 32],
    y: [u8; 32],
    infinity: Choice,
}

impl AffinePoint {
    /// Construct from parts (infinity yields canonical `(0, 0)` coordinates).
    pub(crate) fn from_parts(x: [u8; 32], y: [u8; 32], infinity: Choice) -> Self {
        Self { x, y, infinity }
    }

    /// The affine coordinates as canonical bytes (infinity yields `(0, 0)`).
    pub(crate) fn coords(&self) -> ([u8; 32], [u8; 32]) {
        let zero = [0u8; 32];
        (
            if bool::from(self.infinity) { zero } else { self.x },
            if bool::from(self.infinity) { zero } else { self.y },
        )
    }

    /// Obtain the opaque projective handle for this point.
    pub(crate) fn to_native(&self) -> drv::P256OpsImplProjectivePoint {
        if bool::from(self.infinity) {
            drv::P256OpsImpl::projective_identity()
        } else {
            drv::P256OpsImpl::point_from_canonical(&drv::P256AffinePoint { x: self.x, y: self.y })
        }
    }

    fn from_canonical_coords(x: [u8; 32], y: [u8; 32]) -> CtOption<Self> {
        // v2 contract: invalid input maps to the IDENTITY (defined fallback),
        // and a valid affine input never decodes to the identity — so
        // `projective_is_identity` is the complete validity check.
        let p = drv::P256OpsImpl::point_from_canonical(&drv::P256AffinePoint { x, y });
        let ok = Choice::from((!drv::P256OpsImpl::projective_is_identity(&p)) as u8);
        CtOption::new(Self::from_parts(x, y, Choice::from(0)), ok)
    }
}

impl Default for AffinePoint {
    fn default() -> Self {
        Self::identity()
    }
}
impl DefaultIsZeroes for AffinePoint {}

impl fmt::Debug for AffinePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if bool::from(self.infinity) {
            return f.write_str("AffinePoint(identity)");
        }
        f.write_str("AffinePoint(0x")?;
        for b in self.x.iter().chain(self.y.iter()) {
            write!(f, "{b:02x}")?;
        }
        f.write_str(")")
    }
}

impl PartialEq for AffinePoint {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}
impl Eq for AffinePoint {}

impl ConstantTimeEq for AffinePoint {
    fn ct_eq(&self, other: &Self) -> Choice {
        ct_eq_bytes(&self.coords().0, &other.coords().0)
            & ct_eq_bytes(&self.coords().1, &other.coords().1)
            & self.infinity.ct_eq(&other.infinity)
    }
}

impl ConditionallySelectable for AffinePoint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self {
            x: ct_select_bytes(&a.x, &b.x, choice),
            y: ct_select_bytes(&a.y, &b.y, choice),
            infinity: Choice::conditional_select(&a.infinity, &b.infinity, choice),
        }
    }
}

impl AffineCoordinates for AffinePoint {
    type FieldRepr = FieldBytes<NistP256>;

    fn x(&self) -> Self::FieldRepr {
        let mut out = FieldBytes::<NistP256>::default();
        out.copy_from_slice(&self.coords().0);
        out
    }

    fn y_is_odd(&self) -> Choice {
        Choice::from(self.coords().1[31] & 1)
    }
}

impl FromEncodedPoint<NistP256> for AffinePoint {
    fn from_encoded_point(encoded_point: &Sec1EncodedPoint<NistP256>) -> CtOption<Self> {
        match encoded_point.coordinates() {
            Coordinates::Identity => CtOption::new(Self::identity(), Choice::from(1)),
            Coordinates::Uncompressed { x, y } => {
                let mut x_bytes = [0u8; 32];
                x_bytes.copy_from_slice(x.as_ref());
                let mut y_bytes = [0u8; 32];
                y_bytes.copy_from_slice(y.as_ref());
                Self::from_canonical_coords(x_bytes, y_bytes)
            }
            // The P256Ops driver exposes affine decode/encode but no base-field
            // square-root operation, so x-only SEC1 forms cannot be recovered
            // without adding a software field backend. Reject them explicitly.
            Coordinates::Compact { .. } | Coordinates::Compressed { .. } => {
                CtOption::new(Self::identity(), Choice::from(0))
            }
        }
    }
}

impl ToEncodedPoint<NistP256> for AffinePoint {
    fn to_encoded_point(&self, compress: bool) -> Sec1EncodedPoint<NistP256> {
        if bool::from(self.infinity) {
            return Sec1EncodedPoint::<NistP256>::identity();
        }

        let mut x = FieldBytes::<NistP256>::default();
        x.copy_from_slice(&self.x);
        let mut y = FieldBytes::<NistP256>::default();
        y.copy_from_slice(&self.y);
        Sec1EncodedPoint::<NistP256>::from_affine_coordinates(&x, &y, compress)
    }
}

impl GroupEncoding for AffinePoint {
    type Repr = EncodingRepr;

    /// `0x04 || x || y`; the identity encodes as all zeroes.
    fn to_bytes(&self) -> Self::Repr {
        let mut out = EncodingRepr::default();
        if bool::from(self.infinity) {
            return out; // all zeroes
        }
        out.0[0] = 0x04;
        out.0[1..33].copy_from_slice(&self.x);
        out.0[33..65].copy_from_slice(&self.y);
        out
    }

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        let b: &[u8] = bytes.as_ref();
        let all_zero = b
            .iter()
            .fold(Choice::from(1), |acc, &x| acc & Choice::from((x == 0) as u8));
        if bool::from(all_zero) {
            return CtOption::new(Self::identity(), Choice::from(1));
        }
        if b[0] != 0x04 {
            return CtOption::new(Self::identity(), Choice::from(0));
        }
        let x: [u8; 32] = b[1..33].try_into().expect("length checked");
        let y: [u8; 32] = b[33..65].try_into().expect("length checked");

        Self::from_canonical_coords(x, y)
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        let b: &[u8] = bytes.as_ref();
        let all_zero = b
            .iter()
            .fold(Choice::from(1), |acc, &x| acc & Choice::from((x == 0) as u8));
        if bool::from(all_zero) {
            return CtOption::new(Self::identity(), Choice::from(1));
        }
        if b[0] != 0x04 {
            return CtOption::new(Self::identity(), Choice::from(0));
        }
        let x: [u8; 32] = b[1..33].try_into().expect("length checked");
        let y: [u8; 32] = b[33..65].try_into().expect("length checked");
        CtOption::new(
            Self {
                x,
                y,
                infinity: Choice::from(0),
            },
            Choice::from(1),
        )
    }
}

impl PrimeCurveAffine for AffinePoint {
    type Scalar = Scalar;
    type Curve = ProjectivePoint;

    fn identity() -> Self {
        Self {
            x: [0u8; 32],
            y: [0u8; 32],
            infinity: Choice::from(1),
        }
    }

    fn generator() -> Self {
        Self {
            x: GENERATOR_X,
            y: GENERATOR_Y,
            infinity: Choice::from(0),
        }
    }

    fn is_identity(&self) -> Choice {
        self.infinity
    }

    fn to_curve(&self) -> Self::Curve {
        ProjectivePoint::from_native(self.to_native())
    }
}

/// -(x, y) = (x, p - y), computed via the backend (scalar -1) since the driver
/// exposes no base-field arithmetic.
impl Neg for AffinePoint {
    type Output = Self;
    fn neg(self) -> Self {
        let minus_one = Scalar::from_canon(N_MINUS_ONE);
        let p = drv::P256OpsImpl::scalar_mul_projective(&minus_one.to_opaque(), &self.to_native());
        ProjectivePoint::from_native(p).to_affine()
    }
}

macro_rules! impl_affine_mul {
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
impl_affine_mul!(AffinePoint, Scalar);
impl_affine_mul!(AffinePoint, &Scalar);
impl_affine_mul!(&AffinePoint, Scalar);
impl_affine_mul!(&AffinePoint, &Scalar);
