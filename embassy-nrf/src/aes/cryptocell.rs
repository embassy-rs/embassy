//! CryptoCell AES engine primitives.

use core::ptr;

use super::{BLOCK_LEN, Direction, Key};
use crate::cryptocell::dma::{self, Flow};
use crate::pac;
use crate::pac::cc_aes::vals::{DecKey0, ModeKey0, NkKey0, Status};

#[cfg(feature = "_cryptocell-312")]
pub(super) const KEY_LENGTHS: &[usize] = &[16, 24, 32];
#[cfg(not(feature = "_cryptocell-312"))]
pub(super) const KEY_LENGTHS: &[usize] = &[16];

/// Keeps the CryptoCell powered while the driver exists.
pub(super) struct Handle {
    _activation: crate::cryptocell::CryptoCellActivationHandle,
}

impl Handle {
    pub(super) fn new() -> Self {
        Self {
            _activation: crate::cryptocell::activate(),
        }
    }
}

fn key_size(len: usize) -> NkKey0 {
    match len {
        16 => NkKey0::_128bits,
        #[cfg(feature = "_cryptocell-312")]
        24 => NkKey0::_192bits,
        #[cfg(feature = "_cryptocell-312")]
        32 => NkKey0::_256bits,
        _ => unreachable!(),
    }
}

fn direction(dir: Direction) -> DecKey0 {
    match dir {
        Direction::Encrypt => DecKey0::Encrypt,
        Direction::Decrypt => DecKey0::Decrypt,
    }
}

fn clocks(enable: bool) {
    pac::CC_MISC.aes_clk().write(|w| w.set_enable(enable));
    pac::CC_MISC.dma_clk().write(|w| w.set_enable(enable));
}

fn wait_aes_idle() {
    while pac::CC_AES.aes_busy().read().status() == Status::Busy {}
}

/// Enables the clocks and configures mode, direction and key.
fn setup(mode: ModeKey0, dir: DecKey0, key: &Key) {
    clocks(true);
    dma::prepare(Flow::AesActive);
    let r = pac::CC_AES;
    r.aes_control().write(|w| {
        w.set_dec_key0(dir);
        w.set_mode_key0(mode);
        w.set_nk_key0(key_size(key.as_slice().len()));
    });
    r.aes_remaining_bytes().write_value(0);
    for (i, word) in key.as_slice().chunks_exact(4).enumerate() {
        r.aes_key_0(i).write_value(u32::from_le_bytes(word.try_into().unwrap()));
    }
}

fn load_iv(iv: &[u8; BLOCK_LEN]) {
    for (i, word) in iv.chunks_exact(4).enumerate() {
        pac::CC_AES
            .aes_iv_0(i)
            .write_value(u32::from_le_bytes(word.try_into().unwrap()));
    }
}

fn read_iv() -> [u8; BLOCK_LEN] {
    let mut iv = [0; BLOCK_LEN];
    for (i, word) in iv.chunks_exact_mut(4).enumerate() {
        word.copy_from_slice(&pac::CC_AES.aes_iv_0(i).read().to_le_bytes());
    }
    iv
}

fn load_ctr(ctr: &[u8; BLOCK_LEN]) {
    for (i, word) in ctr.chunks_exact(4).enumerate() {
        pac::CC_AES
            .aes_ctr(i)
            .write_value(u32::from_le_bytes(word.try_into().unwrap()));
    }
}

/// ECB over whole blocks.
pub(super) unsafe fn ecb(key: &Key, dir: Direction, input: *const u8, output: *mut u8, len: usize) {
    critical_section::with(|_| {
        setup(ModeKey0::Ecb, direction(dir), key);
        unsafe { dma::transfer(input, output, len) };
        clocks(false);
    })
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
        // Read it before the (possibly in-place) transfer overwrites it.
        unsafe { ptr::copy_nonoverlapping(input.add(len - BLOCK_LEN), last_ciphertext.as_mut_ptr(), BLOCK_LEN) };
    }
    critical_section::with(|_| {
        setup(ModeKey0::Cbc, direction(dir), key);
        load_iv(iv);
        unsafe { dma::transfer(input, output, len) };
        clocks(false);
    });
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
    critical_section::with(|_| {
        setup(ModeKey0::Ctr, DecKey0::Encrypt, key);
        load_ctr(counter);
        unsafe { dma::transfer(input, output, len) };
        clocks(false);
    })
}

/// CBC-MAC over whole blocks, chaining from and updating `state`.
pub(super) fn cbc_mac(key: &Key, state: &mut [u8; BLOCK_LEN], blocks: &[u8]) {
    critical_section::with(|_| {
        setup(ModeKey0::CbcMac, DecKey0::Encrypt, key);
        load_iv(state);
        unsafe { dma::transfer(blocks.as_ptr(), ptr::null_mut(), blocks.len()) };
        *state = read_iv();
        clocks(false);
    })
}

/// CMAC over whole, non-final blocks.
pub(super) fn cmac_update(key: &Key, state: &mut [u8; BLOCK_LEN], started: &mut bool, blocks: &[u8]) {
    critical_section::with(|_| {
        setup(ModeKey0::Cmac, DecKey0::Encrypt, key);
        load_iv(state);
        pac::CC_AES.aes_cmac_init().write(|w| w.set_enable(true));
        wait_aes_idle();
        unsafe { dma::transfer(blocks.as_ptr(), ptr::null_mut(), blocks.len()) };
        *state = read_iv();
        clocks(false);
    });
    *started = true;
}

/// CMAC finalization over the last 1 to 16 bytes of the message, or over an empty message
/// (`tail` empty, `started` false).
pub(super) fn cmac_final(key: &Key, state: &[u8; BLOCK_LEN], started: bool, tail: &[u8]) -> [u8; BLOCK_LEN] {
    debug_assert!(!tail.is_empty() || !started);
    let mut buf = [0u8; BLOCK_LEN];
    buf[..tail.len()].copy_from_slice(tail);
    critical_section::with(|_| {
        setup(ModeKey0::Cmac, DecKey0::Encrypt, key);
        load_iv(state);
        pac::CC_AES.aes_cmac_init().write(|w| w.set_enable(true));
        wait_aes_idle();
        if tail.is_empty() {
            pac::CC_AES.aes_cmac_size0_kick().write(|w| w.set_enable(true));
            wait_aes_idle();
        } else {
            pac::CC_AES.aes_remaining_bytes().write_value(tail.len() as u32);
            unsafe { dma::transfer(buf.as_ptr(), ptr::null_mut(), tail.len()) };
        }
        let tag = read_iv();
        clocks(false);
        tag
    })
}

#[cfg(feature = "_cryptocell-312")]
mod gcm {
    use core::ptr;

    use super::super::{BLOCK_LEN, CtrState, Direction, Error, Key};
    use crate::cryptocell::dma::{self, Flow};
    use crate::pac;
    use crate::pac::cc_hash::vals::Engine;
    use crate::util::for_each_ram_chunk;

    /// GCM state: GHASH subkey and accumulator, J0 and the CTR state.
    #[derive(Clone, Copy, Default)]
    pub(crate) struct GcmState {
        h: [u8; BLOCK_LEN],
        j0: [u8; BLOCK_LEN],
        ghash: [u8; BLOCK_LEN],
        ctr: Option<CtrState>,
    }

    /// Runs GHASH over whole blocks, updating `state`.
    fn ghash(h: &[u8; BLOCK_LEN], state: &mut [u8; BLOCK_LEN], blocks: &[u8]) {
        critical_section::with(|_| {
            pac::CC_MISC.hash_clk().write(|w| w.set_enable(true));
            pac::CC_MISC.dma_clk().write(|w| w.set_enable(true));
            dma::prepare(Flow::HashActive);
            pac::CC_GHASH.ghash_init().write(|w| w.set_enable(true));
            for (i, word) in h.chunks_exact(4).enumerate() {
                pac::CC_GHASH
                    .ghash_subkey(i)
                    .write_value(u32::from_le_bytes(word.try_into().unwrap()));
            }
            for (i, word) in state.chunks_exact(4).enumerate() {
                pac::CC_GHASH
                    .ghash_iv(i)
                    .write_value(u32::from_le_bytes(word.try_into().unwrap()));
            }
            pac::CC_HASH.hash_select().write(|w| w.set_engine(Engine::Ghash));
            pac::CC_HASH.hash_xor_din().write_value(0);
            unsafe { dma::transfer(blocks.as_ptr(), ptr::null_mut(), blocks.len()) };
            for (i, word) in state.chunks_exact_mut(4).enumerate() {
                word.copy_from_slice(&pac::CC_GHASH.ghash_iv(i).read().to_le_bytes());
            }
            pac::CC_HASH.hash_select().write(|w| w.set_engine(Engine::Hash));
            pac::CC_MISC.hash_clk().write(|w| w.set_enable(false));
            pac::CC_MISC.dma_clk().write(|w| w.set_enable(false));
        })
    }

    /// GHASHes `len` bytes, zero-padding a trailing partial block.
    unsafe fn ghash_data(st: &mut GcmState, data: *const u8, len: usize) {
        let full = len / BLOCK_LEN * BLOCK_LEN;
        let blocks = unsafe { core::slice::from_raw_parts(data, full) };
        for_each_ram_chunk(blocks, super::super::MAX_CHUNK, |chunk| {
            ghash(&st.h, &mut st.ghash, chunk)
        });
        if len > full {
            let mut block = [0u8; BLOCK_LEN];
            unsafe { ptr::copy_nonoverlapping(data.add(full), block.as_mut_ptr(), len - full) };
            ghash(&st.h, &mut st.ghash, &block);
        }
    }

    pub(crate) fn gcm_init(key: &Key, iv: &[u8; 12], _dir: Direction) -> GcmState {
        let zero = [0u8; BLOCK_LEN];
        let mut h = [0u8; BLOCK_LEN];
        unsafe { super::ecb(key, Direction::Encrypt, zero.as_ptr(), h.as_mut_ptr(), BLOCK_LEN) };
        let mut j0 = [0u8; BLOCK_LEN];
        j0[..12].copy_from_slice(iv);
        j0[15] = 1;
        let mut counter = j0;
        counter[15] = 2;
        GcmState {
            h,
            j0,
            ghash: [0; BLOCK_LEN],
            ctr: Some(CtrState::new(counter)),
        }
    }

    pub(crate) fn gcm_aad(st: &mut GcmState, _key: &Key, blocks: &[u8]) {
        ghash(&st.h, &mut st.ghash, blocks);
    }

    pub(crate) unsafe fn gcm_payload(
        st: &mut GcmState,
        key: &Key,
        dir: Direction,
        input: *const u8,
        output: *mut u8,
        len: usize,
        _last: bool,
        _aad_len: usize,
        _payload_len: usize,
    ) -> Result<(), Error> {
        let mut ctr = st.ctr.take().unwrap();
        match dir {
            Direction::Encrypt => {
                unsafe { ctr.apply(key, input, output, len) };
                unsafe { ghash_data(st, output as *const u8, len) };
            }
            Direction::Decrypt => {
                unsafe { ghash_data(st, input, len) };
                unsafe { ctr.apply(key, input, output, len) };
            }
        }
        st.ctr = Some(ctr);
        Ok(())
    }

    pub(crate) fn gcm_finish(
        st: &mut GcmState,
        key: &Key,
        aad_len: usize,
        payload_len: usize,
    ) -> Result<[u8; BLOCK_LEN], Error> {
        let mut lengths = [0u8; BLOCK_LEN];
        lengths[..8].copy_from_slice(&((aad_len as u64) * 8).to_be_bytes());
        lengths[8..].copy_from_slice(&((payload_len as u64) * 8).to_be_bytes());
        ghash(&st.h, &mut st.ghash, &lengths);
        let mut tag = [0u8; BLOCK_LEN];
        unsafe { super::ctr(key, &st.j0, st.ghash.as_ptr(), tag.as_mut_ptr(), BLOCK_LEN) };
        Ok(tag)
    }
}

#[cfg(feature = "_cryptocell-312")]
pub(super) use gcm::{GcmState, gcm_aad, gcm_finish, gcm_init, gcm_payload};
