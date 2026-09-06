//! Driver interface: the unitraits that implementations register.
//!
//! Each operation of the public API is served by exactly one *driver*: a type
//! implementing the corresponding trait here, registered with the matching
//! `*_impl!` macro by exactly one crate in the dependency tree. Dispatch is
//! resolved at link time (see [`unitrait`]): the public API calls through
//! `extern "Rust"` symbols, so an application links against whichever driver
//! its dependency tree registers, with no generics or `dyn` involved. If no
//! crate registers a driver for an operation the application uses, or more
//! than one does, linking fails.
//!
//! Drivers are registered one operation at a time, so an application can take
//! SHA-256 from its HAL's hardware accelerator and AES-GCM from a software
//! crate, for instance. The crates providing drivers expose one Cargo feature
//! per operation, named `embassy-crypto-<operation>` (`embassy-crypto-sha256`,
//! `embassy-crypto-aes128-gcm`, `embassy-crypto-p256-ecdsa`, ...).
//!
//! # Implementing a driver
//!
//! ```ignore
//! struct MyDriver;
//!
//! impl embassy_crypto::driver::Sha256 for MyDriver {
//!     type Context = MyState;
//!     fn init() -> MyState { ... }
//!     fn update(ctx: &mut MyState, data: &[u8]) { ... }
//!     fn finalize(ctx: MyState, out: &mut [u8; 32]) { ... }
//! }
//!
//! embassy_crypto::sha256_impl!(MyDriver);
//! ```
//!
//! Stateful operations declare an opaque `Context` type: the driver picks its
//! own state type, which must fit the declared size and alignment (checked at
//! compile time). The state is stored inline in the public API type, so no
//! allocation is involved.

/// Error type for crypto operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// The driver does not support the requested parameters.
    Unsupported,
    /// A key is malformed or out of range.
    InvalidKey,
    /// An input (length, nonce, tag size, ...) is invalid.
    InvalidInput,
    /// A signature or authentication tag did not verify.
    InvalidSignature,
    /// An output buffer is too small.
    BufferTooSmall,
    /// The hardware reported an error.
    HardwareError,
}

/// Error returned when a pair of input/output slices have unequal lengths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NotEqualError;

/// A buffer that is read from and written to, which may or may not alias.
///
/// Drivers receive one of these for every cipher operation, covering both the
/// in-place case (one buffer, encrypted in place) and the separate-buffer case
/// (plaintext in one buffer, ciphertext to another) with a single entry point.
pub struct InOutBuf<'inp, 'out, T> {
    in_ptr: *const T,
    out_ptr: *mut T,
    len: usize,
    _pd: core::marker::PhantomData<(&'inp T, &'out mut T)>,
}

impl<'a, T> From<&'a mut [T]> for InOutBuf<'a, 'a, T> {
    fn from(buf: &'a mut [T]) -> Self {
        let p = buf.as_mut_ptr();
        Self {
            in_ptr: p,
            out_ptr: p,
            len: buf.len(),
            _pd: core::marker::PhantomData,
        }
    }
}

impl<'inp, 'out, T> InOutBuf<'inp, 'out, T> {
    /// Create an in/out buffer from separate input and output slices of equal length.
    pub fn new(in_buf: &'inp [T], out_buf: &'out mut [T]) -> Result<Self, NotEqualError> {
        if in_buf.len() != out_buf.len() {
            return Err(NotEqualError);
        }

        Ok(Self {
            in_ptr: in_buf.as_ptr(),
            out_ptr: out_buf.as_mut_ptr(),
            len: in_buf.len(),
            _pd: core::marker::PhantomData,
        })
    }

    /// Construct from raw pointers.
    ///
    /// # Safety
    ///
    /// `in_ptr` must be valid for reads and `out_ptr` for writes of `len`
    /// elements for the respective lifetimes. They may be equal; otherwise the
    /// regions must not overlap.
    pub unsafe fn from_raw(in_ptr: *const T, out_ptr: *mut T, len: usize) -> Self {
        Self {
            in_ptr,
            out_ptr,
            len,
            _pd: core::marker::PhantomData,
        }
    }

    /// The input side.
    pub fn get_in(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.in_ptr, self.len) }
    }

    /// The output side.
    pub fn get_out(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.out_ptr, self.len) }
    }

    /// Whether the input and output are the same buffer.
    pub fn is_in_place(&self) -> bool {
        core::ptr::eq(self.in_ptr, self.out_ptr)
    }

    /// Split into the input and output slices.
    ///
    /// When the buffer is in place, the returned slices alias. This is not
    /// exposed as safe API for that reason; use [`get_in`](Self::get_in),
    /// [`get_out`](Self::get_out) or [`into_out_with_copied_in`](Self::into_out_with_copied_in).
    ///
    /// # Safety
    ///
    /// The caller must not use both slices when [`is_in_place`](Self::is_in_place).
    pub unsafe fn split(self) -> (&'inp [T], &'out mut [T]) {
        unsafe {
            (
                core::slice::from_raw_parts(self.in_ptr, self.len),
                core::slice::from_raw_parts_mut(self.out_ptr, self.len),
            )
        }
    }

    /// Return the output slice, after copying the input into it when they differ.
    ///
    /// The natural entry point for drivers that only operate in place.
    pub fn into_out_with_copied_in(self) -> &'out mut [T]
    where
        T: Copy,
    {
        if !core::ptr::eq(self.in_ptr, self.out_ptr) {
            unsafe { core::ptr::copy(self.in_ptr, self.out_ptr, self.len) };
        }
        unsafe { core::slice::from_raw_parts_mut(self.out_ptr, self.len) }
    }

    /// Reborrow a sub-range of the buffer.
    pub fn reborrow_range(&mut self, start: usize, end: usize) -> InOutBuf<'_, '_, T> {
        assert!(start <= end && end <= self.len);
        InOutBuf {
            in_ptr: unsafe { self.in_ptr.add(start) },
            out_ptr: unsafe { self.out_ptr.add(start) },
            len: end - start,
            _pd: core::marker::PhantomData,
        }
    }

    /// Consume the buffer and return the raw pointers.
    pub fn into_raw(self) -> (*const T, *mut T) {
        (self.in_ptr, self.out_ptr)
    }

    /// Length of the buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Source of cryptographically secure random bytes.
///
/// Passed to operations that need fresh randomness (key generation, ECDSA
/// nonces). Object-safe so drivers can take it as `&mut dyn Rng`. Production
/// implementations wrap a hardware TRNG or a CSPRNG seeded from one; tests may
/// inject a deterministic implementation to get reproducible outputs.
pub trait Rng {
    /// Fill `buf` with cryptographically secure random bytes.
    fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), Error>;
}

impl<R: Rng + ?Sized> Rng for &mut R {
    fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        (**self).fill_bytes(buf)
    }
}

// ===========================================================================
// Digests
// ===========================================================================

unitrait::unitrait! {
    /// MD5 driver.
    ///
    /// The digest is `16` bytes. `finalize` writes it to `out`, which is exactly
    /// `16` bytes long.
    #[symbol_prefix = "_embassy_crypto_md5"]
    pub trait Md5 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-md5"), opaque(size = 256, align = 16))]
        #[opaque(size = 128, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new hash computation.
        fn init() -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the digest to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 16]);
    }

    /// The global [`Md5`] implementation.
    pub struct Md5Impl;

    /// Register the global [`Md5`] implementation.
    macro md5_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// SHA-1 driver.
    ///
    /// The digest is `20` bytes. `finalize` writes it to `out`, which is exactly
    /// `20` bytes long.
    #[symbol_prefix = "_embassy_crypto_sha1"]
    pub trait Sha1 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha1"), opaque(size = 256, align = 16))]
        #[opaque(size = 128, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new hash computation.
        fn init() -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the digest to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 20]);
    }

    /// The global [`Sha1`] implementation.
    pub struct Sha1Impl;

    /// Register the global [`Sha1`] implementation.
    macro sha1_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// SHA-224 driver.
    ///
    /// The digest is `28` bytes. `finalize` writes it to `out`, which is exactly
    /// `28` bytes long.
    #[symbol_prefix = "_embassy_crypto_sha224"]
    pub trait Sha224 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha224"), opaque(size = 256, align = 16))]
        #[opaque(size = 128, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new hash computation.
        fn init() -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the digest to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 28]);
    }

    /// The global [`Sha224`] implementation.
    pub struct Sha224Impl;

    /// Register the global [`Sha224`] implementation.
    macro sha224_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// SHA-256 driver.
    ///
    /// The digest is `32` bytes. `finalize` writes it to `out`, which is exactly
    /// `32` bytes long.
    #[symbol_prefix = "_embassy_crypto_sha256"]
    pub trait Sha256 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha256"), opaque(size = 256, align = 16))]
        #[opaque(size = 128, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new hash computation.
        fn init() -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the digest to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 32]);
    }

    /// The global [`Sha256`] implementation.
    pub struct Sha256Impl;

    /// Register the global [`Sha256`] implementation.
    macro sha256_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// SHA-384 driver.
    ///
    /// The digest is `48` bytes. `finalize` writes it to `out`, which is exactly
    /// `48` bytes long.
    #[symbol_prefix = "_embassy_crypto_sha384"]
    pub trait Sha384 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha384"), opaque(size = 640, align = 16))]
        #[opaque(size = 256, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new hash computation.
        fn init() -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the digest to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 48]);
    }

    /// The global [`Sha384`] implementation.
    pub struct Sha384Impl;

    /// Register the global [`Sha384`] implementation.
    macro sha384_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// SHA-512/224 driver.
    ///
    /// The digest is `28` bytes. `finalize` writes it to `out`, which is exactly
    /// `28` bytes long.
    #[symbol_prefix = "_embassy_crypto_sha512_224"]
    pub trait Sha512_224 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha512-224"), opaque(size = 640, align = 16))]
        #[opaque(size = 256, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new hash computation.
        fn init() -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the digest to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 28]);
    }

    /// The global [`Sha512_224`] implementation.
    pub struct Sha512_224Impl;

    /// Register the global [`Sha512_224`] implementation.
    macro sha512_224_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// SHA-512/256 driver.
    ///
    /// The digest is `32` bytes. `finalize` writes it to `out`, which is exactly
    /// `32` bytes long.
    #[symbol_prefix = "_embassy_crypto_sha512_256"]
    pub trait Sha512_256 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha512-256"), opaque(size = 640, align = 16))]
        #[opaque(size = 256, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new hash computation.
        fn init() -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the digest to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 32]);
    }

    /// The global [`Sha512_256`] implementation.
    pub struct Sha512_256Impl;

    /// Register the global [`Sha512_256`] implementation.
    macro sha512_256_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// SHA-512 driver.
    ///
    /// The digest is `64` bytes. `finalize` writes it to `out`, which is exactly
    /// `64` bytes long.
    #[symbol_prefix = "_embassy_crypto_sha512"]
    pub trait Sha512 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha512"), opaque(size = 640, align = 16))]
        #[opaque(size = 256, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new hash computation.
        fn init() -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the digest to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 64]);
    }

    /// The global [`Sha512`] implementation.
    pub struct Sha512Impl;

    /// Register the global [`Sha512`] implementation.
    macro sha512_impl(path = $crate::driver);
}

// ===========================================================================
// HMAC
// ===========================================================================

unitrait::unitrait! {
    /// HMAC-SHA-1 driver.
    ///
    /// Keys of any length are accepted (longer than the block size are hashed
    /// first, per RFC 2104). The tag is `20` bytes.
    #[symbol_prefix = "_embassy_crypto_hmac_sha1"]
    pub trait HmacSha1 {
        /// Opaque storage for the implementation's MAC state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha1"), opaque(size = 512, align = 16))]
        #[opaque(size = 350, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new MAC computation with `key`.
        fn init(key: &[u8]) -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the tag to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 20]);
    }

    /// The global [`HmacSha1`] implementation.
    pub struct HmacSha1Impl;

    /// Register the global [`HmacSha1`] implementation.
    macro hmac_sha1_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// HMAC-SHA-224 driver.
    ///
    /// Keys of any length are accepted (longer than the block size are hashed
    /// first, per RFC 2104). The tag is `28` bytes.
    #[symbol_prefix = "_embassy_crypto_hmac_sha224"]
    pub trait HmacSha224 {
        /// Opaque storage for the implementation's MAC state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha224"), opaque(size = 512, align = 16))]
        #[opaque(size = 350, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new MAC computation with `key`.
        fn init(key: &[u8]) -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the tag to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 28]);
    }

    /// The global [`HmacSha224`] implementation.
    pub struct HmacSha224Impl;

    /// Register the global [`HmacSha224`] implementation.
    macro hmac_sha224_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// HMAC-SHA-256 driver.
    ///
    /// Keys of any length are accepted (longer than the block size are hashed
    /// first, per RFC 2104). The tag is `32` bytes.
    #[symbol_prefix = "_embassy_crypto_hmac_sha256"]
    pub trait HmacSha256 {
        /// Opaque storage for the implementation's MAC state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha256"), opaque(size = 512, align = 16))]
        #[opaque(size = 350, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new MAC computation with `key`.
        fn init(key: &[u8]) -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the tag to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 32]);
    }

    /// The global [`HmacSha256`] implementation.
    pub struct HmacSha256Impl;

    /// Register the global [`HmacSha256`] implementation.
    macro hmac_sha256_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// HMAC-SHA-384 driver.
    ///
    /// Keys of any length are accepted (longer than the block size are hashed
    /// first, per RFC 2104). The tag is `48` bytes.
    #[symbol_prefix = "_embassy_crypto_hmac_sha384"]
    pub trait HmacSha384 {
        /// Opaque storage for the implementation's MAC state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha384"), opaque(size = 1024, align = 16))]
        #[opaque(size = 600, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new MAC computation with `key`.
        fn init(key: &[u8]) -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the tag to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 48]);
    }

    /// The global [`HmacSha384`] implementation.
    pub struct HmacSha384Impl;

    /// Register the global [`HmacSha384`] implementation.
    macro hmac_sha384_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// HMAC-SHA-512/224 driver.
    ///
    /// Keys of any length are accepted (longer than the block size are hashed
    /// first, per RFC 2104). The tag is `28` bytes.
    #[symbol_prefix = "_embassy_crypto_hmac_sha512_224"]
    pub trait HmacSha512_224 {
        /// Opaque storage for the implementation's MAC state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha512-224"), opaque(size = 1024, align = 16))]
        #[opaque(size = 600, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new MAC computation with `key`.
        fn init(key: &[u8]) -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the tag to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 28]);
    }

    /// The global [`HmacSha512_224`] implementation.
    pub struct HmacSha512_224Impl;

    /// Register the global [`HmacSha512_224`] implementation.
    macro hmac_sha512_224_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// HMAC-SHA-512/256 driver.
    ///
    /// Keys of any length are accepted (longer than the block size are hashed
    /// first, per RFC 2104). The tag is `32` bytes.
    #[symbol_prefix = "_embassy_crypto_hmac_sha512_256"]
    pub trait HmacSha512_256 {
        /// Opaque storage for the implementation's MAC state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha512-256"), opaque(size = 1024, align = 16))]
        #[opaque(size = 600, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new MAC computation with `key`.
        fn init(key: &[u8]) -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the tag to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 32]);
    }

    /// The global [`HmacSha512_256`] implementation.
    pub struct HmacSha512_256Impl;

    /// Register the global [`HmacSha512_256`] implementation.
    macro hmac_sha512_256_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// HMAC-SHA-512 driver.
    ///
    /// Keys of any length are accepted (longer than the block size are hashed
    /// first, per RFC 2104). The tag is `64` bytes.
    #[symbol_prefix = "_embassy_crypto_hmac_sha512"]
    pub trait HmacSha512 {
        /// Opaque storage for the implementation's MAC state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha512"), opaque(size = 1024, align = 16))]
        #[opaque(size = 600, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Start a new MAC computation with `key`.
        fn init(key: &[u8]) -> Self::Context;

        /// Absorb `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the tag to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 64]);
    }

    /// The global [`HmacSha512`] implementation.
    pub struct HmacSha512Impl;

    /// Register the global [`HmacSha512`] implementation.
    macro hmac_sha512_impl(path = $crate::driver);
}

// ===========================================================================
// AES
// ===========================================================================

unitrait::unitrait! {
    /// AES-128 block cipher (ECB) driver.
    ///
    /// `blocks` is a whole number of 16-byte blocks; each is processed independently.
    #[symbol_prefix = "_embassy_crypto_aes128ecb"]
    pub trait Aes128Ecb {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 384, align = 16))]
        #[opaque(size = 1024, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Initialize with a 128-bit key.
        fn init(key: &[u8; 16]) -> Self::Context;

        /// Encrypt 16-byte blocks.
        fn encrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks.
        fn decrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes128Ecb`] implementation.
    pub struct Aes128EcbImpl;

    /// Register the global [`Aes128Ecb`] implementation.
    macro aes128_ecb_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// AES-128 CBC mode driver.
    ///
    /// `blocks` is a whole number of 16-byte blocks. The chaining state is carried
    /// in the context across calls, so a message may be processed in several calls.
    #[symbol_prefix = "_embassy_crypto_aes128cbc"]
    pub trait Aes128Cbc {
        /// Opaque storage for the encryptor's key schedule and chaining state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 400, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type EncryptContext: Send + Sync + Clone + Drop;

        /// Opaque storage for the decryptor's key schedule and chaining state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 400, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type DecryptContext: Send + Sync + Clone + Drop;

        /// Initialize an encryptor with a 128-bit key and 128-bit IV.
        fn encrypt_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::EncryptContext;

        /// Initialize a decryptor with a 128-bit key and 128-bit IV.
        fn decrypt_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::DecryptContext;

        /// Encrypt 16-byte blocks, updating the chaining state.
        fn encrypt_blocks(ctx: &mut Self::EncryptContext, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks, updating the chaining state.
        fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes128Cbc`] implementation.
    pub struct Aes128CbcImpl;

    /// Register the global [`Aes128Cbc`] implementation.
    macro aes128_cbc_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// AES-128 CTR mode driver.
    ///
    /// The counter is a 128-bit big-endian integer, incremented after each block
    /// (NIST SP 800-38A). Encryption and decryption are the same operation. The
    /// counter and any unused keystream are carried in the context across calls,
    /// so `buf` need not be block-aligned.
    #[symbol_prefix = "_embassy_crypto_aes128ctr"]
    pub trait Aes128Ctr {
        /// Opaque storage for the key schedule, counter and partial keystream.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 432, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Initialize with a 128-bit key and 128-bit initial counter block.
        fn init(key: &[u8; 16], iv: &[u8; 16]) -> Self::Context;

        /// XOR the keystream into `buf`.
        fn apply_keystream(ctx: &mut Self::Context, buf: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes128Ctr`] implementation.
    pub struct Aes128CtrImpl;

    /// Register the global [`Aes128Ctr`] implementation.
    macro aes128_ctr_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// AES-128 GCM driver.
    ///
    /// `nonce` is 12 bytes; other lengths return [`Error::InvalidInput`]. The
    /// tag is 16 bytes. `decrypt` verifies the tag in constant time and returns
    /// [`Error::InvalidSignature`] on mismatch; the buffer contents are then
    /// unspecified.
    #[symbol_prefix = "_embassy_crypto_aes128gcm"]
    pub trait Aes128Gcm {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 450, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Initialize with a 128-bit key.
        fn init(key: &[u8; 16]) -> Self::Context;

        /// Encrypt `buffer` and produce the authentication tag.
        fn encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8; 16],
        ) -> Result<(), Error>;

        /// Verify the authentication tag and decrypt `buffer`.
        fn decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8; 16],
        ) -> Result<(), Error>;
    }

    /// The global [`Aes128Gcm`] implementation.
    pub struct Aes128GcmImpl;

    /// Register the global [`Aes128Gcm`] implementation.
    macro aes128_gcm_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// AES-128 CCM driver.
    ///
    /// `nonce` is 7 to 13 bytes and the tag is 4, 6, 8, 10, 12, 14 or 16 bytes
    /// (NIST SP 800-38C); the tag length is the length of the `tag` slice. Other
    /// lengths return [`Error::InvalidInput`]. `decrypt` verifies the tag in
    /// constant time and returns [`Error::InvalidSignature`] on mismatch; the
    /// buffer contents are then unspecified.
    #[symbol_prefix = "_embassy_crypto_aes128ccm"]
    pub trait Aes128Ccm {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 512, align = 16))]
        #[opaque(size = 1024, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Initialize with a 128-bit key.
        fn init(key: &[u8; 16]) -> Self::Context;

        /// Encrypt `buffer` and produce the authentication tag.
        fn encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8],
        ) -> Result<(), Error>;

        /// Verify the authentication tag and decrypt `buffer`.
        fn decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8],
        ) -> Result<(), Error>;
    }

    /// The global [`Aes128Ccm`] implementation.
    pub struct Aes128CcmImpl;

    /// Register the global [`Aes128Ccm`] implementation.
    macro aes128_ccm_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// AES-128 CMAC driver (NIST SP 800-38B).
    #[symbol_prefix = "_embassy_crypto_aes128cmac"]
    pub trait Aes128Cmac {
        /// Opaque storage for the implementation's CMAC state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 432, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Initialize with a 128-bit key.
        fn init(key: &[u8; 16]) -> Self::Context;

        /// Absorb message data.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the 16-byte tag to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 16]);

        /// Reset the context to its post-init, pre-message state.
        fn reset(ctx: &mut Self::Context);
    }

    /// The global [`Aes128Cmac`] implementation.
    pub struct Aes128CmacImpl;

    /// Register the global [`Aes128Cmac`] implementation.
    macro aes128_cmac_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// AES-256 block cipher (ECB) driver.
    ///
    /// `blocks` is a whole number of 16-byte blocks; each is processed independently.
    #[symbol_prefix = "_embassy_crypto_aes256ecb"]
    pub trait Aes256Ecb {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 512, align = 16))]
        #[opaque(size = 1024, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Initialize with a 256-bit key.
        fn init(key: &[u8; 32]) -> Self::Context;

        /// Encrypt 16-byte blocks.
        fn encrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks.
        fn decrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes256Ecb`] implementation.
    pub struct Aes256EcbImpl;

    /// Register the global [`Aes256Ecb`] implementation.
    macro aes256_ecb_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// AES-256 CBC mode driver.
    ///
    /// `blocks` is a whole number of 16-byte blocks. The chaining state is carried
    /// in the context across calls, so a message may be processed in several calls.
    #[symbol_prefix = "_embassy_crypto_aes256cbc"]
    pub trait Aes256Cbc {
        /// Opaque storage for the encryptor's key schedule and chaining state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 528, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type EncryptContext: Send + Sync + Clone + Drop;

        /// Opaque storage for the decryptor's key schedule and chaining state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 528, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type DecryptContext: Send + Sync + Clone + Drop;

        /// Initialize an encryptor with a 256-bit key and 128-bit IV.
        fn encrypt_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::EncryptContext;

        /// Initialize a decryptor with a 256-bit key and 128-bit IV.
        fn decrypt_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::DecryptContext;

        /// Encrypt 16-byte blocks, updating the chaining state.
        fn encrypt_blocks(ctx: &mut Self::EncryptContext, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks, updating the chaining state.
        fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes256Cbc`] implementation.
    pub struct Aes256CbcImpl;

    /// Register the global [`Aes256Cbc`] implementation.
    macro aes256_cbc_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// AES-256 CTR mode driver.
    ///
    /// The counter is a 128-bit big-endian integer, incremented after each block
    /// (NIST SP 800-38A). Encryption and decryption are the same operation. The
    /// counter and any unused keystream are carried in the context across calls,
    /// so `buf` need not be block-aligned.
    #[symbol_prefix = "_embassy_crypto_aes256ctr"]
    pub trait Aes256Ctr {
        /// Opaque storage for the key schedule, counter and partial keystream.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 560, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Initialize with a 256-bit key and 128-bit initial counter block.
        fn init(key: &[u8; 32], iv: &[u8; 16]) -> Self::Context;

        /// XOR the keystream into `buf`.
        fn apply_keystream(ctx: &mut Self::Context, buf: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes256Ctr`] implementation.
    pub struct Aes256CtrImpl;

    /// Register the global [`Aes256Ctr`] implementation.
    macro aes256_ctr_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// AES-256 GCM driver.
    ///
    /// `nonce` is 12 bytes; other lengths return [`Error::InvalidInput`]. The
    /// tag is 16 bytes. `decrypt` verifies the tag in constant time and returns
    /// [`Error::InvalidSignature`] on mismatch; the buffer contents are then
    /// unspecified.
    #[symbol_prefix = "_embassy_crypto_aes256gcm"]
    pub trait Aes256Gcm {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 540, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Initialize with a 256-bit key.
        fn init(key: &[u8; 32]) -> Self::Context;

        /// Encrypt `buffer` and produce the authentication tag.
        fn encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8; 16],
        ) -> Result<(), Error>;

        /// Verify the authentication tag and decrypt `buffer`.
        fn decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8; 16],
        ) -> Result<(), Error>;
    }

    /// The global [`Aes256Gcm`] implementation.
    pub struct Aes256GcmImpl;

    /// Register the global [`Aes256Gcm`] implementation.
    macro aes256_gcm_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// AES-256 CCM driver.
    ///
    /// `nonce` is 7 to 13 bytes and the tag is 4, 6, 8, 10, 12, 14 or 16 bytes
    /// (NIST SP 800-38C); the tag length is the length of the `tag` slice. Other
    /// lengths return [`Error::InvalidInput`]. `decrypt` verifies the tag in
    /// constant time and returns [`Error::InvalidSignature`] on mismatch; the
    /// buffer contents are then unspecified.
    #[symbol_prefix = "_embassy_crypto_aes256ccm"]
    pub trait Aes256Ccm {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 512, align = 16))]
        #[opaque(size = 1024, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Initialize with a 256-bit key.
        fn init(key: &[u8; 32]) -> Self::Context;

        /// Encrypt `buffer` and produce the authentication tag.
        fn encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8],
        ) -> Result<(), Error>;

        /// Verify the authentication tag and decrypt `buffer`.
        fn decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8],
        ) -> Result<(), Error>;
    }

    /// The global [`Aes256Ccm`] implementation.
    pub struct Aes256CcmImpl;

    /// Register the global [`Aes256Ccm`] implementation.
    macro aes256_ccm_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// AES-256 CMAC driver (NIST SP 800-38B).
    #[symbol_prefix = "_embassy_crypto_aes256cmac"]
    pub trait Aes256Cmac {
        /// Opaque storage for the implementation's CMAC state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 560, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Send + Sync + Clone + Drop;

        /// Initialize with a 256-bit key.
        fn init(key: &[u8; 32]) -> Self::Context;

        /// Absorb message data.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finish the computation, writing the 16-byte tag to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 16]);

        /// Reset the context to its post-init, pre-message state.
        fn reset(ctx: &mut Self::Context);
    }

    /// The global [`Aes256Cmac`] implementation.
    pub struct Aes256CmacImpl;

    /// Register the global [`Aes256Cmac`] implementation.
    macro aes256_cmac_impl(path = $crate::driver);
}

// ===========================================================================
// P-256 (secp256r1)
// ===========================================================================

/// P-256 (secp256r1) scalar: an integer modulo the curve order `n`, as `32` big-endian bytes.
///
/// Values crossing the driver boundary are canonical, i.e. in `[0, n)`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct P256Scalar(pub [u8; 32]);

/// P-256 (secp256r1) point in affine coordinates: `x` and `y` as `32` big-endian bytes each.
///
/// Values crossing the driver boundary are valid points on the curve. The
/// point at infinity has no affine encoding and is never represented by this
/// type; driver operations that can produce it return `None` instead.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct P256Point {
    /// X coordinate.
    pub x: [u8; 32],
    /// Y coordinate.
    pub y: [u8; 32],
}

/// ECDSA/P-256 (secp256r1) signature: canonical `(r, s)`, each in `[1, n)`.
///
/// Signatures produced by drivers are low-S normalized (`s <= n/2`), as TLS 1.3
/// requires. Verification accepts both low-S and high-S signatures.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct P256Signature {
    /// `r` component.
    pub r: P256Scalar,
    /// `s` component.
    pub s: P256Scalar,
}

unitrait::unitrait! {
    /// P-256 (secp256r1) scalar and point arithmetic driver.
    ///
    /// The building blocks of every P-256 (secp256r1) protocol. A hardware accelerator
    /// (a public-key accelerator, or a coprocessor with a scalar-multiplication
    /// primitive) implements this; the high-level [`P256Ecdh`] and
    /// [`P256Ecdsa`] drivers can then be built on top of it.
    ///
    /// ## Contract
    ///
    /// - Scalars are canonical, in `[0, n)`; results are canonical too.
    /// - Points are valid, on-curve, and not the identity (which has no affine
    ///   encoding). Operations whose result is the identity return `None`.
    /// - Scalars may be secret: `scalar_*`, `point_mul` and `point_mul_base`
    ///   must not have scalar-dependent timing. Whether a result is the identity
    ///   is not treated as secret. `point_lincomb` is only used with public
    ///   inputs and may be variable-time.
    #[symbol_prefix = "_embassy_crypto_p256_arith"]
    pub trait P256Arith {
        /// `a + b mod n`.
        fn scalar_add(a: &P256Scalar, b: &P256Scalar) -> P256Scalar;

        /// `a - b mod n`.
        fn scalar_sub(a: &P256Scalar, b: &P256Scalar) -> P256Scalar;

        /// `a * b mod n`.
        fn scalar_mul(a: &P256Scalar, b: &P256Scalar) -> P256Scalar;

        /// `a^-1 mod n`. `a` is nonzero.
        fn scalar_invert(a: &P256Scalar) -> P256Scalar;

        /// Whether `p` is a point on the curve.
        ///
        /// The one operation whose input is not guaranteed valid: it is how
        /// untrusted points get validated.
        fn point_is_valid(p: &P256Point) -> bool;

        /// `p + q`, or `None` if the sum is the identity (`q == -p`).
        fn point_add(p: &P256Point, q: &P256Point) -> Option<P256Point>;

        /// `k * p`, or `None` if `k` is zero.
        fn point_mul(k: &P256Scalar, p: &P256Point) -> Option<P256Point>;

        /// `k * G` for the curve's base point `G`, or `None` if `k` is zero.
        fn point_mul_base(k: &P256Scalar) -> Option<P256Point>;

        /// `k1 * p1 + k2 * p2`, or `None` if the sum is the identity.
        ///
        /// What ECDSA verification needs; implementations may use a combined
        /// ladder (Shamir's trick).
        fn point_lincomb(
            k1: &P256Scalar,
            p1: &P256Point,
            k2: &P256Scalar,
            p2: &P256Point,
        ) -> Option<P256Point>;
    }

    /// The global [`P256Arith`] implementation.
    pub struct P256ArithImpl;

    /// Register the global [`P256Arith`] implementation.
    macro p256_arith_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// P-256 (secp256r1) ECDH driver.
    ///
    /// ## Contract
    ///
    /// - Private scalars are canonical and nonzero; other values return
    ///   [`Error::InvalidKey`].
    /// - `peer` is untrusted: implementations validate that it is a point on
    ///   the curve and return [`Error::InvalidKey`] otherwise.
    /// - No secret-dependent timing with respect to the private scalar.
    /// - Implementations wipe copies of secrets they materialize in RAM.
    #[symbol_prefix = "_embassy_crypto_p256_ecdh"]
    pub trait P256Ecdh {
        /// The public key `k * G` of the private scalar `k`.
        fn public_key(k: &P256Scalar) -> Result<P256Point, Error>;

        /// The shared secret: X coordinate of `k * peer`.
        fn shared_secret(k: &P256Scalar, peer: &P256Point) -> Result<[u8; 32], Error>;
    }

    /// The global [`P256Ecdh`] implementation.
    pub struct P256EcdhImpl;

    /// Register the global [`P256Ecdh`] implementation.
    macro p256_ecdh_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// ECDSA/P-256 (secp256r1) driver, over pre-hashed messages.
    ///
    /// ## Contract
    ///
    /// - Private scalars are canonical and nonzero; other values return
    ///   [`Error::InvalidKey`].
    /// - `sign` draws its nonce from `rng` by rejection sampling into `[1, n)`.
    ///   Hardware that generates the nonce on-chip from its own entropy source
    ///   may ignore `rng`, and must document it. The signature is low-S
    ///   normalized.
    /// - `verify` accepts both low-S and high-S signatures. `q` is untrusted:
    ///   implementations validate that it is a point on the curve. Failure of
    ///   any kind is reported as [`Error::InvalidSignature`], except an invalid
    ///   `q`, which is [`Error::InvalidKey`].
    /// - No secret-dependent timing with respect to the private scalar and the
    ///   nonce. `verify` only handles public data and may be variable-time.
    /// - Implementations wipe copies of secrets they materialize in RAM.
    #[symbol_prefix = "_embassy_crypto_p256_ecdsa"]
    pub trait P256Ecdsa {
        /// The public key `k * G` of the private scalar `k`.
        fn public_key(k: &P256Scalar) -> Result<P256Point, Error>;

        /// Sign the 32-byte `digest` with the private scalar `k`.
        fn sign(k: &P256Scalar, digest: &[u8; 32], rng: &mut dyn Rng) -> Result<P256Signature, Error>;

        /// Verify the signature of the 32-byte `digest` with the public key `q`.
        fn verify(q: &P256Point, digest: &[u8; 32], sig: &P256Signature) -> Result<(), Error>;
    }

    /// The global [`P256Ecdsa`] implementation.
    pub struct P256EcdsaImpl;

    /// Register the global [`P256Ecdsa`] implementation.
    macro p256_ecdsa_impl(path = $crate::driver);
}

// ===========================================================================
// P-384 (secp384r1)
// ===========================================================================

/// P-384 (secp384r1) scalar: an integer modulo the curve order `n`, as `48` big-endian bytes.
///
/// Values crossing the driver boundary are canonical, i.e. in `[0, n)`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct P384Scalar(pub [u8; 48]);

/// P-384 (secp384r1) point in affine coordinates: `x` and `y` as `48` big-endian bytes each.
///
/// Values crossing the driver boundary are valid points on the curve. The
/// point at infinity has no affine encoding and is never represented by this
/// type; driver operations that can produce it return `None` instead.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct P384Point {
    /// X coordinate.
    pub x: [u8; 48],
    /// Y coordinate.
    pub y: [u8; 48],
}

/// ECDSA/P-384 (secp384r1) signature: canonical `(r, s)`, each in `[1, n)`.
///
/// Signatures produced by drivers are low-S normalized (`s <= n/2`), as TLS 1.3
/// requires. Verification accepts both low-S and high-S signatures.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct P384Signature {
    /// `r` component.
    pub r: P384Scalar,
    /// `s` component.
    pub s: P384Scalar,
}

unitrait::unitrait! {
    /// P-384 (secp384r1) scalar and point arithmetic driver.
    ///
    /// The building blocks of every P-384 (secp384r1) protocol. A hardware accelerator
    /// (a public-key accelerator, or a coprocessor with a scalar-multiplication
    /// primitive) implements this; the high-level [`P384Ecdh`] and
    /// [`P384Ecdsa`] drivers can then be built on top of it.
    ///
    /// ## Contract
    ///
    /// - Scalars are canonical, in `[0, n)`; results are canonical too.
    /// - Points are valid, on-curve, and not the identity (which has no affine
    ///   encoding). Operations whose result is the identity return `None`.
    /// - Scalars may be secret: `scalar_*`, `point_mul` and `point_mul_base`
    ///   must not have scalar-dependent timing. Whether a result is the identity
    ///   is not treated as secret. `point_lincomb` is only used with public
    ///   inputs and may be variable-time.
    #[symbol_prefix = "_embassy_crypto_p384_arith"]
    pub trait P384Arith {
        /// `a + b mod n`.
        fn scalar_add(a: &P384Scalar, b: &P384Scalar) -> P384Scalar;

        /// `a - b mod n`.
        fn scalar_sub(a: &P384Scalar, b: &P384Scalar) -> P384Scalar;

        /// `a * b mod n`.
        fn scalar_mul(a: &P384Scalar, b: &P384Scalar) -> P384Scalar;

        /// `a^-1 mod n`. `a` is nonzero.
        fn scalar_invert(a: &P384Scalar) -> P384Scalar;

        /// Whether `p` is a point on the curve.
        ///
        /// The one operation whose input is not guaranteed valid: it is how
        /// untrusted points get validated.
        fn point_is_valid(p: &P384Point) -> bool;

        /// `p + q`, or `None` if the sum is the identity (`q == -p`).
        fn point_add(p: &P384Point, q: &P384Point) -> Option<P384Point>;

        /// `k * p`, or `None` if `k` is zero.
        fn point_mul(k: &P384Scalar, p: &P384Point) -> Option<P384Point>;

        /// `k * G` for the curve's base point `G`, or `None` if `k` is zero.
        fn point_mul_base(k: &P384Scalar) -> Option<P384Point>;

        /// `k1 * p1 + k2 * p2`, or `None` if the sum is the identity.
        ///
        /// What ECDSA verification needs; implementations may use a combined
        /// ladder (Shamir's trick).
        fn point_lincomb(
            k1: &P384Scalar,
            p1: &P384Point,
            k2: &P384Scalar,
            p2: &P384Point,
        ) -> Option<P384Point>;
    }

    /// The global [`P384Arith`] implementation.
    pub struct P384ArithImpl;

    /// Register the global [`P384Arith`] implementation.
    macro p384_arith_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// P-384 (secp384r1) ECDH driver.
    ///
    /// ## Contract
    ///
    /// - Private scalars are canonical and nonzero; other values return
    ///   [`Error::InvalidKey`].
    /// - `peer` is untrusted: implementations validate that it is a point on
    ///   the curve and return [`Error::InvalidKey`] otherwise.
    /// - No secret-dependent timing with respect to the private scalar.
    /// - Implementations wipe copies of secrets they materialize in RAM.
    #[symbol_prefix = "_embassy_crypto_p384_ecdh"]
    pub trait P384Ecdh {
        /// The public key `k * G` of the private scalar `k`.
        fn public_key(k: &P384Scalar) -> Result<P384Point, Error>;

        /// The shared secret: X coordinate of `k * peer`.
        fn shared_secret(k: &P384Scalar, peer: &P384Point) -> Result<[u8; 48], Error>;
    }

    /// The global [`P384Ecdh`] implementation.
    pub struct P384EcdhImpl;

    /// Register the global [`P384Ecdh`] implementation.
    macro p384_ecdh_impl(path = $crate::driver);
}

unitrait::unitrait! {
    /// ECDSA/P-384 (secp384r1) driver, over pre-hashed messages.
    ///
    /// ## Contract
    ///
    /// - Private scalars are canonical and nonzero; other values return
    ///   [`Error::InvalidKey`].
    /// - `sign` draws its nonce from `rng` by rejection sampling into `[1, n)`.
    ///   Hardware that generates the nonce on-chip from its own entropy source
    ///   may ignore `rng`, and must document it. The signature is low-S
    ///   normalized.
    /// - `verify` accepts both low-S and high-S signatures. `q` is untrusted:
    ///   implementations validate that it is a point on the curve. Failure of
    ///   any kind is reported as [`Error::InvalidSignature`], except an invalid
    ///   `q`, which is [`Error::InvalidKey`].
    /// - No secret-dependent timing with respect to the private scalar and the
    ///   nonce. `verify` only handles public data and may be variable-time.
    /// - Implementations wipe copies of secrets they materialize in RAM.
    #[symbol_prefix = "_embassy_crypto_p384_ecdsa"]
    pub trait P384Ecdsa {
        /// The public key `k * G` of the private scalar `k`.
        fn public_key(k: &P384Scalar) -> Result<P384Point, Error>;

        /// Sign the 48-byte `digest` with the private scalar `k`.
        fn sign(k: &P384Scalar, digest: &[u8; 48], rng: &mut dyn Rng) -> Result<P384Signature, Error>;

        /// Verify the signature of the 48-byte `digest` with the public key `q`.
        fn verify(q: &P384Point, digest: &[u8; 48], sig: &P384Signature) -> Result<(), Error>;
    }

    /// The global [`P384Ecdsa`] implementation.
    pub struct P384EcdsaImpl;

    /// Register the global [`P384Ecdsa`] implementation.
    macro p384_ecdsa_impl(path = $crate::driver);
}
// ===========================================================================
// X25519
// ===========================================================================

/// X25519 private key: a 32-byte Curve25519 scalar, little-endian (RFC 7748).
///
/// Stored unclamped; drivers clamp as part of the X25519 function.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct X25519SecretKey(pub [u8; 32]);

/// X25519 public key: a 32-byte Curve25519 u-coordinate, little-endian (RFC 7748).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct X25519PublicKey(pub [u8; 32]);

unitrait::unitrait! {
    /// X25519 (Curve25519 ECDH, RFC 7748) driver.
    ///
    /// ## Contract
    ///
    /// - X25519 accepts every 32-byte string as a public key, so no validation
    ///   is performed on `peer`. The public API rejects an all-zero shared
    ///   secret (a low-order peer point) itself.
    /// - No secret-dependent timing with respect to the private scalar.
    /// - Implementations wipe copies of secrets they materialize in RAM.
    #[symbol_prefix = "_embassy_crypto_x25519"]
    pub trait X25519 {
        /// The public key `X25519(k, 9)` of the private scalar `k`.
        fn public_key(k: &X25519SecretKey) -> Result<X25519PublicKey, Error>;

        /// The shared secret `X25519(k, peer)`.
        fn shared_secret(k: &X25519SecretKey, peer: &X25519PublicKey) -> Result<[u8; 32], Error>;
    }

    /// The global [`X25519`] implementation.
    pub struct X25519Impl;

    /// Register the global [`X25519`] implementation.
    macro x25519_impl(path = $crate::driver);
}
