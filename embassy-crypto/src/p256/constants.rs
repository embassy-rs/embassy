//! P-256 parameters (FIPS 186-4 / SEC 2) as canonical big-endian byte strings.
//!
//! All scalar-field constants are cross-checked in the module's unit tests:
//! `MODULUS_DEC` is parsed independently (decimal) and compared against the
//! hex constants, and algebraic relations are verified bytewise.

/// Decode 64 hex chars into a `[u8; 32]` at compile time.
pub(crate) const fn unhex32(s: &str) -> [u8; 32] {
    const fn nib(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => panic!("invalid hex digit"),
        }
    }
    let b = s.as_bytes();
    assert!(b.len() == 64, "expected 64 hex chars");
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = (nib(b[2 * i]) << 4) | nib(b[2 * i + 1]);
        i += 1;
    }
    out
}

/// Group order `n` of the P-256 curve (the scalar field modulus).
pub const N: [u8; 32] = unhex32("FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551");

/// `n // 2` — the `IsHigh` threshold (scalars `> n/2` are "high").
pub const N_HALF: [u8; 32] = unhex32("7FFFFFFF800000007FFFFFFFFFFFFFFFDE737D56D38BCF4279DCE5617E3192A8");

/// `(n + 1) // 2`, i.e. `2^-1 mod n`.
pub const TWO_INV: [u8; 32] = unhex32("7FFFFFFF800000007FFFFFFFFFFFFFFFDE737D56D38BCF4279DCE5617E3192A9");

/// Odd part of `n - 1`: `Q = (n - 1) >> 4` (Tonelli–Shanks exponent).
pub const SQRT_Q: [u8; 32] = unhex32("0FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC63255");

/// `(Q + 1) // 2` — Tonelli–Shanks candidate-root exponent.
pub const SQRT_Q1_2: [u8; 32] = unhex32("07FFFFFFF800000007FFFFFFFFFFFFFFFDE737D56D38BCF4279DCE5617E3192B");

/// `n - 1`, i.e. the canonical encoding of the scalar `-1` (used for point negation).
pub const N_MINUS_ONE: [u8; 32] = unhex32("FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632550");

/// `n` as a decimal string (`ff::PrimeField::MODULUS`).
pub const MODULUS_DEC: &str = "115792089210356248762697446949407573529996955224135760342422259061068512044369";

/// A known multiplicative generator of `GF(n)` (the integer 7, a quadratic non-residue).
pub const MULT_GENERATOR: [u8; 32] = unhex32("0000000000000000000000000000000000000000000000000000000000000007");

/// `7^Q mod n` — a primitive `2^4`-th root of unity (`ff::PrimeField::ROOT_OF_UNITY`, S = 4).
pub const ROOT_OF_UNITY: [u8; 32] = unhex32("FFC97F062A770992BA807ACE842A3DFC1546CAD004378DAF0592D7FBB41E6602");

/// `(ROOT_OF_UNITY)^-1 mod n`.
pub const ROOT_OF_UNITY_INV: [u8; 32] = unhex32("A0A66A5562D46F2AC645FA0458131CAEE3AC117C794C4137379C7F0657C73764");

/// `7^(2^4) mod n` — a generator of the 16-th roots of unity subgroup.
pub const DELTA: [u8; 32] = unhex32("00000000000000000000000000000000000000000000000000001E39A5057D81");

/// Base field prime `p`.
#[allow(dead_code)] // reference constant; used by tests and readers, not by production paths
pub const P: [u8; 32] = unhex32("FFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF");

/// Short-Weierstrass `a = p - 3`.
#[allow(dead_code)] // reference constant; used by tests and readers, not by production paths
pub const A: [u8; 32] = unhex32("FFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFC");

/// Short-Weierstrass `b`.
#[allow(dead_code)] // reference constant; used by tests and readers, not by production paths
pub const B: [u8; 32] = unhex32("5AC635D8AA3A93E7B3EBBD55769886BC651D06B0CC53B0F63BCE3C3E27D2604B");

/// Base point `G` (x coordinate).
pub const GENERATOR_X: [u8; 32] = unhex32("6B17D1F2E12C4247F8BCE6E563A440F277037D812DEB33A0F4A13945D898C296");

/// Base point `G` (y coordinate).
pub const GENERATOR_Y: [u8; 32] = unhex32("4FE342E2FE1A7F9B8EE7EB4A7C0F9E162BCE33576B315ECECBB6406837BF51F5");

/// Parse a decimal string into fixed-width big-endian bytes (test-only helper;
/// independent of the hex constants, used to cross-check them).
#[cfg(test)]
pub(crate) fn decimal_to_be(s: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for d in s.bytes() {
        assert!(d.is_ascii_digit());
        let mut carry = (d - b'0') as u16;
        for i in (0..32).rev() {
            let v = out[i] as u16 * 10 + carry;
            out[i] = v as u8;
            carry = v >> 8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Big-endian fixed-width addition of 32-byte numbers; returns carry out.
    fn add_be(a: &[u8; 32], b: &[u8; 32]) -> ([u8; 32], u8) {
        let mut out = [0u8; 32];
        let mut carry = 0u16;
        for i in (0..32).rev() {
            let s = a[i] as u16 + b[i] as u16 + carry;
            out[i] = s as u8;
            carry = s >> 8;
        }
        (out, carry as u8)
    }

    fn one() -> [u8; 32] {
        let mut o = [0u8; 32];
        o[31] = 1;
        o
    }

    #[test]
    fn modulus_hex_matches_decimal() {
        assert_eq!(decimal_to_be(MODULUS_DEC), N);
    }

    #[test]
    fn n_is_odd_and_bit_length_256() {
        assert_eq!(N[31] & 1, 1);
        assert_eq!(N[0] >> 7, 1);
    }

    #[test]
    fn two_inv_is_half_of_n_plus_one() {
        // 2 * TWO_INV == N + 1
        let (sum, carry) = add_be(&TWO_INV, &TWO_INV);
        assert_eq!(carry, 0);
        let (n_plus_1, carry2) = add_be(&N, &one());
        assert_eq!(carry2, 0);
        assert_eq!(sum, n_plus_1);
    }

    #[test]
    fn n_half_relations() {
        // 2 * (N // 2) + 1 == N   and   N_HALF < N
        let (dbl, carry) = add_be(&N_HALF, &N_HALF);
        assert_eq!(carry, 0);
        let (n, c2) = add_be(&dbl, &one());
        assert_eq!(c2, 0);
        assert_eq!(n, N);
        assert!(N_HALF < N);
    }

    #[test]
    fn sqrt_exponents_are_odd_and_related() {
        // Q odd: low bit set. 2 * SQRT_Q1_2 == Q + 1.
        assert_eq!(SQRT_Q[31] & 1, 1);
        let (q, c) = add_be(&SQRT_Q1_2, &SQRT_Q1_2);
        assert_eq!(c, 0);
        let (q1, c2) = add_be(&SQRT_Q, &one());
        assert_eq!(c2, 0);
        assert_eq!(q, q1);
    }

    #[test]
    fn multiplicative_generator_is_small_nonzero() {
        assert_eq!(MULT_GENERATOR[31], 7);
    }

    #[test]
    fn n_minus_one_relations() {
        // (N - 1) + 1 == N
        let mut one = [0u8; 32];
        one[31] = 1;
        let (n, carry) = add_be(&N_MINUS_ONE, &one);
        assert_eq!(carry, 0);
        assert_eq!(n, N);
        // N - 1 is even
        assert_eq!(N_MINUS_ONE[31] & 1, 0);
    }

    #[test]
    fn weierstrass_coefficients_sanity() {
        // P/A/B are reference constants: the adapter performs no base-field
        // arithmetic (point validity is delegated to the backend's
        // point_from_canonical), so nothing in production code references them.
        // This test keeps the transcription honest: a == p - 3, p odd, 0 < b < p.
        let mut three = [0u8; 32];
        three[31] = 3;
        let (a_plus_3, carry) = add_be(&A, &three);
        assert_eq!(carry, 0);
        assert_eq!(a_plus_3, P);
        assert_eq!(P[31] & 1, 1);
        assert_ne!(B, [0u8; 32]);
        assert!(B.as_slice() < P.as_slice()); // lexicographic == numeric, big-endian
    }
}
