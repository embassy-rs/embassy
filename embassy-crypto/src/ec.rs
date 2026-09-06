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
        ecdh = $ecdh:ident,
        ecdsa = $ecdsa:ident,
    ) => {
        use crate::driver::{self, $dpoint, $dscalar, $dsig};
        use crate::{Error, Rng, ct};

        /// The curve order `n`, big-endian.
        pub const ORDER: [u8; $n] = $order;

        /// Size of a field element, scalar and coordinate, in bytes.
        pub const FIELD_SIZE: usize = $n;

        /// Draw a scalar uniformly from `[1, n)` by rejection sampling.
        fn random_nonzero_scalar<R: Rng + ?Sized>(rng: &mut R) -> Result<$dscalar, Error> {
            let mut k = $dscalar([0u8; $n]);
            loop {
                rng.fill_bytes(&mut k.0)?;
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

        /// A point on the curve, in affine coordinates.
        ///
        /// Always a valid point, and never the point at infinity, which has no
        /// affine encoding: operations that can produce it return `None`.
        /// Arithmetic is served by the curve's arithmetic driver.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct Point($dpoint);

        impl Point {
            /// The base point `G`.
            pub const GENERATOR: Self = Self($dpoint { x: $gx, y: $gy });

            /// Build a point from its coordinates, checking that it is on the curve.
            pub fn from_xy(x: &[u8; $n], y: &[u8; $n]) -> Result<Self, Error> {
                Self::from_driver($dpoint { x: *x, y: *y })
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

            /// Wrap a driver point, checking that it is on the curve.
            pub fn from_driver(p: $dpoint) -> Result<Self, Error> {
                if !driver::$arith::point_is_valid(&p) {
                    return Err(Error::InvalidKey);
                }
                Ok(Self(p))
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

            /// `self + other`, or `None` if the sum is the point at infinity.
            pub fn add(&self, other: &Self) -> Option<Self> {
                driver::$arith::point_add(&self.0, &other.0).map(Self)
            }

            /// `k * self`, or `None` if `k` is zero.
            pub fn mul(&self, k: &Scalar) -> Option<Self> {
                driver::$arith::point_mul(&k.0, &self.0).map(Self)
            }

            /// `k * G`, or `None` if `k` is zero.
            pub fn mul_base(k: &Scalar) -> Option<Self> {
                driver::$arith::point_mul_base(&k.0).map(Self)
            }

            /// `k1 * p1 + k2 * p2`, or `None` if the sum is the point at infinity.
            ///
            /// May be variable-time: only for public inputs.
            pub fn lincomb(k1: &Scalar, p1: &Self, k2: &Scalar, p2: &Self) -> Option<Self> {
                driver::$arith::point_lincomb(&k1.0, &p1.0, &k2.0, &p2.0).map(Self)
            }
        }

        impl From<Point> for $dpoint {
            fn from(p: Point) -> Self {
                p.0
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
            pub fn generate<R: Rng + ?Sized>(rng: &mut R) -> Result<Self, Error> {
                random_nonzero_scalar(rng).map(Self)
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

        impl From<Point> for PublicKey {
            fn from(p: Point) -> Self {
                Self(p.0)
            }
        }

        impl TryFrom<PublicKey> for Point {
            type Error = Error;

            fn try_from(p: PublicKey) -> Result<Self, Error> {
                Point::from_driver(p.0)
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
            pub fn generate<R: Rng + ?Sized>(rng: &mut R) -> Result<Self, Error> {
                random_nonzero_scalar(rng).map(Self)
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
            /// `rng` provides the signature nonce. The signature is low-S normalized.
            pub fn sign_prehash<R: Rng + ?Sized>(&self, digest: &[u8; $n], rng: &mut R) -> Result<Signature, Error> {
                let mut rng = rng;
                driver::$ecdsa::sign(&self.0, digest, &mut rng).map(Signature)
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

        impl From<Point> for VerifyingKey {
            fn from(p: Point) -> Self {
                Self(p.0)
            }
        }

        impl TryFrom<VerifyingKey> for Point {
            type Error = Error;

            fn try_from(p: VerifyingKey) -> Result<Self, Error> {
                Point::from_driver(p.0)
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
