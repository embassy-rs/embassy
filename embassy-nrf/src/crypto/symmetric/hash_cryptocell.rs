//! CryptoCell HASH engine primitives.

use core::ptr;

use super::Kind;
use crate::crypto::cryptocell::dma::{self, Flow};
use crate::pac;
use crate::pac::cc_hash::vals::Mode;

/// Runs the compression function over whole blocks, updating `state` (big-endian words).
pub(super) fn compress(kind: Kind, state: &mut [u8], blocks: &[u8]) {
    debug_assert!(blocks.len() % 64 == 0);
    let mode = match kind {
        Kind::Sha1 => Mode::Sha1,
        Kind::Sha256 => Mode::Sha256,
        _ => unreachable!(),
    };
    let words = state.len() / 4;

    pac::CC_MISC.hash_clk().write(|w| w.set_enable(true));
    pac::CC_MISC.dma_clk().write(|w| w.set_enable(true));
    dma::prepare(Flow::HashActive);

    let r = pac::CC_HASH;
    r.hash_control().write(|w| w.set_mode(mode));
    #[cfg(feature = "_cryptocell-312")]
    r.hash_select()
        .write(|w| w.set_engine(pac::cc_hash::vals::Engine::Hash));
    // No hardware padding: the caller pads, so the engine only compresses whole blocks.
    r.hash_pad().write(|w| w.set_enable(true));
    r.hash_pad_auto().write(|w| w.set_hwpad(false));
    r.hash_cur_len_0().write_value(0);
    r.hash_cur_len_1().write_value(0);
    // The state registers must be written in descending order (H7 first); ascending
    // writes leave the engine with a different initial state.
    for i in (0..words).rev() {
        r.hash_h(i)
            .write_value(u32::from_be_bytes(state[4 * i..4 * i + 4].try_into().unwrap()));
    }

    unsafe { dma::transfer(blocks.as_ptr(), ptr::null_mut(), blocks.len()) };

    for i in 0..words {
        state[4 * i..4 * i + 4].copy_from_slice(&r.hash_h(i).read().to_be_bytes());
    }

    pac::CC_MISC.hash_clk().write(|w| w.set_enable(false));
    pac::CC_MISC.dma_clk().write(|w| w.set_enable(false));
}
