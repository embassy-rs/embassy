//! Public key accelerator.
//!
//! The driver runs on the PKA engine of the CryptoCell (nRF52840, nRF91, nRF5340) or the
//! public key engine of CRACEN (nRF54L). It provides:
//!
//! - **ECDSA** signature generation and verification.
//! - **ECDH** and public key derivation, through scalar multiplication of a curve point.
//! - **RSA**, through modular exponentiation, with or without the Chinese remainder theorem.
//!
//! Curve points, scalars, moduli and signatures are big-endian byte slices, as in SEC 1 and
//! PKCS#1.
//!
//! - Every value of a curve operation must be exactly [`Curve::size`] bytes long. Pad it
//!   with zeros on the left if needed.
//! - Every value of an RSA operation must be at most as long as the modulus.
//!
//! # Example
//!
//! ```no_run
//! use embassy_nrf::crypto::pka::{Pka, PointMut, curve};
//!
//! # let p: embassy_nrf::Peripherals = todo!();
//! # let private_key = [0u8; 32];
//! // On CRACEN (nRF54L) the constructor also takes the engine microcode.
//! let mut pka = Pka::new_blocking(p.CRYPTO_PKA);
//!
//! // Derive the public key of a private key.
//! let mut x = [0; 32];
//! let mut y = [0; 32];
//! pka.blocking_public_key(&curve::NIST_P256, &private_key, PointMut { x: &mut x, y: &mut y })?;
//! # Ok::<(), embassy_nrf::crypto::pka::Error>(())
//! ```

use core::marker::PhantomData;

use crate::mode::Mode;

pub mod curve;
#[cfg(feature = "_embassy-crypto-pka")]
mod driver;

#[cfg_attr(feature = "_cryptocell", path = "cryptocell/mod.rs")]
#[cfg_attr(feature = "_cracen", path = "cracen.rs")]
mod hw;

/// Largest curve size in bytes that the driver accepts. This fits NIST P-521.
pub const MAX_CURVE_LEN: usize = 66;

/// Largest modulus in bytes that the driver accepts. This fits RSA-4096.
///
/// Not every operation works with a modulus this large. Operations that hold more values at
/// once have a lower limit, and return [`Error::InvalidModulus`] above it.
pub const MAX_MODULUS_LEN: usize = 512;

/// PKA error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// A value has the wrong length for the curve or modulus it belongs to.
    InvalidLength,
    /// The modulus is even, or too large for the operation.
    InvalidModulus,
    /// A scalar is zero or not smaller than the order of the curve.
    InvalidScalar,
    /// A point is not on the curve, or is the point at infinity.
    InvalidPoint,
    /// The signature does not match the message and public key.
    InvalidSignature,
    /// The value has no inverse modulo the modulus.
    NotInvertible,
    /// The nonce produced an unusable signature. Draw a new one and retry.
    RetryWithNewK,
    /// The hardware reported an error.
    Hardware,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::InvalidLength => "invalid value length",
            Self::InvalidModulus => "invalid modulus",
            Self::InvalidScalar => "invalid scalar",
            Self::InvalidPoint => "invalid curve point",
            Self::InvalidSignature => "invalid signature",
            Self::NotInvertible => "value is not invertible",
            Self::RetryWithNewK => "retry with a new nonce",
            Self::Hardware => "hardware error",
        };
        f.write_str(s)
    }
}

impl core::error::Error for Error {}

/// A curve whose parameters the hardware holds itself.
#[cfg(feature = "_cracen")]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(crate) enum Predefined {
    /// NIST P-192.
    P192,
    /// NIST P-256.
    P256,
    /// NIST P-384.
    P384,
    /// NIST P-521.
    P521,
}

/// Domain parameters of a short Weierstrass curve `y² = x³ + a·x + b` over a prime field.
///
/// The [`curve`] module has the common curves. Use [`Curve::new`] for any other.
pub struct Curve {
    pub(crate) p: &'static [u8],
    pub(crate) n: &'static [u8],
    pub(crate) gx: &'static [u8],
    pub(crate) gy: &'static [u8],
    pub(crate) a: &'static [u8],
    pub(crate) b: &'static [u8],
    pub(crate) mod_bits: u16,
    pub(crate) order_bits: u16,
    #[cfg(feature = "_cracen")]
    pub(crate) predefined: Option<Predefined>,
}

impl Curve {
    /// Builds a curve from its domain parameters.
    ///
    /// All values are big-endian and must have the same length:
    /// - `p`: the field modulus.
    /// - `a`, `b`: the curve coefficients.
    /// - `gx`, `gy`: the coordinates of the generator.
    /// - `n`: the order of the generator. Pad it with zeros on the left if it is shorter
    ///   than the field.
    ///
    /// # Panics
    ///
    /// Panics if the values do not all have the same length, or if that length is larger than
    /// [`MAX_CURVE_LEN`].
    pub const fn new(
        p: &'static [u8],
        a: &'static [u8],
        b: &'static [u8],
        gx: &'static [u8],
        gy: &'static [u8],
        n: &'static [u8],
    ) -> Self {
        let len = p.len();
        core::assert!(len <= MAX_CURVE_LEN, "curve is too large");
        core::assert!(
            a.len() == len && b.len() == len && gx.len() == len && gy.len() == len && n.len() == len,
            "curve parameters must all have the same length"
        );
        Self {
            mod_bits: bit_len(p),
            order_bits: bit_len(n),
            p,
            n,
            gx,
            gy,
            a,
            b,
            #[cfg(feature = "_cracen")]
            predefined: None,
        }
    }

    /// Length in bytes of a coordinate, a scalar or a signature component on this curve.
    pub const fn size(&self) -> usize {
        self.p.len()
    }

    /// Length in bits of the order of the generator.
    pub const fn order_bits(&self) -> u32 {
        self.order_bits as u32
    }

    /// Length in bits of the field modulus.
    pub const fn field_bits(&self) -> u32 {
        self.mod_bits as u32
    }

    /// The generator point.
    pub const fn generator(&self) -> Point<'static> {
        Point { x: self.gx, y: self.gy }
    }

    /// The order of the generator, big-endian and [`Curve::size`] bytes long.
    ///
    /// Scalars, private keys and signature components must be nonzero and smaller than this.
    pub const fn n_bytes(&self) -> &'static [u8] {
        self.n
    }
}

/// Length in bits of a big-endian value.
pub(crate) const fn bit_len(v: &[u8]) -> u16 {
    let mut i = 0;
    while i < v.len() {
        if v[i] != 0 {
            return ((v.len() - i - 1) * 8 + (8 - v[i].leading_zeros() as usize)) as u16;
        }
        i += 1;
    }
    0
}

/// A curve point in affine coordinates.
///
/// Both coordinates are big-endian and [`Curve::size`] bytes long.
#[derive(Copy, Clone)]
pub struct Point<'a> {
    /// X coordinate.
    pub x: &'a [u8],
    /// Y coordinate.
    pub y: &'a [u8],
}

/// Buffers receiving a curve point in affine coordinates.
///
/// Both coordinates are big-endian and [`Curve::size`] bytes long.
pub struct PointMut<'a> {
    /// X coordinate.
    pub x: &'a mut [u8],
    /// Y coordinate.
    pub y: &'a mut [u8],
}

/// An ECDSA signature.
///
/// Both components are big-endian and [`Curve::size`] bytes long.
#[derive(Copy, Clone)]
pub struct Signature<'a> {
    /// First component, `r`.
    pub r: &'a [u8],
    /// Second component, `s`.
    pub s: &'a [u8],
}

/// Buffers receiving an ECDSA signature.
///
/// Both components are big-endian and [`Curve::size`] bytes long.
pub struct SignatureMut<'a> {
    /// First component, `r`.
    pub r: &'a mut [u8],
    /// Second component, `s`.
    pub s: &'a mut [u8],
}

/// Driver for the public key accelerator.
///
/// See the [module](self) documentation.
pub struct Pka<'d, M: Mode> {
    _hw: hw::Handle,
    _phantom: PhantomData<(&'d (), M)>,
}

#[cfg(all(feature = "_cryptocell", not(feature = "_embassy-crypto-pka")))]
impl<'d> Pka<'d, crate::mode::Blocking> {
    /// Creates a new blocking PKA driver.
    pub fn new_blocking(_peri: crate::Peri<'d, crate::peripherals::CRYPTO_PKA>) -> Self {
        Self::new_inner()
    }
}

#[cfg(feature = "_cryptocell")]
impl<'d, M: Mode> Pka<'d, M> {
    // Used by the `embassy-crypto` drivers, which have no peripheral token.
    pub(crate) fn new_inner() -> Self {
        Self {
            _hw: hw::Handle::new(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(all(feature = "_cracen", not(feature = "_embassy-crypto-pka")))]
impl<'d> Pka<'d, crate::mode::Blocking> {
    /// Creates a new blocking PKA driver and loads the engine's microcode.
    ///
    /// The public key engine of CRACEN needs a microcode program, which is not part of the
    /// hardware. It is lost whenever CRACEN powers down. The driver keeps CRACEN powered
    /// while it exists, so the microcode is loaded once, here.
    ///
    /// The microcode is not bundled with this crate because of its license. Nordic
    /// distributes it with the nRF Connect SDK, as the `ba414ep_ucode` array in
    /// `subsys/nrf_security/src/drivers/cracen/common/src/cracen/hardware/microcode_binary.h`.
    ///
    /// # Panics
    ///
    /// Panics if the microcode does not fit in the engine's RAM.
    pub fn new_blocking(_peri: crate::Peri<'d, crate::peripherals::CRYPTO_PKA>, microcode: &[u32]) -> Self {
        Self::new_inner(microcode)
    }
}

#[cfg(feature = "_cracen")]
impl<'d, M: Mode> Pka<'d, M> {
    // Used by the `embassy-crypto` drivers, which have no peripheral token.
    pub(crate) fn new_inner(microcode: &[u32]) -> Self {
        assert!(
            microcode.len() <= hw::MICROCODE_WORDS,
            "the microcode does not fit in the engine"
        );
        Self {
            _hw: hw::Handle::new(microcode),
            _phantom: PhantomData,
        }
    }
}

/// Registers the microcode of the CRACEN public key engine for the `embassy-crypto` drivers.
///
/// Available with the `embassy-crypto-p256-*` and `embassy-crypto-p384-*` features. These
/// take over the engine and remove the `CRYPTO_PKA` peripheral, so there is no [`Pka`] to
/// load the microcode. The drivers load the microcode registered here instead.
///
/// Call it once, before the first `embassy-crypto` operation. The drivers panic if no
/// microcode is registered. See `Pka::new_blocking` for where the microcode comes from.
///
/// # Panics
///
/// Panics if the microcode does not fit in the engine's RAM.
#[cfg(all(feature = "_cracen", feature = "_embassy-crypto-pka"))]
pub fn set_microcode(microcode: &'static [u32]) {
    assert!(
        microcode.len() <= hw::MICROCODE_WORDS,
        "the microcode does not fit in the engine"
    );
    driver::MICROCODE.lock(|m| m.set(Some(microcode)));
}

impl<'d, M: Mode> Pka<'d, M> {
    /// Computes `base ^ exponent mod modulus`, the RSA primitive.
    ///
    /// With the public exponent this is the RSA public key operation. With the private
    /// exponent it is the private key operation. For the latter, prefer
    /// [`Self::blocking_rsa_crt`] when the CRT parameters are available. It is about four
    /// times faster.
    ///
    /// - The modulus must be odd.
    /// - `base` and `exponent` must not be longer than the modulus.
    /// - `base` must be smaller than the modulus.
    /// - `output` must be as long as the modulus. The result is padded with zeros on the
    ///   left.
    ///
    /// This function does not apply or check any padding scheme. `base` must already be the
    /// encoded message or signature representative.
    ///
    /// Errors: `InvalidLength`, `InvalidModulus`.
    pub fn blocking_mod_exp(
        &mut self,
        base: &[u8],
        exponent: &[u8],
        modulus: &[u8],
        output: &mut [u8],
    ) -> Result<(), Error> {
        check_modulus(modulus)?;
        if base.len() > modulus.len() || exponent.len() > modulus.len() || output.len() != modulus.len() {
            return Err(Error::InvalidLength);
        }
        hw::mod_exp(base, exponent, modulus, output)
    }

    /// Computes the RSA private key operation using the Chinese remainder theorem.
    ///
    /// - `p`, `q`: the prime factors of the modulus.
    /// - `dp`, `dq`: the private exponent reduced modulo `p - 1` and `q - 1`.
    /// - `qinv`: the inverse of `q` modulo `p`.
    ///
    /// All five must have the same length, half the modulus length. `input` and `output`
    /// must be as long as the modulus.
    ///
    /// On the CryptoCell this operation is limited to a 2560-bit modulus, where
    /// [`Self::blocking_mod_exp`] still works up to 4096 bits.
    ///
    /// Errors: `InvalidLength`, `InvalidModulus`.
    pub fn blocking_rsa_crt(
        &mut self,
        input: &[u8],
        p: &[u8],
        q: &[u8],
        dp: &[u8],
        dq: &[u8],
        qinv: &[u8],
        output: &mut [u8],
    ) -> Result<(), Error> {
        check_modulus(p)?;
        check_modulus(q)?;
        if q.len() != p.len() || dp.len() != p.len() || dq.len() != p.len() || qinv.len() != p.len() {
            return Err(Error::InvalidLength);
        }
        if input.len() != 2 * p.len() || output.len() != 2 * p.len() {
            return Err(Error::InvalidLength);
        }
        hw::rsa_crt(input, p, q, dp, dq, qinv, output)
    }

    /// Multiplies a curve point by a scalar, the ECDH primitive.
    ///
    /// For ECDH, multiply the peer's public key by your private key. The shared secret is the
    /// X coordinate of the result. Hash it before using it as a key.
    ///
    /// - The point must be on the curve, which is checked: a point off the curve would leak
    ///   the scalar through the result.
    /// - The scalar must be nonzero and smaller than the curve order.
    ///
    /// Errors: `InvalidLength`, `InvalidScalar`, `InvalidPoint`.
    pub fn blocking_ecc_mul(
        &mut self,
        curve: &Curve,
        scalar: &[u8],
        point: Point<'_>,
        output: PointMut<'_>,
    ) -> Result<(), Error> {
        check_curve_len(curve, &[scalar, point.x, point.y])?;
        check_curve_len(curve, &[output.x, output.y])?;
        check_scalar(curve, scalar)?;
        // A point off the curve would leak the scalar through the result (invalid curve
        // attack), and the CryptoCell does not check it itself.
        point_check(curve, point.x, point.y)?;
        ecc_mul(curve, scalar, point.x, point.y, output.x, output.y, true)
    }

    /// Derives the public key of a private key.
    ///
    /// The private key must be nonzero and smaller than the curve order.
    ///
    /// Errors: `InvalidLength`, `InvalidScalar`.
    pub fn blocking_public_key(
        &mut self,
        curve: &Curve,
        private_key: &[u8],
        output: PointMut<'_>,
    ) -> Result<(), Error> {
        check_curve_len(curve, &[private_key, output.x, output.y])?;
        check_scalar(curve, private_key)?;
        ecc_mul(curve, private_key, curve.gx, curve.gy, output.x, output.y, true)
    }

    /// Checks that a point is on the curve and is not the point at infinity.
    ///
    /// Do this on every public key that comes from an untrusted source, before using it.
    ///
    /// Errors: `InvalidLength`, `InvalidPoint`.
    pub fn blocking_point_check(&mut self, curve: &Curve, point: Point<'_>) -> Result<(), Error> {
        check_curve_len(curve, &[point.x, point.y])?;
        point_check(curve, point.x, point.y)
    }

    /// Signs a hash with ECDSA.
    ///
    /// - `hash` is the hash of the message. It may have any length. ECDSA uses its leftmost
    ///   [`Curve::order_bits`] bits.
    /// - The nonce is drawn from `rng`, uniformly in `1..n` by rejection sampling, and never
    ///   leaves the driver.
    ///
    /// Errors: `InvalidLength`, `InvalidScalar`.
    pub fn blocking_ecdsa_sign<RM: Mode>(
        &mut self,
        curve: &Curve,
        private_key: &[u8],
        hash: &[u8],
        rng: &mut crate::crypto::rng::Rng<'_, RM>,
        signature: SignatureMut<'_>,
    ) -> Result<(), Error> {
        check_curve_len(curve, &[private_key])?;
        check_curve_len(curve, &[signature.r, signature.s])?;
        check_scalar(curve, private_key)?;
        let mut h = [0u8; MAX_CURVE_LEN];
        let h = &mut h[..curve.size()];
        truncate_hash(curve, hash, h);
        let mut k = [0u8; MAX_CURVE_LEN];
        let k = &mut k[..curve.size()];
        let result = loop {
            random_scalar(curve, rng, k);
            match hw::ecdsa_sign(curve, private_key, k, h, signature.r, signature.s) {
                // The nonce gave a zero signature component. Vanishingly unlikely;
                // another one fixes it.
                Err(Error::RetryWithNewK) => continue,
                result => break result,
            }
        };
        for b in k.iter_mut() {
            // Volatile so that the write is not optimized away.
            unsafe { core::ptr::write_volatile(b, 0) };
        }
        result
    }

    /// Signs a hash with ECDSA and a caller-supplied nonce.
    ///
    /// Prefer [`Self::blocking_ecdsa_sign`], which draws the nonce itself. This is for
    /// deterministic signatures (RFC 6979) and known-answer tests.
    ///
    /// - `hash` is the hash of the message. It may have any length. ECDSA uses its leftmost
    ///   [`Curve::order_bits`] bits.
    /// - `k` is the nonce. It must be uniformly random, nonzero and smaller than the
    ///   curve order, fresh for every signature, and secret: signing two messages with the
    ///   same `k`, or with a predictable or biased one, reveals the private key.
    ///
    /// Errors: `InvalidLength`, `InvalidScalar`, and `RetryWithNewK` if `k` happens to
    /// produce an unusable signature. This is vanishingly unlikely. Draw a new `k` and sign
    /// again.
    pub fn blocking_ecdsa_sign_with_nonce(
        &mut self,
        curve: &Curve,
        private_key: &[u8],
        k: &[u8],
        hash: &[u8],
        signature: SignatureMut<'_>,
    ) -> Result<(), Error> {
        check_curve_len(curve, &[private_key, k])?;
        check_curve_len(curve, &[signature.r, signature.s])?;
        check_scalar(curve, private_key)?;
        check_scalar(curve, k)?;
        let mut h = [0u8; MAX_CURVE_LEN];
        let h = &mut h[..curve.size()];
        truncate_hash(curve, hash, h);
        hw::ecdsa_sign(curve, private_key, k, h, signature.r, signature.s)
    }

    /// Verifies an ECDSA signature over a hash.
    ///
    /// - `hash` is the hash of the message. It may have any length. ECDSA uses its leftmost
    ///   [`Curve::order_bits`] bits.
    /// - The public key must be on the curve. This is not checked. Use
    ///   [`Self::blocking_point_check`] on the first use of a key from an untrusted source.
    ///
    /// Errors: `InvalidLength`, and `InvalidSignature` if the signature does not verify.
    pub fn blocking_ecdsa_verify(
        &mut self,
        curve: &Curve,
        public_key: Point<'_>,
        signature: Signature<'_>,
        hash: &[u8],
    ) -> Result<(), Error> {
        check_curve_len(curve, &[public_key.x, public_key.y])?;
        check_curve_len(curve, &[signature.r, signature.s])?;
        // A zero or out-of-range signature component is invalid, and the hardware is not
        // required to notice.
        if !in_range(signature.r, curve.n) || !in_range(signature.s, curve.n) {
            return Err(Error::InvalidSignature);
        }
        let mut h = [0u8; MAX_CURVE_LEN];
        let h = &mut h[..curve.size()];
        truncate_hash(curve, hash, h);
        ecdsa_verify(curve, public_key.x, public_key.y, signature.r, signature.s, h)
    }
}

/// Verifies a signature whose components are in `1..n`, over a hash reduced to the curve
/// size.
///
/// The CryptoCell's Strauss ladder cannot compute `u1*G + u2*Q` when an intermediate sum is
/// the point at infinity, which a valid signature can arrange. It hands `u1` and `u2` back
/// then, and the sum is computed here as two multiplications and an addition.
fn ecdsa_verify(curve: &Curve, qx: &[u8], qy: &[u8], r: &[u8], s: &[u8], hash: &[u8]) -> Result<(), Error> {
    let n = curve.size();
    let mut u1 = [0u8; MAX_CURVE_LEN];
    let mut u2 = [0u8; MAX_CURVE_LEN];
    let (u1, u2) = (&mut u1[..n], &mut u2[..n]);
    if hw::ecdsa_verify(curve, qx, qy, r, s, hash, u1, u2)? {
        return Ok(());
    }

    // u2 is nonzero: it is r/s with both in 1..n. u1 is zero when the hash is a multiple of
    // the order.
    let mut bx = [0u8; MAX_CURVE_LEN];
    let mut by = [0u8; MAX_CURVE_LEN];
    let (bx, by) = (&mut bx[..n], &mut by[..n]);
    ecc_mul(curve, u2, qx, qy, bx, by, false)?;
    let mut rx = [0u8; MAX_CURVE_LEN];
    let mut ry = [0u8; MAX_CURVE_LEN];
    let (rx, ry) = (&mut rx[..n], &mut ry[..n]);
    if is_zero(u1) {
        rx.copy_from_slice(bx);
    } else {
        let mut ax = [0u8; MAX_CURVE_LEN];
        let mut ay = [0u8; MAX_CURVE_LEN];
        let (ax, ay) = (&mut ax[..n], &mut ay[..n]);
        ecc_mul(curve, u1, curve.gx, curve.gy, ax, ay, false)?;
        if ax != bx {
            hw::ecc_add(curve, ax, ay, bx, by, rx, ry)?;
        } else if ay == by {
            hw::ecc_double(curve, ax, ay, rx, ry)?;
        } else {
            // The sum is the point at infinity, which has no X coordinate to compare.
            return Err(Error::InvalidSignature);
        }
    }

    // The signature is valid when x(R) mod n equals r. x(R) < p < 2n, so the reduction is
    // at most one subtraction.
    if !less_than(rx, curve.n) {
        let mut t = [0u8; MAX_CURVE_LEN];
        let t = &mut t[..n];
        sub_be(rx, curve.n, t);
        rx.copy_from_slice(t);
    }
    if rx == r { Ok(()) } else { Err(Error::InvalidSignature) }
}

/// Multiplies a point by a scalar in `1..n`.
///
/// On CRACEN the engine handles every scalar. On the CryptoCell the ladder cannot compute a
/// few tiny multiples, see its `scalar_mult`. Those fall back to double-and-add on affine
/// points here, over the shorter of `k` and `n - k`.
#[allow(clippy::too_many_arguments)]
fn ecc_mul(
    curve: &Curve,
    k: &[u8],
    px: &[u8],
    py: &[u8],
    rx: &mut [u8],
    ry: &mut [u8],
    blind: bool,
) -> Result<(), Error> {
    if hw::ecc_mul(curve, k, px, py, rx, ry, blind)? {
        return Ok(());
    }

    let n = curve.size();
    let mut neg_k = [0u8; MAX_CURVE_LEN];
    let neg_k = &mut neg_k[..n];
    sub_be(curve.n, k, neg_k);
    let negate = bit_len(neg_k) < bit_len(k);
    let k = if negate { &*neg_k } else { k };

    let mut ax = [0u8; MAX_CURVE_LEN];
    let mut ay = [0u8; MAX_CURVE_LEN];
    let (ax, ay) = (&mut ax[..n], &mut ay[..n]);
    let mut tx = [0u8; MAX_CURVE_LEN];
    let mut ty = [0u8; MAX_CURVE_LEN];
    let (tx, ty) = (&mut tx[..n], &mut ty[..n]);
    let mut acc = false;
    for i in (0..bit_len(k) as usize).rev() {
        if acc {
            // A point of a prime-order curve never doubles to infinity.
            hw::ecc_double(curve, ax, ay, tx, ty)?;
            ax.copy_from_slice(tx);
            ay.copy_from_slice(ty);
        }
        if (k[n - 1 - i / 8] >> (i % 8)) & 1 == 1 {
            if !acc {
                ax.copy_from_slice(px);
                ay.copy_from_slice(py);
                acc = true;
            } else if ax == px {
                if ay == py {
                    hw::ecc_double(curve, ax, ay, tx, ty)?;
                } else {
                    // Adding the opposite: the sum is infinity, which `k` in `1..n` never
                    // reaches at the end, so this is an intermediate that the next
                    // doubling leaves at infinity. Represent it by restarting the sum.
                    acc = false;
                    continue;
                }
                ax.copy_from_slice(tx);
                ay.copy_from_slice(ty);
            } else {
                hw::ecc_add(curve, ax, ay, px, py, tx, ty)?;
                ax.copy_from_slice(tx);
                ay.copy_from_slice(ty);
            }
        }
    }
    if !acc {
        return Err(Error::Hardware);
    }
    rx.copy_from_slice(ax);
    if negate {
        sub_be(curve.p, ay, ry);
    } else {
        ry.copy_from_slice(ay);
    }
    Ok(())
}

/// `out = a - b` for big-endian `a >= b` of the same length.
fn sub_be(a: &[u8], b: &[u8], out: &mut [u8]) {
    let mut borrow = 0i16;
    for i in (0..a.len()).rev() {
        let d = a[i] as i16 - b[i] as i16 - borrow;
        out[i] = d as u8;
        borrow = (d < 0) as i16;
    }
}

/// Checks that a point is on the curve and is not the point at infinity.
fn point_check(curve: &Curve, x: &[u8], y: &[u8]) -> Result<(), Error> {
    // The point at infinity has no affine coordinates, and coordinates outside the field
    // are not a valid encoding even where they reduce onto the curve.
    if is_zero(x) && is_zero(y) {
        return Err(Error::InvalidPoint);
    }
    if !less_than(x, curve.p) || !less_than(y, curve.p) {
        return Err(Error::InvalidPoint);
    }
    hw::point_check(curve, x, y)
}

fn check_curve_len(curve: &Curve, values: &[&[u8]]) -> Result<(), Error> {
    for v in values {
        if v.len() != curve.size() {
            return Err(Error::InvalidLength);
        }
    }
    Ok(())
}

fn check_modulus(modulus: &[u8]) -> Result<(), Error> {
    if modulus.is_empty() || modulus.len() > MAX_MODULUS_LEN {
        return Err(Error::InvalidLength);
    }
    // Barrett reduction in the engine needs an odd modulus.
    if modulus[modulus.len() - 1] & 1 == 0 {
        return Err(Error::InvalidModulus);
    }
    Ok(())
}

/// Fills `out` (the size of the curve) with a uniformly random scalar in `1..n` from `rng`.
///
/// Rejection sampling: the bits above the order are masked off, so at most half the draws are
/// rejected, and a draw that is zero or not below the order is thrown away.
fn random_scalar<M: Mode>(curve: &Curve, rng: &mut crate::crypto::rng::Rng<'_, M>, out: &mut [u8]) {
    let n = curve.size();
    let unused_bits = n * 8 - curve.order_bits as usize;
    loop {
        rng.blocking_fill_bytes(out);
        // Zero the whole leading bytes and mask the partial one.
        let full = unused_bits / 8;
        out[..full].fill(0);
        if unused_bits % 8 != 0 {
            out[full] &= 0xff >> (unused_bits % 8);
        }
        if in_range(out, curve.n) {
            return;
        }
    }
}

fn check_scalar(curve: &Curve, scalar: &[u8]) -> Result<(), Error> {
    if in_range(scalar, curve.n) {
        Ok(())
    } else {
        Err(Error::InvalidScalar)
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

/// Returns whether `0 < v < limit`, both big-endian and of the same length.
fn in_range(v: &[u8], limit: &[u8]) -> bool {
    !is_zero(v) && less_than(v, limit)
}

/// Reduces a hash to its leftmost `order_bits` bits, right-aligned in `out`.
///
/// This is the hash-to-integer conversion of FIPS 186-4.
fn truncate_hash(curve: &Curve, hash: &[u8], out: &mut [u8]) {
    out.fill(0);
    let out_len = out.len();
    let bits = curve.order_bits as usize;
    let used = hash.len().min(bits.div_ceil(8));
    let hash = &hash[..used];
    let n = out_len.min(used);
    out[out_len - n..].copy_from_slice(&hash[used - n..]);
    // If the hash is longer than the order, its leftmost `bits` bits are used, which for a
    // bit length that is not a multiple of 8 means shifting the value right.
    let shift = if hash.len() * 8 > bits { (8 - bits % 8) % 8 } else { 0 };
    if shift != 0 {
        let mut carry = 0u8;
        for b in out.iter_mut() {
            let v = *b;
            *b = (v >> shift) | carry;
            carry = v << (8 - shift);
        }
    }
}
