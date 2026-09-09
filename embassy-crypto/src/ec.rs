//! Shared implementation of the prime-order curve modules ([`p256`](crate::p256), [`p384`](crate::p384)).

/// Instantiate the public API for one curve.
///
/// `$n` is the size of a field element in bytes, `$order` the curve order as
/// big-endian bytes, and `$gx`/`$gy` the base point coordinates.
macro_rules! curve_api {
    (
        n = $n:literal,
        order = $order:expr,
        gx = $gx:expr,
        gy = $gy:expr,
        scalar = $dscalar:ident,
        point = $dpoint:ident,
        signature = $dsig:ident,
        arith = $arith:ident,
        arith_point = $apoint:ident,
        ecdh = $ecdh:ident,
        ecdsa = $ecdsa:ident,
    ) => {
        use crate::driver::{self, RngImpl, $dpoint, $dscalar, $dsig};
        use crate::{Error, ct};

        /// The curve order `n`, big-endian.
        pub const ORDER: [u8; $n] = $order;

        /// Size of a field element, scalar and coordinate, in bytes.
        pub const FIELD_SIZE: usize = $n;

        /// Draw a scalar uniformly from `[1, n)` by rejection sampling.
        fn random_nonzero_scalar() -> Result<$dscalar, Error> {
            let mut k = $dscalar([0u8; $n]);
            loop {
                RngImpl::fill_bytes(&mut k.0);
                if !ct::is_zero(&k.0) && ct::lt(&k.0, &ORDER) {
                    return Ok(k);
                }
            }
        }

        fn checked_scalar(bytes: &[u8; $n]) -> Result<$dscalar, Error> {
            if !ct::lt(bytes, &ORDER) {
                return Err(Error::InvalidKey);
            }
            Ok($dscalar(*bytes))
        }

        fn checked_nonzero_scalar(bytes: &[u8; $n]) -> Result<$dscalar, Error> {
            if ct::is_zero(bytes) {
                return Err(Error::InvalidKey);
            }
            checked_scalar(bytes)
        }

        // =====================================================================
        // Arithmetic
        // =====================================================================

        /// A scalar: an integer modulo the curve order `n`.
        ///
        /// Always canonical (in `[0, n)`). Arithmetic is served by the curve's
        /// arithmetic driver.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct Scalar($dscalar);

        impl Scalar {
            /// The scalar zero.
            pub const ZERO: Self = Self($dscalar([0u8; $n]));

            /// Parse a big-endian scalar. Fails if it is not less than `n`.
            pub fn from_bytes(bytes: &[u8; $n]) -> Result<Self, Error> {
                checked_scalar(bytes).map(Self)
            }

            /// The scalar as big-endian bytes.
            pub fn to_bytes(&self) -> [u8; $n] {
                self.0.0
            }

            /// The scalar as big-endian bytes.
            pub fn as_bytes(&self) -> &[u8; $n] {
                &self.0.0
            }

            /// The driver representation.
            pub fn as_driver(&self) -> &$dscalar {
                &self.0
            }

            /// Whether the scalar is zero, in constant time.
            pub fn is_zero(&self) -> bool {
                ct::is_zero(&self.0.0)
            }

            /// `self + other mod n`.
            pub fn add(&self, other: &Self) -> Self {
                Self(driver::$arith::scalar_add(&self.0, &other.0))
            }

            /// `self - other mod n`.
            pub fn sub(&self, other: &Self) -> Self {
                Self(driver::$arith::scalar_sub(&self.0, &other.0))
            }

            /// `self * other mod n`.
            pub fn mul(&self, other: &Self) -> Self {
                Self(driver::$arith::scalar_mul(&self.0, &other.0))
            }

            /// `self^-1 mod n`, or `None` if `self` is zero.
            pub fn invert(&self) -> Option<Self> {
                if self.is_zero() {
                    return None;
                }
                Some(Self(driver::$arith::scalar_invert(&self.0)))
            }
        }

        impl From<Scalar> for $dscalar {
            fn from(s: Scalar) -> Self {
                s.0
            }
        }

        /// A point on the curve, the point at infinity included.
        ///
        /// Held in the arithmetic driver's own representation (projective
        /// coordinates, typically), so a chain of operations converts to
        /// affine coordinates once, when asked to through [`Self::to_affine`].
        /// Comparing two points converts both. Arithmetic is served by the
        /// curve's arithmetic driver.
        #[derive(Clone, Copy)]
        pub struct Point(driver::$apoint);

        impl Point {
            /// The base point `G`.
            pub fn generator() -> Self {
                Self(driver::$arith::point_from_affine_unchecked(&$dpoint {
                    x: $gx,
                    y: $gy,
                }))
            }

            /// The point at infinity.
            pub fn identity() -> Self {
                Self(driver::$arith::point_identity())
            }

            /// Build a point from its coordinates, checking that it is on the curve.
            pub fn from_xy(x: &[u8; $n], y: &[u8; $n]) -> Result<Self, Error> {
                Self::from_affine(&$dpoint { x: *x, y: *y })
            }

            /// Parse an uncompressed SEC1 encoding (`0x04 || x || y`), checking
            /// that the point is on the curve.
            pub fn from_sec1(bytes: &[u8; 2 * $n + 1]) -> Result<Self, Error> {
                if bytes[0] != 0x04 {
                    return Err(Error::InvalidKey);
                }
                let mut x = [0u8; $n];
                let mut y = [0u8; $n];
                x.copy_from_slice(&bytes[1..1 + $n]);
                y.copy_from_slice(&bytes[1 + $n..]);
                Self::from_xy(&x, &y)
            }

            /// Import an affine point, checking that it is on the curve.
            pub fn from_affine(p: &$dpoint) -> Result<Self, Error> {
                driver::$arith::point_from_affine(p)
                    .map(Self)
                    .ok_or(Error::InvalidKey)
            }

            /// The affine coordinates, or `None` for the point at infinity.
            pub fn to_affine(&self) -> Option<$dpoint> {
                driver::$arith::point_to_affine(&self.0)
            }

            /// The uncompressed SEC1 encoding (`0x04 || x || y`), or `None`
            /// for the point at infinity.
            pub fn to_sec1(&self) -> Option<[u8; 2 * $n + 1]> {
                let p = self.to_affine()?;
                let mut out = [0u8; 2 * $n + 1];
                out[0] = 0x04;
                out[1..1 + $n].copy_from_slice(&p.x);
                out[1 + $n..].copy_from_slice(&p.y);
                Some(out)
            }

            /// Whether this is the point at infinity.
            pub fn is_identity(&self) -> bool {
                driver::$arith::point_is_identity(&self.0)
            }

            /// Wrap a point in the driver representation.
            pub fn from_driver(p: driver::$apoint) -> Self {
                Self(p)
            }

            /// The driver representation.
            pub fn as_driver(&self) -> &driver::$apoint {
                &self.0
            }

            /// `-self`.
            pub fn neg(&self) -> Self {
                Self(driver::$arith::point_neg(&self.0))
            }

            /// `self + other`.
            pub fn add(&self, other: &Self) -> Self {
                Self(driver::$arith::point_add(&self.0, &other.0))
            }

            /// `k * self`.
            pub fn mul(&self, k: &Scalar) -> Self {
                Self(driver::$arith::point_mul(&k.0, &self.0))
            }

            /// `k * G`.
            pub fn mul_base(k: &Scalar) -> Self {
                Self(driver::$arith::point_mul_base(&k.0))
            }

            /// `a * p + b * q`.
            ///
            /// Constant-time in the scalars: safe for secret scalars, as in a
            /// PAKE or a Schnorr-style signature. Cheaper than the equivalent
            /// `mul`s and `add` when the driver shares the doublings.
            pub fn lincomb(a: &Scalar, p: &Self, b: &Scalar, q: &Self) -> Self {
                Self(driver::$arith::point_lincomb(&a.0, &p.0, &b.0, &q.0))
            }

            /// `a * p + b * q`.
            ///
            /// May be variable-time: only for public inputs, as in signature
            /// verification. Faster than [`Self::lincomb`].
            pub fn lincomb_vartime(a: &Scalar, p: &Self, b: &Scalar, q: &Self) -> Self {
                Self(driver::$arith::point_lincomb_vartime(&a.0, &p.0, &b.0, &q.0))
            }
        }

        impl PartialEq for Point {
            fn eq(&self, other: &Self) -> bool {
                self.to_affine() == other.to_affine()
            }
        }

        impl Eq for Point {}

        impl core::fmt::Debug for Point {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_tuple("Point").field(&self.to_affine()).finish()
            }
        }

        #[cfg(feature = "defmt")]
        impl defmt::Format for Point {
            fn format(&self, f: defmt::Formatter<'_>) {
                defmt::write!(f, "Point({:?})", self.to_affine())
            }
        }

        impl TryFrom<Point> for $dpoint {
            type Error = Error;

            /// Fails for the point at infinity.
            fn try_from(p: Point) -> Result<Self, Error> {
                p.to_affine().ok_or(Error::InvalidKey)
            }
        }

        // =====================================================================
        // ECDH
        // =====================================================================

        /// An ECDH private key: a nonzero scalar.
        ///
        /// Served by the curve's ECDH driver. Zeroed on drop.
        #[derive(Clone)]
        pub struct SecretKey($dscalar);

        impl SecretKey {
            /// Generate a random private key.
            pub fn generate() -> Result<Self, Error> {
                random_nonzero_scalar().map(Self)
            }

            /// Load a private key from its big-endian encoding. Fails if it is
            /// zero or not less than `n`.
            pub fn from_bytes(bytes: &[u8; $n]) -> Result<Self, Error> {
                checked_nonzero_scalar(bytes).map(Self)
            }

            /// The private key as big-endian bytes.
            pub fn to_bytes(&self) -> [u8; $n] {
                self.0.0
            }

            /// The private key as a scalar.
            pub fn to_scalar(&self) -> Scalar {
                Scalar(self.0)
            }

            /// The corresponding public key.
            pub fn public_key(&self) -> Result<PublicKey, Error> {
                driver::$ecdh::public_key(&self.0).map(PublicKey)
            }

            /// The shared secret with `peer`: the X coordinate of `self * peer`.
            pub fn diffie_hellman(&self, peer: &PublicKey) -> Result<SharedSecret, Error> {
                driver::$ecdh::shared_secret(&self.0, &peer.0).map(SharedSecret)
            }
        }

        impl TryFrom<Scalar> for SecretKey {
            type Error = Error;

            fn try_from(k: Scalar) -> Result<Self, Error> {
                if k.is_zero() {
                    return Err(Error::InvalidKey);
                }
                Ok(Self(k.0))
            }
        }

        impl Drop for SecretKey {
            fn drop(&mut self) {
                ct::zeroize(&mut self.0.0);
            }
        }

        impl core::fmt::Debug for SecretKey {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("SecretKey(..)")
            }
        }

        /// An ECDH public key: a point on the curve.
        ///
        /// Parsing does not validate the point; the ECDH driver does, when it
        /// is used. Convert to a [`Point`] to validate it explicitly.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct PublicKey($dpoint);

        impl PublicKey {
            /// Build a public key from its coordinates.
            pub fn from_xy(x: &[u8; $n], y: &[u8; $n]) -> Self {
                Self($dpoint { x: *x, y: *y })
            }

            /// Parse an uncompressed SEC1 encoding (`0x04 || x || y`).
            pub fn from_sec1(bytes: &[u8; 2 * $n + 1]) -> Result<Self, Error> {
                if bytes[0] != 0x04 {
                    return Err(Error::InvalidKey);
                }
                let mut x = [0u8; $n];
                let mut y = [0u8; $n];
                x.copy_from_slice(&bytes[1..1 + $n]);
                y.copy_from_slice(&bytes[1 + $n..]);
                Ok(Self::from_xy(&x, &y))
            }

            /// The uncompressed SEC1 encoding (`0x04 || x || y`).
            pub fn to_sec1(&self) -> [u8; 2 * $n + 1] {
                let mut out = [0u8; 2 * $n + 1];
                out[0] = 0x04;
                out[1..1 + $n].copy_from_slice(&self.0.x);
                out[1 + $n..].copy_from_slice(&self.0.y);
                out
            }

            /// The X coordinate, big-endian.
            pub fn x(&self) -> &[u8; $n] {
                &self.0.x
            }

            /// The Y coordinate, big-endian.
            pub fn y(&self) -> &[u8; $n] {
                &self.0.y
            }

            /// The driver representation.
            pub fn as_driver(&self) -> &$dpoint {
                &self.0
            }
        }

        impl TryFrom<Point> for PublicKey {
            type Error = Error;

            /// Fails for the point at infinity.
            fn try_from(p: Point) -> Result<Self, Error> {
                p.to_affine().map(Self).ok_or(Error::InvalidKey)
            }
        }

        impl TryFrom<PublicKey> for Point {
            type Error = Error;

            fn try_from(p: PublicKey) -> Result<Self, Error> {
                Point::from_affine(&p.0)
            }
        }

        /// An ECDH shared secret: the X coordinate of the shared point.
        ///
        /// Feed it to a KDF; do not use it as a key directly. Zeroed on drop.
        pub struct SharedSecret([u8; $n]);

        impl SharedSecret {
            /// The raw shared secret.
            pub fn as_bytes(&self) -> &[u8; $n] {
                &self.0
            }
        }

        impl Drop for SharedSecret {
            fn drop(&mut self) {
                ct::zeroize(&mut self.0);
            }
        }

        impl core::fmt::Debug for SharedSecret {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("SharedSecret(..)")
            }
        }

        // =====================================================================
        // ECDSA
        // =====================================================================

        /// An ECDSA signing key: a nonzero scalar.
        ///
        /// Signs pre-hashed messages: the caller hashes the message with the
        /// hash of matching size (SHA-256 for P-256, SHA-384 for P-384) and
        /// passes the digest. Served by the curve's ECDSA driver. Zeroed on drop.
        #[derive(Clone)]
        pub struct SigningKey($dscalar);

        impl SigningKey {
            /// Generate a random signing key.
            pub fn generate() -> Result<Self, Error> {
                random_nonzero_scalar().map(Self)
            }

            /// Load a signing key from its big-endian encoding. Fails if it is
            /// zero or not less than `n`.
            pub fn from_bytes(bytes: &[u8; $n]) -> Result<Self, Error> {
                checked_nonzero_scalar(bytes).map(Self)
            }

            /// The signing key as big-endian bytes.
            pub fn to_bytes(&self) -> [u8; $n] {
                self.0.0
            }

            /// The signing key as a scalar.
            pub fn to_scalar(&self) -> Scalar {
                Scalar(self.0)
            }

            /// The corresponding verifying key.
            pub fn verifying_key(&self) -> Result<VerifyingKey, Error> {
                driver::$ecdsa::public_key(&self.0).map(VerifyingKey)
            }

            /// Sign a message digest.
            ///
            /// The nonce is drawn from [`driver::RngImpl`]. The signature is low-S normalized.
            pub fn sign_prehash(&self, digest: &[u8; $n]) -> Result<Signature, Error> {
                driver::$ecdsa::sign(&self.0, digest).map(Signature)
            }
        }

        impl TryFrom<Scalar> for SigningKey {
            type Error = Error;

            fn try_from(k: Scalar) -> Result<Self, Error> {
                if k.is_zero() {
                    return Err(Error::InvalidKey);
                }
                Ok(Self(k.0))
            }
        }

        impl Drop for SigningKey {
            fn drop(&mut self) {
                ct::zeroize(&mut self.0.0);
            }
        }

        impl core::fmt::Debug for SigningKey {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("SigningKey(..)")
            }
        }

        /// An ECDSA verifying key: a point on the curve.
        ///
        /// Parsing does not validate the point; the ECDSA driver does, when it
        /// is used. Convert to a [`Point`] to validate it explicitly.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct VerifyingKey($dpoint);

        impl VerifyingKey {
            /// Build a verifying key from its coordinates.
            pub fn from_xy(x: &[u8; $n], y: &[u8; $n]) -> Self {
                Self($dpoint { x: *x, y: *y })
            }

            /// Parse an uncompressed SEC1 encoding (`0x04 || x || y`).
            pub fn from_sec1(bytes: &[u8; 2 * $n + 1]) -> Result<Self, Error> {
                PublicKey::from_sec1(bytes).map(|p| Self(p.0))
            }

            /// The uncompressed SEC1 encoding (`0x04 || x || y`).
            pub fn to_sec1(&self) -> [u8; 2 * $n + 1] {
                PublicKey(self.0).to_sec1()
            }

            /// The X coordinate, big-endian.
            pub fn x(&self) -> &[u8; $n] {
                &self.0.x
            }

            /// The Y coordinate, big-endian.
            pub fn y(&self) -> &[u8; $n] {
                &self.0.y
            }

            /// The driver representation.
            pub fn as_driver(&self) -> &$dpoint {
                &self.0
            }

            /// Verify the signature of a message digest.
            ///
            /// Both low-S and high-S signatures are accepted.
            pub fn verify_prehash(&self, digest: &[u8; $n], signature: &Signature) -> Result<(), Error> {
                driver::$ecdsa::verify(&self.0, digest, &signature.0)
            }
        }

        impl TryFrom<Point> for VerifyingKey {
            type Error = Error;

            /// Fails for the point at infinity.
            fn try_from(p: Point) -> Result<Self, Error> {
                p.to_affine().map(Self).ok_or(Error::InvalidKey)
            }
        }

        impl TryFrom<VerifyingKey> for Point {
            type Error = Error;

            fn try_from(p: VerifyingKey) -> Result<Self, Error> {
                Point::from_affine(&p.0)
            }
        }

        /// An ECDSA signature `(r, s)`.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct Signature($dsig);

        impl Signature {
            /// Parse the fixed-size encoding `r || s`, big-endian. Fails if either
            /// component is zero or not less than `n`.
            pub fn from_bytes(bytes: &[u8; 2 * $n]) -> Result<Self, Error> {
                let mut r = [0u8; $n];
                let mut s = [0u8; $n];
                r.copy_from_slice(&bytes[..$n]);
                s.copy_from_slice(&bytes[$n..]);
                Self::from_scalars(&r, &s)
            }

            /// Build a signature from its components, big-endian. Fails if
            /// either is zero or not less than `n`.
            pub fn from_scalars(r: &[u8; $n], s: &[u8; $n]) -> Result<Self, Error> {
                Ok(Self($dsig {
                    r: checked_nonzero_scalar(r).map_err(|_| Error::InvalidSignature)?,
                    s: checked_nonzero_scalar(s).map_err(|_| Error::InvalidSignature)?,
                }))
            }

            /// The fixed-size encoding `r || s`, big-endian.
            pub fn to_bytes(&self) -> [u8; 2 * $n] {
                let mut out = [0u8; 2 * $n];
                out[..$n].copy_from_slice(&self.0.r.0);
                out[$n..].copy_from_slice(&self.0.s.0);
                out
            }

            /// The `r` component, big-endian.
            pub fn r(&self) -> &[u8; $n] {
                &self.0.r.0
            }

            /// The `s` component, big-endian.
            pub fn s(&self) -> &[u8; $n] {
                &self.0.s.0
            }

            /// The driver representation.
            pub fn as_driver(&self) -> &$dsig {
                &self.0
            }
        }
    };
}

pub(crate) use curve_api;
