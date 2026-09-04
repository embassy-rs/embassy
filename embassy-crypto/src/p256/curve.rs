//! The curve type: [`NistP256`].

use crypto_bigint::U256;
use elliptic_curve::consts::U32;
use elliptic_curve::pkcs8::{AssociatedOid, ObjectIdentifier};
use elliptic_curve::{Curve, CurveArithmetic, FieldBytesEncoding, PrimeCurve, PrimeCurveArithmetic};

use crate::p256::{AffinePoint, ProjectivePoint, Scalar};

/// The NIST P-256 curve (secp256r1 / prime256v1).
///
/// Unlike a conventional `p256`-style crate there is **no backend type parameter**:
/// the `embassy-crypto-driver` unitrait implementation is selected at link time
/// (see the crate-level docs in `lib.rs`).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NistP256;

impl Curve for NistP256 {
    type FieldBytesSize = U32;
    type Uint = U256;

    const ORDER: U256 = U256::from_be_hex("FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551");
}

impl PrimeCurve for NistP256 {}

impl AssociatedOid for NistP256 {
    const OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.3.1.7");
}

/// Big-endian `U256` <-> 32-byte field element encoding.
///
/// `FieldBytesEncoding` has defaulted methods and no blanket impl in
/// `elliptic-curve 0.13` — each curve crate provides this empty impl (as the
/// `p256` crate does for its own `NistP256`).
impl FieldBytesEncoding<NistP256> for U256 {}

/// Stage 4 wiring: `elliptic_curve::CurveArithmetic` for P-256.
impl CurveArithmetic for NistP256 {
    type Scalar = Scalar;
    type AffinePoint = AffinePoint;
    type ProjectivePoint = ProjectivePoint;
}

impl PrimeCurveArithmetic for NistP256 {
    type CurveGroup = ProjectivePoint;
}

#[cfg(test)]
mod tests {
    use elliptic_curve::bigint::ArrayEncoding as _;

    use super::*;
    use crate::p256::constants::{MODULUS_DEC, N, decimal_to_be};

    #[test]
    fn order_matches_decimal_modulus() {
        assert_eq!(
            AsRef::<[u8]>::as_ref(&NistP256::ORDER.to_be_byte_array()),
            &decimal_to_be(MODULUS_DEC)[..]
        );
        assert_eq!(
            &N[..],
            AsRef::<[u8]>::as_ref(&<NistP256 as Curve>::ORDER.to_be_byte_array())
        );
    }
}
