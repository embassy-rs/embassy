//! CRACEN BA411E AES engine primitives.
//!
//! Register offsets, mode bits and descriptor tags follow `sxsymcrypt` in Nordic's
//! `nrf_security` (`blkcipher.c`, `cmac.c`, `aead.c`).

use core::ptr;

use super::{BLOCK_LEN, Direction, Error, Key};
use crate::crypto::cracen::cmdma::{self, Chain, ENGINE_AES, REALIGN, tag_config, tag_data, tag_ignore};

pub(super) const KEY_LENGTHS: &[usize] = &[16, 24, 32];

const TAG_CFG: u32 = tag_config(ENGINE_AES, 0x00);
const TAG_KEY: u32 = tag_config(ENGINE_AES, 0x08);
const TAG_IV: u32 = tag_config(ENGINE_AES, 0x28);
const TAG_MASK: u32 = tag_config(ENGINE_AES, 0x68);
const TAG_DATA: u32 = tag_data(ENGINE_AES, 0);
const TAG_AAD: u32 = tag_data(ENGINE_AES, 1);

/// Configuration word: mode of operation bits [16:8].
const MODE_ECB: u32 = 1 << 8;
const MODE_CBC: u32 = 1 << 9;
const MODE_CTR: u32 = 1 << 10;
const MODE_GCM: u32 = 1 << 14;
const MODE_CMAC: u32 = 1 << 16;
const CFG_DECRYPT: u32 = 1;
const CFG_CTX_LOAD: u32 = 1 << 4;
const CFG_CTX_SAVE: u32 = 1 << 5;

/// Loads a fresh random mask into the AES engine for its side-channel countermeasures.
///
/// NCS does the same before AES operations. CRACEN must be powered. The mask is lost when it
/// powers down.
pub(crate) fn load_countermeasure_mask() {
    let mask = crate::crypto::cracen::random_word();
    let mut input = Chain::<1>::new();
    input.push(&mask as *const u32 as *const u8, 4, REALIGN, TAG_MASK);
    let mut output = Chain::<1>::new();
    output.push(ptr::null(), 0, 0, 0);
    unsafe { cmdma::run(&mut input, &mut output) };
}

fn direction(dir: Direction) -> u32 {
    match dir {
        Direction::Encrypt => 0,
        Direction::Decrypt => CFG_DECRYPT,
    }
}

/// Runs one block-cipher transaction over whole blocks.
unsafe fn cipher(cfg: u32, key: &Key, iv: Option<&[u8; BLOCK_LEN]>, input: *const u8, output: *mut u8, len: usize) {
    debug_assert!(len % BLOCK_LEN == 0 && len != 0);
    let cfg = cfg;
    let mut input_chain = Chain::<4>::new();
    input_chain.push(&cfg as *const u32 as *const u8, 4, REALIGN, TAG_CFG);
    input_chain.push(key.as_slice().as_ptr(), key.as_slice().len(), REALIGN, TAG_KEY);
    if let Some(iv) = iv {
        input_chain.push(iv.as_ptr(), BLOCK_LEN, REALIGN, TAG_IV);
    }
    input_chain.push(input, len, REALIGN, TAG_DATA);
    let mut output_chain = Chain::<1>::new();
    output_chain.push(output, len, 0, 0);
    unsafe { cmdma::run(&mut input_chain, &mut output_chain) };
}

/// ECB over whole blocks.
pub(super) unsafe fn ecb(key: &Key, dir: Direction, input: *const u8, output: *mut u8, len: usize) {
    unsafe { cipher(MODE_ECB | direction(dir), key, None, input, output, len) }
}

/// CBC over whole blocks. `iv` is updated to the last ciphertext block.
pub(super) unsafe fn cbc(
    key: &Key,
    dir: Direction,
    iv: &mut [u8; BLOCK_LEN],
    input: *const u8,
    output: *mut u8,
    len: usize,
) {
    let mut last_ciphertext = [0u8; BLOCK_LEN];
    if dir == Direction::Decrypt {
        unsafe { ptr::copy_nonoverlapping(input.add(len - BLOCK_LEN), last_ciphertext.as_mut_ptr(), BLOCK_LEN) };
    }
    let iv_in = *iv;
    unsafe { cipher(MODE_CBC | direction(dir), key, Some(&iv_in), input, output, len) };
    if dir == Direction::Encrypt {
        unsafe {
            ptr::copy_nonoverlapping(
                output.add(len - BLOCK_LEN) as *const u8,
                last_ciphertext.as_mut_ptr(),
                BLOCK_LEN,
            )
        };
    }
    *iv = last_ciphertext;
}

/// CTR over whole blocks, starting at `counter`.
pub(super) unsafe fn ctr(key: &Key, counter: &[u8; BLOCK_LEN], input: *const u8, output: *mut u8, len: usize) {
    let counter = *counter;
    unsafe { cipher(MODE_CTR, key, Some(&counter), input, output, len) }
}

/// CBC-MAC over whole blocks, chaining from and updating `state`.
///
/// Runs CBC encryption and keeps only the last ciphertext block.
pub(super) fn cbc_mac(key: &Key, state: &mut [u8; BLOCK_LEN], blocks: &[u8]) {
    let cfg = MODE_CBC;
    let iv = *state;
    let mut out = [0u8; BLOCK_LEN];
    let mut input_chain = Chain::<4>::new();
    input_chain.push(&cfg as *const u32 as *const u8, 4, REALIGN, TAG_CFG);
    input_chain.push(key.as_slice().as_ptr(), key.as_slice().len(), REALIGN, TAG_KEY);
    input_chain.push(iv.as_ptr(), BLOCK_LEN, REALIGN, TAG_IV);
    input_chain.push(blocks.as_ptr(), blocks.len(), REALIGN, TAG_DATA);
    let mut output_chain = Chain::<2>::new();
    output_chain.push_discard(blocks.len() - BLOCK_LEN);
    output_chain.push(out.as_mut_ptr(), BLOCK_LEN, 0, 0);
    unsafe { cmdma::run(&mut input_chain, &mut output_chain) };
    *state = out;
}

/// CMAC over whole, non-final blocks. The engine's context is saved into `state`.
pub(super) fn cmac_update(key: &Key, state: &mut [u8; BLOCK_LEN], started: &mut bool, blocks: &[u8]) {
    let cfg = MODE_CMAC | CFG_CTX_SAVE | if *started { CFG_CTX_LOAD } else { 0 };
    let state_in = *state;
    let mut out = [0u8; BLOCK_LEN];
    let mut input_chain = Chain::<4>::new();
    input_chain.push(&cfg as *const u32 as *const u8, 4, REALIGN, TAG_CFG);
    input_chain.push(key.as_slice().as_ptr(), key.as_slice().len(), REALIGN, TAG_KEY);
    if *started {
        input_chain.push(state_in.as_ptr(), BLOCK_LEN, REALIGN, TAG_IV);
    }
    input_chain.push(blocks.as_ptr(), blocks.len(), REALIGN, TAG_DATA);
    let mut output_chain = Chain::<1>::new();
    output_chain.push(out.as_mut_ptr(), BLOCK_LEN, 0, 0);
    unsafe { cmdma::run(&mut input_chain, &mut output_chain) };
    *state = out;
    *started = true;
}

/// CMAC finalization over the last 1 to 16 bytes of the message, or over an empty message
/// (`tail` empty, `started` false).
pub(super) fn cmac_final(key: &Key, state: &[u8; BLOCK_LEN], started: bool, tail: &[u8]) -> [u8; BLOCK_LEN] {
    debug_assert!(!tail.is_empty() || !started);
    let cfg = MODE_CMAC | if started { CFG_CTX_LOAD } else { 0 };
    let state_in = *state;
    let mut buf = [0u8; BLOCK_LEN];
    buf[..tail.len()].copy_from_slice(tail);
    let mut tag = [0u8; BLOCK_LEN];
    let mut input_chain = Chain::<4>::new();
    input_chain.push(&cfg as *const u32 as *const u8, 4, REALIGN, TAG_CFG);
    input_chain.push(key.as_slice().as_ptr(), key.as_slice().len(), REALIGN, TAG_KEY);
    if started {
        input_chain.push(state_in.as_ptr(), BLOCK_LEN, REALIGN, TAG_IV);
    }
    // The engine works on whole blocks, and needs one even when the message is empty: the
    // bytes that are not message are marked as ignored, all sixteen of them if need be.
    input_chain.push(
        buf.as_ptr(),
        BLOCK_LEN,
        REALIGN,
        TAG_DATA | tag_ignore(BLOCK_LEN - tail.len()),
    );
    let mut output_chain = Chain::<1>::new();
    output_chain.push(tag.as_mut_ptr(), BLOCK_LEN, 0, 0);
    unsafe { cmdma::run(&mut input_chain, &mut output_chain) };
    tag
}

/// GCM state. Holds the IV before the first transaction, then the saved engine context.
#[derive(Clone, Copy, Default)]
pub(crate) struct GcmState {
    iv: [u8; 12],
    dir: u32,
    state: [u8; 32],
    loaded: bool,
    tag: Option<[u8; BLOCK_LEN]>,
}

/// Runs one GCM transaction: optional AAD, optional data, and with `finish` the final length
/// block.
///
/// Without `finish` the engine context is saved into `st`. With it the tag is returned.
unsafe fn gcm_run(
    st: &mut GcmState,
    key: &Key,
    aad: &[u8],
    input: *const u8,
    output: *mut u8,
    len: usize,
    finish: Option<[u8; BLOCK_LEN]>,
) -> Option<[u8; BLOCK_LEN]> {
    let cfg =
        MODE_GCM | st.dir | if st.loaded { CFG_CTX_LOAD } else { 0 } | if finish.is_none() { CFG_CTX_SAVE } else { 0 };
    let state_in = st.state;
    let mut state_out = [0u8; 32];
    let mut tag = [0u8; BLOCK_LEN];
    let full = len / BLOCK_LEN * BLOCK_LEN;
    let rem = len - full;
    let mut tail = [0u8; BLOCK_LEN];
    unsafe { ptr::copy_nonoverlapping(input.add(full), tail.as_mut_ptr(), rem) };
    let lengths = finish.unwrap_or([0; BLOCK_LEN]);

    let mut input_chain = Chain::<7>::new();
    input_chain.push(&cfg as *const u32 as *const u8, 4, REALIGN, TAG_CFG);
    input_chain.push(key.as_slice().as_ptr(), key.as_slice().len(), REALIGN, TAG_KEY);
    if st.loaded {
        input_chain.push(state_in.as_ptr(), 32, REALIGN, TAG_IV);
    } else {
        input_chain.push(st.iv.as_ptr(), 12, REALIGN, TAG_IV);
    }
    if !aad.is_empty() {
        input_chain.push(aad.as_ptr(), aad.len(), REALIGN, TAG_AAD);
    }
    if full != 0 {
        input_chain.push(input, full, REALIGN, TAG_DATA);
    }
    if rem != 0 {
        input_chain.push_padded(tail.as_ptr(), rem, BLOCK_LEN, TAG_DATA);
    }
    if finish.is_some() {
        input_chain.push(lengths.as_ptr(), BLOCK_LEN, REALIGN, TAG_DATA);
    }

    let mut output_chain = Chain::<5>::new();
    output_chain.push_discard(aad.len());
    if full != 0 {
        output_chain.push(output, full, 0, 0);
    }
    if rem != 0 {
        output_chain.push_out_padded(unsafe { output.add(full) }, rem, BLOCK_LEN);
    }
    if finish.is_some() {
        output_chain.push(tag.as_mut_ptr(), BLOCK_LEN, 0, 0);
    } else {
        output_chain.push(state_out.as_mut_ptr(), 32, 0, 0);
    }

    unsafe { cmdma::run(&mut input_chain, &mut output_chain) };

    if finish.is_some() {
        Some(tag)
    } else {
        st.state = state_out;
        st.loaded = true;
        None
    }
}

fn gcm_lengths(aad_len: usize, payload_len: usize) -> [u8; BLOCK_LEN] {
    let mut lengths = [0u8; BLOCK_LEN];
    lengths[..8].copy_from_slice(&((aad_len as u64) * 8).to_be_bytes());
    lengths[8..].copy_from_slice(&((payload_len as u64) * 8).to_be_bytes());
    lengths
}

pub(super) fn gcm_init(_key: &Key, iv: &[u8; 12], dir: Direction) -> GcmState {
    GcmState {
        iv: *iv,
        dir: direction(dir),
        state: [0; 32],
        loaded: false,
        tag: None,
    }
}

/// Feeds whole blocks of AAD.
pub(super) fn gcm_aad(st: &mut GcmState, key: &Key, blocks: &[u8]) {
    unsafe { gcm_run(st, key, blocks, ptr::null(), ptr::null_mut(), 0, None) };
}

/// Processes payload. `last` allows a partial final block and computes the tag.
pub(super) unsafe fn gcm_payload(
    st: &mut GcmState,
    key: &Key,
    _dir: Direction,
    input: *const u8,
    output: *mut u8,
    len: usize,
    last: bool,
    aad_len: usize,
    payload_len: usize,
) -> Result<(), Error> {
    if last {
        let lengths = gcm_lengths(aad_len, payload_len);
        st.tag = unsafe { gcm_run(st, key, &[], input, output, len, Some(lengths)) };
    } else if len != 0 {
        unsafe { gcm_run(st, key, &[], input, output, len, None) };
    }
    Ok(())
}

pub(super) fn gcm_finish(
    st: &mut GcmState,
    key: &Key,
    aad_len: usize,
    payload_len: usize,
) -> Result<[u8; BLOCK_LEN], Error> {
    if let Some(tag) = st.tag {
        return Ok(tag);
    }
    let lengths = gcm_lengths(aad_len, payload_len);
    Ok(unsafe { gcm_run(st, key, &[], ptr::null(), ptr::null_mut(), 0, Some(lengths)) }.unwrap())
}
