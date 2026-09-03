//! CRACEN BA417 ChaCha20-Poly1305 engine primitives.
//!
//! Configuration words and descriptor tags follow `sxsymcrypt/src/chachapoly.c` in Nordic's
//! `nrf_security`.

use super::{BLOCK_LEN, KEY_LEN, NONCE_LEN};
use crate::cracen::cmdma::{self, Chain, ENGINE_CHACHA, REALIGN, tag_config, tag_data};

const TAG_CFG: u32 = tag_config(ENGINE_CHACHA, 0x00);
const TAG_KEY: u32 = tag_config(ENGINE_CHACHA, 0x04);
/// Block counter, or the saved context when resuming.
const TAG_COUNTER: u32 = tag_config(ENGINE_CHACHA, 0x28);
const TAG_NONCE: u32 = tag_config(ENGINE_CHACHA, 0x2C);
const TAG_DATA: u32 = tag_data(ENGINE_CHACHA, 0);
const TAG_AAD: u32 = tag_data(ENGINE_CHACHA, 1);

const MODE_CHACHA20_POLY1305: u32 = 0;
const MODE_CHACHA20: u32 = 1;
const CFG_DECRYPT: u32 = 1 << 2;
const CFG_CTX_LOAD: u32 = 1 << 5;
const CFG_CTX_SAVE: u32 = 1 << 6;

/// Keeps CRACEN powered while the driver exists.
pub(super) struct Handle {
    _activation: crate::cracen::CracenActivationHandle,
}

impl Handle {
    pub(super) fn new() -> Self {
        Self {
            _activation: crate::cracen::activate(),
        }
    }
}

/// Applies the keystream over whole blocks, starting at block `counter`.
pub(super) unsafe fn apply(
    key: &[u8; KEY_LEN],
    nonce: &[u8; NONCE_LEN],
    counter: u32,
    input: *const u8,
    output: *mut u8,
    len: usize,
) {
    debug_assert!(len % BLOCK_LEN == 0 && len != 0);
    let cfg = MODE_CHACHA20;
    // The engine takes the counter as big-endian bytes (NCS feeds `00 00 00 01` for 1).
    let counter = counter.to_be_bytes();
    let key = *key;
    let nonce = *nonce;
    let mut input_chain = Chain::<5>::new();
    input_chain.push(&cfg as *const u32 as *const u8, 4, REALIGN, TAG_CFG);
    input_chain.push(key.as_ptr(), KEY_LEN, REALIGN, TAG_KEY);
    input_chain.push(counter.as_ptr(), 4, REALIGN, TAG_COUNTER);
    input_chain.push(nonce.as_ptr(), NONCE_LEN, REALIGN, TAG_NONCE);
    input_chain.push(input, len, REALIGN, TAG_DATA);
    let mut output_chain = Chain::<1>::new();
    output_chain.push(output, len, 0, 0);
    unsafe { cmdma::run(&mut input_chain, &mut output_chain) };
}

pub(super) mod aead {
    use core::ptr;

    use super::super::{BLOCK_LEN, Direction, Error, KEY_LEN, NONCE_LEN};
    use super::{
        CFG_CTX_LOAD, CFG_CTX_SAVE, CFG_DECRYPT, MODE_CHACHA20_POLY1305, TAG_AAD, TAG_CFG, TAG_COUNTER, TAG_DATA,
        TAG_KEY, TAG_NONCE,
    };
    use crate::cracen::cmdma::{self, Chain, REALIGN};

    /// Poly1305 tag length.
    const TAG_LEN: usize = 16;
    /// Size of the saved engine context (ChaCha20 state and Poly1305 accumulator).
    const STATE_LEN: usize = 48;
    /// AAD and data are padded to this in the engine.
    const ALIGN: usize = 16;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Phase {
        Aad,
        Payload,
    }

    /// State of an in-progress ChaCha20-Poly1305 operation, created by
    /// [`ChaCha::start_aead`](super::super::ChaCha::start_aead).
    #[derive(Clone)]
    pub struct AeadContext {
        key: [u8; KEY_LEN],
        nonce: [u8; NONCE_LEN],
        dir: u32,
        phase: Phase,
        /// Saved engine context, valid once `loaded`.
        state: [u8; STATE_LEN],
        loaded: bool,
        /// Partial block of AAD.
        aad_buf: [u8; ALIGN],
        aad_buf_len: u8,
        aad_len: usize,
        payload_len: usize,
        tag: Option<[u8; TAG_LEN]>,
    }

    impl AeadContext {
        pub(crate) fn new(key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN], dir: Direction) -> Self {
            Self {
                key: *key,
                nonce: *nonce,
                dir: match dir {
                    Direction::Encrypt => 0,
                    Direction::Decrypt => CFG_DECRYPT,
                },
                phase: Phase::Aad,
                state: [0; STATE_LEN],
                loaded: false,
                aad_buf: [0; ALIGN],
                aad_buf_len: 0,
                aad_len: 0,
                payload_len: 0,
                tag: None,
            }
        }

        /// One transaction: optional AAD (multiple of 16 bytes), optional data, and
        /// optionally the final length block producing the tag.
        unsafe fn run(&mut self, aad: &[u8], input: *const u8, output: *mut u8, len: usize, finish: bool) {
            let cfg = MODE_CHACHA20_POLY1305
                | self.dir
                | if self.loaded { CFG_CTX_LOAD } else { 0 }
                | if finish { 0 } else { CFG_CTX_SAVE };
            let key = self.key;
            let nonce = self.nonce;
            // The data counter starts at 1; block 0 derives the Poly1305 key.
            let counter = [0u8, 0, 0, 1];
            let state_in = self.state;
            let mut state_out = [0u8; STATE_LEN];
            let mut tag = [0u8; TAG_LEN];
            let full = len / ALIGN * ALIGN;
            let rem = len - full;
            let mut tail = [0u8; ALIGN];
            unsafe { ptr::copy_nonoverlapping(input.add(full), tail.as_mut_ptr(), rem) };
            let mut lengths = [0u8; 16];
            lengths[..8].copy_from_slice(&(self.aad_len as u64).to_le_bytes());
            lengths[8..].copy_from_slice(&(self.payload_len as u64).to_le_bytes());

            let mut input_chain = Chain::<8>::new();
            input_chain.push(&cfg as *const u32 as *const u8, 4, REALIGN, TAG_CFG);
            input_chain.push(key.as_ptr(), KEY_LEN, REALIGN, TAG_KEY);
            if self.loaded {
                input_chain.push(state_in.as_ptr(), STATE_LEN, REALIGN, TAG_COUNTER);
            } else {
                input_chain.push(counter.as_ptr(), 4, REALIGN, TAG_COUNTER);
                input_chain.push(nonce.as_ptr(), NONCE_LEN, REALIGN, TAG_NONCE);
            }
            if !aad.is_empty() {
                input_chain.push(aad.as_ptr(), aad.len(), REALIGN, TAG_AAD);
            }
            if full != 0 {
                input_chain.push(input, full, REALIGN, TAG_DATA);
            }
            if rem != 0 {
                input_chain.push_padded(tail.as_ptr(), rem, ALIGN, TAG_DATA);
            }
            if finish {
                input_chain.push(lengths.as_ptr(), 16, REALIGN, TAG_DATA);
            }

            let mut output_chain = Chain::<5>::new();
            output_chain.push_discard(aad.len());
            if full != 0 {
                output_chain.push(output, full, 0, 0);
            }
            if rem != 0 {
                output_chain.push_out_padded(unsafe { output.add(full) }, rem, ALIGN);
            }
            if finish {
                output_chain.push(tag.as_mut_ptr(), TAG_LEN, 0, 0);
            } else {
                output_chain.push(state_out.as_mut_ptr(), STATE_LEN, 0, 0);
            }

            unsafe { cmdma::run(&mut input_chain, &mut output_chain) };

            if finish {
                self.tag = Some(tag);
            } else {
                self.state = state_out;
                self.loaded = true;
            }
        }

        fn feed_aad_blocks(&mut self, blocks: &[u8]) {
            if !blocks.is_empty() {
                unsafe { self.run(blocks, ptr::null(), ptr::null_mut(), 0, false) };
            }
        }

        fn end_aad(&mut self) {
            if self.aad_buf_len != 0 {
                self.aad_buf[self.aad_buf_len as usize..].fill(0);
                let block = self.aad_buf;
                self.aad_buf_len = 0;
                self.feed_aad_blocks(&block);
            }
            self.phase = Phase::Payload;
        }

        pub(crate) fn aad(&mut self, aad: &[u8], last: bool) -> Result<(), Error> {
            if self.phase != Phase::Aad {
                return Err(Error::AadAfterPayload);
            }
            self.aad_len += aad.len();
            let mut data = aad;
            if self.aad_buf_len != 0 {
                let n = (ALIGN - self.aad_buf_len as usize).min(data.len());
                self.aad_buf[self.aad_buf_len as usize..][..n].copy_from_slice(&data[..n]);
                self.aad_buf_len += n as u8;
                data = &data[n..];
                if self.aad_buf_len as usize == ALIGN {
                    let block = self.aad_buf;
                    self.aad_buf_len = 0;
                    self.feed_aad_blocks(&block);
                }
            }
            let full = data.len() / ALIGN * ALIGN;
            crate::util::for_each_ram_chunk(&data[..full], super::super::MAX_CHUNK, |chunk| {
                self.feed_aad_blocks(chunk)
            });
            self.aad_buf[..data.len() - full].copy_from_slice(&data[full..]);
            self.aad_buf_len += (data.len() - full) as u8;
            if last {
                self.end_aad();
            }
            Ok(())
        }

        pub(crate) unsafe fn payload(
            &mut self,
            input: *const u8,
            output: *mut u8,
            len: usize,
            last: bool,
        ) -> Result<(), Error> {
            if self.phase == Phase::Aad {
                self.end_aad();
            }
            if !last && len % BLOCK_LEN != 0 {
                return Err(Error::InvalidLength);
            }
            self.payload_len += len;
            if last {
                unsafe {
                    crate::util::for_each_ram_chunk_inout(input, output, len, super::super::MAX_CHUNK, |i, o, l| {
                        // Each chunk but the last saves the context; the last one finishes.
                        let is_last = i.add(l) == input.add(len);
                        self.run(&[], i, o, l, is_last);
                    })
                };
                if len == 0 {
                    unsafe { self.run(&[], ptr::null(), ptr::null_mut(), 0, true) };
                }
            } else {
                unsafe {
                    crate::util::for_each_ram_chunk_inout(input, output, len, super::super::MAX_CHUNK, |i, o, l| {
                        self.run(&[], i, o, l, false)
                    })
                };
            }
            Ok(())
        }

        pub(crate) fn finish(mut self) -> Result<[u8; TAG_LEN], Error> {
            if self.phase == Phase::Aad {
                self.end_aad();
            }
            if let Some(tag) = self.tag {
                return Ok(tag);
            }
            unsafe { self.run(&[], ptr::null(), ptr::null_mut(), 0, true) };
            Ok(self.tag.unwrap())
        }
    }
}
