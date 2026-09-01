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
    pub trait Md5 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 256, align = 16)]
        #[symbol = "_emb_crypto_md5_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_md5_init"]
        pub fn md5_init() -> Self::Context;

        /// Hash init
        #[symbol = "_emb_crypto_md5_clone"]
        pub fn md5_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_md5_update"]
        pub fn md5_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_md5_finalize"]
        pub fn md5_finalize(ctx: Self::Context, data: &mut [u8]);
    }

    macro embassy_crypto_md5_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha1 trait
    pub trait Sha1 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 256, align = 16)]
        #[symbol = "_emb_crypto_sha1_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha1_init"]
        pub fn sha1_init() -> Self::Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha1_clone"]
        pub fn sha1_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_sha1_update"]
        pub fn sha1_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_sha1_finalize"]
        pub fn sha1_finalize(ctx: Self::Context, data: &mut [u8]);
    }

    macro embassy_crypto_sha1_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha224 trait
    pub trait Sha224 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 256, align = 16)]
        #[symbol = "_emb_crypto_sha224_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha224_init"]
        pub fn sha224_init() -> Self::Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha224_clone"]
        pub fn sha224_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_sha224_update"]
        pub fn sha224_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_sha224_finalize"]
        pub fn sha224_finalize(ctx: Self::Context, data: &mut [u8]);
    }

    macro embassy_crypto_sha224_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha256 trait
    pub trait Sha256 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 256, align = 16)]
        #[symbol = "_emb_crypto_sha256_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha256_init"]
        pub fn sha256_init() -> Self::Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha256_clone"]
        pub fn sha256_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_sha256_update"]
        pub fn sha256_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_sha256_finalize"]
        pub fn sha256_finalize(ctx: Self::Context, data: &mut [u8]);
    }

    macro embassy_crypto_sha256_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha384 trait
    pub trait Sha384 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 640, align = 16)]
        #[symbol = "_emb_crypto_sha384_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha384_init"]
        pub fn sha384_init() -> Self::Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha384_clone"]
        pub fn sha384_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_sha384_update"]
        pub fn sha384_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_sha384_finalize"]
        pub fn sha384_finalize(ctx: Self::Context, data: &mut [u8]);
    }

    macro embassy_crypto_sha384_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512_224 trait
    pub trait Sha512_224 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 640, align = 16)]
        #[symbol = "_emb_crypto_sha512_224_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha512_224_init"]
        pub fn sha512_224_init() -> Self::Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha512_224_clone"]
        pub fn sha512_224_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_sha512_224_update"]
        pub fn sha512_224_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_sha512_224_finalize"]
        pub fn sha512_224_finalize(ctx: Self::Context, data: &mut [u8]);
    }

    macro embassy_crypto_sha512_224_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512_256 trait
    pub trait Sha512_256 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 640, align = 16)]
        #[symbol = "_emb_crypto_sha512_256_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha512_256_init"]
        pub fn sha512_256_init() -> Self::Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha512_256_clone"]
        pub fn sha512_256_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_sha512_256_update"]
        pub fn sha512_256_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_sha512_256_finalize"]
        pub fn sha512_256_finalize(ctx: Self::Context, data: &mut [u8]);
    }

    macro embassy_crypto_sha512_256_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512 trait
    pub trait Sha512 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 640, align = 16)]
        #[symbol = "_emb_crypto_sha512_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha512_init"]
        pub fn sha512_init() -> Self::Context;

        /// Hash init
        #[symbol = "_emb_crypto_sha512_clone"]
        pub fn sha512_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_sha512_update"]
        pub fn sha512_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_sha512_finalize"]
        pub fn sha512_finalize(ctx: Self::Context, data: &mut [u8]);
    }

    macro embassy_crypto_sha512_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha1 trait
    pub trait HmacSha1 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 512, align = 16)]
        #[symbol = "_emb_crypto_hmac_sha1_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_hmac_sha1_init"]
        pub fn hmac_sha1_init(key: &[u8]) -> Self::Context;

        /// Hash clone
        #[symbol = "_emb_crypto_hmac_sha1_clone"]
        pub fn hmac_sha1_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_hmac_sha1_update"]
        pub fn hmac_sha1_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_hmac_sha1_finalize"]
        pub fn hmac_sha1_finalize(ctx: Self::Context, data: &mut [u8]);

        /// Hash reset – restores the context to its post-init, pre-message state.
        #[symbol = "_emb_crypto_hmac_sha1_reset"]
        pub fn hmac_sha1_reset(ctx: &mut Self::Context);
    }

    macro embassy_crypto_hmac_sha1_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha224 trait
    pub trait HmacSha224 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 512, align = 16)]
        #[symbol = "_emb_crypto_hmac_sha224_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_hmac_sha224_init"]
        pub fn hmac_sha224_init(key: &[u8]) -> Self::Context;

        /// Hash clone
        #[symbol = "_emb_crypto_hmac_sha224_clone"]
        pub fn hmac_sha224_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_hmac_sha224_update"]
        pub fn hmac_sha224_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_hmac_sha224_finalize"]
        pub fn hmac_sha224_finalize(ctx: Self::Context, data: &mut [u8]);

        /// Hash reset – restores the context to its post-init, pre-message state.
        #[symbol = "_emb_crypto_hmac_sha224_reset"]
        pub fn hmac_sha224_reset(ctx: &mut Self::Context);
    }

    macro embassy_crypto_hmac_sha224_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha256 trait
    pub trait HmacSha256 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 512, align = 16)]
        #[symbol = "_emb_crypto_hmac_sha256_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_hmac_sha256_init"]
        pub fn hmac_sha256_init(key: &[u8]) -> Self::Context;

        /// Hash clone
        #[symbol = "_emb_crypto_hmac_sha256_clone"]
        pub fn hmac_sha256_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_hmac_sha256_update"]
        pub fn hmac_sha256_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_hmac_sha256_finalize"]
        pub fn hmac_sha256_finalize(ctx: Self::Context, data: &mut [u8]);

        /// Hash reset – restores the context to its post-init, pre-message state.
        #[symbol = "_emb_crypto_hmac_sha256_reset"]
        pub fn hmac_sha256_reset(ctx: &mut Self::Context);
    }

    macro embassy_crypto_hmac_sha256_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha384 trait
    pub trait HmacSha384 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_hmac_sha384_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_hmac_sha384_init"]
        pub fn hmac_sha384_init(key: &[u8]) -> Self::Context;

        /// Hash clone
        #[symbol = "_emb_crypto_hmac_sha384_clone"]
        pub fn hmac_sha384_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_hmac_sha384_update"]
        pub fn hmac_sha384_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_hmac_sha384_finalize"]
        pub fn hmac_sha384_finalize(ctx: Self::Context, data: &mut [u8]);

        /// Hash reset – restores the context to its post-init, pre-message state.
        #[symbol = "_emb_crypto_hmac_sha384_reset"]
        pub fn hmac_sha384_reset(ctx: &mut Self::Context);
    }

    macro embassy_crypto_hmac_sha384_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512_224 trait
    pub trait HmacSha512_224 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_hmac_sha512_224_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_hmac_sha512_224_init"]
        pub fn hmac_sha512_224_init(key: &[u8]) -> Self::Context;

        /// Hash clone
        #[symbol = "_emb_crypto_hmac_sha512_224_clone"]
        pub fn hmac_sha512_224_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_hmac_sha512_224_update"]
        pub fn hmac_sha512_224_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_hmac_sha512_224_finalize"]
        pub fn hmac_sha512_224_finalize(ctx: Self::Context, data: &mut [u8]);

        /// Hash reset – restores the context to its post-init, pre-message state.
        #[symbol = "_emb_crypto_hmac_sha512_224_reset"]
        pub fn hmac_sha512_224_reset(ctx: &mut Self::Context);
    }

    macro embassy_crypto_hmac_sha512_224_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512_256 trait
    pub trait HmacSha512_256 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_hmac_sha512_256_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_hmac_sha512_256_init"]
        pub fn hmac_sha512_256_init(key: &[u8]) -> Self::Context;

        /// Hash clone
        #[symbol = "_emb_crypto_hmac_sha512_256_clone"]
        pub fn hmac_sha512_256_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_hmac_sha512_256_update"]
        pub fn hmac_sha512_256_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_hmac_sha512_256_finalize"]
        pub fn hmac_sha512_256_finalize(ctx: Self::Context, data: &mut [u8]);

        /// Hash reset – restores the context to its post-init, pre-message state.
        #[symbol = "_emb_crypto_hmac_sha512_256_reset"]
        pub fn hmac_sha512_256_reset(ctx: &mut Self::Context);
    }

    macro embassy_crypto_hmac_sha512_256_impl(path = $crate);
}

unitrait::unitrait! {
    /// Sha512 trait
    pub trait HmacSha512 {
        /// Opaque storage for the implementation's hash state.
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_hmac_sha512_context"]
        pub type Context;

        /// Hash init
        #[symbol = "_emb_crypto_hmac_sha512_init"]
        pub fn hmac_sha512_init(key: &[u8]) -> Self::Context;

        /// Hash clone
        #[symbol = "_emb_crypto_hmac_sha512_clone"]
        pub fn hmac_sha512_clone(ctx: &Self::Context) -> Self::Context;

        /// Hash update
        #[symbol = "_emb_crypto_hmac_sha512_update"]
        pub fn hmac_sha512_update(ctx: &mut Self::Context, data: &[u8]);

        /// Hash finalize
        #[symbol = "_emb_crypto_hmac_sha512_finalize"]
        pub fn hmac_sha512_finalize(ctx: Self::Context, data: &mut [u8]);

        /// Hash reset – restores the context to its post-init, pre-message state.
        #[symbol = "_emb_crypto_hmac_sha512_reset"]
        pub fn hmac_sha512_reset(ctx: &mut Self::Context);
    }

    macro embassy_crypto_hmac_sha512_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-128 ECB block cipher trait.
    pub trait Aes128Ecb {
        /// Opaque storage for the implementation's key schedule.
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_aes128ecb_context"]
        pub type Context;

        /// Initialize with a 128-bit key.
        #[symbol = "_emb_crypto_aes128ecb_init"]
        pub fn aes128ecb_init(key: &[u8; 16]) -> Self::Context;

        /// Clone the context.
        #[symbol = "_emb_crypto_aes128ecb_clone"]
        pub fn aes128ecb_clone(ctx: &Self::Context) -> Self::Context;

        /// Encrypt 16-byte blocks in-place.
        #[symbol = "_emb_crypto_aes128ecb_encrypt_block"]
        pub fn aes128ecb_encrypt_block(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks in-place.
        #[symbol = "_emb_crypto_aes128ecb_decrypt_block"]
        pub fn aes128ecb_decrypt_block(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);
    }

    macro embassy_crypto_aes128ecb_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-256 ECB block cipher trait.
    pub trait Aes256Ecb {
        /// Opaque storage for the implementation's key schedule.
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_aes256ecb_context"]
        pub type Context;

        /// Initialize with a 256-bit key.
        #[symbol = "_emb_crypto_aes256ecb_init"]
        pub fn aes256ecb_init(key: &[u8; 32]) -> Self::Context;

        /// Clone the context.
        #[symbol = "_emb_crypto_aes256ecb_clone"]
        pub fn aes256ecb_clone(ctx: &Self::Context) -> Self::Context;

        /// Encrypt 16-byte blocks in-place.
        #[symbol = "_emb_crypto_aes256ecb_encrypt_block"]
        pub fn aes256ecb_encrypt_block(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks in-place.
        #[symbol = "_emb_crypto_aes256ecb_decrypt_block"]
        pub fn aes256ecb_decrypt_block(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>);
    }

    macro embassy_crypto_aes256ecb_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-128 CBC block cipher trait.
    pub trait Aes128Cbc {
        /// Opaque storage for the implementation's key schedule and chaining state.
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_aes128cbc_context"]
        pub type Context;

        /// Initialize with a 128-bit key and 128-bit IV.
        #[symbol = "_emb_crypto_aes128cbc_init"]
        pub fn aes128cbc_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::Context;

        /// Clone the context.
        #[symbol = "_emb_crypto_aes128cbc_clone"]
        pub fn aes128cbc_clone(ctx: &Self::Context) -> Self::Context;

        /// Encrypt 16-byte blocks in-place (updates internal chaining state).
        #[symbol = "_emb_crypto_aes128cbc_encrypt_block"]
        pub fn aes128cbc_encrypt_block(ctx: &mut Self::Context, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks in-place (updates internal chaining state).
        #[symbol = "_emb_crypto_aes128cbc_decrypt_block"]
        pub fn aes128cbc_decrypt_block(ctx: &mut Self::Context, blocks: InOutBuf<'_, '_, u8>);
    }

    macro embassy_crypto_aes128cbc_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-256 CBC block cipher trait.
    pub trait Aes256Cbc {
        /// Opaque storage for the implementation's key schedule and chaining state.
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_aes256cbc_context"]
        pub type Context;

        /// Initialize with a 256-bit key and 128-bit IV.
        #[symbol = "_emb_crypto_aes256cbc_init"]
        pub fn aes256cbc_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::Context;

        /// Clone the context.
        #[symbol = "_emb_crypto_aes256cbc_clone"]
        pub fn aes256cbc_clone(ctx: &Self::Context) -> Self::Context;

        /// Encrypt 16-byte blocks in-place (updates internal chaining state).
        #[symbol = "_emb_crypto_aes256cbc_encrypt_block"]
        pub fn aes256cbc_encrypt_block(ctx: &mut Self::Context, blocks: InOutBuf<'_, '_, u8>);

        /// Decrypt 16-byte blocks in-place (updates internal chaining state).
        #[symbol = "_emb_crypto_aes256cbc_decrypt_block"]
        pub fn aes256cbc_decrypt_block(ctx: &mut Self::Context, blocks: InOutBuf<'_, '_, u8>);
    }

    macro embassy_crypto_aes256cbc_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-128 GCM AEAD trait.
    pub trait Aes128Gcm {
        /// Opaque storage for the implementation's key schedule.
        #[opaque(size = 1280, align = 16)]
        #[symbol = "_emb_crypto_aes128gcm_context"]
        pub type Context;

        /// Initialize with a 128-bit key.
        #[symbol = "_emb_crypto_aes128gcm_init"]
        pub fn aes128gcm_init(key: &[u8; 16]) -> Self::Context;

        /// Clone the context.
        #[symbol = "_emb_crypto_aes128gcm_clone"]
        pub fn aes128gcm_clone(ctx: &Self::Context) -> Self::Context;

        /// Encrypt plaintext in-place and produce a 16-byte authentication tag.
        #[symbol = "_emb_crypto_aes128gcm_encrypt"]
        pub fn aes128gcm_encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8; 16],
        ) -> Result<(), CryptoError>;

        /// Decrypt ciphertext in-place and verify a 16-byte authentication tag.
        #[symbol = "_emb_crypto_aes128gcm_decrypt"]
        pub fn aes128gcm_decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8; 16],
        ) -> Result<(), CryptoError>;
    }

    macro embassy_crypto_aes128gcm_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-256 GCM AEAD trait.
    pub trait Aes256Gcm {
        /// Opaque storage for the implementation's key schedule.
        #[opaque(size = 1280, align = 16)]
        #[symbol = "_emb_crypto_aes256gcm_context"]
        pub type Context;

        /// Initialize with a 256-bit key.
        #[symbol = "_emb_crypto_aes256gcm_init"]
        pub fn aes256gcm_init(key: &[u8; 32]) -> Self::Context;

        /// Clone the context.
        #[symbol = "_emb_crypto_aes256gcm_clone"]
        pub fn aes256gcm_clone(ctx: &Self::Context) -> Self::Context;

        /// Encrypt plaintext in-place and produce a 16-byte authentication tag.
        #[symbol = "_emb_crypto_aes256gcm_encrypt"]
        pub fn aes256gcm_encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8; 16],
        ) -> Result<(), CryptoError>;

        /// Decrypt ciphertext in-place and verify a 16-byte authentication tag.
        #[symbol = "_emb_crypto_aes256gcm_decrypt"]
        pub fn aes256gcm_decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8; 16],
        ) -> Result<(), CryptoError>;
    }

    macro embassy_crypto_aes256gcm_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-128 CCM AEAD trait.
    ///
    /// The tag and nonce sizes are validated at runtime by the HAL.
    pub trait Aes128Ccm {
        /// Opaque storage for the implementation's key schedule.
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_aes128ccm_context"]
        pub type Context;

        /// Initialize with a 128-bit key.
        #[symbol = "_emb_crypto_aes128ccm_init"]
        pub fn aes128ccm_init(key: &[u8; 16]) -> Self::Context;

        /// Clone the context.
        #[symbol = "_emb_crypto_aes128ccm_clone"]
        pub fn aes128ccm_clone(ctx: &Self::Context) -> Self::Context;

        /// Encrypt plaintext in-place and produce an authentication tag.
        /// The tag length is determined by the length of the `tag` slice.
        #[symbol = "_emb_crypto_aes128ccm_encrypt"]
        pub fn aes128ccm_encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8],
        ) -> Result<(), CryptoError>;

        /// Decrypt ciphertext in-place and verify an authentication tag.
        /// The tag length is determined by the length of the `tag` slice.
        #[symbol = "_emb_crypto_aes128ccm_decrypt"]
        pub fn aes128ccm_decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8],
        ) -> Result<(), CryptoError>;
    }

    macro embassy_crypto_aes128ccm_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-256 CCM AEAD trait.
    ///
    /// The tag and nonce sizes are validated at runtime by the HAL.
    pub trait Aes256Ccm {
        /// Opaque storage for the implementation's key schedule.
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_aes256ccm_context"]
        pub type Context;

        /// Initialize with a 256-bit key.
        #[symbol = "_emb_crypto_aes256ccm_init"]
        pub fn aes256ccm_init(key: &[u8; 32]) -> Self::Context;

        /// Clone the context.
        #[symbol = "_emb_crypto_aes256ccm_clone"]
        pub fn aes256ccm_clone(ctx: &Self::Context) -> Self::Context;

        /// Encrypt plaintext in-place and produce an authentication tag.
        /// The tag length is determined by the length of the `tag` slice.
        #[symbol = "_emb_crypto_aes256ccm_encrypt"]
        pub fn aes256ccm_encrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &mut [u8],
        ) -> Result<(), CryptoError>;

        /// Decrypt ciphertext in-place and verify an authentication tag.
        /// The tag length is determined by the length of the `tag` slice.
        #[symbol = "_emb_crypto_aes256ccm_decrypt"]
        pub fn aes256ccm_decrypt(
            ctx: &Self::Context,
            nonce: &[u8],
            aad: &[u8],
            buffer: InOutBuf<'_, '_, u8>,
            tag: &[u8],
        ) -> Result<(), CryptoError>;
    }

    macro embassy_crypto_aes256ccm_impl(path = $crate);
}

unitrait::unitrait! {
    /// P256 ECDH (Elliptic Curve Diffie-Hellman) trait.
    ///
    /// Used for TLS 1.2/1.3 key exchange and Bluetooth LE Secure Connections.
    pub trait P256Ecdh {
        /// Generate a new P256 keypair.
        ///
        /// `private_key` receives the 32-byte scalar.
        /// `public_key` receives the 65-byte uncompressed point (0x04 || x || y).
        #[symbol = "_emb_crypto_p256ecdh_generate_keypair"]
        pub fn p256ecdh_generate_keypair(
            private_key: &mut [u8; 32],
            public_key: &mut [u8; 65],
        ) -> Result<(), CryptoError>;

        /// Derive the public key from a private key.
        ///
        /// `private_key` is the 32-byte scalar.
        /// `public_key` receives the 65-byte uncompressed point.
        #[symbol = "_emb_crypto_p256ecdh_derive_public_key"]
        pub fn p256ecdh_derive_public_key(
            private_key: &[u8; 32],
            public_key: &mut [u8; 65],
        ) -> Result<(), CryptoError>;

        /// Compute the ECDH shared secret.
        ///
        /// `private_key` is the local 32-byte scalar.
        /// `peer_public_key` is the peer's 65-byte uncompressed point.
        /// `shared_secret` receives the 32-byte x-coordinate of the shared point.
        #[symbol = "_emb_crypto_p256ecdh_shared_secret"]
        pub fn p256ecdh_shared_secret(
            private_key: &[u8; 32],
            peer_public_key: &[u8; 65],
            shared_secret: &mut [u8; 32],
        ) -> Result<(), CryptoError>;
    }

    macro embassy_crypto_p256ecdh_impl(path = $crate);
}

unitrait::unitrait! {
    /// P256 ECDSA (Elliptic Curve Digital Signature Algorithm) trait.
    ///
    /// Used for TLS 1.2/1.3 certificate verification and authentication.
    pub trait P256Ecdsa {
        /// Sign a message digest with a private key.
        ///
        /// `private_key` is the 32-byte scalar.
        /// `digest` is the pre-hashed message (e.g. SHA-256 digest).
        /// `signature` receives the 64-byte raw signature (r || s, big-endian).
        #[symbol = "_emb_crypto_p256ecdsa_sign"]
        pub fn p256ecdsa_sign(
            private_key: &[u8; 32],
            digest: &[u8],
            signature: &mut [u8; 64],
        ) -> Result<(), CryptoError>;

        /// Verify a message digest signature with a public key.
        ///
        /// `public_key` is the 65-byte uncompressed point.
        /// `digest` is the pre-hashed message.
        /// `signature` is the 64-byte raw signature (r || s, big-endian).
        #[symbol = "_emb_crypto_p256ecdsa_verify"]
        pub fn p256ecdsa_verify(
            public_key: &[u8; 65],
            digest: &[u8],
            signature: &[u8; 64],
        ) -> Result<(), CryptoError>;
    }

    macro embassy_crypto_p256ecdsa_impl(path = $crate);
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
    pub trait Aes128Ctr {
        /// Opaque storage for key schedule, counter state, and partial-block buffer.
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_aes128ctr_context"]
        pub type Context;

        /// Initialize with a 128-bit key and 128-bit initial counter (IV).
        #[symbol = "_emb_crypto_aes128ctr_init"]
        pub fn aes128ctr_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::Context;

        /// Clone the context (including current counter position).
        #[symbol = "_emb_crypto_aes128ctr_clone"]
        pub fn aes128ctr_clone(ctx: &Self::Context) -> Self::Context;

        /// Apply keystream to `buf` in-place (encrypt == decrypt for CTR).
        #[symbol = "_emb_crypto_aes128ctr_apply_keystream"]
        pub fn aes128ctr_apply_keystream(ctx: &mut Self::Context, buf: InOutBuf<'_, '_, u8>);
    }

    macro embassy_crypto_aes128ctr_impl(path = $crate);
}

unitrait::unitrait! {
    /// AES-256 CTR stream cipher trait.
    ///
    /// See [`Aes128Ctr`] for CTR mode semantics. Uses a 256-bit key.
    pub trait Aes256Ctr {
        #[opaque(size = 1024, align = 16)]
        #[symbol = "_emb_crypto_aes256ctr_context"]
        pub type Context;

        #[symbol = "_emb_crypto_aes256ctr_init"]
        pub fn aes256ctr_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::Context;

        #[symbol = "_emb_crypto_aes256ctr_clone"]
        pub fn aes256ctr_clone(ctx: &Self::Context) -> Self::Context;

        #[symbol = "_emb_crypto_aes256ctr_apply_keystream"]
        pub fn aes256ctr_apply_keystream(ctx: &mut Self::Context, buf: InOutBuf<'_, '_, u8>);
    }

    macro embassy_crypto_aes256ctr_impl(path = $crate);
}
