#![no_std]
#![allow(missing_docs)]
#![doc = include_str!("../README.md")]

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

/// Discriminated union of all AES symmetric crypto operations.
#[non_exhaustive]
pub enum AesOperation<'a> {
    Aes128EcbEncrypt {
        block: &'a mut [u8; 16],
        key: &'a [u8; 16],
    },
    Aes128EcbDecrypt {
        block: &'a mut [u8; 16],
        key: &'a [u8; 16],
    },
    Aes128Cmac {
        key: &'a [u8; 16],
        data: &'a [u8],
        out: &'a mut [u8; 16],
    },
    AesCcm128Encrypt {
        key: &'a [u8; 16],
        nonce: &'a [u8],
        aad: &'a [u8],
        plaintext: &'a [u8],
        ciphertext: &'a mut [u8],
        tag: &'a mut [u8; 16],
    },
    AesCcm128Decrypt {
        key: &'a [u8; 16],
        nonce: &'a [u8],
        aad: &'a [u8],
        ciphertext: &'a [u8],
        plaintext: &'a mut [u8],
        tag: &'a [u8; 16],
    },
    AesCcm8_128Encrypt {
        key: &'a [u8; 16],
        nonce: &'a [u8],
        aad: &'a [u8],
        plaintext: &'a [u8],
        ciphertext: &'a mut [u8],
        tag: &'a mut [u8; 8],
    },
    AesCcm8_128Decrypt {
        key: &'a [u8; 16],
        nonce: &'a [u8],
        aad: &'a [u8],
        ciphertext: &'a [u8],
        plaintext: &'a mut [u8],
        tag: &'a [u8; 8],
    },
    AesGcm128Encrypt {
        key: &'a [u8; 16],
        nonce: &'a [u8],
        aad: &'a [u8],
        plaintext: &'a [u8],
        ciphertext: &'a mut [u8],
        tag: &'a mut [u8; 16],
    },
    AesGcm128Decrypt {
        key: &'a [u8; 16],
        nonce: &'a [u8],
        aad: &'a [u8],
        ciphertext: &'a [u8],
        plaintext: &'a mut [u8],
        tag: &'a [u8; 16],
    },
    AesGcm256Encrypt {
        key: &'a [u8; 32],
        nonce: &'a [u8],
        aad: &'a [u8],
        plaintext: &'a [u8],
        ciphertext: &'a mut [u8],
        tag: &'a mut [u8; 16],
    },
    AesGcm256Decrypt {
        key: &'a [u8; 32],
        nonce: &'a [u8],
        aad: &'a [u8],
        ciphertext: &'a [u8],
        plaintext: &'a mut [u8],
        tag: &'a [u8; 16],
    },
    Aes128CbcEncrypt {
        iv: &'a [u8; 16],
        buffer: &'a mut [u8],
        key: &'a [u8; 16],
    },
    Aes128CbcDecrypt {
        iv: &'a [u8; 16],
        block: &'a mut [u8],
        key: &'a [u8; 16],
    },
    Aes256CbcEncrypt {
        iv: &'a [u8; 16],
        block: &'a mut [u8],
        key: &'a [u8; 32],
    },
    Aes256CbcDecrypt {
        iv: &'a [u8; 16],
        block: &'a mut [u8],
        key: &'a [u8; 32],
    },
    AesCcm4_128Encrypt {
        key: &'a [u8; 16],
        nonce: &'a [u8],
        aad: &'a [u8],
        plaintext: &'a [u8],
        ciphertext: &'a mut [u8],
        tag: &'a mut [u8; 4],
    },
    AesCcm4_128Decrypt {
        key: &'a [u8; 16],
        nonce: &'a [u8],
        aad: &'a [u8],
        ciphertext: &'a [u8],
        plaintext: &'a mut [u8],
        tag: &'a [u8; 4],
    },
}

unitrait::unitrait! {
    /// AES trait
    pub trait Aes {
        /// Hash init
        #[symbol = "_emb_crypto_aes_exec"]
        pub fn aes_exec(op: AesOperation<'_>) -> Result<(), CryptoError>;
    }

    macro embassy_crypto_aes_impl(path = $crate);
}
