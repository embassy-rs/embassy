//! ChaCha and ChaCha-Poly1305.

use super::{Direction, Error, MAX_CHUNK, Symmetric};
use crate::mode::Mode;
use crate::util::for_each_ram_chunk_inout;

#[cfg_attr(feature = "_cryptocell", path = "chacha_cryptocell.rs")]
#[cfg_attr(feature = "_cracen", path = "chacha_cracen.rs")]
mod hw;

/// ChaCha20 key length in bytes.
pub const CHACHA_KEY_LEN: usize = 32;
/// ChaCha20 nonce length in bytes.
pub const CHACHA_NONCE_LEN: usize = 12;
/// ChaCha20 block length in bytes.
pub const CHACHA_BLOCK_LEN: usize = 64;
const KEY_LEN: usize = CHACHA_KEY_LEN;
const NONCE_LEN: usize = CHACHA_NONCE_LEN;
const BLOCK_LEN: usize = CHACHA_BLOCK_LEN;

/// Number of rounds of the ChaCha block function.
///
/// ChaCha20 is the standard (RFC 8439) variant. The reduced-round ChaCha12 and ChaCha8 are
/// faster but have a smaller security margin; only the CryptoCell engine supports them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ChaChaVariant {
    /// 20 rounds (RFC 8439).
    ChaCha20,
    /// 12 rounds.
    #[cfg(feature = "_cryptocell")]
    ChaCha12,
    /// 8 rounds.
    #[cfg(feature = "_cryptocell")]
    ChaCha8,
}

/// State of a ChaCha keystream in progress. Created by [`Symmetric::chacha_start`].
#[derive(Clone)]
pub struct ChaChaContext {
    variant: ChaChaVariant,
    key: [u8; KEY_LEN],
    nonce: [u8; NONCE_LEN],
    counter: u32,
    /// Unused keystream bytes are stored at the end of this buffer.
    ks: [u8; BLOCK_LEN],
    ks_len: u8,
}

impl ChaChaContext {
    /// XORs the keystream into `len` bytes from `input` to `output`.
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
                    hw::apply(self.variant, &self.key, &self.nonce, self.counter, i, o, l);
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
                    self.variant,
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

impl<'d, M: Mode> Symmetric<'d, M> {
    /// Starts a ChaCha keystream with the given key, nonce and initial block counter.
    ///
    /// The (key, nonce) pair must never be reused.
    pub fn chacha_start(
        &mut self,
        variant: ChaChaVariant,
        key: &[u8; KEY_LEN],
        nonce: &[u8; NONCE_LEN],
        counter: u32,
    ) -> ChaChaContext {
        ChaChaContext {
            variant,
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
    ///
    /// Errors: `InvalidLength` if `input` and `output` have different lengths.
    pub fn chacha_blocking_apply_keystream(
        &mut self,
        ctx: &mut ChaChaContext,
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
    pub fn chacha_blocking_apply_keystream_in_place(&mut self, ctx: &mut ChaChaContext, data: &mut [u8]) {
        unsafe { ctx.apply(data.as_ptr(), data.as_mut_ptr(), data.len()) };
    }
}

pub use hw::aead::ChaChaPolyContext;

impl<'d, M: Mode> Symmetric<'d, M> {
    /// Starts a ChaCha-Poly1305 authenticated encryption or decryption. With
    /// [`ChaChaVariant::ChaCha20`] this is RFC 8439 ChaCha20-Poly1305.
    ///
    /// The (key, nonce) pair must never be reused. The returned context holds all the state
    /// of the operation. Pass it to the other `chachapoly_blocking_*` methods.
    pub fn chachapoly_start(
        &mut self,
        variant: ChaChaVariant,
        key: &[u8; KEY_LEN],
        nonce: &[u8; NONCE_LEN],
        dir: Direction,
    ) -> ChaChaPolyContext {
        ChaChaPolyContext::new(variant, key, nonce, dir)
    }

    /// Feeds additional authenticated data (AAD).
    ///
    /// - Call it before feeding any payload.
    /// - Chunks may have any size.
    /// - `last` marks the final chunk. It is optional. The AAD phase also ends when the
    ///   first payload chunk is fed, or when the operation is finished.
    ///
    /// Errors: `AadAfterPayload`.
    pub fn chachapoly_blocking_aad(
        &mut self,
        ctx: &mut ChaChaPolyContext,
        aad: &[u8],
        last: bool,
    ) -> Result<(), Error> {
        ctx.aad(aad, last)
    }

    /// Processes payload data from `input` into `output`.
    ///
    /// - `input` and `output` must have the same length.
    /// - `last` marks the final chunk.
    /// - Every chunk except the last must be a multiple of the block length, 64 bytes.
    ///
    /// Errors: `InvalidLength`.
    pub fn chachapoly_blocking_payload(
        &mut self,
        ctx: &mut ChaChaPolyContext,
        input: &[u8],
        output: &mut [u8],
        last: bool,
    ) -> Result<(), Error> {
        if input.len() != output.len() {
            return Err(Error::InvalidLength);
        }
        unsafe { ctx.payload(input.as_ptr(), output.as_mut_ptr(), input.len(), last) }
    }

    /// Processes payload data in place. See [`Self::chachapoly_blocking_payload`].
    pub fn chachapoly_blocking_payload_in_place(
        &mut self,
        ctx: &mut ChaChaPolyContext,
        data: &mut [u8],
        last: bool,
    ) -> Result<(), Error> {
        unsafe { ctx.payload(data.as_ptr(), data.as_mut_ptr(), data.len(), last) }
    }

    /// Finishes the operation and returns the 16-byte Poly1305 tag.
    ///
    /// When decrypting, compare the returned tag with the received one in constant time
    /// before using the plaintext.
    pub fn chachapoly_blocking_finish(&mut self, ctx: ChaChaPolyContext) -> Result<[u8; 16], Error> {
        ctx.finish()
    }
}
