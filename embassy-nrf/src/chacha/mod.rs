//! ChaCha20 stream cipher hardware accelerator.
//!
//! This driver uses the CHACHA engine of the CryptoCell subsystem (nRF52840, nRF91, nRF5340)
//! or of the CRACEN CryptoMaster (nRF54L).
//!
//! ChaCha20 with a 256-bit key, 96-bit nonce and 32-bit block counter, and ChaCha20-Poly1305
//! authenticated encryption, both as specified by RFC 8439.
//!
//! On the CryptoCell the Poly1305 authenticator runs on the PKA engine, so a driver of this
//! module and one of the [`pka`](crate::pka) module must not be used at the same time from
//! different execution contexts.
//!
//! Each hardware transaction runs in a critical section and processes at most 1 KiB of data,
//! so interrupt latency stays bounded. Input data may be anywhere in memory (data outside RAM
//! is copied through a bounce buffer before the DMA reads it).

use core::marker::PhantomData;

use crate::mode::{Blocking, Mode};
use crate::util::for_each_ram_chunk_inout;
use crate::{Peri, peripherals};

#[cfg_attr(feature = "_cryptocell", path = "cryptocell.rs")]
#[cfg_attr(feature = "_cracen", path = "cracen.rs")]
mod hw;

/// Key length in bytes.
pub const KEY_LEN: usize = 32;
/// Nonce length in bytes.
pub const NONCE_LEN: usize = 12;
/// Block length in bytes.
pub const BLOCK_LEN: usize = 64;

/// Largest amount of data processed per hardware transaction.
const MAX_CHUNK: usize = 1024;

/// ChaCha error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// The input and output lengths differ, or a chunk that is not the last one is not a
    /// multiple of the block length where required.
    InvalidLength,
    /// Additional authenticated data was supplied after payload data.
    AadAfterPayload,
    /// The hardware reported an error.
    Hardware,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::InvalidLength => "invalid data length",
            Self::AadAfterPayload => "additional authenticated data after payload",
            Self::Hardware => "hardware error",
        };
        f.write_str(s)
    }
}

impl core::error::Error for Error {}

/// Cipher direction for ChaCha20-Poly1305.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Encryption.
    Encrypt,
    /// Decryption.
    Decrypt,
}

/// State of an in-progress ChaCha20 keystream, created by [`ChaCha::start`].
#[derive(Clone)]
pub struct Context {
    key: [u8; KEY_LEN],
    nonce: [u8; NONCE_LEN],
    counter: u32,
    /// Unused keystream bytes are stored at the end of this buffer.
    ks: [u8; BLOCK_LEN],
    ks_len: u8,
}

impl Context {
    /// XORs the keystream into `len` bytes from `input` to `output`, which may alias.
    ///
    /// # Safety
    ///
    /// `input` must be readable and `output` writable for `len` bytes. If they overlap, they
    /// must be equal.
    unsafe fn apply(&mut self, input: *const u8, output: *mut u8, len: usize) {
        let mut done = 0;

        let n = (self.ks_len as usize).min(len);
        for i in 0..n {
            let k = self.ks[BLOCK_LEN - self.ks_len as usize + i];
            unsafe { output.add(i).write(input.add(i).read() ^ k) };
        }
        self.ks_len -= n as u8;
        done += n;

        while len - done >= BLOCK_LEN {
            let blocks = (len - done) / BLOCK_LEN;
            // Never let the 32-bit counter wrap inside a transaction.
            let until_wrap = (u32::MAX - self.counter) as u64 + 1;
            let n = (blocks as u64).min(until_wrap) as usize * BLOCK_LEN;
            unsafe {
                for_each_ram_chunk_inout(input.add(done), output.add(done), n, MAX_CHUNK, |i, o, l| {
                    hw::apply(&self.key, &self.nonce, self.counter, i, o, l);
                    self.counter = self.counter.wrapping_add((l / BLOCK_LEN) as u32);
                })
            };
            done += n;
        }

        if done < len {
            let zero = [0u8; BLOCK_LEN];
            let mut ks = [0u8; BLOCK_LEN];
            unsafe {
                hw::apply(
                    &self.key,
                    &self.nonce,
                    self.counter,
                    zero.as_ptr(),
                    ks.as_mut_ptr(),
                    BLOCK_LEN,
                )
            };
            self.counter = self.counter.wrapping_add(1);
            let rem = len - done;
            for i in 0..rem {
                unsafe { output.add(done + i).write(input.add(done + i).read() ^ ks[i]) };
            }
            self.ks = ks;
            self.ks_len = (BLOCK_LEN - rem) as u8;
        }
    }
}

/// ChaCha driver.
pub struct ChaCha<'d, M: Mode> {
    _hw: hw::Handle,
    _phantom: PhantomData<(&'d (), M)>,
}

impl<'d> ChaCha<'d, Blocking> {
    /// Creates a new blocking ChaCha driver.
    pub fn new_blocking(_peri: Peri<'d, peripherals::CHACHA>) -> Self {
        Self {
            _hw: hw::Handle::new(),
            _phantom: PhantomData,
        }
    }
}

impl<'d, M: Mode> ChaCha<'d, M> {
    /// Starts a ChaCha20 keystream with the given key, nonce and initial block counter.
    ///
    /// The (key, nonce) pair must never be reused.
    pub fn start(&mut self, key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN], counter: u32) -> Context {
        Context {
            key: *key,
            nonce: *nonce,
            counter,
            ks: [0; BLOCK_LEN],
            ks_len: 0,
        }
    }

    /// XORs the next bytes of the keystream into `input`, writing the result to `output`.
    ///
    /// Encryption and decryption are the same operation. Data may have any length.
    pub fn blocking_apply_keystream(
        &mut self,
        ctx: &mut Context,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<(), Error> {
        if input.len() != output.len() {
            return Err(Error::InvalidLength);
        }
        unsafe { ctx.apply(input.as_ptr(), output.as_mut_ptr(), input.len()) };
        Ok(())
    }

    /// XORs the next bytes of the keystream into `data` in place.
    pub fn blocking_apply_keystream_in_place(&mut self, ctx: &mut Context, data: &mut [u8]) {
        unsafe { ctx.apply(data.as_ptr(), data.as_mut_ptr(), data.len()) };
    }
}

impl<'d, M: Mode> Drop for ChaCha<'d, M> {
    fn drop(&mut self) {
        // The hardware is powered down by the handle when the last user releases it.
    }
}

pub use hw::aead::AeadContext;

impl<'d, M: Mode> ChaCha<'d, M> {
    /// Starts a ChaCha20-Poly1305 (RFC 8439) authenticated encryption or decryption.
    ///
    /// The (key, nonce) pair must never be reused.
    pub fn start_aead(&mut self, key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN], dir: Direction) -> AeadContext {
        AeadContext::new(key, nonce, dir)
    }

    /// Feeds additional authenticated data (AAD). Must be called before any payload.
    ///
    /// AAD may be fed in chunks of any size. `last` marks the final chunk; it is optional,
    /// the AAD phase also ends when the first payload chunk is fed or the operation is
    /// finished.
    pub fn blocking_aad(&mut self, ctx: &mut AeadContext, aad: &[u8], last: bool) -> Result<(), Error> {
        ctx.aad(aad, last)
    }

    /// Processes payload data from `input` into `output`, which must have the same length.
    ///
    /// `last` marks the final chunk. Chunks that are not the last one must be a multiple of
    /// the block length (64 bytes).
    pub fn blocking_payload(
        &mut self,
        ctx: &mut AeadContext,
        input: &[u8],
        output: &mut [u8],
        last: bool,
    ) -> Result<(), Error> {
        if input.len() != output.len() {
            return Err(Error::InvalidLength);
        }
        unsafe { ctx.payload(input.as_ptr(), output.as_mut_ptr(), input.len(), last) }
    }

    /// Processes payload data in place. See [`Self::blocking_payload`].
    pub fn blocking_payload_in_place(
        &mut self,
        ctx: &mut AeadContext,
        data: &mut [u8],
        last: bool,
    ) -> Result<(), Error> {
        unsafe { ctx.payload(data.as_ptr(), data.as_mut_ptr(), data.len(), last) }
    }

    /// Finishes the operation and returns the 16-byte Poly1305 tag.
    ///
    /// When decrypting, compare the returned tag against the received one in constant time
    /// before using the plaintext.
    pub fn blocking_finish(&mut self, ctx: AeadContext) -> Result<[u8; 16], Error> {
        ctx.finish()
    }
}
