//! RustCrypto `elliptic-curve` / `group` trait implementations for P-256 over
//! the `embassy-crypto-driver` v2 `P256Ops` unitrait.
//!
//! This module is the in-tree merge of the `p256-ec` bridge crate (Stages 2-4):
//! scalar field ([`Scalar`]: `ff::Field`, `ff::PrimeField`, arithmetic, `Invert`,
//! `IsHigh`, `Reduce` family), point types ([`AffinePoint`], [`ProjectivePoint`]:
//! `group` 0.13 `Group`/`Curve`/`PrimeGroup`/`PrimeCurve`/`PrimeCurveAffine`/
//! `GroupEncoding`, `AffineCoordinates`, `MulByGenerator`, `LinearCombination`),
//! and [`elliptic_curve::CurveArithmetic`] + [`elliptic_curve::PrimeCurveArithmetic`]
//! wiring for [`NistP256`].
//!
//! Backend contract (driver v2): `scalar_inv`/`scalar_inv_vartime` return 0 for
//! `a == 0` (defined fallback; validity is `!scalar_is_zero(a)`); invalid
//! `point_from_canonical` input maps to the identity (validity is
//! `!projective_is_identity(decoded)`); `point_to_canonical(identity)` is `(0, 0)`.
//! `p256`'s formulas are complete, so no backend-side special cases are needed.

mod affine;
mod constants;
mod ct;
mod curve;
mod projective;
mod scalar;

pub use affine::{AffinePoint, EncodingRepr};
pub use curve::NistP256;
pub use projective::ProjectivePoint;
pub use scalar::Scalar;

/// Compile-time coherence checks for the merged module (same role as in the
/// original `p256-ec` crate: proves every `CurveArithmetic` bound holds).
#[allow(dead_code)]
fn assert_stage2_coherence() {
    use elliptic_curve::ops::Invert;
    use elliptic_curve::scalar::IsHigh;
    use elliptic_curve::{Curve, FieldBytes, PrimeCurve};
    use subtle::CtOption;
    use zeroize::DefaultIsZeroes;

    fn is_field<T: ff::Field>() {}
    fn is_prime_field<T: ff::PrimeField<Repr = FieldBytes<NistP256>>>() {}
    fn is_invert<T: Invert<Output = CtOption<T>>>() {}
    fn is_is_high<T: IsHigh>() {}
    fn is_default_is_zeroes<T: DefaultIsZeroes>() {}
    fn is_send_sync_static<T: Send + Sync + 'static>() {}
    fn is_as_ref_scalar<T: AsRef<T>>() {}
    fn is_into_field_bytes<T: Into<FieldBytes<NistP256>>>() {}
    fn is_into_uint<T: Into<elliptic_curve::bigint::U256>>() {}
    fn is_into_primitive<T: Into<elliptic_curve::ScalarPrimitive<NistP256>>>() {}
    fn is_from_primitive<T: From<elliptic_curve::ScalarPrimitive<NistP256>>>() {}
    fn is_from_u64<T: From<u64>>() {}
    fn is_partial_ord<T: PartialOrd>() {}
    fn is_shr_assign<T: core::ops::ShrAssign<usize>>() {}

    is_field::<Scalar>();
    is_prime_field::<Scalar>();
    is_invert::<Scalar>();
    is_is_high::<Scalar>();
    is_default_is_zeroes::<Scalar>();
    is_send_sync_static::<Scalar>();
    is_as_ref_scalar::<Scalar>();
    is_into_field_bytes::<Scalar>();
    is_into_uint::<Scalar>();
    is_into_primitive::<Scalar>();
    is_from_primitive::<Scalar>();
    is_from_u64::<Scalar>();
    is_partial_ord::<Scalar>();
    is_shr_assign::<Scalar>();

    use elliptic_curve::bigint::U256;
    use elliptic_curve::ops::{Reduce, ReduceNonZero};
    use elliptic_curve::scalar::FromUintUnchecked;
    fn is_reduce<T: Reduce<U256, Bytes = FieldBytes<NistP256>>>() {}
    fn is_reduce_nonzero<T: ReduceNonZero<U256>>() {}
    fn is_from_uint_unchecked<T: FromUintUnchecked<Uint = U256>>() {}
    is_reduce::<Scalar>();
    is_reduce_nonzero::<Scalar>();
    is_from_uint_unchecked::<Scalar>();

    use elliptic_curve::{CurveArithmetic, PrimeCurveArithmetic};
    fn is_curve_arithmetic<C: CurveArithmetic>() {}
    fn is_prime_curve_arithmetic<C: PrimeCurveArithmetic>() {}
    is_curve_arithmetic::<NistP256>();
    is_prime_curve_arithmetic::<NistP256>();

    fn is_curve<C: Curve>() {}
    fn is_prime_curve<C: PrimeCurve>() {}
    is_curve::<NistP256>();
    is_prime_curve::<NistP256>();

    // ---- Point-side bounds (CurveArithmetic::ProjectivePoint / ::AffinePoint) ----
    use elliptic_curve::ops::{LinearCombination, MulByGenerator};
    use elliptic_curve::point::AffineCoordinates;
    use group::prime::{PrimeCurve as GroupPrimeCurve, PrimeCurveAffine, PrimeGroup};
    use group::{Curve as GroupCurve, Group, GroupEncoding};

    fn is_group<T: Group>() {}
    fn is_group_curve<T: GroupCurve>() {}
    fn is_prime_group<T: PrimeGroup>() {}
    fn is_group_prime_curve<T: GroupPrimeCurve>() {}
    fn is_prime_curve_affine<T: PrimeCurveAffine>() {}
    fn is_group_encoding<T: GroupEncoding>() {}
    fn is_affine_coordinates<T: AffineCoordinates<FieldRepr = FieldBytes<NistP256>>>() {}
    fn is_lincomb<T: LinearCombination>() {}
    fn is_mul_by_gen<T: MulByGenerator>() {}
    fn is_from_affine<T: From<AffinePoint>>() {}
    fn is_into_affine<T: Into<AffinePoint>>() {}

    is_group::<ProjectivePoint>();
    is_group_curve::<ProjectivePoint>();
    is_prime_group::<ProjectivePoint>();
    is_group_prime_curve::<ProjectivePoint>();
    is_prime_curve_affine::<AffinePoint>();
    is_group_encoding::<ProjectivePoint>();
    is_group_encoding::<AffinePoint>();
    is_affine_coordinates::<AffinePoint>();
    is_lincomb::<ProjectivePoint>();
    is_mul_by_gen::<ProjectivePoint>();
    is_default_is_zeroes::<ProjectivePoint>();
    is_default_is_zeroes::<AffinePoint>();
    is_send_sync_static::<ProjectivePoint>();
    is_send_sync_static::<AffinePoint>();
    is_from_affine::<ProjectivePoint>();
    is_into_affine::<ProjectivePoint>();
}
