#![no_std]
#![doc = include_str!("../README.md")]
#![allow(missing_docs)]

/// Error type for crypto operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CryptoError {
    Unsupported,
    InvalidKey,
    InvalidInput,
    InvalidSignature,
    BufferTooSmall,
    HardwareError,
}

/// Error returned when a pair of input/output slices have unequal lengths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotEqualError;

/// A lightweight I/O buffer abstraction without depending on the RustCrypto `inout` crate.
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
    /// Create an in/out buffer from separate input and output slices.
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
    pub unsafe fn from_raw(in_ptr: *const T, out_ptr: *mut T, len: usize) -> Self {
        Self {
            in_ptr,
            out_ptr,
            len,
            _pd: core::marker::PhantomData,
        }
    }

    /// Access the input side without copying.
    pub fn get_in(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.in_ptr, self.len) }
    }

    /// Access the output side without copying.
    pub fn get_out(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.out_ptr, self.len) }
    }

    /// Return the output slice while preserving the caller's write access.
    pub fn into_out(self) -> &'out mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.out_ptr, self.len) }
    }

    /// Return the output slice, copying input into output when the regions differ.
    pub fn into_out_with_copied_in(self) -> &'out mut [T]
    where
        T: Copy,
    {
        if !core::ptr::eq(self.in_ptr, self.out_ptr) {
            unsafe { core::ptr::copy(self.in_ptr, self.out_ptr, self.len) };
        }
        unsafe { core::slice::from_raw_parts_mut(self.out_ptr, self.len) }
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

unitrait::unitrait! {
    /// Md5 trait
    #[symbol_prefix = "_embassy_crypto_md5"]
    pub trait Md5 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-md5"), opaque(size = 256, align = 16))]
        #[opaque(size = 128, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init() -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);
    }

    /// The global [`Md5`] implementation.
    pub struct Md5Impl;

    macro md5_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha1 trait
    #[symbol_prefix = "_embassy_crypto_sha1"]
    pub trait Sha1 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha1"), opaque(size = 256, align = 16))]
        #[opaque(size = 128, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init() -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);
    }

    /// The global [`Sha1`] implementation.
    pub struct Sha1Impl;

    macro sha1_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha224 trait
    #[symbol_prefix = "_embassy_crypto_sha224"]
    pub trait Sha224 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha224"), opaque(size = 256, align = 16))]
        #[opaque(size = 128, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init() -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);
    }

    /// The global [`Sha224`] implementation.
    pub struct Sha224Impl;

    macro sha224_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha256 trait
    #[symbol_prefix = "_embassy_crypto_sha256"]
    pub trait Sha256 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha256"), opaque(size = 256, align = 16))]
        #[opaque(size = 128, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init() -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);
    }

    /// The global [`Sha256`] implementation.
    pub struct Sha256Impl;

    macro sha256_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha384 trait
    #[symbol_prefix = "_embassy_crypto_sha384"]
    pub trait Sha384 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha384"), opaque(size = 640, align = 16))]
        #[opaque(size = 256, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init() -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);
    }

    /// The global [`Sha384`] implementation.
    pub struct Sha384Impl;

    macro sha384_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512_224 trait
    #[symbol_prefix = "_embassy_crypto_sha512_224"]
    pub trait Sha512_224 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha512-224"), opaque(size = 640, align = 16))]
        #[opaque(size = 256, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init() -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);
    }

    /// The global [`Sha512_224`] implementation.
    pub struct Sha512_224Impl;

    macro sha512_224_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512_256 trait
    #[symbol_prefix = "_embassy_crypto_sha512_256"]
    pub trait Sha512_256 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha512-256"), opaque(size = 640, align = 16))]
        #[opaque(size = 256, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init() -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);
    }

    /// The global [`Sha512_256`] implementation.
    pub struct Sha512_256Impl;

    macro sha512_256_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512 trait
    #[symbol_prefix = "_embassy_crypto_sha512"]
    pub trait Sha512 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-sha512"), opaque(size = 640, align = 16))]
        #[opaque(size = 256, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init() -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);
    }

    /// The global [`Sha512`] implementation.
    pub struct Sha512Impl;

    macro sha512_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha1 trait
    #[symbol_prefix = "_embassy_crypto_hmac_sha1"]
    pub trait HmacSha1 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha1"), opaque(size = 512, align = 16))]
        #[opaque(size = 350, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init(key: &[u8]) -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);

    }

    /// The global [`HmacSha1`] implementation.
    pub struct HmacSha1Impl;

    macro hmac_sha1_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha224 trait
    #[symbol_prefix = "_embassy_crypto_hmac_sha224"]
    pub trait HmacSha224 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha224"), opaque(size = 512, align = 16))]
        #[opaque(size = 350, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init(key: &[u8]) -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);

    }

    /// The global [`HmacSha224`] implementation.
    pub struct HmacSha224Impl;

    macro hmac_sha224_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha256 trait
    #[symbol_prefix = "_embassy_crypto_hmac_sha256"]
    pub trait HmacSha256 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha256"), opaque(size = 512, align = 16))]
        #[opaque(size = 350, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init(key: &[u8]) -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);

    }

    /// The global [`HmacSha256`] implementation.
    pub struct HmacSha256Impl;

    macro hmac_sha256_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha384 trait
    #[symbol_prefix = "_embassy_crypto_hmac_sha384"]
    pub trait HmacSha384 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha384"), opaque(size = 1024, align = 16))]
        #[opaque(size = 600, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init(key: &[u8]) -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);

    }

    /// The global [`HmacSha384`] implementation.
    pub struct HmacSha384Impl;

    macro hmac_sha384_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512_224 trait
    #[symbol_prefix = "_embassy_crypto_hmac_sha512_224"]
    pub trait HmacSha512_224 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha512-224"), opaque(size = 1024, align = 16))]
        #[opaque(size = 600, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init(key: &[u8]) -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);

    }

    /// The global [`HmacSha512_224`] implementation.
    pub struct HmacSha512_224Impl;

    macro hmac_sha512_224_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512_256 trait
    #[symbol_prefix = "_embassy_crypto_hmac_sha512_256"]
    pub trait HmacSha512_256 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha512-256"), opaque(size = 1024, align = 16))]
        #[opaque(size = 600, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init(key: &[u8]) -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);

    }

    /// The global [`HmacSha512_256`] implementation.
    pub struct HmacSha512_256Impl;

    macro hmac_sha512_256_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512 trait
    #[symbol_prefix = "_embassy_crypto_hmac_sha512"]
    pub trait HmacSha512 {
        /// Opaque storage for the implementation's hash state.
        #[cfg_attr(all(target_pointer_width = "32", feature = "large-hmac-sha512"), opaque(size = 1024, align = 16))]
        #[opaque(size = 600, align = 16)]
        pub type Context: Drop + Clone;

        /// Hash init
        fn init(key: &[u8]) -> Self::Context;

        /// Hash update
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        fn finalize(ctx: Self::Context, data: &mut [u8]);

    }

    /// The global [`HmacSha512`] implementation.
    pub struct HmacSha512Impl;

    macro hmac_sha512_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-128 ECB block cipher trait.
    #[symbol_prefix = "_embassy_crypto_aes128ecb"]
    pub trait Aes128Ecb {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 384, align = 16))]
        #[opaque(size = 1024, align = 16)]
        pub type Context: Drop + Clone;

        /// Initialize with a 128-bit key.
        fn init(key: &[u8; 16]) -> Self::Context;

        /// Encrypt 16-byte blocks in-place.
        fn encrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks in-place.
        fn decrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes128Ecb`] implementation.
    pub struct Aes128EcbImpl;

    macro aes128ecb_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-256 ECB block cipher trait.
    #[symbol_prefix = "_embassy_crypto_aes256ecb"]
    pub trait Aes256Ecb {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 512, align = 16))]
        #[opaque(size = 1024, align = 16)]
        pub type Context: Drop + Clone;

        /// Initialize with a 256-bit key.
        fn init(key: &[u8; 32]) -> Self::Context;

        /// Encrypt 16-byte blocks in-place.
        fn encrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks in-place.
        fn decrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes256Ecb`] implementation.
    pub struct Aes256EcbImpl;

    macro aes256ecb_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-128 CBC block cipher trait.
    #[symbol_prefix = "_embassy_crypto_aes128cbc"]
    pub trait Aes128Cbc {
        /// Opaque storage for the encryptor's key schedule and chaining state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 400, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type EncryptContext: Drop;

        /// Opaque storage for the decryptor's key schedule and chaining state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 400, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type DecryptContext: Drop;

        /// Initialize encryptor with a 128-bit key and 128-bit IV.
        fn encrypt_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::EncryptContext;

        /// Initialize decryptor with a 128-bit key and 128-bit IV.
        fn decrypt_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::DecryptContext;

        /// Encrypt 16-byte blocks in-place (updates internal chaining state).
        fn encrypt_blocks(ctx: &mut Self::EncryptContext, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks in-place (updates internal chaining state).
        fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes128Cbc`] implementation.
    pub struct Aes128CbcImpl;

    macro aes128cbc_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-256 CBC block cipher trait.
    #[symbol_prefix = "_embassy_crypto_aes256cbc"]
    pub trait Aes256Cbc {
        /// Opaque storage for the encryptor's key schedule and chaining state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 528, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type EncryptContext: Drop;

        /// Opaque storage for the decryptor's key schedule and chaining state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 528, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type DecryptContext: Drop;

        /// Initialize encryptor with a 256-bit key and 128-bit IV.
        fn encrypt_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::EncryptContext;

        /// Initialize decryptor with a 256-bit key and 128-bit IV.
        fn decrypt_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::DecryptContext;

        /// Encrypt 16-byte blocks in-place (updates internal chaining state).
        fn encrypt_blocks(ctx: &mut Self::EncryptContext, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks in-place (updates internal chaining state).
        fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes256Cbc`] implementation.
    pub struct Aes256CbcImpl;

    macro aes256cbc_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-128 GCM AEAD trait.
    #[symbol_prefix = "_embassy_crypto_aes128gcm"]
    pub trait Aes128Gcm {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 450, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Drop + Clone;

        /// Initialize with a 128-bit key.
        fn init(key: &[u8; 16]) -> Self::Context;

        /// Encrypt plaintext in-place and produce a 16-byte authentication tag.
        fn encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8; 16],
        ) -> Result<(), CryptoError>;

        /// Decrypt ciphertext in-place and verify a 16-byte authentication tag.
        fn decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8; 16],
        ) -> Result<(), CryptoError>;
    }

    /// The global [`Aes128Gcm`] implementation.
    pub struct Aes128GcmImpl;

    macro aes128gcm_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-256 GCM AEAD trait.
    #[symbol_prefix = "_embassy_crypto_aes256gcm"]
    pub trait Aes256Gcm {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 540, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Drop + Clone;

        /// Initialize with a 256-bit key.
        fn init(key: &[u8; 32]) -> Self::Context;

        /// Encrypt plaintext in-place and produce a 16-byte authentication tag.
        fn encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8; 16],
        ) -> Result<(), CryptoError>;

        /// Decrypt ciphertext in-place and verify a 16-byte authentication tag.
        fn decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8; 16],
        ) -> Result<(), CryptoError>;
    }

    /// The global [`Aes256Gcm`] implementation.
    pub struct Aes256GcmImpl;

    macro aes256gcm_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-128 CCM AEAD trait.
    ///
    /// The tag and nonce sizes are validated at runtime by the HAL.
    #[symbol_prefix = "_embassy_crypto_aes128ccm"]
    pub trait Aes128Ccm {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 512, align = 16))]
        #[opaque(size = 1024, align = 16)]
        pub type Context: Drop + Clone;

        /// Initialize with a 128-bit key.
        fn init(key: &[u8; 16]) -> Self::Context;

        /// Encrypt plaintext in-place and produce an authentication tag.
        /// The tag length is determined by the length of the `tag` slice.
        fn encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8],
        ) -> Result<(), CryptoError>;

        /// Decrypt ciphertext in-place and verify an authentication tag.
        /// The tag length is determined by the length of the `tag` slice.
        fn decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8],
        ) -> Result<(), CryptoError>;
    }

    /// The global [`Aes128Ccm`] implementation.
    pub struct Aes128CcmImpl;

    macro aes128ccm_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-256 CCM AEAD trait.
    ///
    /// The tag and nonce sizes are validated at runtime by the HAL.
    #[symbol_prefix = "_embassy_crypto_aes256ccm"]
    pub trait Aes256Ccm {
        /// Opaque storage for the implementation's key schedule.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 512, align = 16))]
        #[opaque(size = 1024, align = 16)]
        pub type Context: Drop + Clone;

        /// Initialize with a 256-bit key.
        fn init(key: &[u8; 32]) -> Self::Context;

        /// Encrypt plaintext in-place and produce an authentication tag.
        /// The tag length is determined by the length of the `tag` slice.
        fn encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8],
        ) -> Result<(), CryptoError>;

        /// Decrypt ciphertext in-place and verify an authentication tag.
        /// The tag length is determined by the length of the `tag` slice.
        fn decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8],
        ) -> Result<(), CryptoError>;
    }

    /// The global [`Aes256Ccm`] implementation.
    pub struct Aes256CcmImpl;

    macro aes256ccm_impl(path = $crate);
}

// ===========================================================================
// AES-128 CTR stream cipher trait
// ===========================================================================

unitrait::unitrait! {
    /// AES-128 CTR stream cipher trait.
    ///
    /// CTR mode turns a block cipher into a synchronous stream cipher.
    /// Encryption and decryption are identical: XOR data with the AES-ECB
    /// encrypted counter keystream. The counter is a 128-bit big-endian integer
    /// incremented after each block, matching NIST SP 800-38A.
    #[symbol_prefix = "_embassy_crypto_aes128ctr"]
    pub trait Aes128Ctr {
        /// Opaque storage for key schedule, counter state, and partial-block buffer.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 432, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Drop;

        /// Initialize with a 128-bit key and 128-bit initial counter (IV).
        fn init(key: &[u8; 16], iv: &[u8; 16]) -> Self::Context;

        /// Apply keystream to `buf` in-place (encrypt == decrypt for CTR).
        fn apply_keystream(ctx: &mut Self::Context, buf: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes128Ctr`] implementation.
    pub struct Aes128CtrImpl;

    macro aes128ctr_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-256 CTR stream cipher trait.
    ///
    /// See [`Aes128Ctr`] for CTR mode semantics. Uses a 256-bit key.
    #[symbol_prefix = "_embassy_crypto_aes256ctr"]
    pub trait Aes256Ctr {
        #[cfg_attr(target_pointer_width = "32", opaque(size = 560, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Drop;

        fn init(key: &[u8; 32], iv: &[u8; 16]) -> Self::Context;

        fn apply_keystream(ctx: &mut Self::Context, buf: InOutBuf<'_, '_, u8>);
    }

    /// The global [`Aes256Ctr`] implementation.
    pub struct Aes256CtrImpl;

    macro aes256ctr_impl(path = $crate);
}

// ===========================================================================
// AES-128 CMAC trait
// ===========================================================================

unitrait::unitrait! {
    /// AES-128 CMAC (Cipher-based Message Authentication Code) trait.
    ///
    /// Produces a 128-bit authentication tag using AES-128 as the underlying
    /// block cipher (NIST SP 800-38B).
    #[symbol_prefix = "_embassy_crypto_aes128cmac"]
    pub trait Aes128Cmac {
        /// Opaque storage for the implementation's CMAC state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 432, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Drop + Clone;

        /// Initialize with a 128-bit key.
        fn init(key: &[u8; 16]) -> Self::Context;

        /// Update the CMAC state with message data.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finalize and write the 16-byte tag to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 16]);

        /// Reset the context to its post-init, pre-message state.
        fn reset(ctx: &mut Self::Context);
    }

    /// The global [`Aes128Cmac`] implementation.
    pub struct Aes128CmacImpl;

    macro aes128cmac_impl(path = $crate);
}

// ===========================================================================
// AES-256 CMAC trait
// ===========================================================================

unitrait::unitrait! {
    /// AES-256 CMAC (Cipher-based Message Authentication Code) trait.
    ///
    /// Produces a 128-bit authentication tag using AES-256 as the underlying
    /// block cipher (NIST SP 800-38B).
    #[symbol_prefix = "_embassy_crypto_aes256cmac"]
    pub trait Aes256Cmac {
        /// Opaque storage for the implementation's CMAC state.
        #[cfg_attr(target_pointer_width = "32", opaque(size = 560, align = 16))]
        #[opaque(size = 1280, align = 16)]
        pub type Context: Drop + Clone;

        /// Initialize with a 256-bit key.
        fn init(key: &[u8; 32]) -> Self::Context;

        /// Update the CMAC state with message data.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Finalize and write the 16-byte tag to `out`.
        fn finalize(ctx: Self::Context, out: &mut [u8; 16]);

        /// Reset the context to its post-init, pre-message state.
        fn reset(ctx: &mut Self::Context);
    }

    /// The global [`Aes256Cmac`] implementation.
    pub struct Aes256CmacImpl;

    macro aes256cmac_impl(path = $crate);
}

// ===================================================================
// P-256 scalar multiplication
// ===================================================================

/// Canonical P-256 scalar: big-endian, 32 bytes.
///
/// This is the portable representation that crosses the backend boundary.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct P256Scalar(pub [u8; 32]);

/// Canonical P-256 affine point: big-endian (x, y), uncompressed.
///
/// This is the portable representation that crosses the backend boundary.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct P256AffinePoint {
    /// X coordinate, big-endian, 32 bytes.
    pub x: [u8; 32],
    /// Y coordinate, big-endian, 32 bytes.
    pub y: [u8; 32],
}

unitrait::unitrait! {
    /// P-256 scalar multiplication accelerator.
    ///
    /// Scalar multiplication dominates the cost of every P-256 protocol
    /// operation (ECDH, ECDSA sign/verify, key generation) by orders of
    /// magnitude, so this is the one hook a backend needs to provide for
    /// hardware acceleration. Everything else (scalar field arithmetic,
    /// point addition, encoding) stays in software, in `embassy-crypto`.
    ///
    /// Input and output are canonical big-endian byte arrays; conversion to
    /// and from whatever the backend uses internally happens inside the
    /// implementation.
    ///
    /// ## Contract
    ///
    /// The caller guarantees:
    ///
    /// - `k` is in the range `[1, n-1]`, where `n` is the curve order.
    /// - `p` is a valid, on-curve affine point (and therefore not the identity,
    ///   which has no affine encoding).
    ///
    /// Given the above, the result is never the identity, since P-256 has
    /// prime order. The implementation must not be given secret-dependent
    /// timing: `k` (and, for `mul_affine`, `p`) may be secret.
    #[symbol_prefix = "_embassy_crypto_p256_scalar_mul"]
    pub trait P256ScalarMul {
        /// Fixed-base scalar multiplication: `k * G`.
        fn mul_base(k: P256Scalar) -> P256AffinePoint;

        /// Variable-base scalar multiplication: `k * P`.
        fn mul_affine(k: P256Scalar, p: P256AffinePoint) -> P256AffinePoint;
    }

    /// The global [`P256ScalarMul`] implementation.
    pub struct P256ScalarMulImpl;

    macro p256_scalar_mul_impl(path = $crate);
}

unitrait::unitrait! {
    /// P-256 scalar field inversion accelerator (optional).
    ///
    /// ECDSA signing and verification each need one inversion modulo the curve
    /// order. `embassy-crypto` routes to this trait only when its
    /// `p256-scalar-invert` feature is enabled.
    ///
    /// ## Contract
    ///
    /// The caller guarantees `k` is in the range `[1, n-1]`, where `n` is the
    /// curve order, so the inverse always exists. The result is canonical,
    /// in the same range.
    #[symbol_prefix = "_embassy_crypto_p256_scalar_invert"]
    pub trait P256ScalarInvert {
        /// `k^-1 mod n`. Must not have secret-dependent timing: `k` may be secret.
        fn invert(k: P256Scalar) -> P256Scalar;

        /// `k^-1 mod n`, variable time. Callers only pass public values.
        fn invert_vartime(k: P256Scalar) -> P256Scalar;
    }

    /// The global [`P256ScalarInvert`] implementation.
    pub struct P256ScalarInvertImpl;

    macro p256_scalar_invert_impl(path = $crate);
}

unitrait::unitrait! {
    /// P-256 double-base scalar multiplication accelerator (optional).
    ///
    /// Computes `k1 * p1 + k2 * p2` in one operation, which is what ECDSA
    /// verification needs; backends can use a combined ladder (Shamir's trick).
    ///
    /// ## Contract
    ///
    /// The caller guarantees `k1` and `k2` are in `[1, n-1]` and `p1`, `p2` are
    /// valid on-curve affine points. The sum can be the identity (when
    /// `k1 * p1 == -(k2 * p2)`), which has no affine encoding; the
    /// implementation returns `None` in that case.
    ///
    /// Must not have secret-dependent timing with respect to the scalars and
    /// points. Whether the result is the identity is not treated as secret.
    #[symbol_prefix = "_embassy_crypto_p256_lincomb"]
    pub trait P256Lincomb {
        /// `k1 * p1 + k2 * p2`, or `None` if the sum is the identity.
        fn lincomb(
            k1: P256Scalar,
            p1: P256AffinePoint,
            k2: P256Scalar,
            p2: P256AffinePoint,
        ) -> Option<P256AffinePoint>;
    }

    /// The global [`P256Lincomb`] implementation.
    pub struct P256LincombImpl;

    macro p256_lincomb_impl(path = $crate);
}
