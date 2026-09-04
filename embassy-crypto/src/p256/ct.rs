//! Constant-time big-endian byte-slice helpers (the canonical domain of [`crate::p256::Scalar`]).
//!
//! These operate on `[u8; 32]` and rely only on `u8` compares compiling to
//! branchless `setcc` sequences (the same assumption `subtle` documents for its
//! slice implementations). Secret data only ever reaches these through
//! `scalar_to_canonical`, which must be constant-time per the backend contract.

use subtle::Choice;

/// Constant-time `a == b`.
#[inline]
pub(crate) fn ct_eq_bytes(a: &[u8; 32], b: &[u8; 32]) -> Choice {
    let mut acc = 0u8;
    for i in 0..32 {
        acc |= a[i] ^ b[i];
    }
    Choice::from((acc == 0) as u8)
}

/// Constant-time big-endian `a < b`.
#[inline]
pub(crate) fn ct_lt_bytes(a: &[u8; 32], b: &[u8; 32]) -> Choice {
    let mut lt = Choice::from(0);
    let mut eq = Choice::from(1);
    for i in 0..32 {
        let byte_lt = Choice::from((a[i] < b[i]) as u8);
        let byte_eq = Choice::from((a[i] == b[i]) as u8);
        lt |= eq & byte_lt;
        eq &= byte_eq;
    }
    lt
}

/// Constant-time big-endian `a > b`.
#[inline]
pub(crate) fn ct_gt_bytes(a: &[u8; 32], b: &[u8; 32]) -> Choice {
    ct_lt_bytes(b, a)
}

/// Constant-time select: `choice == 1` returns `b`, else `a`.
#[inline]
pub(crate) fn ct_select_bytes(a: &[u8; 32], b: &[u8; 32], choice: Choice) -> [u8; 32] {
    let mask = 0u8.wrapping_sub(choice.unwrap_u8());
    let mut out = [0u8; 32];
    for i in 0..32 {
        // subtle contract: choice == 1 -> b. (The original Stage-2 code had the
        // operands swapped, returning `a` on choice == 1; first executed by
        // p256::ct::tests in the embassy merge.)
        out[i] = (b[i] & mask) | (a[i] & !mask);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(x: u8) -> [u8; 32] {
        [x; 32]
    }

    #[test]
    fn eq_properties() {
        assert!(bool::from(ct_eq_bytes(&b(0), &b(0))));
        assert!(!bool::from(ct_eq_bytes(&b(0), &b(1))));
        let mut a = b(0xAA);
        a[17] = 0x55;
        assert!(bool::from(ct_eq_bytes(&a, &a)));
        let mut c = a;
        c[17] = 0x54;
        assert!(!bool::from(ct_eq_bytes(&a, &c)));
    }

    #[test]
    fn ordering_lexicographic_big_endian() {
        let mut hi = [0u8; 32];
        hi[0] = 1;
        let mut hi2 = [0u8; 32];
        hi2[5] = 1;
        assert!(bool::from(ct_lt_bytes(&b(0), &hi)));
        assert!(bool::from(ct_gt_bytes(&hi, &b(0))));
        // hi[0]=1 is the MOST significant byte: hi = 2^248 > hi2 = 2^208.
        assert!(bool::from(ct_gt_bytes(&hi, &hi2)));
        assert!(bool::from(ct_lt_bytes(&hi2, &hi)));
        assert!(!bool::from(ct_lt_bytes(&hi, &hi)));
        assert!(!bool::from(ct_gt_bytes(&hi, &hi)));
        // low byte matters
        let mut x = [0u8; 32];
        x[31] = 1;
        let mut y = [0u8; 32];
        y[31] = 2;
        assert!(bool::from(ct_lt_bytes(&x, &y)));
    }

    #[test]
    fn select_picks_correct_side() {
        let t = Choice::from(1);
        let f = Choice::from(0);
        assert_eq!(ct_select_bytes(&b(1), &b(2), f), b(1));
        assert_eq!(ct_select_bytes(&b(1), &b(2), t), b(2));
        // selecting with wrong-size masks still yields exact bytes
        let a = [0xFFu8; 32];
        let c = [0x0Fu8; 32];
        assert_eq!(ct_select_bytes(&a, &c, t), c);
        assert_eq!(ct_select_bytes(&a, &c, f), a);
    }
}
