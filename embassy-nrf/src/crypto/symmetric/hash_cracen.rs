//! CRACEN BA413 hash engine primitives.
//!
//! Configuration words and descriptor tags follow `sxsymcrypt/src/hash.c` in Nordic's
//! `nrf_security`.
//!
//! Every transaction loads the state and exports it again. The engine's own padding and
//! finalization are never used. The shared layer in `hash.rs` pads.

use super::Kind;
use crate::crypto::cracen::cmdma::{self, Chain, ENGINE_HASH, REALIGN, TAG_LAST, tag_config, tag_data};

const TAG_CFG: u32 = tag_config(ENGINE_HASH, 0x00);
/// Initial state. Nordic sets `LAST` on this tag even though more descriptors follow.
const TAG_STATE: u32 = tag_data(ENGINE_HASH, 1) | TAG_LAST;
const TAG_DATA: u32 = tag_data(ENGINE_HASH, 0);

/// Configuration word: algorithm bits, with hardware padding and finalization off.
fn mode(kind: Kind) -> u32 {
    match kind {
        Kind::Sha1 => 0x02,
        Kind::Sha256 => 0x08,
        Kind::Sha384 => 0x10,
        Kind::Sha512 => 0x20,
    }
}

/// Runs the compression function over whole blocks, updating `state` (big-endian words).
pub(super) fn compress(kind: Kind, state: &mut [u8], blocks: &[u8]) {
    let cfg = mode(kind);
    let mut state_in = [0u8; 64];
    state_in[..state.len()].copy_from_slice(state);
    let mut state_out = [0u8; 64];

    let mut input_chain = Chain::<3>::new();
    input_chain.push(&cfg as *const u32 as *const u8, 4, REALIGN, TAG_CFG);
    input_chain.push(state_in.as_ptr(), state.len(), REALIGN, TAG_STATE);
    input_chain.push(blocks.as_ptr(), blocks.len(), 0, TAG_DATA);
    let mut output_chain = Chain::<1>::new();
    output_chain.push(state_out.as_mut_ptr(), state.len(), 0, 0);
    unsafe { cmdma::run(&mut input_chain, &mut output_chain) };

    state.copy_from_slice(&state_out[..state.len()]);
}
