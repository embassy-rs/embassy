#![cfg(feature = "driver-p256")]
//! Backend-linked known-answer tests — the first tests that actually execute
//! driver arithmetic. Gated on `driver-p256`: this crate's own rustcrypto
//! backend (`P256OpsDriver`, driver v2) provides the unitrait symbols, so
//! `cargo test --features driver-p256 --test kat` runs them end to end.

use crypto_bigint::U256;
use elliptic_curve::ops::{Invert, MulByGenerator, Reduce, ReduceNonZero};
use elliptic_curve::scalar::FromUintUnchecked;
use elliptic_curve::{CurveArithmetic, PublicKey};
use embassy_crypto::p256::{AffinePoint, NistP256, ProjectivePoint, Scalar};
use ff::{Field as _, PrimeField};
use group::prime::PrimeCurveAffine as _;
use group::{Group, GroupEncoding};
use subtle::ConstantTimeEq as _;

// Vectors computed independently (affine EC over GF(p), Python).
const A_HEX: &str = "C9AFA9D845BA75166B5C215767B1D6934E50C3DB36E89B127B8A622B120F6721";
const B_HEX: &str = "0F1E2D3C4B5A69788796A5B4C3D2E1F00112233445566778899AABBCCDDEEFF00";
const A_PLUS_B_HEX: &str = "C9AFA9D845BA75166B5C215767B1D6934E50C3DB36E89B127B8A622B120F6724";
const A_TIMES_B_HEX: &str = "5D0EFD8AD12F5F4142146406371583BA71245636568A942D8B2B90FB3D67EAC1";
const NEG_A_HEX: &str = "36505626BA458AEA94A3DEA8984E296C6E9636D2702F0372782F6897EA53BE30";
const INV_A_HEX: &str = "FF24A4EEB2B46CEE2BF82C197F40FC7F2B34205E3CBF997C4A9C6F32EC7E38D4";
const REDUCE_FF_HEX: &str = "00000000FFFFFFFF00000000000000004319055258E8617B0C46353D039CDAAE";
const N_HEX: &str = "FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551";
const THREE_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000003";
const AG_X: &str = "60FED4BA255A9D31C961EB74C6356D68C049B8923B61FA6CE669622E60F29FB6";
const AG_Y: &str = "7903FE1008B8BC99A41AE9E95628BC64F2F1B20C2D7E9F5177A3C294D4462299";
const TWO_G_X: &str = "7CF27B188D034F7E8A52380304B51AC3C08969E277F21B35A60B48FC47669978";
const TWO_G_Y: &str = "07775510DB8ED040293D9AC69F7430DBBA7DADE63CE982299E04B79D227873D1";
const NEG_TWO_G_Y: &str = "F888AAEE24712FC0D6C26539608BCF244582521AC3167DD661FB4862DD878C2E";
const G_SEC1: &str = "046B17D1F2E12C4247F8BCE6E563A440F277037D812DEB33A0F4A13945D898C296\
4FE342E2FE1A7F9B8EE7EB4A7C0F9E162BCE33576B315ECECBB6406837BF51F5";

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
        .collect()
}
fn unhex32(s: &str) -> [u8; 32] {
    unhex(s).try_into().unwrap()
}

fn scalar(hex: &str) -> Scalar {
    let arr = unhex32(hex);
    let mut fb = <Scalar as PrimeField>::Repr::default();
    fb.copy_from_slice(&arr);
    Option::<Scalar>::from(<Scalar as PrimeField>::from_repr(fb)).expect("scalar in range")
}

fn affine_from_xy(x_hex: &str, y_hex: &str) -> AffinePoint {
    let mut bytes = [0u8; 65];
    bytes[0] = 0x04;
    bytes[1..33].copy_from_slice(&unhex32(x_hex));
    bytes[33..65].copy_from_slice(&unhex32(y_hex));
    let mut repr = <AffinePoint as GroupEncoding>::Repr::default();
    repr.as_mut().copy_from_slice(&bytes);
    Option::<AffinePoint>::from(<AffinePoint as GroupEncoding>::from_bytes(&repr)).expect("on curve")
}

#[test]
fn scalar_arithmetic_kats() {
    let (a, b) = (scalar(A_HEX), scalar(THREE_HEX));
    assert_eq!(a + b, scalar(A_PLUS_B_HEX));
    assert_eq!(a * b, scalar(A_TIMES_B_HEX));
    assert_eq!(-a, scalar(NEG_A_HEX));
    assert_eq!(a - b, a + (-b));
}

#[test]
fn scalar_invert_kats() {
    let a = scalar(A_HEX);
    let inv = Option::<Scalar>::from(Invert::invert(&a)).expect("invertible");
    assert_eq!(inv, scalar(INV_A_HEX));
    assert_eq!(a * inv, Scalar::ONE);
    let inv_vt = Option::<Scalar>::from(Invert::invert_vartime(&a)).expect("invertible (vartime)");
    assert_eq!(inv_vt, inv);
    assert!(bool::from(Invert::invert(&Scalar::ZERO).is_none()));
}

#[test]
fn reduce_kats() {
    let ff = U256::from_be_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    assert_eq!(<Scalar as Reduce<U256>>::reduce(ff), scalar(REDUCE_FF_HEX));
    assert_eq!(Scalar::from_uint_unchecked(ff), scalar(REDUCE_FF_HEX));
    // n + 1 reduces to 1
    let n_plus_1 = U256::from_be_hex(N_HEX).wrapping_add(&U256::ONE);
    assert_eq!(<Scalar as Reduce<U256>>::reduce(n_plus_1), Scalar::ONE);
    // a non-zero multiple of n maps to 1 under ReduceNonZero
    let rn = <Scalar as ReduceNonZero<U256>>::reduce_nonzero(U256::from_be_hex(N_HEX));
    assert_eq!(rn, Scalar::ONE);
}

#[test]
fn generator_and_doubling_kats() {
    let two_g = affine_from_xy(TWO_G_X, TWO_G_Y);
    let g = ProjectivePoint::generator();
    assert!(bool::from((g + g).ct_eq(&two_g.to_curve())));
    assert!(bool::from(g.double().ct_eq(&two_g.to_curve())));
    // distinct projective reps of the same point compare equal (affine-domain ct_eq)
    assert!(bool::from((g + g).ct_eq(&g.double())));
}

#[test]
fn scalar_mul_and_ecdh_kats() {
    let (a, b) = (scalar(A_HEX), scalar(B_HEX));
    let ag = affine_from_xy(AG_X, AG_Y).to_curve();
    assert!(bool::from(ProjectivePoint::mul_by_generator(&a).ct_eq(&ag)));
    assert!(bool::from((ProjectivePoint::generator() * a).ct_eq(&ag)));
    assert!(bool::from((AffinePoint::generator() * a).ct_eq(&ag)));
    // ECDH commutativity: a * (b * G) == b * (a * G)
    let ab = ProjectivePoint::mul_by_generator(&b) * a;
    let ba = ProjectivePoint::mul_by_generator(&a) * b;
    assert!(bool::from(ab.ct_eq(&ba)));
}

#[test]
fn negation_and_identity_kats() {
    let p = affine_from_xy(TWO_G_X, TWO_G_Y).to_curve();
    let neg = affine_from_xy(TWO_G_X, NEG_TWO_G_Y).to_curve();
    assert!(bool::from((-p).ct_eq(&neg)));
    assert!(bool::from(ProjectivePoint::identity().is_identity()));
    assert!(!bool::from(ProjectivePoint::generator().is_identity()));
    assert!(bool::from(AffinePoint::identity().is_identity()));
    let g = ProjectivePoint::generator();
    assert!(bool::from((g + ProjectivePoint::identity()).ct_eq(&g)));
    assert!(bool::from(ProjectivePoint::identity().double().is_identity()));
    // v2 contract: k == 0 yields the identity
    assert!(bool::from(
        ProjectivePoint::mul_by_generator(&Scalar::ZERO).is_identity()
    ));
}

#[test]
fn group_encoding_kats() {
    let g = AffinePoint::generator();
    assert_eq!(g.to_bytes().as_ref(), &unhex(G_SEC1)[..]);
    let back = Option::<AffinePoint>::from(<AffinePoint as GroupEncoding>::from_bytes(&g.to_bytes())).unwrap();
    assert!(bool::from(back.ct_eq(&g)));
    // identity <-> all-zero encoding
    let z = AffinePoint::identity().to_bytes();
    assert!(z.as_ref().iter().all(|&x| x == 0));
    let back = Option::<AffinePoint>::from(<AffinePoint as GroupEncoding>::from_bytes(&z)).unwrap();
    assert!(bool::from(back.is_identity()));
    // (0, 0) is off-curve and must be rejected (v2 identity fallback + predicate)
    let mut bad = <AffinePoint as GroupEncoding>::Repr::default();
    bad.as_mut()[0] = 0x04;
    assert!(bool::from(<AffinePoint as GroupEncoding>::from_bytes(&bad).is_none()));
    // projective round-trip
    let p = ProjectivePoint::generator();
    let back = Option::<ProjectivePoint>::from(<ProjectivePoint as GroupEncoding>::from_bytes(&p.to_bytes())).unwrap();
    assert!(bool::from(back.ct_eq(&p)));
}

#[test]
fn curve_arithmetic_wiring_and_public_key_smoke() {
    let p: <NistP256 as CurveArithmetic>::ProjectivePoint = ProjectivePoint::mul_by_generator(&scalar(A_HEX));
    assert!(bool::from(p.ct_eq(&affine_from_xy(AG_X, AG_Y).to_curve())));
    let a: <NistP256 as CurveArithmetic>::AffinePoint = AffinePoint::generator();
    let pk = PublicKey::<NistP256>::from_affine(a).expect("generator is not identity");
    assert!(bool::from(pk.as_affine().ct_eq(&a)));
    assert!(PublicKey::<NistP256>::from_affine(AffinePoint::identity()).is_err());
}
