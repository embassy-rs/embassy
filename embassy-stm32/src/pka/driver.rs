//! `embassy-crypto` drivers over the PKA: curve arithmetic, ECDH and ECDSA for P-256 and
//! P-384, one per `embassy-crypto-p*-*` feature.
//!
//! - Scalar arithmetic is done on the host with `crypto-bigint`, in constant time. It is
//!   cheaper than an engine setup, and the manual does not mark the engine's modular
//!   operations as side-channel protected. Points go through the engine.
//! - Points are affine or infinity. The engine takes and returns affine, so projective buys nothing.
//! - Point addition uses the engine's complete addition (covers doubling), converted back to
//!   affine via the Jacobian conversion. `P + (-P)` is caught on the host: the engine cannot
//!   represent infinity.
//! - `pka_v1c` has no point addition and no double base ladder. Addition is done on the host
//!   with one field inversion, and linear combinations as separate multiplications.
//! - Scalar multiplication uses the protected operation. The manual marks it, ECDSA sign and
//!   protected exponentiation as side-channel protected. It erases engine RAM when done, so no
//!   copy of a private scalar survives there.
//! - Secret linear combinations: one protected multiplication per scalar, then add.
//! - Variable-time linear combinations: the double base ladder. It errors when the result is
//!   infinity, then we fall back to separate multiplications.
//! - ECDSA sign uses the engine's protected signing with a nonce from the `embassy-crypto` RNG
//!   driver. Verify uses the engine's verification.
//! - One peripheral shared by all drivers, behind a critical-section mutex, clocked only during
//!   an operation.
//! - The engine seeds its RAM from the RNG on every enable, so the RNG must be running: with
//!   `embassy-crypto-rng` the drivers start it, otherwise an [`Rng`](crate::rng::Rng) must be
//!   alive.

#![allow(dead_code)]

use crypto_bigint::modular::{FixedMontyForm, FixedMontyParams};
use crypto_bigint::{NonZero, Uint};
use embassy_crypto::Error as CryptoError;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

#[cfg(not(pka_v1c))]
use super::EccProjectivePoint;
use super::{EccPoint, EcdsaCurveParams, EcdsaPublicKey, EcdsaSignature, Error, Pka};
use crate::mode::Blocking;
use crate::suspend::ResumablePeripheral;

foreach_peripheral!(
    (pka, $inst:ident) => {
        type BlockingPka = Pka<'static, crate::peripherals::$inst, Blocking>;

        static DRIVER: Mutex<CriticalSectionRawMutex, ResumablePeripheral<BlockingPka>> =
            Mutex::new(ResumablePeripheral::new_suspended(unsafe { crate::peripherals::$inst::steal() }));
    };
);

/// Runs `f` with the engine enabled and initialized.
fn with_pka<R>(f: impl FnOnce(&mut BlockingPka) -> R) -> R {
    #[cfg(feature = "embassy-crypto-rng")]
    crate::rng::driver::ensure_running();
    let mut driver = DRIVER.try_lock().expect("the PKA is in use");
    let mut pka = driver.borrow();
    #[cfg(any(pka_v1a, pka_n6))]
    assert!(
        !pka.is_limited(),
        "the PKA of this chip only verifies ECDSA signatures (limited mode)"
    );
    f(&mut pka)
}

/// How many nonces ECDSA signing tries before giving up on the hardware.
const SIGN_ATTEMPTS: usize = 8;

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

/// An affine point of `N`-byte coordinates.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Aff<const N: usize> {
    x: [u8; N],
    y: [u8; N],
}

impl<const N: usize> Aff<N> {
    fn from_ecc_point(p: &EccPoint) -> Self {
        Self {
            x: p.x[..N].try_into().unwrap(),
            y: p.y[..N].try_into().unwrap(),
        }
    }
}

/// The arithmetic driver's point: affine coordinates, or `None` for the point at infinity.
type Pt<const N: usize> = Option<Aff<N>>;

/// Every value crossing the arithmetic driver boundary is valid, so the engine can only fail
/// on a hardware fault, which there is no way to report through the driver traits.
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

/// Whether `(x, y)` is a point on the curve, other than the point at infinity.
fn is_on_curve(pka: &mut BlockingPka, curve: &EcdsaCurveParams, x: &[u8], y: &[u8]) -> bool {
    // The point at infinity has no affine coordinates, and coordinates outside the field are
    // not a valid encoding even where they reduce onto the curve.
    if is_zero(x) && is_zero(y) {
        return false;
    }
    if !less_than(x, curve.p_modulus) || !less_than(y, curve.p_modulus) {
        return false;
    }
    expect(pka.point_check_blocking(curve, x, y))
}

/// `k * P` for `k` in `1..n` and `P` on the curve, which is never the point at infinity.
///
/// The engine reports the point at infinity as an error, which such inputs never produce.
fn try_mul<const N: usize>(
    pka: &mut BlockingPka,
    curve: &EcdsaCurveParams,
    k: &[u8; N],
    p: &Aff<N>,
) -> Result<Aff<N>, Error> {
    let mut r = EccPoint::new(N);
    pka.ecc_mul_blocking(curve, k, &p.x, &p.y, &mut r)?;
    Ok(Aff::from_ecc_point(&r))
}

/// `k * P` for `k` in `1..n`, which is never the point at infinity.
fn mul<const N: usize>(pka: &mut BlockingPka, curve: &EcdsaCurveParams, k: &[u8; N], p: &Aff<N>) -> Aff<N> {
    expect(try_mul(pka, curve, k, p))
}

/// `k * P`, with `k` possibly zero and `P` possibly at infinity.
fn mul_pt<const N: usize>(pka: &mut BlockingPka, curve: &EcdsaCurveParams, k: &[u8; N], p: &Pt<N>) -> Pt<N> {
    if is_zero(k) {
        return None;
    }
    p.map(|p| mul(pka, curve, k, &p))
}

/// `P + Q`, or `None` for the point at infinity, through the engine's complete addition.
#[cfg(not(pka_v1c))]
fn add<const N: usize, const L: usize>(
    pka: &mut BlockingPka,
    curve: &EcdsaCurveParams,
    _field: &FixedMontyParams<L>,
    p: &Aff<N>,
    q: &Aff<N>,
) -> Pt<N> {
    if p.x == q.x && p.y != q.y {
        // Same X and different Y: the points are opposite.
        return None;
    }
    let pp = EccProjectivePoint::from_affine(&p.x, &p.y);
    let qq = EccProjectivePoint::from_affine(&q.x, &q.y);
    let mut sum = EccProjectivePoint::new(N);
    expect(pka.ecc_complete_add_blocking(curve, &pp, &qq, &mut sum));
    let mut r = EccPoint::new(N);
    expect(pka.jacobian_to_affine_blocking(curve.p_modulus, &sum, &mut r));
    Some(Aff::from_ecc_point(&r))
}

/// `P + Q`, or `None` for the point at infinity, on the host: the `pka_v1c` engine has no
/// point addition. The affine formulas, with one field inversion, in constant time.
#[cfg(pka_v1c)]
fn add<const N: usize, const L: usize>(
    _pka: &mut BlockingPka,
    curve: &EcdsaCurveParams,
    field: &FixedMontyParams<L>,
    p: &Aff<N>,
    q: &Aff<N>,
) -> Pt<N> {
    let fe = |v: &[u8; N]| FixedMontyForm::new(&uint(v), field);
    let (x1, y1, x2, y2) = (fe(&p.x), fe(&p.y), fe(&q.x), fe(&q.y));
    let (num, den) = if p.x == q.x {
        if p.y != q.y || is_zero(&p.y) {
            // Opposite points, or a point of order two, which a prime-order curve does not have.
            return None;
        }
        // Doubling: λ = (3x² + a) / 2y.
        let mut a = FixedMontyForm::new(&Uint::from_be_slice(curve.a_coefficient), field);
        if curve.a_coefficient_sign != 0 {
            a = a.neg();
        }
        let xx = x1.square();
        (xx.double().add(&xx).add(&a), y1.double())
    } else {
        // λ = (y2 - y1) / (x2 - x1).
        (y2.sub(&y1), x2.sub(&x1))
    };
    // `den` is nonzero here, so the inverse exists.
    let lambda = num.mul(&den.invert().unwrap_or(FixedMontyForm::zero(field)));
    // x3 = λ² - x1 - x2, y3 = λ(x1 - x3) - y1.
    let x3 = lambda.square().sub(&x1).sub(&x2);
    let y3 = lambda.mul(&x1.sub(&x3)).sub(&y1);
    Some(Aff {
        x: bytes(&x3.retrieve()),
        y: bytes(&y3.retrieve()),
    })
}

/// `P + Q`, either possibly at infinity.
fn add_pt<const N: usize, const L: usize>(
    pka: &mut BlockingPka,
    curve: &EcdsaCurveParams,
    field: &FixedMontyParams<L>,
    p: &Pt<N>,
    q: &Pt<N>,
) -> Pt<N> {
    match (p, q) {
        (None, q) => *q,
        (p, None) => *p,
        (Some(p), Some(q)) => add(pka, curve, field, p, q),
    }
}

/// `a*P + b*Q` for public values, through the engine's double base ladder.
#[cfg(not(pka_v1c))]
fn lincomb<const N: usize, const L: usize>(
    pka: &mut BlockingPka,
    curve: &EcdsaCurveParams,
    field: &FixedMontyParams<L>,
    a: &[u8; N],
    p: &Pt<N>,
    b: &[u8; N],
    q: &Pt<N>,
) -> Pt<N> {
    let (Some(pa), Some(qa)) = (p, q) else {
        return lincomb_separate(pka, curve, field, a, p, b, q);
    };
    if is_zero(a) || is_zero(b) {
        return lincomb_separate(pka, curve, field, a, p, b, q);
    }
    let pp = EccProjectivePoint::from_affine(&pa.x, &pa.y);
    let qq = EccProjectivePoint::from_affine(&qa.x, &qa.y);
    let mut r = EccPoint::new(N);
    match pka.double_base_ladder_blocking(curve, a, &pp, b, &qq, &mut r) {
        Ok(()) => Some(Aff::from_ecc_point(&r)),
        // The ladder reports an error when the result is the point at infinity, which it has
        // no way to express. Separate multiplications handle it, and anything else the ladder
        // may object to.
        Err(_) => lincomb_separate(pka, curve, field, a, p, b, q),
    }
}

/// `a*P + b*Q` for public values: the `pka_v1c` engine has no double base ladder.
#[cfg(pka_v1c)]
fn lincomb<const N: usize, const L: usize>(
    pka: &mut BlockingPka,
    curve: &EcdsaCurveParams,
    field: &FixedMontyParams<L>,
    a: &[u8; N],
    p: &Pt<N>,
    b: &[u8; N],
    q: &Pt<N>,
) -> Pt<N> {
    lincomb_separate(pka, curve, field, a, p, b, q)
}

/// `a*P + b*Q` as two multiplications and an addition, which handle every special case.
fn lincomb_separate<const N: usize, const L: usize>(
    pka: &mut BlockingPka,
    curve: &EcdsaCurveParams,
    field: &FixedMontyParams<L>,
    a: &[u8; N],
    p: &Pt<N>,
    b: &[u8; N],
    q: &Pt<N>,
) -> Pt<N> {
    let ap = mul_pt(pka, curve, a, p);
    let bq = mul_pt(pka, curve, b, q);
    add_pt(pka, curve, field, &ap, &bq)
}

/// The public key `k * G` of a private scalar, checked to be in `1..n`.
fn public_key<const N: usize>(
    curve: &EcdsaCurveParams,
    generator: &Aff<N>,
    k: &[u8; N],
) -> Result<Aff<N>, CryptoError> {
    if !ct_in_range(k, curve.order) {
        return Err(CryptoError::InvalidKey);
    }
    with_pka(|pka| try_mul(pka, curve, k, generator)).map_err(|_| CryptoError::HardwareError)
}

/// The X coordinate of `k * peer`, for a private scalar in `1..n` and an untrusted peer point.
fn shared_secret<const N: usize>(curve: &EcdsaCurveParams, k: &[u8; N], peer: &Aff<N>) -> Result<[u8; N], CryptoError> {
    if !ct_in_range(k, curve.order) {
        return Err(CryptoError::InvalidKey);
    }
    with_pka(|pka| {
        if !is_on_curve(pka, curve, &peer.x, &peer.y) {
            return Err(CryptoError::InvalidKey);
        }
        let r = try_mul(pka, curve, k, peer).map_err(|_| CryptoError::HardwareError)?;
        Ok(r.x)
    })
}

/// Signs a digest with a private scalar in `1..n`: `(r, s)`, low-S normalized.
fn sign<const N: usize, const L: usize>(
    curve: &EcdsaCurveParams,
    order: &NonZero<Uint<L>>,
    half_order: &[u8; N],
    k: &[u8; N],
    digest: &[u8; N],
) -> Result<([u8; N], [u8; N]), CryptoError> {
    if !ct_in_range(k, curve.order) {
        return Err(CryptoError::InvalidKey);
    }
    let mut nonce = [0u8; N];
    let mut r = [0u8; N];
    let mut s = [0u8; N];
    let result = with_pka(|pka| {
        for _ in 0..SIGN_ATTEMPTS {
            // Rejection sampling into `1..n`.
            loop {
                embassy_crypto::rng_fill_bytes(&mut nonce);
                if ct_in_range(&nonce, curve.order) {
                    break;
                }
            }
            match pka.ecdsa_sign_blocking(curve, k, &nonce, digest, &mut r, &mut s) {
                Ok(()) => return Ok(()),
                // The engine reports a zero `r` or `s`, which another nonce fixes, the same
                // way as an operation error.
                Err(Error::OperationError) => continue,
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
    curve: &EcdsaCurveParams,
    q: &Aff<N>,
    digest: &[u8; N],
    r: &[u8; N],
    s: &[u8; N],
) -> Result<(), CryptoError> {
    with_pka(|pka| {
        if !is_on_curve(pka, curve, &q.x, &q.y) {
            return Err(CryptoError::InvalidKey);
        }
        // A zero or out-of-range component is invalid, and the engine is not required to
        // notice.
        if !ct_in_range(r, curve.order) || !ct_in_range(s, curve.order) {
            return Err(CryptoError::InvalidSignature);
        }
        let public_key = EcdsaPublicKey { x: &q.x, y: &q.y };
        let signature = EcdsaSignature { r, s };
        match pka.ecdsa_verify_blocking(curve, &public_key, &signature, digest) {
            Ok(true) => Ok(()),
            _ => Err(CryptoError::InvalidSignature),
        }
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
        mod $curve_mod {
            #[allow(unused_imports)]
            use embassy_crypto::driver::{$point, $scalar, $signature};

            use super::*;

            const N: usize = $n;
            const L: usize = N / crypto_bigint::Limb::BYTES;
            const CURVE: EcdsaCurveParams = $curve;
            static ORDER: FixedMontyParams<L> = monty(CURVE.order);
            static FIELD: FixedMontyParams<L> = monty(CURVE.p_modulus);
            /// `n / 2`, the largest low `s`.
            static HALF_ORDER: [u8; N] = half(CURVE.order);

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
                    x: CURVE.generator_x.try_into().unwrap(),
                    y: CURVE.generator_y.try_into().unwrap(),
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
                        let on_curve = with_pka(|pka| is_on_curve(pka, &CURVE, &p.x, &p.y));
                        on_curve.then_some(Some(from_point(p)))
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
                        with_pka(|pka| add_pt(pka, &CURVE, &FIELD, p, q))
                    }

                    fn point_mul(k: &$scalar, p: &Pt<N>) -> Pt<N> {
                        with_pka(|pka| mul_pt(pka, &CURVE, &k.0, p))
                    }

                    fn point_mul_base(k: &$scalar) -> Pt<N> {
                        with_pka(|pka| mul_pt(pka, &CURVE, &k.0, &Some(generator())))
                    }

                    fn point_lincomb(a: &$scalar, p: &Pt<N>, b: &$scalar, q: &Pt<N>) -> Pt<N> {
                        with_pka(|pka| lincomb_separate(pka, &CURVE, &FIELD, &a.0, p, &b.0, q))
                    }

                    fn point_lincomb_vartime(a: &$scalar, p: &Pt<N>, b: &$scalar, q: &Pt<N>) -> Pt<N> {
                        with_pka(|pka| lincomb(pka, &CURVE, &FIELD, &a.0, p, &b.0, q))
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
                        public_key(&CURVE, &generator(), &k.0).map(|p| to_point(&p))
                    }

                    fn shared_secret(k: &$scalar, peer: &$point) -> Result<[u8; N], CryptoError> {
                        shared_secret(&CURVE, &k.0, &from_point(peer))
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
                        public_key(&CURVE, &generator(), &k.0).map(|p| to_point(&p))
                    }

                    fn sign(k: &$scalar, digest: &[u8; N]) -> Result<$signature, CryptoError> {
                        let (r, s) = sign(&CURVE, ORDER.modulus().as_nz_ref(), &HALF_ORDER, &k.0, digest)?;
                        Ok($signature {
                            r: $scalar(r),
                            s: $scalar(s),
                        })
                    }

                    fn verify(q: &$point, digest: &[u8; N], sig: &$signature) -> Result<(), CryptoError> {
                        verify(&CURVE, &from_point(q), digest, &sig.r.0, &sig.s.0)
                    }
                }

                embassy_crypto::$ecdsa_register!(Driver);
            }
        }
    };
}

impl_curve!(
    p256,
    EcdsaCurveParams::nist_p256(),
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
    EcdsaCurveParams::nist_p384(),
    48,
    scalar = P384Scalar,
    point = P384Point,
    signature = P384Signature,
    arith = ("embassy-crypto-p384-arith", P384Arith, p384_arith_impl),
    ecdh = ("embassy-crypto-p384-ecdh", P384Ecdh, p384_ecdh_impl),
    ecdsa = ("embassy-crypto-p384-ecdsa", P384Ecdsa, p384_ecdsa_impl),
);
