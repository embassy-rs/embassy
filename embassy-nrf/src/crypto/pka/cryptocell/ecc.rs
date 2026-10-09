//! Elliptic curve arithmetic on the CryptoCell PKA.
//!
//! The PKA has no curve operations of its own. Point arithmetic is a sequence of modular
//! operations issued by software.
//!
//! The formulas work in modified Jacobian coordinates `(X, Y, Z, T)`. They stand for the
//! affine point `(X/Z², Y/Z³)`, with `T = a·Z⁴` so a doubling does not recompute it.
//! Additions take the second point in affine or Jacobian form and produce a modified point.
//!
//! Intermediate results stay unreduced. The `_nfr` operations leave up to eight extra bits.
//! Subtractions add a multiple of the modulus first, so nothing goes negative. `N4`, `N8` and
//! `N12` hold 4, 8 and 12 times the modulus for that.
//!
//! Scalar multiplication follows Arm's CryptoCell runtime library
//! (`pka_ec_wrst_smul_scap.c`). It scans the scalar two bits at a time and always adds one of
//! `±P`, `±2P`, `±4P`, so the operation sequence does not depend on the scalar. When `-k` is
//! longer than `k`, it multiplies by `-k` instead, to hide the scalar length.
//!
//! Verification only handles public values, so it uses the faster Strauss algorithm
//! (`pka_ec_wrst.c`) to compute the sum of two scalar multiplications at once.

use super::engine::{self as pka, LEN_FULL, LEN_MAX, LEN_N, REG_COUNT, REG_N, REG_NP, REG_T0, REG_T1};
use crate::crypto::pka::{Curve, Error, MAX_CURVE_LEN};

// Registers shared by every curve operation.
const R_T: u8 = 2;
const R_T1: u8 = 3;
const R_T2: u8 = 4;
const R_T3: u8 = 5;
/// Holds `1/Z` while converting to affine coordinates.
const R_AQ: u8 = 6;
/// Holds `n - 2` while inverting by exponentiation.
const R_NM2: u8 = 7;
/// Multiples of the modulus, so that subtractions never go negative.
const R_N4: u8 = 8;
const R_N8: u8 = 9;
const R_N12: u8 = 10;
/// Curve coefficient `a`.
const R_EC_A: u8 = 11;
const R_T4: u8 = 12;
/// `Z` of the affine addition.
const R_AAA_Z: u8 = 13;

// Registers of the scalar multiplication.
const S_X2: u8 = 12;
const S_Y2: u8 = 13;
const S_Z2: u8 = 14;
const S_T2: u8 = 15;
const S_X4: u8 = 16;
const S_Y4: u8 = 17;
const S_Z4: u8 = 18;
const S_T4: u8 = 19;
const S_XS: u8 = 20;
const S_YS: u8 = 21;
const S_ZS: u8 = 22;
const S_TS: u8 = 23;
const S_ZP: u8 = 24;
const S_TP: u8 = 25;
/// Scratch of the doubling, and the curve order before the multiplication starts.
const S_ZR: u8 = 26;
const S_ORD: u8 = 26;
const S_RK: u8 = 27;
const S_XP: u8 = 28;
const S_YP: u8 = 29;

// Registers of the signature verification.
const V_F: u8 = 2;
const V_D: u8 = 3;
const V_H: u8 = 4;
const V_TMP: u8 = 5;
const V_XPQ: u8 = 14;
const V_YPQ: u8 = 15;
const V_ZR: u8 = 16;
const V_TR: u8 = 17;
const V_H1: u8 = 18;
const V_H2: u8 = 19;
const V_GX: u8 = 20;
const V_GY: u8 = 21;
const V_WX: u8 = 22;
const V_WY: u8 = 23;
const V_RX: u8 = 24;
const V_RY: u8 = 25;
const V_TMP_N: u8 = 26;
const V_TMP_NP: u8 = 27;
const V_C: u8 = 28;

/// Doubles a point, keeping it in modified Jacobian coordinates.
///
/// `t` is used as scratch, so it must not be the same register as `t1`.
#[allow(clippy::too_many_arguments)]
fn double_mdf2mdf(x: u8, y: u8, z: u8, t: u8, x1: u8, y1: u8, z1: u8, t1: u8) {
    debug_assert!(t != t1);
    pka::add(LEN_FULL, t, y1, y1);
    pka::mod_mul_nfr(LEN_N, z, t, z1);
    pka::mod_mul_nfr(LEN_N, y, y1, y1);
    pka::add(LEN_FULL, t, x1, x1);
    pka::add(LEN_FULL, t, t, t);
    pka::mod_mul_nfr(LEN_N, t, y, t);
    pka::mod_mul_nfr(LEN_N, R_T2, x1, x1);
    pka::add(LEN_FULL, x, R_T2, R_T2);
    pka::add(LEN_FULL, R_T2, R_T2, x);
    pka::add(LEN_FULL, R_T2, t1, R_T2);
    pka::sub(LEN_FULL, t, R_N4, t);
    pka::mod_mul_acc_nfr(LEN_N, x, R_T2, R_T2, t);
    pka::add(LEN_FULL, x, t, x);
    pka::add(LEN_FULL, t, x, t);
    pka::sub(LEN_FULL, R_T3, R_N12, t);
    pka::add(LEN_FULL, y, y, y);
    pka::mod_mul_nfr(LEN_N, y, y, y);
    pka::add(LEN_FULL, y, y, y);
    pka::add(LEN_FULL, t, y, y);
    pka::mod_mul_nfr(LEN_N, t, t, t1);
    pka::sub(LEN_FULL, y, R_N8, y);
    pka::mod_mul_acc_nfr(LEN_N, y, R_T3, R_T2, y);
}

/// Doubles a point, keeping it in modified Jacobian coordinates.
///
/// Unlike [`double_mdf2mdf`], `t` and `t1` may be the same register. Uses `R_T4` as scratch.
#[allow(clippy::too_many_arguments)]
fn double_mdf2mdf_t4(x: u8, y: u8, z: u8, t: u8, x1: u8, y1: u8, z1: u8, t1: u8) {
    pka::add(LEN_FULL, R_T4, y1, y1);
    pka::mod_mul_nfr(LEN_N, z, R_T4, z1);
    pka::mod_mul_nfr(LEN_N, y, y1, y1);
    pka::add(LEN_FULL, R_T4, x1, x1);
    pka::add(LEN_FULL, R_T4, R_T4, R_T4);
    pka::mod_mul_nfr(LEN_N, R_T4, y, R_T4);
    pka::mod_mul_nfr(LEN_N, R_T2, x1, x1);
    pka::add(LEN_FULL, x, R_T2, R_T2);
    pka::add(LEN_FULL, R_T2, R_T2, x);
    pka::add(LEN_FULL, R_T2, t1, R_T2);
    pka::sub(LEN_FULL, R_T4, R_N4, R_T4);
    pka::mod_mul_acc_nfr(LEN_N, x, R_T2, R_T2, R_T4);
    pka::add(LEN_FULL, x, R_T4, x);
    pka::add(LEN_FULL, R_T4, x, R_T4);
    pka::sub(LEN_FULL, R_T3, R_N12, R_T4);
    pka::add(LEN_FULL, y, y, y);
    pka::mod_mul_nfr(LEN_N, y, y, y);
    pka::add(LEN_FULL, y, y, y);
    pka::add(LEN_FULL, R_T4, y, y);
    pka::mod_mul_nfr(LEN_N, R_T4, R_T4, t1);
    pka::sub(LEN_FULL, y, R_N8, y);
    pka::mod_mul_acc_nfr(LEN_N, y, R_T3, R_T2, y);
    pka::copy(LEN_FULL, t, R_T4);
}

/// Doubles a modified Jacobian point into a plain Jacobian one.
#[allow(clippy::too_many_arguments)]
fn double_mdf2jcb(x: u8, y: u8, z: u8, x1: u8, y1: u8, z1: u8, t1: u8) {
    pka::add(LEN_FULL, R_T, y1, y1);
    pka::mod_mul_nfr(LEN_N, z, R_T, z1);
    pka::mod_mul_nfr(LEN_N, y, y1, y1);
    pka::add(LEN_FULL, R_T, x1, x1);
    pka::add(LEN_FULL, R_T, R_T, R_T);
    pka::mod_mul_nfr(LEN_N, R_T, y, R_T);
    pka::mod_mul_nfr(LEN_N, R_T2, x1, x1);
    pka::add(LEN_FULL, x, R_T2, R_T2);
    pka::add(LEN_FULL, R_T2, R_T2, x);
    pka::add(LEN_FULL, R_T2, t1, R_T2);
    pka::sub(LEN_FULL, R_T, R_N4, R_T);
    pka::mod_mul_acc_nfr(LEN_N, x, R_T2, R_T2, R_T);
    pka::add(LEN_FULL, x, R_T, x);
    pka::add(LEN_FULL, R_T, x, R_T);
    pka::sub(LEN_FULL, R_T3, R_N12, R_T);
    pka::add(LEN_FULL, y, y, y);
    pka::mod_mul_nfr(LEN_N, y, y, y);
    pka::add(LEN_FULL, y, y, y);
    pka::sub(LEN_FULL, y, R_N8, y);
    pka::mod_mul_acc_nfr(LEN_N, y, R_T3, R_T2, y);
}

/// Adds an affine point to a Jacobian one, giving a modified Jacobian point.
#[allow(clippy::too_many_arguments)]
fn add_jcb_afn2mdf(x: u8, y: u8, z: u8, t: u8, x1: u8, y1: u8, z1: u8, x2: u8, y2: u8) {
    pka::mod_mul_nfr(LEN_N, t, z1, z1);
    pka::sub(LEN_FULL, x, R_N12, x1);
    pka::mod_mul_acc_nfr(LEN_N, R_T1, x2, t, x);
    pka::mod_mul_nfr(LEN_N, t, z1, t);
    pka::mod_mul_nfr(LEN_N, t, y2, t);
    pka::sub(LEN_FULL, t, R_N4, t);
    pka::add(LEN_FULL, t, y1, t);
    pka::mod_mul_nfr(LEN_N, z, z1, R_T1);
    pka::mod_mul_nfr(LEN_N, R_T2, R_T1, R_T1);
    pka::mod_mul_nfr(LEN_N, R_T1, R_T1, R_T2);
    pka::sub(LEN_FULL, R_T1, R_N4, R_T1);
    pka::mod_mul_nfr(LEN_N, y, R_T1, y1);
    pka::mod_mul_nfr(LEN_N, R_T2, x, R_T2);
    pka::mod_mul_acc_nfr(LEN_N, x, t, t, R_T1);
    pka::add(LEN_FULL, x, R_T2, x);
    pka::add(LEN_FULL, x, R_T2, x);
    pka::add(LEN_FULL, R_T2, x, R_T2);
    pka::mod_mul_acc_nfr(LEN_N, y, t, R_T2, y);
    pka::mod_mul_nfr(LEN_N, t, z, z);
    pka::mod_mul_nfr(LEN_N, t, t, t);
    pka::mod_mul_nfr(LEN_N, t, R_EC_A, t);
}

/// Adds two Jacobian points, giving a modified Jacobian point.
#[allow(clippy::too_many_arguments)]
fn add_jcb_jcb2mdf(x: u8, y: u8, z: u8, t: u8, x1: u8, y1: u8, z1: u8, x2: u8, y2: u8, z2: u8) {
    pka::mod_mul_nfr(LEN_N, t, z2, z2);
    pka::mod_mul_nfr(LEN_N, x, x1, t);
    pka::sub(LEN_FULL, x, R_N4, x);
    pka::mod_mul_nfr(LEN_N, t, z2, t);
    pka::mod_mul_nfr(LEN_N, y, y1, t);
    pka::sub(LEN_FULL, y, R_N4, y);
    pka::mod_mul_nfr(LEN_N, t, z1, z1);
    pka::mod_mul_acc_nfr(LEN_N, R_T1, x2, t, x);
    pka::mod_mul_nfr(LEN_N, t, z1, t);
    pka::mod_mul_acc_nfr(LEN_N, t, y2, t, y);
    pka::mod_mul_nfr(LEN_N, z, z1, z2);
    pka::mod_mul_nfr(LEN_N, z, z, R_T1);
    pka::mod_mul_nfr(LEN_N, R_T2, R_T1, R_T1);
    pka::mod_mul_nfr(LEN_N, R_T1, R_T1, R_T2);
    pka::sub(LEN_FULL, R_T1, R_N4, R_T1);
    pka::mod_mul_nfr(LEN_N, y, R_T1, y);
    pka::mod_mul_nfr(LEN_N, R_T2, x, R_T2);
    pka::mod_mul_acc_nfr(LEN_N, x, t, t, R_T1);
    pka::add(LEN_FULL, x, R_T2, x);
    pka::add(LEN_FULL, x, R_T2, x);
    pka::add(LEN_FULL, R_T2, x, R_T2);
    pka::mod_mul_acc_nfr(LEN_N, y, t, R_T2, y);
    pka::sub(LEN_FULL, y, R_N4, y);
    pka::mod_mul_nfr(LEN_N, t, z, z);
    pka::mod_mul_nfr(LEN_N, t, t, t);
    pka::mod_mul_nfr(LEN_N, t, R_EC_A, t);
}

/// Adds two Jacobian points.
#[allow(clippy::too_many_arguments)]
fn add_jcb_jcb2jcb(x: u8, y: u8, z: u8, x1: u8, y1: u8, z1: u8, x2: u8, y2: u8, z2: u8) {
    pka::mod_mul_nfr(LEN_N, R_T, z2, z2);
    pka::mod_mul_nfr(LEN_N, x, x1, R_T);
    pka::sub(LEN_FULL, x, R_N4, x);
    pka::mod_mul_nfr(LEN_N, R_T, z2, R_T);
    pka::mod_mul_nfr(LEN_N, y, y1, R_T);
    pka::sub(LEN_FULL, y, R_N4, y);
    pka::mod_mul_nfr(LEN_N, R_T, z1, z1);
    pka::mod_mul_acc_nfr(LEN_N, R_T1, x2, R_T, x);
    pka::mod_mul_nfr(LEN_N, R_T, z1, R_T);
    pka::mod_mul_acc_nfr(LEN_N, R_T, y2, R_T, y);
    pka::mod_mul_nfr(LEN_N, z, z1, z2);
    pka::mod_mul_nfr(LEN_N, z, z, R_T1);
    pka::mod_mul_nfr(LEN_N, R_T2, R_T1, R_T1);
    pka::mod_mul_nfr(LEN_N, R_T1, R_T1, R_T2);
    pka::sub(LEN_FULL, R_T1, R_N4, R_T1);
    pka::mod_mul_nfr(LEN_N, y, R_T1, y);
    pka::mod_mul_nfr(LEN_N, R_T2, x, R_T2);
    pka::mod_mul_acc_nfr(LEN_N, x, R_T, R_T, R_T1);
    pka::add(LEN_FULL, x, R_T2, x);
    pka::add(LEN_FULL, x, R_T2, x);
    pka::add(LEN_FULL, R_T2, x, R_T2);
    pka::mod_mul_acc_nfr(LEN_N, y, R_T, R_T2, y);
    pka::sub(LEN_FULL, y, R_N4, y);
}

/// Converts a Jacobian point to affine coordinates in place.
///
/// With `secure`, the inversion runs in constant time by exponentiation. Without it, the
/// faster variable-time engine inversion is used.
fn jcb2afn(secure: bool, x: u8, y: u8, z: u8) {
    if secure {
        pka::mod_inv_exp(R_AQ, z, R_NM2);
    } else {
        pka::mod_inv(LEN_N, R_AQ, z);
    }
    pka::mod_mul_nfr(LEN_N, y, y, R_AQ);
    pka::mod_mul_nfr(LEN_N, R_AQ, R_AQ, R_AQ);
    pka::mod_mul_nfr(LEN_N, x, x, R_AQ);
    pka::mod_mul_nfr(LEN_N, y, y, R_AQ);
    pka::reduce(LEN_N, x, x);
    pka::reduce(LEN_N, y, y);
}

/// Adds two affine points, giving an affine point.
fn add_aff(x: u8, y: u8, x1: u8, y1: u8, x2: u8, y2: u8) {
    pka::sub(LEN_FULL, x, REG_N, x1);
    pka::add(LEN_FULL, R_AAA_Z, x, x2);
    pka::sub(LEN_FULL, R_T, REG_N, y2);
    pka::add(LEN_FULL, R_T, y1, R_T);
    pka::mod_mul_nfr(LEN_N, R_T2, R_AAA_Z, R_AAA_Z);
    pka::mod_mul_nfr(LEN_N, R_T1, R_AAA_Z, R_T2);
    pka::sub(LEN_FULL, R_T1, R_N4, R_T1);
    pka::mod_mul_nfr(LEN_N, y, R_T1, y1);
    pka::mod_mul_nfr(LEN_N, R_T2, x, R_T2);
    pka::mod_mul_acc_nfr(LEN_N, x, R_T, R_T, R_T1);
    pka::add(LEN_FULL, x, R_T2, x);
    pka::add(LEN_FULL, x, R_T2, x);
    pka::add(LEN_FULL, R_T2, x, R_T2);
    pka::mod_mul_acc_nfr(LEN_N, y, R_T, R_T2, y);
    jcb2afn(false, x, y, R_AAA_Z);
}

/// Fills `R_N4`, `R_N8` and `R_N12` with multiples of the modulus.
fn set_multiples() {
    pka::add(LEN_FULL, R_N4, REG_N, REG_N);
    pka::add(LEN_FULL, R_N4, R_N4, R_N4);
    pka::add(LEN_FULL, R_N8, R_N4, R_N4);
    pka::add(LEN_FULL, R_N12, R_N8, R_N4);
}

/// Reads the bits of a scalar held in a PKA register, from the most significant one down.
struct BitReader {
    word: u32,
    fresh: bool,
}

impl BitReader {
    fn new() -> Self {
        Self { word: 0, fresh: true }
    }

    /// Returns bit `i` of the register, which must be read in strictly decreasing order.
    fn next_bit(&mut self, reg: u8, i: usize) -> u32 {
        if self.fresh || i % 32 == 31 {
            self.word = pka::read_word(reg, i / 32);
            if i % 32 != 31 {
                self.word <<= 31 - i % 32;
            }
            self.fresh = false;
        }
        let b = self.word >> 31;
        self.word <<= 1;
        b
    }

    /// Returns bits `i + 1` and `i` of the register, `i` being even and strictly decreasing.
    fn next_2bits(&mut self, reg: u8, i: usize) -> u32 {
        debug_assert!(i % 2 == 0);
        if self.fresh || i % 32 == 30 {
            self.word = pka::read_word(reg, i / 32);
            self.fresh = false;
        }
        (self.word >> (i % 32)) & 3
    }
}

/// Multiplies the point in `S_XP`, `S_YP` by the scalar in `S_RK`, in place.
///
/// Expects the curve order in `S_ORD`, the coefficient `a` in `R_EC_A`, the modulus in
/// `REG_N`, and the sizes table set to the modulus size.
///
/// Returns `false` if the result is wrong. The addition formulas do not handle `P + P` or
/// `P + (-P)`. The ladder hits those when the scalar or its negation is tiny, because the
/// accumulator is then a small multiple of the point. The engine operations are the same
/// either way. Only the checks after each addition differ, and they reveal at most that the
/// scalar is one of a handful of values no secret should be.
fn scalar_mult(order_bits: u32) -> bool {
    set_multiples();
    let mut degenerate = false;

    // Multiply by whichever of k and -k has more leading zero bits, so that the number of
    // iterations does not reveal the length of the scalar. The result is negated at the end
    // if -k was used.
    pka::copy(LEN_FULL, S_TP, S_RK);
    pka::sub(LEN_FULL, S_ZP, S_ORD, S_TP);
    let sz1 = pka::effective_bits(S_RK);
    let sz2 = pka::effective_bits(S_ZP);
    let (size, positive) = if sz1 > sz2 {
        pka::copy(LEN_FULL, S_RK, S_TP);
        (sz1, true)
    } else {
        pka::copy(LEN_FULL, S_RK, S_ZP);
        (sz2, false)
    };
    debug_assert!(size <= order_bits);

    // P in Jacobian coordinates, then 2P and 4P in modified Jacobian coordinates.
    pka::set_value(S_ZP, 1);
    double_mdf2mdf(S_X2, S_Y2, S_Z2, S_T2, S_XP, S_YP, S_ZP, R_EC_A);
    double_mdf2mdf(S_X4, S_Y4, S_Z4, S_T4, S_X2, S_Y2, S_Z2, S_T2);

    // Scan the scalar two bits at a time. `carry` is -1 when the accumulator runs one P
    // ahead of the scalar bits consumed so far, which lets every step add a non-zero
    // multiple of P and so keeps the operation sequence independent of the scalar.
    let mut i = ((size as usize + 1) & !1) - 2;
    let mut bits = BitReader::new();
    let mut carry: i32 = match bits.next_2bits(S_RK, i) {
        1 => {
            copy4(S_XS, S_YS, S_ZS, S_TS, S_X2, S_Y2, S_Z2, S_T2);
            -1
        }
        2 => {
            copy4(S_XS, S_YS, S_ZS, S_TS, S_X2, S_Y2, S_Z2, S_T2);
            0
        }
        // The top bit of the scalar is set, so the two top bits cannot both be zero.
        _ => {
            copy4(S_XS, S_YS, S_ZS, S_TS, S_X4, S_Y4, S_Z4, S_T4);
            -1
        }
    };

    // The `T` coordinates of P, 2P and 4P are no longer needed: reuse them for the negated
    // Y coordinates, which turn an addition into a subtraction.
    pka::sub(LEN_FULL, S_TP, R_N4, S_YP);
    pka::sub(LEN_FULL, S_T2, R_N4, S_Y2);
    pka::sub(LEN_FULL, S_T4, R_N4, S_Y4);

    while i >= 2 {
        i -= 2;
        // S *= 4
        double_mdf2mdf(S_XS, S_YS, S_ZS, S_ZR, S_XS, S_YS, S_ZS, S_TS);
        double_mdf2jcb(S_XS, S_YS, S_ZS, S_XS, S_YS, S_ZS, S_ZR);

        let b2 = bits.next_2bits(S_RK, i) as i32;
        let (px, py, pz, next) = match carry * 4 + b2 {
            -4 => (S_X4, S_T4, S_Z4, 0),
            -3 => (S_X2, S_T2, S_Z2, -1),
            -2 => (S_X2, S_T2, S_Z2, 0),
            -1 => (S_XP, S_TP, S_ZP, 0),
            0 => (S_XP, S_YP, S_ZP, -1),
            1 => (S_XP, S_YP, S_ZP, 0),
            2 => (S_X2, S_Y2, S_Z2, 0),
            _ => (S_X4, S_Y4, S_Z4, -1),
        };
        add_jcb_jcb2mdf(S_XS, S_YS, S_ZS, S_TS, S_XS, S_YS, S_ZS, px, py, pz);
        degenerate |= z_is_zero(S_ZS);
        carry = next;
    }

    // Settle the carry by subtracting the P that the accumulator ran ahead by.
    add_jcb_jcb2jcb(S_X2, S_Y2, S_Z2, S_XS, S_YS, S_ZS, S_XP, S_TP, S_ZP);
    degenerate |= z_is_zero(S_Z2);
    let (x, y, z, neg) = if carry == -1 {
        pka::sub(LEN_FULL, S_T2, R_N4, S_Y2);
        (S_X2, S_Y2, S_Z2, S_T2)
    } else {
        pka::sub(LEN_FULL, S_TS, R_N4, S_YS);
        (S_XS, S_YS, S_ZS, S_TS)
    };
    pka::copy(LEN_FULL, S_XP, x);
    pka::copy(LEN_FULL, S_YP, if positive { y } else { neg });
    pka::copy(LEN_FULL, S_ZP, z);
    jcb2afn(true, S_XP, S_YP, S_ZP);
    !degenerate
}

#[allow(clippy::too_many_arguments)]
fn copy4(x: u8, y: u8, z: u8, t: u8, x1: u8, y1: u8, z1: u8, t1: u8) {
    pka::copy(LEN_FULL, x, x1);
    pka::copy(LEN_FULL, y, y1);
    pka::copy(LEN_FULL, z, z1);
    pka::copy(LEN_FULL, t, t1);
}

/// Loads the curve modulus, its Barrett tag and the coefficient `a`, and sets the sizes
/// table up for operations modulo the field prime.
fn load_field(curve: &Curve) {
    pka::write_be(REG_N, curve.p);
    pka::calc_np(curve.mod_bits as u32, REG_N, REG_NP, R_T, R_T1);
    pka::write_be(R_EC_A, curve.a);
}

/// Multiplies a point by a scalar.
///
/// Returns `Ok(false)` with nothing written if the ladder could not compute this product.
/// See [`scalar_mult`]. The ladder is already constant-time, so `blind` is ignored.
pub(crate) fn ecc_mul(
    curve: &Curve,
    scalar: &[u8],
    px: &[u8],
    py: &[u8],
    rx: &mut [u8],
    ry: &mut [u8],
    _blind: bool,
) -> Result<bool, Error> {
    let bits = (curve.mod_bits as u32).max(curve.order_bits as u32);
    pka::init(bits, REG_COUNT)?;
    pka::set_len(LEN_N, curve.mod_bits as u32);
    load_field(curve);
    pka::write_be(S_ORD, curve.n);
    pka::write_be(S_XP, px);
    pka::write_be(S_YP, py);
    pka::write_be(S_RK, scalar);
    let ok = scalar_mult(curve.order_bits as u32);
    if ok {
        pka::read_be(S_XP, rx);
        pka::read_be(S_YP, ry);
    }
    pka::finish(REG_COUNT);
    Ok(ok)
}

/// Signs a hash. `hash` is already reduced to the size of the curve.
pub(crate) fn ecdsa_sign(
    curve: &Curve,
    private_key: &[u8],
    k: &[u8],
    hash: &[u8],
    out_r: &mut [u8],
    out_s: &mut [u8],
) -> Result<(), Error> {
    // R = k*G, r = x(R).
    let mut ephemeral_x = [0u8; MAX_CURVE_LEN];
    let ephemeral_x = &mut ephemeral_x[..curve.size()];
    let mut ephemeral_y = [0u8; MAX_CURVE_LEN];
    let ephemeral_y = &mut ephemeral_y[..curve.size()];
    // An ephemeral key the ladder cannot handle is one of a handful of tiny values, which
    // would give the private key away: it is as unusable as one that yields a zero component.
    if !ecc_mul(curve, k, curve.gx, curve.gy, ephemeral_x, ephemeral_y, true)? {
        return Err(Error::RetryWithNewK);
    }

    // s = (h + r*d) / k mod n.
    const R_C: u8 = 2;
    const R_M: u8 = 3;
    const R_K: u8 = 4;
    const R_D: u8 = 5;
    const R_KINV: u8 = 6;
    const R_S: u8 = 7;
    const R_TMP: u8 = 8;
    /// Registers 0 to 8, plus the two the engine reserves for itself.
    const REGS: usize = 11;

    let res = (|| {
        pka::init(curve.order_bits as u32, REGS)?;
        pka::write_be(REG_N, curve.n);
        pka::calc_np(curve.order_bits as u32, REG_N, REG_NP, R_C, R_M);
        pka::write_be(R_C, ephemeral_x);
        pka::write_be(R_M, hash);
        pka::write_be(R_K, k);
        pka::write_be(R_D, private_key);

        // The ephemeral key is checked against the order by the caller, but the reduction
        // also guards against a hardware fault leaving a stale value behind.
        pka::div(LEN_MAX, R_TMP, R_K, REG_N);
        if pka::equal_im(LEN_MAX, R_K, 0) {
            return Err(Error::RetryWithNewK);
        }
        pka::mod_inv_exp(R_KINV, R_K, R_TMP);
        if pka::equal_im(LEN_FULL, R_KINV, 0) {
            return Err(Error::RetryWithNewK);
        }
        pka::mod_mul(LEN_N, R_S, R_D, R_C);
        pka::reduce(LEN_N, R_M, R_M);
        pka::mod_add(LEN_FULL, R_S, R_S, R_M);
        pka::mod_mul(LEN_N, R_S, R_S, R_KINV);
        if pka::equal_im(LEN_FULL, R_S, 0) {
            return Err(Error::RetryWithNewK);
        }
        pka::read_be(R_C, out_r);
        pka::read_be(R_S, out_s);
        Ok(())
    })();
    pka::finish(REGS);
    res
}

/// Verifies a signature. `hash` is already reduced to the size of the curve.
///
/// Returns `Ok(false)` when the Strauss ladder could not compute `u1*G + u2*Q`, see
/// [`sum_of_two_scalar_mults`]. `u1` and `u2` are then written out, reduced modulo the
/// order, for the caller to finish the verification another way.
#[allow(clippy::too_many_arguments)]
pub(crate) fn ecdsa_verify(
    curve: &Curve,
    qx: &[u8],
    qy: &[u8],
    r: &[u8],
    s: &[u8],
    hash: &[u8],
    u1: &mut [u8],
    u2: &mut [u8],
) -> Result<bool, Error> {
    let mod_bits = curve.mod_bits as u32;
    let order_bits = curve.order_bits as u32;
    pka::init(mod_bits.max(order_bits), REG_COUNT)?;
    let res = verify_inner(curve, qx, qy, r, s, hash, mod_bits, order_bits, u1, u2);
    pka::finish(REG_COUNT);
    res
}

#[allow(clippy::too_many_arguments)]
fn verify_inner(
    curve: &Curve,
    qx: &[u8],
    qy: &[u8],
    r: &[u8],
    s: &[u8],
    hash: &[u8],
    mod_bits: u32,
    order_bits: u32,
    u1: &mut [u8],
    u2: &mut [u8],
) -> Result<bool, Error> {
    // Operations run modulo the curve order first, so it goes in the modulus register and the
    // field prime waits in a spare one.
    pka::set_len(LEN_N, order_bits);
    pka::write_be(REG_N, curve.n);
    pka::calc_np(order_bits, REG_N, REG_NP, V_GX, V_GY);
    pka::write_be(V_TMP_N, curve.p);
    pka::calc_np(mod_bits, V_TMP_N, V_TMP_NP, V_GX, V_GY);

    pka::write_be(V_C, r);
    pka::write_be(V_D, s);
    pka::write_be(V_F, hash);
    pka::write_be(V_GX, curve.gx);
    pka::write_be(V_GY, curve.gy);
    pka::write_be(V_WX, qx);
    pka::write_be(V_WY, qy);
    pka::write_be(R_EC_A, curve.a);

    // h = 1/s, h1 = h*z, h2 = h*r  (mod n)
    pka::mod_inv_exp(V_H, V_D, V_TMP);
    pka::div(LEN_FULL, V_TMP, V_F, REG_N);
    pka::mod_mul(LEN_N, V_H1, V_F, V_H);
    pka::mod_mul(LEN_N, V_H2, V_C, V_H);

    // Switch to operations modulo the field prime, keeping the order for the final reduction.
    pka::clear(LEN_FULL, REG_T0);
    pka::clear(LEN_FULL, REG_T1);
    pka::set_len(LEN_N, mod_bits);
    pka::copy(LEN_FULL, V_TMP, REG_N);
    pka::copy(LEN_FULL, REG_N, V_TMP_N);
    pka::copy(LEN_FULL, V_TMP_N, V_TMP);
    pka::copy(LEN_FULL, REG_NP, V_TMP_NP);
    set_multiples();

    // R = h1*G + h2*Q. The ladder cannot add a point to itself or to its opposite, which
    // a valid signature can make it do: a public key equal or opposite to the generator, or
    // a hash and public key chosen so that an intermediate sum of the two multiplications is
    // the point at infinity. The caller computes the sum another way then. `h1` may be zero,
    // when the hash is a multiple of the order; the ladder cannot take that either.
    if pka::equal_im(LEN_FULL, V_H1, 0)
        || pka::equal(LEN_FULL, V_WX, V_GX)
        || !sum_of_two_scalar_mults(V_RX, V_RY, V_H1, V_GX, V_GY, V_H2, V_WX, V_WY)?
    {
        pka::read_be(V_H1, u1);
        pka::read_be(V_H2, u2);
        return Ok(false);
    }

    // The signature is valid when x(R) mod n equals r.
    pka::set_len(LEN_N, order_bits);
    pka::div(LEN_FULL, V_TMP, V_RX, V_TMP_N);
    if pka::equal(LEN_FULL, V_RX, V_C) {
        Ok(true)
    } else {
        Err(Error::InvalidSignature)
    }
}

/// Computes `a*P + b*Q` with the Strauss algorithm, scanning both scalars at once.
///
/// `P` and `Q` must be neither equal nor opposite, and both scalars nonzero.
///
/// The addition formulas do not handle `P + P` or `P + (-P)`. Such an addition lands the
/// running sum at infinity, so it is checked for that after every addition. If it happens
/// this returns `Ok(false)`, and the caller computes the sum another way. Only public values
/// go through here, so the check may leak.
#[allow(clippy::too_many_arguments)]
fn sum_of_two_scalar_mults(xr: u8, yr: u8, a: u8, xp: u8, yp: u8, b: u8, xq: u8, yq: u8) -> Result<bool, Error> {
    if pka::equal_im(LEN_FULL, a, 0) || pka::equal_im(LEN_FULL, b, 0) {
        return Err(Error::InvalidScalar);
    }
    let bits = pka::effective_bits(a).max(pka::effective_bits(b));
    let mut i = bits as usize - 1;

    add_aff(V_XPQ, V_YPQ, xp, yp, xq, yq);

    let mut ba = BitReader::new();
    let mut bb = BitReader::new();
    let (sx, sy) = match ba.next_bit(a, i) * 2 + bb.next_bit(b, i) {
        1 => (xq, yq),
        2 => (xp, yp),
        // Both scalars have their top bit below `bits`; at least one is set here.
        _ => (V_XPQ, V_YPQ),
    };
    pka::copy(LEN_FULL, xr, sx);
    pka::copy(LEN_FULL, yr, sy);
    pka::set_value(V_ZR, 1);
    pka::copy(LEN_FULL, V_TR, R_EC_A);

    while i > 0 {
        i -= 1;
        let b2 = ba.next_bit(a, i) * 2 + bb.next_bit(b, i);
        if b2 == 0 {
            double_mdf2mdf_t4(xr, yr, V_ZR, V_TR, xr, yr, V_ZR, V_TR);
        } else {
            double_mdf2jcb(xr, yr, V_ZR, xr, yr, V_ZR, V_TR);
            let (px, py) = match b2 {
                1 => (xq, yq),
                2 => (xp, yp),
                _ => (V_XPQ, V_YPQ),
            };
            add_jcb_afn2mdf(xr, yr, V_ZR, V_TR, xr, yr, V_ZR, px, py);
            if z_is_zero(V_ZR) {
                return Ok(false);
            }
        }
    }
    jcb2afn(false, xr, yr, V_ZR);
    Ok(true)
}

/// Returns whether a possibly unreduced `Z` coordinate is zero, so the point is at infinity.
///
/// The value is reduced by adding zero to it twice, since `_nfr` products go up to twice the
/// modulus. The engine's reduction instruction is not used. It is only safe as the last
/// operation before reading a result, and breaks the operations that follow it.
fn z_is_zero(z: u8) -> bool {
    pka::clear(LEN_FULL, R_T3);
    pka::mod_add_im(LEN_FULL, R_T3, z, 0);
    pka::mod_add_im(LEN_FULL, R_T3, R_T3, 0);
    pka::equal_im(LEN_FULL, R_T3, 0)
}

/// Adds two affine points that are neither equal nor opposite.
pub(crate) fn ecc_add(
    curve: &Curve,
    px: &[u8],
    py: &[u8],
    qx: &[u8],
    qy: &[u8],
    rx: &mut [u8],
    ry: &mut [u8],
) -> Result<(), Error> {
    const R_PX: u8 = 14;
    const R_PY: u8 = 15;
    const R_QX: u8 = 16;
    const R_QY: u8 = 17;
    const R_RX: u8 = 18;
    const R_RY: u8 = 19;
    /// Registers 0 to 19, plus the two the engine reserves for itself.
    const REGS: usize = 22;

    pka::init(curve.mod_bits as u32, REGS)?;
    load_field(curve);
    set_multiples();
    pka::write_be(R_PX, px);
    pka::write_be(R_PY, py);
    pka::write_be(R_QX, qx);
    pka::write_be(R_QY, qy);
    add_aff(R_RX, R_RY, R_PX, R_PY, R_QX, R_QY);
    pka::read_be(R_RX, rx);
    pka::read_be(R_RY, ry);
    pka::finish(REGS);
    Ok(())
}

/// Doubles an affine point.
///
/// The final inversion runs in constant time. The point may be an intermediate of a
/// computation on a secret scalar.
pub(crate) fn ecc_double(curve: &Curve, px: &[u8], py: &[u8], rx: &mut [u8], ry: &mut [u8]) -> Result<(), Error> {
    const R_X: u8 = 14;
    const R_Y: u8 = 15;
    const R_Z: u8 = 16;
    /// Registers 0 to 16, plus the two the engine reserves for itself.
    const REGS: usize = 19;

    pka::init(curve.mod_bits as u32, REGS)?;
    load_field(curve);
    set_multiples();
    pka::write_be(R_X, px);
    pka::write_be(R_Y, py);
    // With Z = 1 the cached a·Z⁴ of the modified Jacobian form is just a.
    pka::set_value(R_Z, 1);
    double_mdf2jcb(R_X, R_Y, R_Z, R_X, R_Y, R_Z, R_EC_A);
    jcb2afn(true, R_X, R_Y, R_Z);
    pka::read_be(R_X, rx);
    pka::read_be(R_Y, ry);
    pka::finish(REGS);
    Ok(())
}

/// Computes `a*P + b*Q` with the Strauss algorithm, for public values.
///
/// `P` and `Q` must be neither equal nor opposite. Both scalars must be nonzero and below
/// the order. Returns `Ok(false)` if the computation passed through infinity. The caller
/// must then compute the sum another way. See [`sum_of_two_scalar_mults`].
#[cfg(feature = "_embassy-crypto-pka")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn ecc_lincomb(
    curve: &Curve,
    a: &[u8],
    px: &[u8],
    py: &[u8],
    b: &[u8],
    qx: &[u8],
    qy: &[u8],
    rx: &mut [u8],
    ry: &mut [u8],
) -> Result<bool, Error> {
    let bits = (curve.mod_bits as u32).max(curve.order_bits as u32);
    pka::init(bits, REG_COUNT)?;
    pka::set_len(LEN_N, curve.mod_bits as u32);
    load_field(curve);
    set_multiples();
    pka::write_be(V_H1, a);
    pka::write_be(V_H2, b);
    pka::write_be(V_GX, px);
    pka::write_be(V_GY, py);
    pka::write_be(V_WX, qx);
    pka::write_be(V_WY, qy);
    let res = sum_of_two_scalar_mults(V_RX, V_RY, V_H1, V_GX, V_GY, V_H2, V_WX, V_WY);
    if let Ok(true) = res {
        pka::read_be(V_RX, rx);
        pka::read_be(V_RY, ry);
    }
    pka::finish(REG_COUNT);
    res
}

/// Checks that a point satisfies the curve equation.
pub(crate) fn point_check(curve: &Curve, x: &[u8], y: &[u8]) -> Result<(), Error> {
    const R_X: u8 = 3;
    const R_Y: u8 = 4;
    const R_A: u8 = 5;
    const R_B: u8 = 6;
    const R_Y2: u8 = 7;
    /// Registers 0 to 7, plus the two the engine reserves for itself.
    const REGS: usize = 10;

    pka::init(curve.mod_bits as u32, REGS)?;
    pka::write_be(REG_N, curve.p);
    pka::calc_np(curve.mod_bits as u32, REG_N, REG_NP, R_T, R_X);
    pka::write_be(R_X, x);
    pka::write_be(R_Y, y);
    pka::write_be(R_A, curve.a);
    pka::write_be(R_B, curve.b);

    // y² == x³ + a·x + b
    pka::mod_mul(LEN_N, R_T, R_X, R_X);
    pka::mod_add(LEN_FULL, R_T, R_T, R_A);
    pka::mod_mul(LEN_N, R_T, R_X, R_T);
    pka::mod_add(LEN_FULL, R_Y2, R_T, R_B);
    pka::mod_mul(LEN_N, R_T, R_Y, R_Y);
    let ok = pka::equal(LEN_FULL, R_Y2, R_T);
    pka::finish(REGS);

    if ok { Ok(()) } else { Err(Error::InvalidPoint) }
}
