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
// P-256 Elliptic Curve Traits
// ===================================================================

/// Canonical P-256 scalar: big-endian, 32 bytes, range [1, n-1].
///
/// This is the portable representation that crosses backend boundaries.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct P256Scalar(pub [u8; 32]);

/// Canonical P-256 affine point: big-endian (x, y), uncompressed.
///
/// This is the portable representation that crosses backend boundaries.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct P256AffinePoint {
    /// X coordinate, big-endian, 32 bytes.
    pub x: [u8; 32],
    /// Y coordinate, big-endian, 32 bytes.
    pub y: [u8; 32],
}

// ------------------------------------------------------------------
// Tier 1: P256ScalarMul — Hardware accelerator hook
// ------------------------------------------------------------------

unitrait::unitrait! {
    /// Scalar multiplication accelerator trait.
    ///
    /// Backends that only have hardware for `k * G` or `k * P` implement this.
    /// Input and output are canonical byte arrays — conversion happens inside
    /// the implementation, which is fine because scalar multiplication dominates
    /// the cost by orders of magnitude.
    pub trait P256ScalarMul {
        /// Fixed-base scalar multiplication: `k * G`.
        #[symbol = "_emb_crypto_p256_mul_base"]
        fn mul_base(k: &P256Scalar) -> P256AffinePoint;

        /// Variable-base scalar multiplication: `k * P`.
        #[symbol = "_emb_crypto_p256_mul_affine"]
        fn mul_affine(k: &P256Scalar, p: &P256AffinePoint) -> P256AffinePoint;
    }

    /// The global [`P256ScalarMul`] implementation.
    pub struct P256ScalarMulImpl;

    macro embassy_crypto_p256_scalar_mul_impl(path = $crate);
}

// ------------------------------------------------------------------
// Tier 2: P256ScalarOps — Full low-level backend
// ------------------------------------------------------------------

unitrait::unitrait! {
    /// Full P-256 backend: scalar field arithmetic mod n, and short-Weierstrass
    /// point arithmetic. One global implementation per program, selected at
    /// link time via `embassy_crypto_p256_ops_impl!(Backend)`.
    ///
    /// ## Global contracts (apply to every function)
    ///
    /// - **No panics, no UB.** Functions are total over their documented
    ///   preconditions; where a precondition is violated, the *result value* is
    ///   the defined fallback and is always safe to drop or re-convert.
    /// - **Stateless and reentrant.** No mutable global state; calls may be
    ///   interleaved freely (interrupt-safe on embedded targets).
    /// - **Platform independence.** Only `bool`, fixed byte arrays, and the opaque
    ///   blobs appear in signatures. Backends adapt to C/assembly internally;
    ///   C `_Bool`/endianness concerns never cross this boundary.
    /// - **Constant-time (CT)**: execution time and memory access are independent
    ///   of secret operand *values*. Functions documented as variable-time, or with
    ///   public-only inputs, must never receive secrets.
    /// - Scalar inputs are whatever the backend prefers internally (Montgomery,
    ///   redundant, ...); callers can rely only on the documented conversions.
    ///
    /// ## Validity via predicates (the tuple workaround)
    ///
    /// - Inverse validity: `scalar_inv(a)` returns `a^-1 mod n`, or **`0` if
    ///   `a == 0`** (defined fallback). Because `a != 0` exactly when the inverse
    ///   exists, `!scalar_is_zero(a)` is the complete validity check.
    /// - Decode validity: `point_from_canonical(p)` returns the point, or the
    ///   **identity** if `p` is off-curve / out of range (defined fallback). A
    ///   valid affine input is *never* the identity (the identity has no affine
    ///   encoding), so `projective_is_identity(point_from_canonical(p))` is false
    ///   exactly when `p` is valid. One predicate serves both purposes.
    /// - Encode disambiguation: `point_to_canonical(p)` returns the real affine
    ///   coordinates, or **`(0, 0)` if `p` is the identity`** (defined fallback).
    ///   Callers must use `projective_is_identity(p)` to distinguish — never
    ///   compare coordinates against `(0, 0)`.
    pub trait P256Ops {
        /// Opaque scalar (backend-specific representation; must hold values in
        /// `[0, n-1]`). Any bit pattern of the storage is a valid value.
        ///
        /// `Send + Sync + Unpin` in addition to `Copy`: the adapter's field/
        /// group types (`ff::Field`, `group::Group`) require `Send + Sync`, so
        /// backends must use plain thread-safe data (the RustCrypto backend
        /// does; anything with interior mutability must synchronize).
        #[cfg_attr(target_pointer_width = "64", opaque(size = 64, align = 16))]
        #[cfg_attr(target_pointer_width = "32", opaque(size = 64, align = 16))]
        pub type Scalar: Copy + Send + Sync + Unpin;

        /// Opaque projective point (backend-specific representation).
        /// `Send + Sync + Unpin` alongside `Copy` — same rationale as `Scalar`.
        #[cfg_attr(target_pointer_width = "64", opaque(size = 128, align = 16))]
        #[cfg_attr(target_pointer_width = "32", opaque(size = 128, align = 16))]
        pub type ProjectivePoint: Copy + Send + Sync + Unpin;

        // ------------------------------------------------------------------
        // Conversions and clones (the only canonical<->opaque crossing points)
        // ------------------------------------------------------------------

        /// Decode a canonical scalar (big-endian, MUST be in `[0, n-1]`; zero
        /// is valid and MUST be accepted). Out-of-range input: result is
        /// unspecified, must not panic. Callers that cannot guarantee the range
        /// must use `scalar_reduce_32bytes` instead.
        #[symbol = "_emb_crypto_p256_scalar_from_canonical"]
        fn scalar_from_canonical(s: &P256Scalar) -> Self::Scalar;

        /// Encode to canonical form (big-endian, always in `[0, n-1]`).
        /// Constant-time with respect to the scalar.
        #[symbol = "_emb_crypto_p256_scalar_to_canonical"]
        fn scalar_to_canonical(s: &Self::Scalar) -> P256Scalar;

        /// Decode a canonical affine point (big-endian x, y).
        ///
        /// If `(x, y)` is a valid, on-curve, in-range affine point, returns it.
        /// Otherwise — off-curve, out of range, or `(0, 0)` — returns the
        /// **identity** (defined fallback). Validity of the input is queried
        /// with `projective_is_identity` on the result (see "Validity via
        /// predicates" above). Not required to be constant-time (inputs are
        /// public in all intended uses: key parsing, verification).
        #[symbol = "_emb_crypto_p256_point_from_canonical"]
        fn point_from_canonical(p: &P256AffinePoint) -> Self::ProjectivePoint;

        /// Encode to canonical affine form.
        ///
        /// Non-identity points yield their real coordinates. The identity
        /// yields `(0, 0)` (defined fallback) — test `projective_is_identity`
        /// to distinguish, never the coordinates. Constant-time with respect
        /// to the point.
        #[symbol = "_emb_crypto_p256_point_to_canonical"]
        fn point_to_canonical(p: &Self::ProjectivePoint) -> P256AffinePoint;

        // ------------------------------------------------------------------
        // Scalar predicates
        // ------------------------------------------------------------------

        /// True iff the scalar is `0 mod n`. Constant-time with respect to the
        /// scalar. Combined with the defined fallbacks of `scalar_inv` /
        /// `scalar_inv_vartime`, this is the inverse-validity check.
        #[symbol = "_emb_crypto_p256_scalar_is_zero"]
        fn scalar_is_zero(a: &Self::Scalar) -> bool;

        // ------------------------------------------------------------------
        // Scalar field arithmetic mod n
        //
        // Precondition for every operand: a value in `[0, n-1]`, i.e. produced
        // by any of these functions, by `scalar_from_canonical` with in-range
        // input, or by `scalar_reduce_32bytes`. All results are in `[0, n-1]`.
        // ------------------------------------------------------------------

        /// `a + b mod n`. Constant-time with respect to both operands.
        #[symbol = "_emb_crypto_p256_scalar_add"]
        fn scalar_add(a: &Self::Scalar, b: &Self::Scalar) -> Self::Scalar;

        /// `a * b mod n`. Constant-time with respect to both operands.
        #[symbol = "_emb_crypto_p256_scalar_mul"]
        fn scalar_mul(a: &Self::Scalar, b: &Self::Scalar) -> Self::Scalar;

        /// `-a mod n` (`0` maps to `0`). Constant-time.
        #[symbol = "_emb_crypto_p256_scalar_neg"]
        fn scalar_neg(a: &Self::Scalar) -> Self::Scalar;

        /// Constant-time modular inverse: `a^-1 mod n`, or **`0` if `a == 0`**
        /// (defined fallback — zero has no inverse; test `scalar_is_zero(a)`
        /// for validity). Constant-time with respect to `a`.
        #[symbol = "_emb_crypto_p256_scalar_inv"]
        fn scalar_inv(a: &Self::Scalar) -> Self::Scalar;

        /// Variable-time modular inverse; identical contract to `scalar_inv`.
        /// MUST NOT be called with secret inputs.
        #[symbol = "_emb_crypto_p256_scalar_inv_vartime"]
        fn scalar_inv_vartime(a: &Self::Scalar) -> Self::Scalar;

        /// Reduce an arbitrary 32-byte big-endian integer modulo n.
        /// Result in `[0, n-1]`. Constant-time.
        #[symbol = "_emb_crypto_p256_scalar_reduce_32bytes"]
        fn scalar_reduce_bytes(bytes: &[u8; 32]) -> Self::Scalar;

        // ------------------------------------------------------------------
        // Projective point predicates
        // ------------------------------------------------------------------

        /// True iff the point is the identity. Constant-time with respect to
        /// the point. Doubles as the validity check for `point_from_canonical`
        /// output (see "Validity via predicates" above) and backs
        /// `Group::is_identity`.
        #[symbol = "_emb_crypto_p256_projective_is_identity"]
        fn projective_is_identity(p: &Self::ProjectivePoint) -> bool;

        // ------------------------------------------------------------------
        // Projective point arithmetic
        //
        // "Complete" means correct for ALL inputs: identity, P == Q, and
        // P == -Q. Backends whose fastest native formulas are incomplete must
        // dispatch the exceptional cases (identity/equality tests are cheap;
        // only the exceptional path is slower).
        // ------------------------------------------------------------------

        /// The identity (point at infinity).
        #[symbol = "_emb_crypto_p256_projective_identity"]
        fn projective_identity() -> Self::ProjectivePoint;

        /// The base point G (SECG secp256r1, FIPS 186-4).
        #[symbol = "_emb_crypto_p256_projective_generator"]
        fn projective_generator() -> Self::ProjectivePoint;

        /// COMPLETE addition `a + b`. Constant-time.
        #[symbol = "_emb_crypto_p256_projective_add"]
        fn projective_add(a: &Self::ProjectivePoint, b: &Self::ProjectivePoint) -> Self::ProjectivePoint;

        /// COMPLETE subtraction `a - b = a + (-b)`. Constant-time.
        #[symbol = "_emb_crypto_p256_projective_sub"]
        fn projective_sub(a: &Self::ProjectivePoint, b: &Self::ProjectivePoint) -> Self::ProjectivePoint;

        /// COMPLETE doubling `2 * p` (`2 * identity == identity`). Constant-time.
        #[symbol = "_emb_crypto_p256_projective_double"]
        fn projective_double(p: &Self::ProjectivePoint) -> Self::ProjectivePoint;

        /// Fixed-base scalar multiplication `k * G`, for `k` in `[0, n-1]`;
        /// `k == 0` yields the identity. Constant-time with respect to `k`.
        #[symbol = "_emb_crypto_p256_scalar_mul_base"]
        fn scalar_mul_base(k: &Self::Scalar) -> Self::ProjectivePoint;

        /// Variable-base scalar multiplication `k * P`, for `k` in `[0, n-1]`
        /// and ANY projective point `P` (including the identity, which yields
        /// the identity). Constant-time with respect to `k` AND `P` (both may
        /// be secret: ECDH).
        #[symbol = "_emb_crypto_p256_scalar_mul_projective"]
        fn scalar_mul_projective(k: &Self::Scalar, p: &Self::ProjectivePoint) -> Self::ProjectivePoint;

        /// Simultaneous double-scalar multiplication: `k1 * p1 + k2 * p2`.
        ///
        /// Backends with a native joint multiplication SHOULD override this.
        /// The default composes two generic multiplications plus an add and is
        /// ~2x slower — it exists so backends without a joint primitive still
        /// satisfy the trait.
        ///
        /// Defined fallbacks (matching the rest of this contract):
        /// `k_i == 0` contributes nothing (result is the other term);
        /// identity operands follow the `scalar_mul_projective` contract.
        /// Constant-time with respect to both scalars.
        #[symbol = "_emb_crypto_p256_projective_lincomb"]
        fn projective_lincomb(
            k1: &Self::Scalar,
            p1: &Self::ProjectivePoint,
            k2: &Self::Scalar,
            p2: &Self::ProjectivePoint,
        ) -> Self::ProjectivePoint;
    }

    /// The global [`P256Ops`] implementation.
    pub struct P256OpsImpl;

    macro embassy_crypto_p256_ops_impl(path = $crate);
}

// ------------------------------------------------------------------
// Tier 3: P256Ecd — High-level protocol operations for TLS/BLE
// ------------------------------------------------------------------

unitrait::unitrait! {
    /// High-level ECDSA/ECDH operations for TLS and BLE.
    ///
    /// This trait is what TLS/BLE protocol code uses. It can be implemented
    /// **natively** on a full backend (Cortex-M4 calls `p256_sign` directly)
    /// or **composed** on a partial backend (ESP32 uses `P256ScalarMul` for
    /// the expensive `k*G` step + software for the rest).
    pub trait P256Ecd {
        /// Derive the public key `Q = d * G` from a private key.
        #[symbol = "_emb_crypto_p256_public_key_from_private"]
        fn public_key_from_private(d: &P256Scalar) -> P256AffinePoint;

        /// ECDSA sign a prehashed message.
        #[symbol = "_emb_crypto_p256_ecdsa_sign"]
        fn ecdsa_sign(d: &P256Scalar, z: &P256Scalar, k: &P256Scalar)
        -> (P256Scalar, P256Scalar);

        /// ECDSA verify a signature on a prehashed message.
        #[symbol = "_emb_crypto_p256_ecdsa_verify"]
        fn ecdsa_verify(
            q: &P256AffinePoint,
            z: &P256Scalar,
            r: &P256Scalar,
            s: &P256Scalar,
        ) -> bool;

        /// ECDH shared secret derivation.
        #[symbol = "_emb_crypto_p256_ecdh_shared_secret"]
        fn ecdh_shared_secret(d: &P256Scalar, q: &P256AffinePoint) -> [u8; 32];
    }

    /// The global [`P256Ops`] implementation.
    pub struct P256EcdImpl;

    macro embassy_crypto_p256_ecd_impl(path = $crate);
}
