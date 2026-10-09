//! AES block cipher and modes of operation.
//!
//! Every type is served by the driver registered for it; see [`driver`].
//! Each cipher method comes in an in-place form, operating on one buffer, and
//! a `_to` form reading from one buffer and writing to another of the same
//! length.

use crate::Error;
use crate::driver::{self, InOutBuf};

fn check_blocks(len: usize) -> Result<(), Error> {
    if len % 16 != 0 {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

fn inout<'i, 'o>(input: &'i [u8], output: &'o mut [u8]) -> Result<InOutBuf<'i, 'o, u8>, Error> {
    InOutBuf::new(input, output).map_err(|_| Error::InvalidInput)
}

macro_rules! impl_ecb {
    ($(#[$meta:meta])* $name:ident, $drv:ident, $ctx:ident, $key_size:literal) => {
        $(#[$meta])*
        ///
        /// Encrypts and decrypts independent 16-byte blocks (ECB). This is the
        /// raw block cipher, not a mode of operation: use it as a building block
        /// (key wrapping, KDFs) and not to encrypt data directly.
        #[derive(Clone)]
        pub struct $name {
            ctx: driver::$ctx,
        }

        impl $name {
            /// Size of the key, in bytes.
            pub const KEY_SIZE: usize = $key_size;

            /// Size of a block, in bytes.
            pub const BLOCK_SIZE: usize = 16;

            /// Initialize with `key`.
            pub fn new(key: &[u8; $key_size]) -> Self {
                Self {
                    ctx: driver::$drv::init(key),
                }
            }

            /// Encrypt one block in place.
            pub fn encrypt_block(&self, block: &mut [u8; 16]) {
                driver::$drv::encrypt_blocks(&self.ctx, block.as_mut_slice().into());
            }

            /// Decrypt one block in place.
            pub fn decrypt_block(&self, block: &mut [u8; 16]) {
                driver::$drv::decrypt_blocks(&self.ctx, block.as_mut_slice().into());
            }

            /// Encrypt a whole number of blocks in place.
            pub fn encrypt_blocks(&self, blocks: &mut [u8]) -> Result<(), Error> {
                check_blocks(blocks.len())?;
                driver::$drv::encrypt_blocks(&self.ctx, blocks.into());
                Ok(())
            }

            /// Decrypt a whole number of blocks in place.
            pub fn decrypt_blocks(&self, blocks: &mut [u8]) -> Result<(), Error> {
                check_blocks(blocks.len())?;
                driver::$drv::decrypt_blocks(&self.ctx, blocks.into());
                Ok(())
            }

            /// Encrypt a whole number of blocks from `input` to `output`.
            pub fn encrypt_blocks_to(&self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                check_blocks(input.len())?;
                driver::$drv::encrypt_blocks(&self.ctx, inout(input, output)?);
                Ok(())
            }

            /// Decrypt a whole number of blocks from `input` to `output`.
            pub fn decrypt_blocks_to(&self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                check_blocks(input.len())?;
                driver::$drv::decrypt_blocks(&self.ctx, inout(input, output)?);
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

macro_rules! impl_cbc {
    ($(#[$meta:meta])* $enc:ident, $dec:ident, $drv:ident, $enc_ctx:ident, $dec_ctx:ident, $key_size:literal) => {
        $(#[$meta])*
        ///
        /// CBC mode encryptor. Data is processed a whole number of blocks at a
        /// time; padding is the caller's responsibility. The chaining state is
        /// kept across calls, so a message may be encrypted in several calls.
        #[derive(Clone)]
        pub struct $enc {
            ctx: driver::$enc_ctx,
        }

        impl $enc {
            /// Initialize with `key` and `iv`.
            pub fn new(key: &[u8; $key_size], iv: &[u8; 16]) -> Self {
                Self {
                    ctx: driver::$drv::encrypt_init(key, iv),
                }
            }

            /// Encrypt a whole number of blocks in place.
            pub fn encrypt(&mut self, blocks: &mut [u8]) -> Result<(), Error> {
                check_blocks(blocks.len())?;
                driver::$drv::encrypt_blocks(&mut self.ctx, blocks.into());
                Ok(())
            }

            /// Encrypt a whole number of blocks from `input` to `output`.
            pub fn encrypt_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                check_blocks(input.len())?;
                driver::$drv::encrypt_blocks(&mut self.ctx, inout(input, output)?);
                Ok(())
            }
        }

        impl core::fmt::Debug for $enc {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($enc)).finish_non_exhaustive()
            }
        }

        $(#[$meta])*
        ///
        /// CBC mode decryptor. Data is processed a whole number of blocks at a
        /// time; padding is the caller's responsibility. The chaining state is
        /// kept across calls, so a message may be decrypted in several calls.
        #[derive(Clone)]
        pub struct $dec {
            ctx: driver::$dec_ctx,
        }

        impl $dec {
            /// Initialize with `key` and `iv`.
            pub fn new(key: &[u8; $key_size], iv: &[u8; 16]) -> Self {
                Self {
                    ctx: driver::$drv::decrypt_init(key, iv),
                }
            }

            /// Decrypt a whole number of blocks in place.
            pub fn decrypt(&mut self, blocks: &mut [u8]) -> Result<(), Error> {
                check_blocks(blocks.len())?;
                driver::$drv::decrypt_blocks(&mut self.ctx, blocks.into());
                Ok(())
            }

            /// Decrypt a whole number of blocks from `input` to `output`.
            pub fn decrypt_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                check_blocks(input.len())?;
                driver::$drv::decrypt_blocks(&mut self.ctx, inout(input, output)?);
                Ok(())
            }
        }

        impl core::fmt::Debug for $dec {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($dec)).finish_non_exhaustive()
            }
        }
    };
}

macro_rules! impl_ctr {
    ($(#[$meta:meta])* $name:ident, $drv:ident, $ctx:ident, $key_size:literal) => {
        $(#[$meta])*
        ///
        /// CTR mode with a 128-bit big-endian counter (NIST SP 800-38A).
        /// Encryption and decryption are the same operation. Data of any length
        /// is accepted, in any number of calls: the keystream position is kept
        /// across calls.
        #[derive(Clone)]
        pub struct $name {
            ctx: driver::$ctx,
        }

        impl $name {
            /// Initialize with `key` and the initial counter block `iv`.
            pub fn new(key: &[u8; $key_size], iv: &[u8; 16]) -> Self {
                Self {
                    ctx: driver::$drv::init(key, iv),
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

macro_rules! impl_gcm {
    ($(#[$meta:meta])* $name:ident, $drv:ident, $ctx:ident, $key_size:literal) => {
        $(#[$meta])*
        ///
        /// GCM authenticated encryption with a 96-bit nonce and a 128-bit tag.
        /// The nonce must never repeat for a given key.
        #[derive(Clone)]
        pub struct $name {
            ctx: driver::$ctx,
        }

        impl $name {
            /// Size of the nonce, in bytes.
            pub const NONCE_SIZE: usize = 12;

            /// Size of the tag, in bytes.
            pub const TAG_SIZE: usize = 16;

            /// Initialize with `key`.
            pub fn new(key: &[u8; $key_size]) -> Self {
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

macro_rules! impl_ccm {
    ($(#[$meta:meta])* $name:ident, $drv:ident, $ctx:ident, $key_size:literal) => {
        $(#[$meta])*
        ///
        /// CCM authenticated encryption (NIST SP 800-38C, RFC 3610). The nonce is
        /// 7 to 13 bytes and the tag 4, 6, 8, 10, 12, 14 or 16 bytes; the tag
        /// length is the length of the `tag` slice passed to each call. The
        /// nonce must never repeat for a given key.
        #[derive(Clone)]
        pub struct $name {
            ctx: driver::$ctx,
        }

        impl $name {
            /// Initialize with `key`.
            pub fn new(key: &[u8; $key_size]) -> Self {
                Self {
                    ctx: driver::$drv::init(key),
                }
            }

            /// Encrypt `buf` in place, authenticating it and `aad`, writing the tag to `tag`.
            pub fn encrypt(&self, nonce: &[u8], aad: &[u8], buf: &mut [u8], tag: &mut [u8]) -> Result<(), Error> {
                driver::$drv::encrypt(&self.ctx, nonce, aad, buf.into(), tag)
            }

            /// Encrypt `input` to `output`, authenticating it and `aad`, writing the tag to `tag`.
            pub fn encrypt_to(
                &self,
                nonce: &[u8],
                aad: &[u8],
                input: &[u8],
                output: &mut [u8],
                tag: &mut [u8],
            ) -> Result<(), Error> {
                driver::$drv::encrypt(&self.ctx, nonce, aad, inout(input, output)?, tag)
            }

            /// Verify `tag` over `buf` and `aad`, then decrypt `buf` in place.
            ///
            /// On [`Error::InvalidSignature`] the contents of `buf` are unspecified.
            pub fn decrypt(&self, nonce: &[u8], aad: &[u8], buf: &mut [u8], tag: &[u8]) -> Result<(), Error> {
                driver::$drv::decrypt(&self.ctx, nonce, aad, buf.into(), tag)
            }

            /// Verify `tag` over `input` and `aad`, then decrypt `input` to `output`.
            ///
            /// On [`Error::InvalidSignature`] the contents of `output` are unspecified.
            pub fn decrypt_to(
                &self,
                nonce: &[u8],
                aad: &[u8],
                input: &[u8],
                output: &mut [u8],
                tag: &[u8],
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

macro_rules! impl_cmac {
    ($(#[$meta:meta])* $name:ident, $drv:ident, $ctx:ident, $key_size:literal) => {
        $(#[$meta])*
        ///
        /// CMAC message authentication code (NIST SP 800-38B), with a 128-bit tag.
        #[derive(Clone)]
        pub struct $name {
            ctx: driver::$ctx,
        }

        impl $name {
            /// Size of the tag, in bytes.
            pub const OUTPUT_SIZE: usize = 16;

            /// Start a new MAC computation with `key`.
            pub fn new(key: &[u8; $key_size]) -> Self {
                Self {
                    ctx: driver::$drv::init(key),
                }
            }

            /// Absorb `data`.
            pub fn update(&mut self, data: &[u8]) {
                driver::$drv::update(&mut self.ctx, data);
            }

            /// Finish the computation and return the tag.
            pub fn finalize(self) -> [u8; 16] {
                let mut out = [0u8; 16];
                driver::$drv::finalize(self.ctx, &mut out);
                out
            }

            /// Finish the computation and compare the tag with `tag` in constant time.
            ///
            /// A truncated `tag` (shorter than 16 bytes) is compared against the
            /// corresponding prefix; an empty one never verifies.
            pub fn verify(self, tag: &[u8]) -> Result<(), Error> {
                let out = self.finalize();
                if tag.is_empty() || tag.len() > out.len() || !crate::ct::eq(&out[..tag.len()], tag) {
                    return Err(Error::InvalidSignature);
                }
                Ok(())
            }

            /// Discard the absorbed data, keeping the key.
            pub fn reset(&mut self) {
                driver::$drv::reset(&mut self.ctx);
            }

            /// Compute the tag of `data` under `key` in one call.
            pub fn mac(key: &[u8; $key_size], data: &[u8]) -> [u8; 16] {
                let mut m = Self::new(key);
                m.update(data);
                m.finalize()
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }
    };
}

impl_ecb!(
    /// AES-128 block cipher.
    Aes128, Aes128EcbImpl, Aes128EcbImplContext, 16
);
impl_ecb!(
    /// AES-256 block cipher.
    Aes256, Aes256EcbImpl, Aes256EcbImplContext, 32
);

impl_cbc!(
    /// AES-128 CBC.
    Aes128CbcEncrypt, Aes128CbcDecrypt, Aes128CbcImpl, Aes128CbcImplEncryptContext, Aes128CbcImplDecryptContext, 16
);
impl_cbc!(
    /// AES-256 CBC.
    Aes256CbcEncrypt, Aes256CbcDecrypt, Aes256CbcImpl, Aes256CbcImplEncryptContext, Aes256CbcImplDecryptContext, 32
);

impl_ctr!(
    /// AES-128 CTR.
    Aes128Ctr, Aes128CtrImpl, Aes128CtrImplContext, 16
);
impl_ctr!(
    /// AES-256 CTR.
    Aes256Ctr, Aes256CtrImpl, Aes256CtrImplContext, 32
);

impl_gcm!(
    /// AES-128 GCM.
    Aes128Gcm, Aes128GcmImpl, Aes128GcmImplContext, 16
);
impl_gcm!(
    /// AES-256 GCM.
    Aes256Gcm, Aes256GcmImpl, Aes256GcmImplContext, 32
);

impl_ccm!(
    /// AES-128 CCM.
    Aes128Ccm, Aes128CcmImpl, Aes128CcmImplContext, 16
);
impl_ccm!(
    /// AES-256 CCM.
    Aes256Ccm, Aes256CcmImpl, Aes256CcmImplContext, 32
);

impl_cmac!(
    /// AES-128 CMAC.
    Aes128Cmac, Aes128CmacImpl, Aes128CmacImplContext, 16
);
impl_cmac!(
    /// AES-256 CMAC.
    Aes256Cmac, Aes256CmacImpl, Aes256CmacImplContext, 32
);
