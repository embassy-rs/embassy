#![no_std]
#![allow(missing_docs)]
#![doc = include_str!("../README.md")]

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
        #[symbol = "_emb_crypto_sha256_init"]
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
        #[symbol = "_emb_crypto_sha384_init"]
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
        #[symbol = "_emb_crypto_sha512_224_init"]
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
        #[symbol = "_emb_crypto_sha512_256_init"]
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
        #[symbol = "_emb_crypto_sha512_init"]
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
