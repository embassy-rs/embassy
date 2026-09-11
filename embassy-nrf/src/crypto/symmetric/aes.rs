//! AES.

use super::{Direction, Error, MAX_CHUNK, Symmetric};
use crate::mode::Mode;
use crate::util::{for_each_ram_chunk, for_each_ram_chunk_inout};

#[cfg_attr(feature = "_cryptocell", path = "aes_cryptocell.rs")]
#[cfg_attr(feature = "_cracen", path = "aes_cracen.rs")]
mod hw;

#[cfg(feature = "_cracen")]
pub(super) use hw::load_countermeasure_mask;

/// AES block length in bytes.
pub const AES_BLOCK_LEN: usize = 16;
const BLOCK_LEN: usize = AES_BLOCK_LEN;

/// Key material, stored inline so ciphers and contexts own their key.
#[derive(Clone, Copy)]
pub(crate) struct Key {
    bytes: [u8; 32],
    len: u8,
}

impl Key {
    fn new(key: &[u8]) -> Result<Self, Error> {
        if !hw::KEY_LENGTHS.contains(&key.len()) {
            return Err(Error::InvalidKeyLength);
        }
        let mut bytes = [0; 32];
        bytes[..key.len()].copy_from_slice(key);
        Ok(Self {
            bytes,
            len: key.len() as u8,
        })
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }
}

/// Mode-specific parameters of a cipher.
pub(crate) enum Params<'a> {
    Ecb,
    Cbc {
        iv: &'a [u8; BLOCK_LEN],
    },
    Ctr {
        iv: &'a [u8; BLOCK_LEN],
    },
    Cmac,
    Ccm {
        nonce: &'a [u8],
        tag_len: usize,
        aad_len: usize,
        payload_len: usize,
    },
    #[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
    Gcm {
        iv: &'a [u8; 12],
    },
}

pub(crate) trait SealedCipher {
    fn key(&self) -> &Key;
    fn params(&self) -> Params<'_>;
}

/// AES cipher mode. Implemented by [`AesEcb`], [`AesCbc`], [`AesCtr`], [`AesCmac`],
/// [`AesCcm`] and `AesGcm`.
///
/// This trait is sealed.
#[allow(private_bounds)]
pub trait Cipher: SealedCipher + Copy {}

/// AES cipher mode that accepts additional authenticated data. Implemented by [`AesCcm`] and
/// `AesGcm`.
///
/// This trait is sealed.
#[allow(private_bounds)]
pub trait AuthenticatedCipher: Cipher {}

/// AES in ECB (electronic codebook) mode.
///
/// Data must be a multiple of the block length.
///
/// ECB does not hide data patterns. Do not use it to encrypt messages directly.
#[derive(Clone, Copy)]
pub struct AesEcb {
    key: Key,
}

impl AesEcb {
    /// Creates an ECB cipher.
    ///
    /// The key is 16, 24 or 32 bytes. See the [module](super) documentation for which key
    /// lengths each chip supports.
    ///
    /// Errors: `InvalidKeyLength`.
    pub fn new(key: &[u8]) -> Result<Self, Error> {
        Ok(Self { key: Key::new(key)? })
    }
}

impl SealedCipher for AesEcb {
    fn key(&self) -> &Key {
        &self.key
    }
    fn params(&self) -> Params<'_> {
        Params::Ecb
    }
}
impl Cipher for AesEcb {}

/// AES in CBC (cipher block chaining) mode.
///
/// Data must be a multiple of the block length.
///
/// The IV must be unpredictable and unique for every message.
#[derive(Clone, Copy)]
pub struct AesCbc {
    key: Key,
    iv: [u8; BLOCK_LEN],
}

impl AesCbc {
    /// Creates a CBC cipher.
    ///
    /// The key is 16, 24 or 32 bytes. See the [module](super) documentation for which key
    /// lengths each chip supports.
    ///
    /// Errors: `InvalidKeyLength`.
    pub fn new(key: &[u8], iv: &[u8; BLOCK_LEN]) -> Result<Self, Error> {
        Ok(Self {
            key: Key::new(key)?,
            iv: *iv,
        })
    }
}

impl SealedCipher for AesCbc {
    fn key(&self) -> &Key {
        &self.key
    }
    fn params(&self) -> Params<'_> {
        Params::Cbc { iv: &self.iv }
    }
}
impl Cipher for AesCbc {}

/// AES in CTR (counter) mode, as in NIST SP 800-38A.
///
/// - Data may have any length.
/// - Encryption and decryption are the same operation.
/// - The 16-byte counter block is a big-endian integer, incremented once per block.
/// - The initial counter block must never repeat for the same key.
#[derive(Clone, Copy)]
pub struct AesCtr {
    key: Key,
    iv: [u8; BLOCK_LEN],
}

impl AesCtr {
    /// Creates a CTR cipher with the given key and initial counter block.
    ///
    /// The key is 16, 24 or 32 bytes. See the [module](super) documentation for which key
    /// lengths each chip supports.
    ///
    /// Errors: `InvalidKeyLength`.
    pub fn new(key: &[u8], iv: &[u8; BLOCK_LEN]) -> Result<Self, Error> {
        Ok(Self {
            key: Key::new(key)?,
            iv: *iv,
        })
    }
}

impl SealedCipher for AesCtr {
    fn key(&self) -> &Key {
        &self.key
    }
    fn params(&self) -> Params<'_> {
        Params::Ctr { iv: &self.iv }
    }
}
impl Cipher for AesCtr {}

/// AES-CMAC message authentication code, as in NIST SP 800-38B.
///
/// - The direction passed to [`Symmetric::aes_start`] is ignored.
/// - Feed the message with [`Symmetric::aes_blocking_payload`]. The output buffer is
///   ignored and may be empty.
/// - [`Symmetric::aes_blocking_finish`] returns the 16-byte tag.
#[derive(Clone, Copy)]
pub struct AesCmac {
    key: Key,
}

impl AesCmac {
    /// Creates a CMAC.
    ///
    /// The key is 16, 24 or 32 bytes. See the [module](super) documentation for which key
    /// lengths each chip supports.
    ///
    /// Errors: `InvalidKeyLength`.
    pub fn new(key: &[u8]) -> Result<Self, Error> {
        Ok(Self { key: Key::new(key)? })
    }
}

impl SealedCipher for AesCmac {
    fn key(&self) -> &Key {
        &self.key
    }
    fn params(&self) -> Params<'_> {
        Params::Cmac
    }
}
impl Cipher for AesCmac {}

/// AES in CCM (counter with CBC-MAC) authenticated mode, as in NIST SP 800-38C.
///
/// CCM needs the total length of the additional authenticated data and of the payload up
/// front. Only the first `tag_len` bytes of the tag returned by
/// [`Symmetric::aes_blocking_finish`] are valid.
#[derive(Clone, Copy)]
pub struct AesCcm {
    key: Key,
    nonce: [u8; 13],
    nonce_len: u8,
    tag_len: u8,
    aad_len: usize,
    payload_len: usize,
}

impl AesCcm {
    /// Creates a CCM cipher.
    ///
    /// - `key`: 16, 24 or 32 bytes. See the [module](super) documentation for which key
    ///   lengths each chip supports.
    /// - `nonce`: 7 to 13 bytes, unique for every message.
    /// - `aad_len`: total length of the additional authenticated data.
    /// - `payload_len`: total length of the payload.
    /// - `tag_len`: 4, 6, 8, 10, 12, 14 or 16 bytes.
    ///
    /// A shorter nonce allows a longer payload. With a 13-byte nonce the payload is limited
    /// to 65535 bytes.
    ///
    /// Errors: `InvalidKeyLength`, `InvalidNonceLength`, `InvalidTagLength`, and
    /// `InvalidLength` if the payload is too long for the nonce.
    pub fn new(key: &[u8], nonce: &[u8], aad_len: usize, payload_len: usize, tag_len: usize) -> Result<Self, Error> {
        let key = Key::new(key)?;
        if !(7..=13).contains(&nonce.len()) {
            return Err(Error::InvalidNonceLength);
        }
        if !(4..=16).contains(&tag_len) || tag_len % 2 != 0 {
            return Err(Error::InvalidTagLength);
        }
        // The payload length must fit in the L = 15 - nonce_len bytes of the length field.
        let l = 15 - nonce.len();
        if l < 8 && (payload_len as u64) >= 1u64 << (8 * l) {
            return Err(Error::InvalidLength);
        }
        let mut n = [0; 13];
        n[..nonce.len()].copy_from_slice(nonce);
        Ok(Self {
            key,
            nonce: n,
            nonce_len: nonce.len() as u8,
            tag_len: tag_len as u8,
            aad_len,
            payload_len,
        })
    }
}

impl SealedCipher for AesCcm {
    fn key(&self) -> &Key {
        &self.key
    }
    fn params(&self) -> Params<'_> {
        Params::Ccm {
            nonce: &self.nonce[..self.nonce_len as usize],
            tag_len: self.tag_len as usize,
            aad_len: self.aad_len,
            payload_len: self.payload_len,
        }
    }
}
impl Cipher for AesCcm {}
impl AuthenticatedCipher for AesCcm {}

/// AES in GCM (Galois/counter) authenticated mode, as in NIST SP 800-38D.
///
/// Not available on nRF52840 and nRF91.
///
/// Only 12-byte IVs are supported. The IV must be unique for every message. Reusing one
/// breaks the security of the mode. The tag returned by [`Symmetric::aes_blocking_finish`]
/// is 16 bytes.
#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
#[derive(Clone, Copy)]
pub struct AesGcm {
    key: Key,
    iv: [u8; 12],
}

#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
impl AesGcm {
    /// Creates a GCM cipher with the given key and 12-byte IV.
    ///
    /// The key is 16, 24 or 32 bytes.
    ///
    /// Errors: `InvalidKeyLength`.
    pub fn new(key: &[u8], iv: &[u8; 12]) -> Result<Self, Error> {
        Ok(Self {
            key: Key::new(key)?,
            iv: *iv,
        })
    }
}

#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
impl SealedCipher for AesGcm {
    fn key(&self) -> &Key {
        &self.key
    }
    fn params(&self) -> Params<'_> {
        Params::Gcm { iv: &self.iv }
    }
}
#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
impl Cipher for AesGcm {}
#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
impl AuthenticatedCipher for AesGcm {}

/// Accumulator for block-based MACs (CMAC, CBC-MAC) and block-aligned AAD.
#[derive(Clone, Copy)]
struct MacAcc {
    state: [u8; BLOCK_LEN],
    buf: [u8; BLOCK_LEN],
    buf_len: u8,
    started: bool,
}

impl MacAcc {
    const fn new() -> Self {
        Self {
            state: [0; BLOCK_LEN],
            buf: [0; BLOCK_LEN],
            buf_len: 0,
            started: false,
        }
    }

    /// Absorbs `data`. Complete blocks are passed to `process`, the rest is buffered.
    ///
    /// With `hold_back`, the last block is kept until [`Self::flush`]. CMAC needs this, so
    /// that `process` knows which block is the final one.
    fn absorb(&mut self, data: &[u8], hold_back: bool, mut process: impl FnMut(&mut Self, &[u8])) {
        let total = self.buf_len as usize + data.len();
        let keep = if hold_back {
            if total == 0 { 0 } else { (total - 1) % BLOCK_LEN + 1 }
        } else {
            total % BLOCK_LEN
        };
        if total - keep == 0 {
            self.buf[self.buf_len as usize..total].copy_from_slice(data);
            self.buf_len = total as u8;
            return;
        }

        let mut data = data;
        if self.buf_len != 0 {
            let fill = BLOCK_LEN - self.buf_len as usize;
            self.buf[self.buf_len as usize..].copy_from_slice(&data[..fill]);
            data = &data[fill..];
            let block = self.buf;
            self.buf_len = 0;
            process(self, &block);
        }
        let direct = data.len() - keep;
        for_each_ram_chunk(&data[..direct], MAX_CHUNK, |chunk| process(self, chunk));
        self.buf[..keep].copy_from_slice(&data[direct..]);
        self.buf_len = keep as u8;
    }

    /// Zero-pads and processes the buffered partial block, if any.
    fn flush(&mut self, mut process: impl FnMut(&mut Self, &[u8])) {
        if self.buf_len != 0 {
            self.buf[self.buf_len as usize..].fill(0);
            let block = self.buf;
            self.buf_len = 0;
            process(self, &block);
        }
    }
}

/// Counter-mode state: the next counter block and unused keystream bytes.
#[derive(Clone, Copy)]
pub(crate) struct CtrState {
    counter: [u8; BLOCK_LEN],
    /// Unused keystream bytes are stored at the end of this buffer.
    ks: [u8; BLOCK_LEN],
    ks_len: u8,
}

impl CtrState {
    pub(crate) const fn new(counter: [u8; BLOCK_LEN]) -> Self {
        Self {
            counter,
            ks: [0; BLOCK_LEN],
            ks_len: 0,
        }
    }

    /// XORs the keystream into `len` bytes from `input` to `output`.
    ///
    /// # Safety
    ///
    /// `input` must be readable and `output` writable for `len` bytes. If they overlap, they
    /// must be equal.
    pub(crate) unsafe fn apply(&mut self, key: &Key, input: *const u8, output: *mut u8, len: usize) {
        let mut done = 0;

        // Use up leftover keystream from the previous call.
        let n = (self.ks_len as usize).min(len);
        for i in 0..n {
            let k = self.ks[BLOCK_LEN - self.ks_len as usize + i];
            unsafe { output.add(i).write(input.add(i).read() ^ k) };
        }
        self.ks_len -= n as u8;
        done += n;

        // Whole blocks go through the hardware. The hardware increments the counter itself, but
        // how many bits of it carry is not guaranteed, so each transaction is limited to what
        // fits before the low 16 bits wrap, and the counter is advanced in software.
        while len - done >= BLOCK_LEN {
            let blocks = (len - done) / BLOCK_LEN;
            let until_wrap = 0x1_0000 - u16::from_be_bytes([self.counter[14], self.counter[15]]) as usize;
            let n = blocks.min(until_wrap) * BLOCK_LEN;
            unsafe {
                for_each_ram_chunk_inout(input.add(done), output.add(done), n, MAX_CHUNK, |i, o, l| {
                    hw::ctr(key, &self.counter, i, o, l);
                    add_be(&mut self.counter, (l / BLOCK_LEN) as u64);
                })
            };
            done += n;
        }

        // A trailing partial block: generate one keystream block and keep the rest.
        if done < len {
            let zero = [0u8; BLOCK_LEN];
            let mut ks = [0u8; BLOCK_LEN];
            unsafe { hw::ctr(key, &self.counter, zero.as_ptr(), ks.as_mut_ptr(), BLOCK_LEN) };
            add_be(&mut self.counter, 1);
            let rem = len - done;
            for i in 0..rem {
                unsafe { output.add(done + i).write(input.add(done + i).read() ^ ks[i]) };
            }
            self.ks = ks;
            self.ks_len = (BLOCK_LEN - rem) as u8;
        }
    }
}

/// Adds `n` to a big-endian 128-bit counter block.
fn add_be(counter: &mut [u8; BLOCK_LEN], n: u64) {
    *counter = u128::from_be_bytes(*counter).wrapping_add(n as u128).to_be_bytes();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Aad,
    Payload,
}

/// State of an AES operation in progress. Created by [`Symmetric::aes_start`].
#[derive(Clone)]
pub struct AesContext<C: Cipher> {
    cipher: C,
    dir: Direction,
    phase: Phase,
    /// CBC chaining value.
    iv: [u8; BLOCK_LEN],
    /// CTR/CCM counter state.
    ctr: CtrState,
    /// CMAC/CBC-MAC accumulator, also used to block-align AAD.
    mac: MacAcc,
    aad_len: usize,
    payload_len: usize,
    #[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
    gcm: hw::GcmState,
}

impl<C: Cipher> AesContext<C> {
    /// Returns the cipher this context was started with.
    pub fn cipher(&self) -> &C {
        &self.cipher
    }

    /// Returns the direction this context was started with.
    pub fn direction(&self) -> Direction {
        self.dir
    }
}

impl<'d, M: Mode> Symmetric<'d, M> {
    /// Starts a cipher operation.
    ///
    /// The returned context holds all the state of the operation. Pass it to the other
    /// `aes_blocking_*` methods.
    pub fn aes_start<C: Cipher>(&mut self, cipher: C, dir: Direction) -> AesContext<C> {
        let key = *cipher.key();
        let mut ctx = AesContext {
            cipher,
            dir,
            phase: Phase::Aad,
            iv: [0; BLOCK_LEN],
            ctr: CtrState::new([0; BLOCK_LEN]),
            mac: MacAcc::new(),
            aad_len: 0,
            payload_len: 0,
            #[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
            gcm: hw::GcmState::default(),
        };
        match ctx.cipher.params() {
            Params::Ecb | Params::Cmac => {}
            Params::Cbc { iv } => ctx.iv = *iv,
            Params::Ctr { iv } => ctx.ctr = CtrState::new(*iv),
            Params::Ccm {
                nonce,
                tag_len,
                aad_len,
                payload_len,
            } => {
                let l = 15 - nonce.len();
                // B0 = flags || nonce || payload length.
                let mut b0 = [0u8; BLOCK_LEN];
                b0[0] = ((aad_len > 0) as u8) << 6 | (((tag_len - 2) / 2) as u8) << 3 | (l - 1) as u8;
                b0[1..1 + nonce.len()].copy_from_slice(nonce);
                let len = (payload_len as u64).to_be_bytes();
                b0[BLOCK_LEN - l..].copy_from_slice(&len[8 - l..]);
                hw::cbc_mac(&key, &mut ctx.mac.state, &b0);
                // Counter block A_1 = flags || nonce || 1. A_0 is used for the tag.
                let mut a1 = [0u8; BLOCK_LEN];
                a1[0] = (l - 1) as u8;
                a1[1..1 + nonce.len()].copy_from_slice(nonce);
                a1[BLOCK_LEN - 1] = 1;
                ctx.ctr = CtrState::new(a1);
                // The AAD is prefixed with its encoded length.
                if aad_len > 0 {
                    let n = if aad_len < 0xFF00 {
                        ctx.mac.buf[..2].copy_from_slice(&(aad_len as u16).to_be_bytes());
                        2
                    } else if (aad_len as u64) < 1 << 32 {
                        ctx.mac.buf[..2].copy_from_slice(&[0xFF, 0xFE]);
                        ctx.mac.buf[2..6].copy_from_slice(&(aad_len as u32).to_be_bytes());
                        6
                    } else {
                        ctx.mac.buf[..2].copy_from_slice(&[0xFF, 0xFF]);
                        ctx.mac.buf[2..10].copy_from_slice(&(aad_len as u64).to_be_bytes());
                        10
                    };
                    ctx.mac.buf_len = n;
                }
            }
            #[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
            Params::Gcm { iv } => ctx.gcm = hw::gcm_init(&key, iv, dir),
        }
        ctx
    }

    /// Feeds additional authenticated data (AAD).
    ///
    /// - Call it before feeding any payload.
    /// - Chunks may have any size.
    /// - `last` marks the final chunk. It is optional. The AAD phase also ends when the
    ///   first payload chunk is fed, or when the operation is finished.
    ///
    /// Errors: `AadAfterPayload`, `InvalidLength` for CCM if the total does not match
    /// `aad_len`.
    pub fn aes_blocking_aad<C: AuthenticatedCipher>(
        &mut self,
        ctx: &mut AesContext<C>,
        aad: &[u8],
        last: bool,
    ) -> Result<(), Error> {
        if ctx.phase != Phase::Aad {
            return Err(Error::AadAfterPayload);
        }
        let key = *ctx.cipher.key();
        match ctx.cipher.params() {
            Params::Ccm { aad_len, .. } => {
                if ctx.aad_len + aad.len() > aad_len {
                    return Err(Error::InvalidLength);
                }
                ctx.mac
                    .absorb(aad, false, |acc, blocks| hw::cbc_mac(&key, &mut acc.state, blocks));
                ctx.aad_len += aad.len();
                if last {
                    Self::aes_end_aad(ctx)?;
                }
            }
            #[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
            Params::Gcm { .. } => {
                let gcm = &mut ctx.gcm;
                ctx.mac.absorb(aad, false, |_, blocks| hw::gcm_aad(gcm, &key, blocks));
                ctx.aad_len += aad.len();
                if last {
                    Self::aes_end_aad(ctx)?;
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    /// Ends the AAD phase of an AEAD context.
    fn aes_end_aad<C: Cipher>(ctx: &mut AesContext<C>) -> Result<(), Error> {
        let key = *ctx.cipher.key();
        match ctx.cipher.params() {
            Params::Ccm { aad_len, .. } => {
                if ctx.aad_len != aad_len {
                    return Err(Error::InvalidLength);
                }
                ctx.mac.flush(|acc, blocks| hw::cbc_mac(&key, &mut acc.state, blocks));
            }
            #[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
            Params::Gcm { .. } => {
                let gcm = &mut ctx.gcm;
                ctx.mac.flush(|_, blocks| hw::gcm_aad(gcm, &key, blocks));
            }
            _ => {}
        }
        ctx.phase = Phase::Payload;
        Ok(())
    }

    /// Processes payload data from `input` into `output`.
    ///
    /// - `input` and `output` must have the same length. For [`AesCmac`], `output` is
    ///   ignored and may be empty.
    /// - `last` marks the final chunk.
    /// - ECB and CBC: every chunk must be a multiple of the block length.
    /// - GCM: every chunk except the last must be a multiple of the block length.
    /// - CTR, CMAC and CCM: chunks may have any length.
    ///
    /// Errors: `InvalidLength`.
    pub fn aes_blocking_payload<C: Cipher>(
        &mut self,
        ctx: &mut AesContext<C>,
        input: &[u8],
        output: &mut [u8],
        last: bool,
    ) -> Result<(), Error> {
        if !matches!(ctx.cipher.params(), Params::Cmac) && input.len() != output.len() {
            return Err(Error::InvalidLength);
        }
        unsafe { Self::aes_payload_raw(ctx, input.as_ptr(), output.as_mut_ptr(), input.len(), last) }
    }

    /// Processes payload data in place. See [`Self::aes_blocking_payload`].
    pub fn aes_blocking_payload_in_place<C: Cipher>(
        &mut self,
        ctx: &mut AesContext<C>,
        data: &mut [u8],
        last: bool,
    ) -> Result<(), Error> {
        unsafe { Self::aes_payload_raw(ctx, data.as_ptr(), data.as_mut_ptr(), data.len(), last) }
    }

    /// # Safety
    ///
    /// `input` must be readable and `output` writable for `len` bytes. If they overlap, they
    /// must be equal.
    unsafe fn aes_payload_raw<C: Cipher>(
        ctx: &mut AesContext<C>,
        input: *const u8,
        output: *mut u8,
        len: usize,
        last: bool,
    ) -> Result<(), Error> {
        let key = *ctx.cipher.key();
        let dir = ctx.dir;
        match ctx.cipher.params() {
            Params::Ecb => {
                if len % BLOCK_LEN != 0 {
                    return Err(Error::InvalidLength);
                }
                unsafe {
                    for_each_ram_chunk_inout(input, output, len, MAX_CHUNK, |i, o, l| hw::ecb(&key, dir, i, o, l))
                };
            }
            Params::Cbc { .. } => {
                if len % BLOCK_LEN != 0 {
                    return Err(Error::InvalidLength);
                }
                let iv = &mut ctx.iv;
                unsafe {
                    for_each_ram_chunk_inout(input, output, len, MAX_CHUNK, |i, o, l| hw::cbc(&key, dir, iv, i, o, l))
                };
            }
            Params::Ctr { .. } => unsafe { ctx.ctr.apply(&key, input, output, len) },
            Params::Cmac => {
                let data = unsafe { core::slice::from_raw_parts(input, len) };
                ctx.mac.absorb(data, true, |acc, blocks| {
                    hw::cmac_update(&key, &mut acc.state, &mut acc.started, blocks)
                });
            }
            Params::Ccm { payload_len, .. } => {
                if ctx.phase == Phase::Aad {
                    Self::aes_end_aad(ctx)?;
                }
                if ctx.payload_len + len > payload_len {
                    return Err(Error::InvalidLength);
                }
                // The MAC is computed over the plaintext.
                let cbc_mac = |acc: &mut MacAcc, blocks: &[u8]| hw::cbc_mac(&key, &mut acc.state, blocks);
                match dir {
                    Direction::Encrypt => {
                        {
                            let plaintext = unsafe { core::slice::from_raw_parts(input, len) };
                            ctx.mac.absorb(plaintext, false, cbc_mac);
                        }
                        unsafe { ctx.ctr.apply(&key, input, output, len) };
                    }
                    Direction::Decrypt => {
                        unsafe { ctx.ctr.apply(&key, input, output, len) };
                        let plaintext = unsafe { core::slice::from_raw_parts(output as *const u8, len) };
                        ctx.mac.absorb(plaintext, false, cbc_mac);
                    }
                }
                ctx.payload_len += len;
                if last {
                    if ctx.payload_len != payload_len {
                        return Err(Error::InvalidLength);
                    }
                    ctx.mac.flush(cbc_mac);
                }
            }
            #[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
            Params::Gcm { .. } => {
                if ctx.phase == Phase::Aad {
                    Self::aes_end_aad(ctx)?;
                }
                if !last && len % BLOCK_LEN != 0 {
                    return Err(Error::InvalidLength);
                }
                let payload_len = ctx.payload_len + len;
                unsafe {
                    hw::gcm_payload(
                        &mut ctx.gcm,
                        &key,
                        dir,
                        input,
                        output,
                        len,
                        last,
                        ctx.aad_len,
                        payload_len,
                    )?
                };
                ctx.payload_len = payload_len;
            }
        }
        Ok(())
    }

    /// Finishes the operation.
    ///
    /// For CMAC, CCM and GCM this returns the authentication tag. For CCM only the first
    /// `tag_len` bytes of it are valid. For the other modes it returns `None`.
    ///
    /// When decrypting with an authenticated mode, compare the returned tag with the
    /// received one in constant time before using the plaintext.
    ///
    /// Errors: `InvalidLength` for CCM if the totals do not match `aad_len` and
    /// `payload_len`.
    pub fn aes_blocking_finish<C: Cipher>(&mut self, mut ctx: AesContext<C>) -> Result<Option<[u8; BLOCK_LEN]>, Error> {
        let key = *ctx.cipher.key();
        let (nonce_len, payload_len) = match ctx.cipher.params() {
            Params::Ccm { nonce, payload_len, .. } => (nonce.len(), payload_len),
            _ => (0, 0),
        };
        match ctx.cipher.params() {
            Params::Ecb | Params::Cbc { .. } | Params::Ctr { .. } => Ok(None),
            Params::Cmac => {
                let tail = &ctx.mac.buf[..ctx.mac.buf_len as usize];
                Ok(Some(hw::cmac_final(&key, &ctx.mac.state, ctx.mac.started, tail)))
            }
            Params::Ccm { .. } => {
                if ctx.phase == Phase::Aad {
                    Self::aes_end_aad(&mut ctx)?;
                }
                if ctx.payload_len != payload_len {
                    return Err(Error::InvalidLength);
                }
                ctx.mac.flush(|acc, blocks| hw::cbc_mac(&key, &mut acc.state, blocks));
                // T = MAC XOR E(K, A_0), with A_0 = flags || nonce || 0.
                let mut a0 = ctx.ctr.counter;
                a0[BLOCK_LEN - (15 - nonce_len)..].fill(0);
                let mut tag = [0u8; BLOCK_LEN];
                unsafe { hw::ctr(&key, &a0, ctx.mac.state.as_ptr(), tag.as_mut_ptr(), BLOCK_LEN) };
                Ok(Some(tag))
            }
            #[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
            Params::Gcm { .. } => {
                if ctx.phase == Phase::Aad {
                    Self::aes_end_aad(&mut ctx)?;
                }
                Ok(Some(hw::gcm_finish(&mut ctx.gcm, &key, ctx.aad_len, ctx.payload_len)?))
            }
        }
    }
}
