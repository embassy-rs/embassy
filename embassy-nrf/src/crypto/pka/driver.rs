//! `embassy-crypto` drivers over the PKA: curve arithmetic, ECDH and ECDSA for P-256 and
//! P-384, one per `embassy-crypto-p*-*` feature.
//!
//! Scalar arithmetic runs on the host with `crypto-bigint`, in constant time. It is cheaper
//! than an engine setup, and CRACEN has no instruction for it.
//!
//! Point arithmetic goes through the engine. Both engines take and return affine points, so
//! the driver's point type is an affine point or the point at infinity. Keeping a projective
//! form between operations would gain nothing.
//!
//! Neither engine's point addition handles `P + P` or `P + (-P)`. Every addition here
//! compares the points first, and doubles or returns infinity instead.
//!
//! A linear combination of secret scalars is done one multiplication at a time, with the
//! engine countermeasures, and the results are added. Sharing the doublings between terms
//! would need a ladder of its own on the CryptoCell, and is not possible on CRACEN. The
//! variable-time combination does share them on the CryptoCell, through the Strauss ladder
//! of signature verification.
//!
//! ECDSA signing runs on the engine, with a nonce drawn from the `embassy-crypto` RNG driver
//! by rejection sampling. Verification runs on the engine too.
//!
//! The features that register these drivers remove the `CRYPTO_PKA` singleton, so the engine
//! has one owner: these drivers. They lock it for every engine operation and panic if it is
//! already locked. A [`Pka`](super::Pka) driver is created per operation, so the engine is
//! powered only while one runs.

// Which helpers are used depends on the enabled feature set.
#![allow(dead_code)]

#[cfg(feature = "_cracen")]
use core::cell::Cell;

#[cfg(any(feature = "embassy-crypto-p256-arith", feature = "embassy-crypto-p384-arith"))]
use crypto_bigint::modular::FixedMontyForm;
use crypto_bigint::modular::FixedMontyParams;
use crypto_bigint::{NonZero, Uint};
use embassy_crypto::Error as CryptoError;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::PanicRawMutex;

use super::{Curve, Error, Pka, curve, ecc_mul, ecdsa_verify, hw, point_check};
use crate::mode::Blocking;

static LOCK: Mutex<PanicRawMutex, ()> = Mutex::new(());

/// The engine's microcode, registered with [`super::set_microcode`].
#[cfg(feature = "_cracen")]
pub(super) static MICROCODE: Mutex<PanicRawMutex, Cell<Option<&'static [u32]>>> = Mutex::new(Cell::new(None));

/// How many nonces ECDSA signing tries before giving up on the hardware.
const SIGN_ATTEMPTS: usize = 8;

/// Runs `f` with the engine locked and powered.
fn with_pka<R>(f: impl FnOnce(&mut Pka<'static, Blocking>) -> R) -> R {
    LOCK.lock(|_| {
        #[cfg(feature = "_cryptocell")]
        let mut pka = Pka::new_inner();
        #[cfg(feature = "_cracen")]
        let mut pka = {
            let microcode = MICROCODE
                .lock(|m| m.get())
                .expect("the embassy-crypto curve drivers need the engine's microcode, see `pka::set_microcode`");
            Pka::new_inner(microcode)
        };
        f(&mut pka)
    })
}

/// An affine point of `N`-byte coordinates.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Aff<const N: usize> {
    x: [u8; N],
    y: [u8; N],
}

impl<const N: usize> Aff<N> {
    const ZERO: Self = Self { x: [0; N], y: [0; N] };
}

/// The driver's point: affine coordinates, or `None` for the point at infinity.
type Pt<const N: usize> = Option<Aff<N>>;

/// Unwraps an engine result.
///
/// Every value crossing the arithmetic driver boundary is valid, so the engine can only fail
/// on a hardware fault. The arithmetic trait has no way to report that.
fn expect<T>(r: Result<T, Error>) -> T {
    match r {
        Ok(v) => v,
        Err(e) => panic!("PKA error: {:?}", e),
    }
}

fn is_zero(v: &[u8]) -> bool {
    v.iter().all(|&b| b == 0)
}

/// Returns whether `v < limit`, both big-endian and of the same length.
fn less_than(v: &[u8], limit: &[u8]) -> bool {
    for (a, b) in v.iter().zip(limit) {
        if a != b {
            return a < b;
        }
    }
    false
}

/// Returns whether `v` is zero, in constant time.
fn ct_is_zero(v: &[u8]) -> bool {
    v.iter().fold(0u8, |acc, &b| acc | b) == 0
}

/// Returns whether `v < limit`, both big-endian and of the same length, in constant time.
fn ct_less_than(v: &[u8], limit: &[u8]) -> bool {
    // A borrow out of the most significant byte of `v - limit` means `v < limit`.
    let mut borrow = 0u16;
    for (&a, &b) in v.iter().rev().zip(limit.iter().rev()) {
        let d = (a as u16).wrapping_sub(b as u16).wrapping_sub(borrow);
        borrow = (d >> 15) & 1;
    }
    borrow == 1
}

/// Returns whether `0 < k < n`, in constant time.
fn ct_in_range(k: &[u8], n: &[u8]) -> bool {
    // Both checks run to completion whatever the value.
    let nonzero = !ct_is_zero(k);
    let below = ct_less_than(k, n);
    nonzero & below
}

/// Overwrites a secret.
fn zeroize(buf: &mut [u8]) {
    for b in buf {
        unsafe { core::ptr::write_volatile(b, 0) };
    }
}

/// Montgomery constants of an odd big-endian modulus of exactly `L` limbs.
const fn monty<const L: usize>(n_be: &[u8]) -> FixedMontyParams<L> {
    FixedMontyParams::new_vartime(Uint::from_be_slice(n_be).to_odd().expect_copied("odd modulus"))
}

/// Big-endian bytes to an integer, `N == L * Limb::BYTES`.
fn uint<const N: usize, const L: usize>(v: &[u8; N]) -> Uint<L> {
    Uint::from_be_slice(v)
}

/// An integer to big-endian bytes, `N == L * Limb::BYTES`.
fn bytes<const N: usize, const L: usize>(v: &Uint<L>) -> [u8; N] {
    let mut out = [0u8; N];
    out.copy_from_slice(&v.to_be_bytes());
    out
}

/// `k * P` for `k` in `1..n` and `P` on the curve, which is never the point at infinity.
///
/// The engine must already be locked.
fn try_mul<const N: usize>(curve: &Curve, k: &[u8; N], p: &Aff<N>, blind: bool) -> Result<Aff<N>, Error> {
    let mut r = Aff::ZERO;
    ecc_mul(curve, k, &p.x, &p.y, &mut r.x, &mut r.y, blind)?;
    Ok(r)
}

/// `k * P` for `k` in `1..n`, which is never the point at infinity.
fn mul<const N: usize>(curve: &Curve, k: &[u8; N], p: &Aff<N>, blind: bool) -> Aff<N> {
    with_pka(|_| expect(try_mul(curve, k, p, blind)))
}

/// `k * P`, with `k` possibly zero and `P` possibly at infinity.
fn mul_pt<const N: usize>(curve: &Curve, k: &[u8; N], p: &Pt<N>, blind: bool) -> Pt<N> {
    if is_zero(k) {
        return None;
    }
    p.map(|p| mul(curve, k, &p, blind))
}

/// `P + Q`, or `None` for the point at infinity.
fn add<const N: usize>(curve: &Curve, p: &Aff<N>, q: &Aff<N>) -> Pt<N> {
    let mut r = Aff::ZERO;
    if p.x == q.x {
        // Same X: the points are equal or opposite.
        if p.y != q.y {
            return None;
        }
        with_pka(|_| expect(hw::ecc_double(curve, &p.x, &p.y, &mut r.x, &mut r.y)));
    } else {
        with_pka(|_| expect(hw::ecc_add(curve, &p.x, &p.y, &q.x, &q.y, &mut r.x, &mut r.y)));
    }
    Some(r)
}

/// `P + Q`, either possibly at infinity.
fn add_pt<const N: usize>(curve: &Curve, p: &Pt<N>, q: &Pt<N>) -> Pt<N> {
    match (p, q) {
        (None, q) => *q,
        (p, None) => *p,
        (Some(p), Some(q)) => add(curve, p, q),
    }
}

/// `a*P + b*Q` for public values, sharing the doublings between the two terms.
#[cfg(feature = "_cryptocell")]
fn lincomb<const N: usize, const L: usize>(
    curve: &Curve,
    order: &NonZero<Uint<L>>,
    a: &[u8; N],
    p: &Aff<N>,
    b: &[u8; N],
    q: &Aff<N>,
) -> Pt<N> {
    match (is_zero(a), is_zero(b)) {
        (true, true) => return None,
        (true, false) => return Some(mul(curve, b, q, false)),
        (false, true) => return Some(mul(curve, a, p, false)),
        (false, false) => {}
    }
    if p.x == q.x {
        // a*P + b*(±P) = (a ± b)*P.
        let (a, b) = (uint(a), uint(b));
        let k: [u8; N] = bytes(&if p.y == q.y {
            a.add_mod(&b, order)
        } else {
            a.sub_mod(&b, order)
        });
        return if is_zero(&k) {
            None
        } else {
            Some(mul(curve, &k, p, false))
        };
    }
    let mut r = Aff::ZERO;
    let done = with_pka(|_| expect(hw::ecc_lincomb(curve, a, &p.x, &p.y, b, &q.x, &q.y, &mut r.x, &mut r.y)));
    if done {
        Some(r)
    } else {
        // The ladder ran into the point at infinity: the terms are related in a way that
        // separate multiplications handle.
        add(curve, &mul(curve, a, p, false), &mul(curve, b, q, false))
    }
}

/// The public key `k * G` of a private scalar, checked to be in `1..n`.
fn public_key<const N: usize>(curve: &Curve, generator: &Aff<N>, k: &[u8; N]) -> Result<Aff<N>, CryptoError> {
    if !ct_in_range(k, curve.n) {
        return Err(CryptoError::InvalidKey);
    }
    with_pka(|_| try_mul(curve, k, generator, true)).map_err(|_| CryptoError::HardwareError)
}

/// The X coordinate of `k * peer`, for a private scalar in `1..n` and an untrusted peer point.
fn shared_secret<const N: usize>(curve: &Curve, k: &[u8; N], peer: &Aff<N>) -> Result<[u8; N], CryptoError> {
    if !ct_in_range(k, curve.n) {
        return Err(CryptoError::InvalidKey);
    }
    with_pka(|_| {
        point_check(curve, &peer.x, &peer.y).map_err(|_| CryptoError::InvalidKey)?;
        let mut r = try_mul(curve, k, peer, true).map_err(|_| CryptoError::HardwareError)?;
        zeroize(&mut r.y);
        Ok(r.x)
    })
}

/// Signs a digest with a private scalar in `1..n`: `(r, s)`, low-S normalized.
///
/// The digest is as long as the curve, so it is already the truncated hash of FIPS 186-4.
fn sign<const N: usize, const L: usize>(
    curve: &Curve,
    order: &NonZero<Uint<L>>,
    half_order: &[u8; N],
    k: &[u8; N],
    digest: &[u8; N],
) -> Result<([u8; N], [u8; N]), CryptoError> {
    if !ct_in_range(k, curve.n) {
        return Err(CryptoError::InvalidKey);
    }
    let mut nonce = [0u8; N];
    let mut r = [0u8; N];
    let mut s = [0u8; N];
    let result = with_pka(|_| {
        for _ in 0..SIGN_ATTEMPTS {
            // Rejection sampling into `1..n`.
            loop {
                embassy_crypto::rng_fill_bytes(&mut nonce);
                if ct_in_range(&nonce, curve.n) {
                    break;
                }
            }
            match hw::ecdsa_sign(curve, k, &nonce, digest, &mut r, &mut s) {
                Ok(()) => return Ok(()),
                // The nonce gave an unusable signature, which another one fixes.
                Err(Error::RetryWithNewK) => continue,
                Err(_) => return Err(CryptoError::HardwareError),
            }
        }
        Err(CryptoError::HardwareError)
    });
    zeroize(&mut nonce);
    result?;
    // Low-S normalization: `s` is public, so its comparison need not be constant-time.
    if !less_than(&s, half_order) && s != *half_order {
        s = bytes(&uint(&s).neg_mod(order));
    }
    Ok((r, s))
}

/// Verifies the signature `(r, s)` of a digest with an untrusted public key.
fn verify<const N: usize>(
    curve: &Curve,
    q: &Aff<N>,
    digest: &[u8; N],
    r: &[u8; N],
    s: &[u8; N],
) -> Result<(), CryptoError> {
    with_pka(|_| {
        point_check(curve, &q.x, &q.y).map_err(|_| CryptoError::InvalidKey)?;
        // A zero or out-of-range component is invalid, and the engine is not required to
        // notice.
        if !ct_in_range(r, curve.n) || !ct_in_range(s, curve.n) {
            return Err(CryptoError::InvalidSignature);
        }
        ecdsa_verify(curve, &q.x, &q.y, r, s, digest).map_err(|_| CryptoError::InvalidSignature)
    })
}

macro_rules! impl_curve {
    (
        $curve_mod:ident, $curve:expr, $n:literal,
        scalar = $scalar:ident, point = $point:ident, signature = $signature:ident,
        arith = ($arith_feature:literal, $arith_trait:ident, $arith_register:ident),
        ecdh = ($ecdh_feature:literal, $ecdh_trait:ident, $ecdh_register:ident),
        ecdsa = ($ecdsa_feature:literal, $ecdsa_trait:ident, $ecdsa_register:ident),
    ) => {
        #[cfg(any(feature = $arith_feature, feature = $ecdh_feature, feature = $ecdsa_feature))]
        mod $curve_mod {
            #[allow(unused_imports)]
            use embassy_crypto::driver::{$point, $scalar, $signature};

            use super::*;

            const N: usize = $n;
            const L: usize = N / crypto_bigint::Limb::BYTES;
            static CURVE: &Curve = &$curve;
            static ORDER: FixedMontyParams<L> = monty($curve.n);
            static FIELD: FixedMontyParams<L> = monty($curve.p);
            /// `n / 2`, the largest low `s`.
            static HALF_ORDER: [u8; N] = half($curve.n);

            const fn half(v: &[u8]) -> [u8; N] {
                let mut out = [0u8; N];
                let mut carry = 0u8;
                let mut i = 0;
                while i < N {
                    out[i] = (v[i] >> 1) | (carry << 7);
                    carry = v[i] & 1;
                    i += 1;
                }
                out
            }

            fn generator() -> Aff<N> {
                Aff {
                    x: CURVE.gx.try_into().unwrap(),
                    y: CURVE.gy.try_into().unwrap(),
                }
            }

            fn to_point(p: &Aff<N>) -> $point {
                $point { x: p.x, y: p.y }
            }

            fn from_point(p: &$point) -> Aff<N> {
                Aff { x: p.x, y: p.y }
            }

            #[cfg(feature = $arith_feature)]
            mod arith {
                use super::*;

                struct Driver;

                impl embassy_crypto::driver::$arith_trait for Driver {
                    type Point = Pt<N>;

                    fn scalar_add(a: &$scalar, b: &$scalar) -> $scalar {
                        $scalar(bytes(
                            &uint(&a.0).add_mod(&uint(&b.0), ORDER.modulus().as_nz_ref()),
                        ))
                    }

                    fn scalar_sub(a: &$scalar, b: &$scalar) -> $scalar {
                        $scalar(bytes(
                            &uint(&a.0).sub_mod(&uint(&b.0), ORDER.modulus().as_nz_ref()),
                        ))
                    }

                    fn scalar_mul(a: &$scalar, b: &$scalar) -> $scalar {
                        // mont(a, R²) = a·R, and mont(a·R, b) = a·b: two Montgomery products.
                        let ar = FixedMontyForm::new(&uint(&a.0), &ORDER);
                        let b = FixedMontyForm::from_montgomery(uint(&b.0), &ORDER);
                        $scalar(bytes(ar.mul(&b).as_montgomery()))
                    }

                    fn scalar_invert(a: &$scalar) -> $scalar {
                        // A constant-time select, not a branch on the input. Zero for zero.
                        $scalar(bytes(
                            &uint(&a.0).invert_odd_mod(ORDER.modulus()).unwrap_or(Uint::ZERO),
                        ))
                    }

                    fn point_identity() -> Pt<N> {
                        None
                    }

                    fn point_from_affine(p: &$point) -> Option<Pt<N>> {
                        with_pka(|_| point_check(CURVE, &p.x, &p.y)).ok()?;
                        Some(Some(from_point(p)))
                    }

                    fn point_from_affine_unchecked(p: &$point) -> Pt<N> {
                        Some(from_point(p))
                    }

                    fn point_to_affine(p: &Pt<N>) -> Option<$point> {
                        p.as_ref().map(to_point)
                    }

                    fn point_is_identity(p: &Pt<N>) -> bool {
                        p.is_none()
                    }

                    fn point_neg(p: &Pt<N>) -> Pt<N> {
                        // A valid point of a prime-order curve never has y = 0.
                        p.map(|p| Aff {
                            x: p.x,
                            y: bytes(&uint(&p.y).neg_mod(FIELD.modulus().as_nz_ref())),
                        })
                    }

                    fn point_add(p: &Pt<N>, q: &Pt<N>) -> Pt<N> {
                        add_pt(CURVE, p, q)
                    }

                    fn point_mul(k: &$scalar, p: &Pt<N>) -> Pt<N> {
                        mul_pt(CURVE, &k.0, p, true)
                    }

                    fn point_mul_base(k: &$scalar) -> Pt<N> {
                        mul_pt(CURVE, &k.0, &Some(generator()), true)
                    }

                    fn point_lincomb(a: &$scalar, p: &Pt<N>, b: &$scalar, q: &Pt<N>) -> Pt<N> {
                        add_pt(
                            CURVE,
                            &mul_pt(CURVE, &a.0, p, true),
                            &mul_pt(CURVE, &b.0, q, true),
                        )
                    }

                    fn point_lincomb_vartime(a: &$scalar, p: &Pt<N>, b: &$scalar, q: &Pt<N>) -> Pt<N> {
                        #[cfg(feature = "_cryptocell")]
                        if let (Some(p), Some(q)) = (p, q) {
                            return lincomb(CURVE, ORDER.modulus().as_nz_ref(), &a.0, p, &b.0, q);
                        }
                        add_pt(
                            CURVE,
                            &mul_pt(CURVE, &a.0, p, false),
                            &mul_pt(CURVE, &b.0, q, false),
                        )
                    }
                }

                embassy_crypto::$arith_register!(Driver);
            }

            #[cfg(feature = $ecdh_feature)]
            mod ecdh {
                use super::*;

                struct Driver;

                impl embassy_crypto::driver::$ecdh_trait for Driver {
                    fn public_key(k: &$scalar) -> Result<$point, CryptoError> {
                        public_key(CURVE, &generator(), &k.0).map(|p| to_point(&p))
                    }

                    fn shared_secret(k: &$scalar, peer: &$point) -> Result<[u8; N], CryptoError> {
                        shared_secret(CURVE, &k.0, &from_point(peer))
                    }
                }

                embassy_crypto::$ecdh_register!(Driver);
            }

            #[cfg(feature = $ecdsa_feature)]
            mod ecdsa {
                use super::*;

                struct Driver;

                impl embassy_crypto::driver::$ecdsa_trait for Driver {
                    fn public_key(k: &$scalar) -> Result<$point, CryptoError> {
                        public_key(CURVE, &generator(), &k.0).map(|p| to_point(&p))
                    }

                    fn sign(k: &$scalar, digest: &[u8; N]) -> Result<$signature, CryptoError> {
                        let (r, s) = sign(CURVE, ORDER.modulus().as_nz_ref(), &HALF_ORDER, &k.0, digest)?;
                        Ok($signature {
                            r: $scalar(r),
                            s: $scalar(s),
                        })
                    }

                    fn verify(q: &$point, digest: &[u8; N], sig: &$signature) -> Result<(), CryptoError> {
                        verify(CURVE, &from_point(q), digest, &sig.r.0, &sig.s.0)
                    }
                }

                embassy_crypto::$ecdsa_register!(Driver);
            }
        }
    };
}

impl_curve!(
    p256,
    curve::NIST_P256,
    32,
    scalar = P256Scalar,
    point = P256Point,
    signature = P256Signature,
    arith = ("embassy-crypto-p256-arith", P256Arith, p256_arith_impl),
    ecdh = ("embassy-crypto-p256-ecdh", P256Ecdh, p256_ecdh_impl),
    ecdsa = ("embassy-crypto-p256-ecdsa", P256Ecdsa, p256_ecdsa_impl),
);
impl_curve!(
    p384,
    curve::NIST_P384,
    48,
    scalar = P384Scalar,
    point = P384Point,
    signature = P384Signature,
    arith = ("embassy-crypto-p384-arith", P384Arith, p384_arith_impl),
    ecdh = ("embassy-crypto-p384-ecdh", P384Ecdh, p384_ecdh_impl),
    ecdsa = ("embassy-crypto-p384-ecdsa", P384Ecdsa, p384_ecdsa_impl),
);
