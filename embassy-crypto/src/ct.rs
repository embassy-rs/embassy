//! Constant-time helpers on byte arrays.

/// Constant-time equality of two equal-length slices.
///
/// Returns `false` if the lengths differ (the lengths are not secret).
pub(crate) fn eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Constant-time `a < b` for big-endian integers of equal width.
pub(crate) fn lt(a: &[u8], b: &[u8]) -> bool {
    debug_assert_eq!(a.len(), b.len());
    // Compute `a - b` from the least significant byte; the final borrow is the result.
    let mut borrow = 0u16;
    for (x, y) in a.iter().rev().zip(b.iter().rev()) {
        let d = u16::from(*x).wrapping_sub(u16::from(*y)).wrapping_sub(borrow);
        borrow = (d >> 15) & 1;
    }
    borrow == 1
}

/// Constant-time check that all bytes are zero.
pub(crate) fn is_zero(a: &[u8]) -> bool {
    let mut acc = 0u8;
    for x in a {
        acc |= x;
    }
    acc == 0
}

/// Overwrite `buf` with zeros in a way the compiler cannot optimize away.
pub(crate) fn zeroize(buf: &mut [u8]) {
    for b in buf {
        // SAFETY: `b` is a valid, aligned `&mut u8`.
        unsafe { core::ptr::write_volatile(b, 0) };
    }
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lt_works() {
        assert!(lt(&[0, 1], &[0, 2]));
        assert!(!lt(&[0, 2], &[0, 2]));
        assert!(!lt(&[1, 0], &[0, 0xff]));
        assert!(lt(&[0, 0xff], &[1, 0]));
        assert!(lt(&[0], &[1]));
        assert!(!lt(&[0xff], &[0xff]));
    }

    #[test]
    fn eq_works() {
        assert!(eq(b"abc", b"abc"));
        assert!(!eq(b"abc", b"abd"));
        assert!(!eq(b"abc", b"ab"));
    }
}
