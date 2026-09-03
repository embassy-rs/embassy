//! Public key accelerator.
//!
//! This driver uses the PKA engine of the CryptoCell subsystem (nRF52840, nRF91, nRF5340) or
//! the BA414EP public key engine of CRACEN (nRF54L). It provides:
//!
//! - **ECDSA** signature generation and verification,
//! - **ECDH** and public key derivation, through scalar multiplication of a curve point,
//! - **RSA**, through modular exponentiation, with or without the Chinese remainder theorem.
//!
//! Curve points, scalars, moduli and signatures are big-endian byte slices, the encoding used
//! by SEC 1 and PKCS#1. Every value of a curve operation must be exactly [`Curve::size`] bytes
//! long, zero-padded on the left if needed; every value of an RSA operation must be at most as
//! long as the modulus.
//!
//! # Example
//!
//! ```no_run
//! use embassy_nrf::pka::{Pka, PointMut, curve};
//!
//! # let p: embassy_nrf::Peripherals = todo!();
//! # let private_key = [0u8; 32];
//! let mut pka = Pka::new_blocking(p.PKA);
//!
//! // Derive the public key of a private key.
//! let mut x = [0; 32];
//! let mut y = [0; 32];
//! pka.blocking_public_key(&curve::NIST_P256, &private_key, PointMut { x: &mut x, y: &mut y })?;
//! # Ok::<(), embassy_nrf::pka::Error>(())
//! ```
//!
//! # Side channels
//!
//! On the CryptoCell, scalar multiplication runs a fixed sequence of point operations whatever
//! the scalar is, and inverts by exponentiation rather than by the engine's variable-time
//! instruction, so the private key does not shape the operation. On CRACEN the operations that
//! touch a private value run with the engine's own countermeasures against power analysis:
//! curve operations randomize the scalar and the projective coordinates, and modular
//! exponentiation randomizes the modulus, each from a fresh random factor. Neither driver
//! claims constant-time behaviour. Signature verification and point validation handle public
//! values only.

use core::marker::PhantomData;

use crate::mode::{Blocking, Mode};
use crate::{Peri, peripherals};

pub mod curve;

#[cfg_attr(feature = "_cryptocell", path = "cryptocell/mod.rs")]
#[cfg_attr(feature = "_cracen", path = "cracen.rs")]
mod hw;

#[cfg(feature = "_cryptocell")]
pub(crate) use hw::poly::Poly1305;

/// Largest curve size in bytes that the drivers accept (NIST P-521).
pub const MAX_CURVE_LEN: usize = 66;

/// Largest modulus in bytes that the drivers accept, enough for RSA-4096.
///
/// A modulus this large does not fit every operation: the more values an operation holds at
/// once, the smaller the largest modulus it can work with. [`Error::InvalidModulus`] says so.
pub const MAX_MODULUS_LEN: usize = 512;

/// PKA error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// A value has the wrong length for the curve or modulus it belongs to.
    InvalidLength,
    /// The modulus is larger than the hardware supports, or is not usable (it must be odd and
    /// its top bit set).
    InvalidModulus,
    /// A scalar is zero or not smaller than the order of the curve.
    InvalidScalar,
    /// A point is not on the curve, or is the point at infinity.
    InvalidPoint,
    /// The signature does not match the message and public key.
    InvalidSignature,
    /// The value has no inverse modulo the modulus.
    NotInvertible,
    /// The ephemeral key produced an unusable signature. Retry with a different one.
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
            Self::RetryWithNewK => "retry with a new ephemeral key",
            Self::Hardware => "hardware error",
        };
        f.write_str(s)
    }
}

impl core::error::Error for Error {}

/// A curve whose parameters the hardware knows without being given them.
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
/// The curves in [`curve`] cover the common choices. [`Curve::new`] builds any other one.
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
    /// All values are big-endian and must have the same length: the field modulus `p`, the
    /// curve coefficients `a` and `b`, the coordinates `gx` and `gy` of the generator, and the
    /// order `n` of the generator. Left-pad the order with zeros if it is shorter than the
    /// field.
    ///
    /// The field modulus must be odd, which every prime above 2 is.
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

    /// The order of the generator, big-endian and [`Curve::size`] bytes long.
    ///
    /// Scalars, private keys and signature components must be smaller than this.
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

/// PKA driver.
pub struct Pka<'d, M: Mode> {
    _hw: hw::Handle,
    _phantom: PhantomData<(&'d (), M)>,
}

#[cfg(feature = "_cryptocell")]
impl<'d> Pka<'d, Blocking> {
    /// Creates a new blocking PKA driver.
    pub fn new_blocking(_peri: Peri<'d, peripherals::PKA>) -> Self {
        Self {
            _hw: hw::Handle::new(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(feature = "_cracen")]
impl<'d> Pka<'d, Blocking> {
    /// Creates a new blocking PKA driver and loads the engine's microcode.
    ///
    /// The public key engine of CRACEN runs a microcode program that is not part of the
    /// hardware: it must be loaded into the engine's RAM before any operation, and it is lost
    /// whenever CRACEN is powered down. The driver keeps CRACEN powered for as long as it
    /// exists, so the microcode is loaded once here.
    ///
    /// Nordic distributes the microcode with the nRF Connect SDK, as the `ba414ep_ucode`
    /// array of `subsys/nrf_security/src/drivers/cracen/common/src/cracen/hardware/`
    /// `microcode_binary.h`. It is licensed separately from this crate, which is why it is
    /// not bundled here.
    ///
    /// # Panics
    ///
    /// Panics if the microcode does not fit in the engine's RAM.
    pub fn new_blocking(_peri: Peri<'d, peripherals::PKA>, microcode: &[u32]) -> Self {
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

impl<'d, M: Mode> Pka<'d, M> {
    /// Computes `base ^ exponent mod modulus`, the RSA primitive.
    ///
    /// This is both the public key operation, with the public exponent, and the private key
    /// operation without the Chinese remainder theorem, with the private exponent. Use
    /// [`Self::blocking_rsa_crt`] instead where the CRT parameters are available: it is about
    /// four times faster.
    ///
    /// The modulus must be odd, and `base` and `exponent` must not be longer than it. `output`
    /// receives as many bytes as the modulus has, left-padded with zeros.
    ///
    /// This function does not apply or check any padding scheme. `base` must already be the
    /// encoded message or signature representative, and must be smaller than the modulus.
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
    /// `p` and `q` are the prime factors of the modulus, `dp` and `dq` the private exponent
    /// reduced modulo `p - 1` and `q - 1`, and `qinv` the inverse of `q` modulo `p`. All five
    /// must have the same length, half of the modulus length, and `input` and `output` must be
    /// as long as the modulus.
    ///
    /// This operation holds more values at once than any other, so it runs out of engine
    /// memory sooner: on the CryptoCell it returns [`Error::InvalidModulus`] above 2560 bits,
    /// where [`Self::blocking_mod_exp`] still works.
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
    /// The shared secret of an ECDH exchange is the X coordinate of the result of multiplying
    /// the peer's public key by the own private key. Hash the result before using it as a key.
    ///
    /// The point must be on the curve; check it with [`Self::blocking_point_check`] first if
    /// it comes from an untrusted source. The scalar must be in `1..n`.
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
        hw::ecc_mul(curve, scalar, point.x, point.y, output.x, output.y)
    }

    /// Derives the public key of a private key, by multiplying the generator by it.
    ///
    /// The private key must be in `1..n`.
    pub fn blocking_public_key(
        &mut self,
        curve: &Curve,
        private_key: &[u8],
        output: PointMut<'_>,
    ) -> Result<(), Error> {
        check_curve_len(curve, &[private_key, output.x, output.y])?;
        check_scalar(curve, private_key)?;
        hw::ecc_mul(curve, private_key, curve.gx, curve.gy, output.x, output.y)
    }

    /// Checks that a point is on the curve and is not the point at infinity.
    ///
    /// Do this on every public key that comes from an untrusted source, before using it.
    pub fn blocking_point_check(&mut self, curve: &Curve, point: Point<'_>) -> Result<(), Error> {
        check_curve_len(curve, &[point.x, point.y])?;
        // The point at infinity has no affine coordinates, and coordinates outside the field
        // are not a valid encoding even where they reduce onto the curve.
        if is_zero(point.x) && is_zero(point.y) {
            return Err(Error::InvalidPoint);
        }
        if !less_than(point.x, curve.p) || !less_than(point.y, curve.p) {
            return Err(Error::InvalidPoint);
        }
        hw::point_check(curve, point.x, point.y)
    }

    /// Signs a hash with ECDSA.
    ///
    /// `hash` is the hash of the message, of any length; ECDSA uses its leftmost
    /// [`Curve::order_bits`] bits. `k` is the per-signature ephemeral key: it must be drawn
    /// uniformly at random from `1..n` for every signature, must never be reused, and must
    /// stay secret. Recovering it, or signing two messages with the same one, reveals the
    /// private key.
    ///
    /// Returns [`Error::RetryWithNewK`] if the ephemeral key happens to produce an unusable
    /// signature, which is vanishingly unlikely. Draw a new one and sign again.
    pub fn blocking_ecdsa_sign(
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
    /// `hash` is the hash of the message, of any length; ECDSA uses its leftmost
    /// [`Curve::order_bits`] bits.
    ///
    /// The public key must be on the curve. Check it with [`Self::blocking_point_check`]
    /// before the first use of a key from an untrusted source: this function does not.
    ///
    /// Returns [`Error::InvalidSignature`] if the signature does not verify.
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
        hw::ecdsa_verify(curve, public_key.x, public_key.y, signature.r, signature.s, h)
    }
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

/// Reduces a hash to the leftmost `order_bits` bits, right-aligned in `out`.
///
/// This is the conversion of a hash to an integer of ANS X9.62 and FIPS 186-4.
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
