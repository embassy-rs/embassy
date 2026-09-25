//! ChaCha stream ciphers and ChaCha-Poly1305 authenticated encryption.
//!
//! ChaCha20 and ChaCha20-Poly1305 are the RFC 8439 algorithms. ChaCha8 and
//! ChaCha12 are the reduced-round variants from the original ChaCha paper, with
//! the same key, nonce and counter layout; they are faster but have a smaller
//! security margin. Each is served by the driver registered for it; see [`driver`].

use crate::Error;
use crate::driver::{self, InOutBuf};

fn inout<'i, 'o>(input: &'i [u8], output: &'o mut [u8]) -> Result<InOutBuf<'i, 'o, u8>, Error> {
    InOutBuf::new(input, output).map_err(|_| Error::InvalidInput)
}

macro_rules! impl_stream {
    ($(#[$meta:meta])* $name:ident, $drv:ident, $ctx:ident) => {
        $(#[$meta])*
        ///
        /// Stream cipher with a 256-bit key, a 96-bit nonce and a 32-bit block
        /// counter. Encryption and decryption are the same operation. Data of any
        /// length is accepted, in any number of calls: the keystream position is
        /// kept across calls. The (key, nonce) pair must never be reused.
        #[derive(Clone)]
        pub struct $name {
            ctx: driver::$ctx,
        }

        impl $name {
            /// Size of the key, in bytes.
            pub const KEY_SIZE: usize = 32;

            /// Size of the nonce, in bytes.
            pub const NONCE_SIZE: usize = 12;

            /// Initialize with `key` and `nonce`, starting at block counter 0.
            pub fn new(key: &[u8; 32], nonce: &[u8; 12]) -> Self {
                Self::with_counter(key, nonce, 0)
            }

            /// Initialize with `key` and `nonce`, starting at block `counter`.
            pub fn with_counter(key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> Self {
                Self {
                    ctx: driver::$drv::init(key, nonce, counter),
                }
            }

            /// XOR the keystream into `buf`.
            pub fn apply_keystream(&mut self, buf: &mut [u8]) {
                driver::$drv::apply_keystream(&mut self.ctx, buf.into());
            }

            /// XOR the keystream with `input`, writing the result to `output`.
            pub fn apply_keystream_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                driver::$drv::apply_keystream(&mut self.ctx, inout(input, output)?);
                Ok(())
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }
    };
}

macro_rules! impl_aead {
    ($(#[$meta:meta])* $name:ident, $drv:ident, $ctx:ident) => {
        $(#[$meta])*
        ///
        /// Authenticated encryption with a 256-bit key, a 96-bit nonce and a 128-bit
        /// tag. The nonce must never repeat for a given key.
        #[derive(Clone)]
        pub struct $name {
            ctx: driver::$ctx,
        }

        impl $name {
            /// Size of the key, in bytes.
            pub const KEY_SIZE: usize = 32;

            /// Size of the nonce, in bytes.
            pub const NONCE_SIZE: usize = 12;

            /// Size of the tag, in bytes.
            pub const TAG_SIZE: usize = 16;

            /// Initialize with `key`.
            pub fn new(key: &[u8; 32]) -> Self {
                Self {
                    ctx: driver::$drv::init(key),
                }
            }

            /// Encrypt `buf` in place, authenticating it and `aad`, and return the tag.
            pub fn encrypt(&self, nonce: &[u8; 12], aad: &[u8], buf: &mut [u8]) -> Result<[u8; 16], Error> {
                let mut tag = [0u8; 16];
                driver::$drv::encrypt(&self.ctx, nonce, aad, buf.into(), &mut tag)?;
                Ok(tag)
            }

            /// Encrypt `input` to `output`, authenticating it and `aad`, and return the tag.
            pub fn encrypt_to(
                &self,
                nonce: &[u8; 12],
                aad: &[u8],
                input: &[u8],
                output: &mut [u8],
            ) -> Result<[u8; 16], Error> {
                let mut tag = [0u8; 16];
                driver::$drv::encrypt(&self.ctx, nonce, aad, inout(input, output)?, &mut tag)?;
                Ok(tag)
            }

            /// Verify `tag` over `buf` and `aad`, then decrypt `buf` in place.
            ///
            /// On [`Error::InvalidSignature`] the contents of `buf` are unspecified.
            pub fn decrypt(&self, nonce: &[u8; 12], aad: &[u8], buf: &mut [u8], tag: &[u8; 16]) -> Result<(), Error> {
                driver::$drv::decrypt(&self.ctx, nonce, aad, buf.into(), tag)
            }

            /// Verify `tag` over `input` and `aad`, then decrypt `input` to `output`.
            ///
            /// On [`Error::InvalidSignature`] the contents of `output` are unspecified.
            pub fn decrypt_to(
                &self,
                nonce: &[u8; 12],
                aad: &[u8],
                input: &[u8],
                output: &mut [u8],
                tag: &[u8; 16],
            ) -> Result<(), Error> {
                driver::$drv::decrypt(&self.ctx, nonce, aad, inout(input, output)?, tag)
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }
    };
}

impl_stream!(
    /// ChaCha8: ChaCha with 8 rounds.
    ChaCha8, ChaCha8Impl, ChaCha8ImplContext
);
impl_stream!(
    /// ChaCha12: ChaCha with 12 rounds.
    ChaCha12, ChaCha12Impl, ChaCha12ImplContext
);
impl_stream!(
    /// ChaCha20 (RFC 8439).
    ChaCha20, ChaCha20Impl, ChaCha20ImplContext
);

impl_aead!(
    /// ChaCha8-Poly1305: ChaCha20-Poly1305 with the 8-round ChaCha8 in place of ChaCha20.
    ChaCha8Poly1305, ChaCha8Poly1305Impl, ChaCha8Poly1305ImplContext
);
impl_aead!(
    /// ChaCha12-Poly1305: ChaCha20-Poly1305 with the 12-round ChaCha12 in place of ChaCha20.
    ChaCha12Poly1305, ChaCha12Poly1305Impl, ChaCha12Poly1305ImplContext
);
impl_aead!(
    /// ChaCha20-Poly1305 (RFC 8439).
    ChaCha20Poly1305, ChaCha20Poly1305Impl, ChaCha20Poly1305ImplContext
);
