//! Driver for the HASHCRYPT peripheral, mode switch skeleton
use embassy_hal_internal::Peri;
use nxp_pac::hashcrypt::vals::{Aesdecrypt, Aeskeysz, Aesmode, Aessecret, Mode};
use nxp_pac::syscon::vals::HashAesRst::Released;

use crate::hashcrypt::inner::Key::{Key128, Key192, Key256};
use crate::pac;
use crate::peripherals::HASHCRYPT;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Key {
    Key128([u8; 16]),
    Key192([u8; 24]),
    Key256([u8; 32]),
}
/// AES key length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySize {
    Bits128,
    Bits192,
    Bits256,
}

/// Errors returned by the AES modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AesError {
    /// Error triggers when `set_key` is called without `set_key_size`
    KeySizeNeeded,
    /// Error triggers when encrypt/decrypt is called before `set_key`
    KeyNeeded,
    /// Error triggers when .encrypt/.decrypt for cbc are called without `set_iv`
    IvNeeded,
    /// Error triggers when .encrypt/.decrypt for ctr are called without `set_counter`
    CounterNeeded,
    /// Error triggers when `set_key` is called with a parameter that does not respect the size established by `set_key_size`
    WrongKeySize,
    /// Error triggers when the key-size register holds a reserved value
    DeviceError,
    /// Error triggers when encrypt/decrypt are called with data and output of different lengths
    LengthMismatch,
    /// Error triggers when, in ECB and CBC mode, in an encrypt or decrypt call, the length of the data is not divisible by 16
    UnalignedLength,
    /// Error triggers when the buffer for the output is too small for the ciphertext or plaintext
    OutputTooSmall,
    /// Error triggers when function from the AesPadded trait is called on data which is not properly padded or that does not have padding at all
    InvalidPadding,
}

// Generic driver type
pub struct GenericHashcrypt<'d> {
    _peri: Peri<'d, HASHCRYPT>,
}

// mode switching implementation of generic driver
impl<'d> GenericHashcrypt<'d> {
    pub fn new(peri: Peri<'d, HASHCRYPT>) -> Self {
        pac::SYSCON.ahbclkctrl2().modify(|w| {
            w.set_hash_aes(true);
        });

        pac::SYSCON.presetctrl2().modify(|w| {
            w.set_hash_aes_rst(Released);
        });
        Self { _peri: peri }
    }

    pub fn sha1(&mut self) -> Sha1<'_, 'd> {
        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_mode(Mode::Sha1);
            w.set_new_hash(true);
        });
        Sha1 { _peri: self }
    }

    pub fn sha256(&mut self) -> Sha256<'_, 'd> {
        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_mode(Mode::Sha2256);
            w.set_new_hash(true);
        });
        Sha256 { _peri: self }
    }

    /// Switches the peripheral to AES-ECB mode.
    pub fn aes_ecb(&mut self) -> AesEcb<'_, 'd> {
        // AES-ECB config via register calls
        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_mode(Mode::Aes);
            w.set_new_hash(true);
        });

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesmode(Aesmode::Ecb);
            w.set_aessecret(Aessecret::NormalWay);
            w.set_streamlast(false);
            w.set_msw1st(true);
            w.set_msw1st_out(true);
            w.set_swapdat(true);
            w.set_swapkey(true);
        });

        AesEcb {
            _peri: self,
            key_size: None,
            key: None,
        }
    }

    /// Switches the peripheral to AES-CBC mode.
    pub fn aes_cbc(&mut self) -> AesCbc<'_, 'd> {
        // AES-CBC config via register calls
        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_mode(Mode::Aes);
            w.set_new_hash(true);
        });

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesmode(Aesmode::Cbc);
            w.set_aessecret(Aessecret::NormalWay);
            w.set_streamlast(false);
            w.set_msw1st(true);
            w.set_msw1st_out(true);
            w.set_swapdat(true);
            w.set_swapkey(true);
        });

        AesCbc {
            _peri: self,
            key_size: None,
            key: None,
            iv: None,
        }
    }

    /// Switches the peripheral to AES-CTR mode.
    pub fn aes_ctr(&mut self) -> AesCtr<'_, 'd> {
        // AES-CTR config via register calls
        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_mode(Mode::Aes);
            w.set_new_hash(true);
        });

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesmode(Aesmode::Ctr);
            w.set_aessecret(Aessecret::NormalWay);
            w.set_streamlast(false);
            w.set_msw1st(true);
            w.set_msw1st_out(true);
            w.set_swapdat(true);
            w.set_swapkey(true);
            w.set_aesctrpos(0);
        });
        AesCtr {
            _peri: self,
            key_size: None,
            key: None,
            counter: None,
        }
    }

    pub(crate) fn wait_data() {
        while !pac::HASHCRYPT.status().read().waiting() {
            cortex_m::asm::nop();
        }
    }

    pub(crate) fn feed_word(word: u32) {
        pac::HASHCRYPT.indata().write(|w| {
            w.set_data(word);
        });
    }

    pub(crate) fn read_digest(count: usize, out: &mut [u8]) {
        // Block until the DIGEST status flag signals the output registers hold a
        // complete result, then read `count` words out of DIGEST0..n.
        while !pac::HASHCRYPT.status().read().digest() {
            cortex_m::asm::nop();
        }
        for i in 0..count {
            let word = pac::HASHCRYPT.digest0(i).read().digest();
            out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
    }
}

// Helper functions for AES modes
impl<'d> GenericHashcrypt<'d> {
    pub(crate) fn wait_key() {
        while !pac::HASHCRYPT.status().read().needkey() {
            cortex_m::asm::nop();
        }
    }

    pub(crate) fn feed_key(key: &Key) {
        Self::wait_data();
        Self::wait_key();

        match key {
            Key128(bytes) => {
                for chunk in bytes.chunks_exact(4) {
                    Self::feed_word(u32::from_le_bytes(chunk.try_into().unwrap()));
                }
            }
            Key192(bytes) => {
                for chunk in bytes.chunks_exact(4) {
                    Self::feed_word(u32::from_le_bytes(chunk.try_into().unwrap()));
                }
            }
            Key256(bytes) => {
                for chunk in bytes.chunks_exact(4) {
                    Self::feed_word(u32::from_le_bytes(chunk.try_into().unwrap()));
                }
            }
        }
    }

    pub(crate) fn wait_iv_counter() {
        while !pac::HASHCRYPT.status().read().neediv() {
            cortex_m::asm::nop();
        }
    }

    pub(crate) fn feed_iv_counter(words: &[u8; 16]) {
        Self::wait_data();
        Self::wait_iv_counter();

        for chunk in words.chunks_exact(4) {
            Self::feed_word(u32::from_le_bytes(chunk.try_into().unwrap()));
        }
    }

    pub(crate) fn process_block(input: &[u8; 16], output: &mut [u8; 16], last: bool) {
        if last {
            pac::HASHCRYPT.cryptcfg().modify(|w| {
                w.set_streamlast(true);
            });
        }

        for word in input.chunks_exact(4) {
            Self::feed_word(u32::from_le_bytes(word.try_into().unwrap()));
        }

        Self::read_digest(4, output);
    }

    pub(crate) fn process_blocks(data: &[u8], output: &mut [u8]) {
        let nr_blocks = data.len() / 16;
        let mut offset = 0;

        for (i, block) in data.chunks_exact(16).enumerate() {
            let block: &[u8; 16] = block.try_into().unwrap();
            let last = if nr_blocks == i + 1 { true } else { false };
            let out: &mut [u8; 16] = (&mut output[offset..offset + 16]).try_into().unwrap();
            Self::process_block(block, out, last);
            offset += 16;
        }
    }

    pub(crate) fn process_blocks_padded(data: &[u8], output: &mut [u8]) {
        let mut offset = 0;
        let mut blocks = data.chunks_exact(16);
        for block in &mut blocks {
            let block: [u8; 16] = block.try_into().unwrap();
            let out: &mut [u8; 16] = (&mut output[offset..offset + 16]).try_into().unwrap();
            Self::process_block(&block, out, false);
            offset += 16;
        }

        let tail = blocks.remainder();
        let pad = (16 - tail.len()) as u8;

        let mut final_block = [0u8; 16];
        final_block[..tail.len()].copy_from_slice(tail);
        final_block[tail.len()..].fill(pad);

        let out: &mut [u8; 16] = (&mut output[offset..offset + 16]).try_into().unwrap();
        Self::process_block(&final_block, out, true);
    }

    pub(crate) fn process_blocks_ctr(data: &[u8], output: &mut [u8]) {
        let blocks = data.chunks_exact(16);
        let tail = blocks.remainder();
        let nr_blocks = data.len() / 16;
        let has_tail = !tail.is_empty();
        let mut offset = 0;

        // process the 16 byte blocks
        for (i, block) in blocks.enumerate() {
            let block: &[u8; 16] = block.try_into().unwrap();
            let out: &mut [u8; 16] = (&mut output[offset..offset + 16]).try_into().unwrap();
            let last = !has_tail && i + 1 == nr_blocks;

            Self::process_block(block, out, last);
            offset += 16;
        }

        // if the tail exist, pad with 0
        if has_tail {
            let mut in_blocks = [0u8; 16];
            in_blocks[..tail.len()].copy_from_slice(tail);
            let mut out_blocks = [0u8; 16];
            Self::process_block(&in_blocks, &mut out_blocks, true);
            output[offset..offset + tail.len()].copy_from_slice(&out_blocks[..tail.len()]);
        }
    }
}

pub trait Digest {
    type Output;
    // the update method is identical between sha1 and sha2 and so, it will be implemented
    // by a macro similar to the key related functions for AES

    /// Returns the digest of the message streamed via `update`
    fn finalise(&mut self) -> Self::Output;
}

/// AES encryption and decryption, implemented by [`AesEcb`], [`AesCbc`] and [`AesCtr`].
pub trait Aes {
    /// Encrypt `data` into `output`.
    ///
    /// `data` and `output` must have the same length. Except in CTR mode, the length must be
    /// a multiple of 16 bytes; for arbitrary-length messages in ECB and CBC, see [`AesPadded`].
    fn encrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError>;

    /// Decrypt `data` into `output`.
    ///
    /// `data` and `output` must have the same length. Except in CTR mode, the length must be
    /// a multiple of 16 bytes; for PKCS#7-padded ciphertext in ECB and CBC, see [`AesPadded`].
    fn decrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError>;
}
/// Arbitrary-length encryption for the block modes ([`AesEcb`] and [`AesCbc`]), using PKCS#7 padding.
///
/// PKCS#7 always pads: a message that is already a multiple of 16 bytes gets a full extra
/// block. This is what lets [`decrypt_padded`](AesPadded::decrypt_padded) find and remove
/// the padding reliably. CTR doesn't need this trait, since it accepts any length directly.
pub trait AesPadded {
    /// Pad `data` with PKCS#7 and encrypt it into `output`.
    ///
    /// `output` must be at least `(data.len() / 16 + 1) * 16` bytes. Returns the slice of
    /// `output` holding the ciphertext.
    fn encrypt_padded<'o>(&mut self, data: &[u8], output: &'o mut [u8]) -> Result<&'o [u8], AesError>;

    /// Decrypt PKCS#7-padded `data` into `output` and strip the padding.
    ///
    /// `data` must be non-empty, a multiple of 16 bytes, and encrypted with the same key
    /// (and IV) using PKCS#7 padding. `output` must be at least `data.len()` bytes. Returns
    /// the slice of `output` holding the original message.
    ///
    /// Returns [`AesError::InvalidPadding`] if the padding is malformed, e.g. for a wrong key
    /// or unpadded ciphertext. This check is best-effort, not an integrity check: random data
    /// passes it occasionally. Don't expose this error to untrusted parties
    /// (padding-oracle attacks).
    fn decrypt_padded<'o>(&mut self, data: &[u8], output: &'o mut [u8]) -> Result<&'o [u8], AesError>;
}

// Specific driver types
pub struct Sha1<'a, 'd> {
    _peri: &'a mut GenericHashcrypt<'d>,
}

impl<'a, 'd> Digest for Sha1<'a, 'd> {
    type Output = [u8; 20];

    fn finalise(&mut self) -> Self::Output {
        todo!("Add finalise method for Sha1");
    }
}

pub struct Sha256<'a, 'd> {
    _peri: &'a mut GenericHashcrypt<'d>,
}

impl<'a, 'd> Digest for Sha256<'a, 'd> {
    type Output = [u8; 32];
    fn finalise(&mut self) -> Self::Output {
        todo!("Add finalise method for SHA-256");
    }
}

/// AES in ECB mode.
pub struct AesEcb<'a, 'd> {
    _peri: &'a mut GenericHashcrypt<'d>,
    key_size: Option<KeySize>,
    key: Option<Key>,
}

impl<'a, 'd> Aes for AesEcb<'a, 'd> {
    fn encrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError> {
        let key = match &self.key {
            Some(k) => k,
            None => return Err(AesError::KeyNeeded),
        };

        if data.len() % 16 != 0 {
            return Err(AesError::UnalignedLength);
        }

        if data.len() != output.len() {
            return Err(AesError::LengthMismatch);
        }

        if data.is_empty() {
            return Ok(());
        }

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesdecrypt(Aesdecrypt::Encrypt);
            w.set_streamlast(false);
        });

        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        GenericHashcrypt::feed_key(key);
        GenericHashcrypt::process_blocks(data, output);

        return Ok(());
    }

    fn decrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError> {
        let key = match &self.key {
            Some(k) => k,
            None => return Err(AesError::KeyNeeded),
        };

        if data.len() % 16 != 0 {
            return Err(AesError::UnalignedLength);
        }

        if data.len() != output.len() {
            return Err(AesError::LengthMismatch);
        }

        if data.is_empty() {
            return Ok(());
        }

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesdecrypt(Aesdecrypt::Decrypt);
            w.set_streamlast(false);
        });

        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        GenericHashcrypt::feed_key(key);
        GenericHashcrypt::process_blocks(data, output);

        return Ok(());
    }
}

impl<'a, 'd> AesPadded for AesEcb<'a, 'd> {
    fn encrypt_padded<'o>(&mut self, data: &[u8], output: &'o mut [u8]) -> Result<&'o [u8], AesError> {
        let key = match &self.key {
            Some(k) => k,
            None => return Err(AesError::KeyNeeded),
        };

        let padded_len = (data.len() / 16 + 1) * 16;
        if output.len() < padded_len {
            return Err(AesError::OutputTooSmall);
        }

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesdecrypt(Aesdecrypt::Encrypt);
            w.set_streamlast(false);
        });

        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        GenericHashcrypt::feed_key(key);
        GenericHashcrypt::process_blocks_padded(data, output);

        return Ok(&output[..padded_len]);
    }

    fn decrypt_padded<'o>(&mut self, data: &[u8], output: &'o mut [u8]) -> Result<&'o [u8], AesError> {
        let key = match &self.key {
            Some(k) => k,
            None => return Err(AesError::KeyNeeded),
        };

        if data.len() == 0 || data.len() % 16 != 0 {
            return Err(AesError::UnalignedLength);
        }
        if output.len() < data.len() {
            return Err(AesError::OutputTooSmall);
        }

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesdecrypt(Aesdecrypt::Decrypt);
            w.set_streamlast(false);
        });

        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        GenericHashcrypt::feed_key(key);
        GenericHashcrypt::process_blocks(data, output);

        let len = data.len();
        let n = output[len - 1] as usize;

        if n == 0 || n > 16 {
            return Err(AesError::InvalidPadding);
        }
        for i in (len - n)..len {
            if output[i] != n as u8 {
                return Err(AesError::InvalidPadding);
            }
        }

        return Ok(&output[..len - n]);
    }
}

impl<'a, 'd> AesEcb<'a, 'd> {
    // Does not require anything past the default Aes and AesPadded methods
}

/// AES in CBC mode. See [`AesCbc::set_iv`] for IV requirements.
pub struct AesCbc<'a, 'd> {
    _peri: &'a mut GenericHashcrypt<'d>,
    key_size: Option<KeySize>,
    key: Option<Key>,
    iv: Option<[u8; 16]>,
}

impl<'a, 'd> Aes for AesCbc<'a, 'd> {
    fn encrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError> {
        let key = match &self.key {
            Some(k) => k,
            None => {
                return Err(AesError::KeyNeeded);
            }
        };

        let iv = match &self.iv {
            Some(v) => v,
            None => {
                return Err(AesError::IvNeeded);
            }
        };

        if data.len() % 16 != 0 {
            return Err(AesError::UnalignedLength);
        }

        if data.len() != output.len() {
            return Err(AesError::LengthMismatch);
        }

        if data.is_empty() {
            return Ok(());
        }

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesdecrypt(Aesdecrypt::Encrypt);
            w.set_streamlast(false);
        });

        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        GenericHashcrypt::feed_key(key);
        GenericHashcrypt::feed_iv_counter(iv);
        GenericHashcrypt::process_blocks(data, output);

        return Ok(());
    }

    fn decrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError> {
        let key = match &self.key {
            Some(k) => k,
            None => {
                return Err(AesError::KeyNeeded);
            }
        };

        let iv = match &self.iv {
            Some(v) => v,
            None => {
                return Err(AesError::IvNeeded);
            }
        };

        if data.len() % 16 != 0 {
            return Err(AesError::UnalignedLength);
        }

        if data.len() != output.len() {
            return Err(AesError::LengthMismatch);
        }

        if data.is_empty() {
            return Ok(());
        }

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesdecrypt(Aesdecrypt::Decrypt);
            w.set_streamlast(false);
        });

        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        GenericHashcrypt::feed_key(key);
        GenericHashcrypt::feed_iv_counter(iv);
        GenericHashcrypt::process_blocks(data, output);

        return Ok(());
    }
}

impl<'a, 'd> AesPadded for AesCbc<'a, 'd> {
    fn encrypt_padded<'o>(&mut self, data: &[u8], output: &'o mut [u8]) -> Result<&'o [u8], AesError> {
        let key = match &self.key {
            Some(k) => k,
            None => {
                return Err(AesError::KeyNeeded);
            }
        };

        let iv = match &self.iv {
            Some(v) => v,
            None => {
                return Err(AesError::IvNeeded);
            }
        };

        let padded_len = (data.len() / 16 + 1) * 16;
        if output.len() < padded_len {
            return Err(AesError::OutputTooSmall);
        }

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesdecrypt(Aesdecrypt::Encrypt);
            w.set_streamlast(false);
        });

        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        GenericHashcrypt::feed_key(key);
        GenericHashcrypt::feed_iv_counter(iv);
        GenericHashcrypt::process_blocks_padded(data, output);

        return Ok(&output[..padded_len]);
    }

    fn decrypt_padded<'o>(&mut self, data: &[u8], output: &'o mut [u8]) -> Result<&'o [u8], AesError> {
        let key = match &self.key {
            Some(k) => k,
            None => return Err(AesError::KeyNeeded),
        };

        let iv = match &self.iv {
            Some(v) => v,
            None => {
                return Err(AesError::IvNeeded);
            }
        };

        if data.len() == 0 || data.len() % 16 != 0 {
            return Err(AesError::UnalignedLength);
        }
        if output.len() < data.len() {
            return Err(AesError::OutputTooSmall);
        }

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesdecrypt(Aesdecrypt::Decrypt);
            w.set_streamlast(false);
        });

        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        GenericHashcrypt::feed_key(key);
        GenericHashcrypt::feed_iv_counter(iv);
        GenericHashcrypt::process_blocks(data, output);

        let len = data.len();
        let n = output[len - 1] as usize;

        if n == 0 || n > 16 {
            return Err(AesError::InvalidPadding);
        }
        for i in (len - n)..len {
            if output[i] != n as u8 {
                return Err(AesError::InvalidPadding);
            }
        }

        return Ok(&output[..len - n]);
    }
}

impl<'a, 'd> AesCbc<'a, 'd> {
    /// Sets the IV used by every following encrypt/decrypt call.
    ///
    /// The IV is not chained between calls: each call starts from this IV, until it's changed
    /// by the next call of this method.
    pub fn set_iv(&mut self, iv: &[u8; 16]) {
        self.iv = Some(*iv);
    }
}

/// AES in CTR mode. See [`AesCtr::set_counter`] for counter requirements.
pub struct AesCtr<'a, 'd> {
    _peri: &'a mut GenericHashcrypt<'d>,
    key_size: Option<KeySize>,
    key: Option<Key>,
    counter: Option<[u8; 16]>,
}

impl<'a, 'd> Aes for AesCtr<'a, 'd> {
    fn encrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError> {
        let key = match &self.key {
            Some(k) => k,
            None => {
                return Err(AesError::KeyNeeded);
            }
        };

        let counter = match &self.counter {
            Some(t) => t,
            None => {
                return Err(AesError::CounterNeeded);
            }
        };

        if data.len() != output.len() {
            return Err(AesError::LengthMismatch);
        }

        if data.is_empty() {
            return Ok(());
        }

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesdecrypt(Aesdecrypt::Encrypt);
            w.set_streamlast(false);
        });

        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        GenericHashcrypt::feed_key(key);
        GenericHashcrypt::feed_iv_counter(counter);
        GenericHashcrypt::process_blocks_ctr(data, output);

        return Ok(());
    }

    fn decrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError> {
        let key = match &self.key {
            Some(k) => k,
            None => {
                return Err(AesError::KeyNeeded);
            }
        };

        let counter = match &self.counter {
            Some(t) => t,
            None => {
                return Err(AesError::CounterNeeded);
            }
        };

        if data.len() != output.len() {
            return Err(AesError::LengthMismatch);
        }

        if data.is_empty() {
            return Ok(());
        }

        pac::HASHCRYPT.cryptcfg().modify(|w| {
            w.set_aesdecrypt(Aesdecrypt::Decrypt);
            w.set_streamlast(false);
        });

        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        GenericHashcrypt::feed_key(key);
        GenericHashcrypt::feed_iv_counter(counter);
        GenericHashcrypt::process_blocks_ctr(data, output);

        return Ok(());
    }
}

impl<'a, 'd> AesCtr<'a, 'd> {
    /// Sets the initial counter block used by every following encrypt/decrypt call.
    ///
    /// The counter is not advanced between calls: each call starts from this value, until it's
    /// changed by the next call of this function.
    pub fn set_counter(&mut self, counter: &[u8; 16]) {
        self.counter = Some(*counter);
    }
}

macro_rules! impl_aes {
    ($ty:ident) => {
        impl<'a, 'd> $ty<'a, 'd> {
            /// Sets the AES key length. Do this before using [`Self::set_key`].
            pub fn set_key_size(&mut self, size: KeySize) {
                let val = match size {
                    KeySize::Bits128 => Aeskeysz::Bits128,
                    KeySize::Bits192 => Aeskeysz::Bits192,
                    KeySize::Bits256 => Aeskeysz::Bits256,
                };

                pac::HASHCRYPT.cryptcfg().modify(|w| {
                    w.set_aeskeysz(val);
                });

                self.key_size = Some(size);
            }

            /// Returns the configured key size in bytes.
            ///
            /// Returns [`AesError::DeviceError`] if the register holds a reserved value.
            pub fn key_size(&self) -> Result<u32, AesError> {
                match pac::HASHCRYPT.cryptcfg().read().aeskeysz() {
                    // Convert from bits to bytes
                    Aeskeysz::Bits128 => return Ok(16),
                    Aeskeysz::Bits192 => return Ok(24),
                    Aeskeysz::Bits256 => return Ok(32),
                    Aeskeysz::_RESERVED_3 => return Err(AesError::DeviceError),
                }
            }

            /// Sets the key used by every following encrypt/decrypt call, until it's changed by the
            /// next call of this method.
            ///
            /// Returns [`AesError::KeySizeNeeded`] if [`Self::set_key_size`] hasn't been called, or
            /// [`AesError::WrongKeySize`] if the key length doesn't match it.
            pub fn set_key(&mut self, key: &[u8]) -> Result<(), AesError> {
                let expected_size = match self.key_size {
                    Some(s) => s,
                    None => return Err(AesError::KeySizeNeeded),
                };

                let size = match expected_size {
                    KeySize::Bits128 => 16,
                    KeySize::Bits192 => 24,
                    KeySize::Bits256 => 32,
                };

                if size == key.len() as u32 {
                    self.key = Some(match size {
                        16 => {
                            let mut buf = [0u8; 16];
                            buf.copy_from_slice(key);
                            Key::Key128(buf)
                        }
                        24 => {
                            let mut buf = [0u8; 24];
                            buf.copy_from_slice(key);
                            Key::Key192(buf)
                        }
                        32 => {
                            let mut buf = [0u8; 32];
                            buf.copy_from_slice(key);
                            Key::Key256(buf)
                        }

                        _ => unreachable!(),
                    });

                    return Ok(());
                } else {
                    return Err(AesError::WrongKeySize);
                }
            }
        }
    };
}

impl_aes!(AesEcb);
impl_aes!(AesCbc);
impl_aes!(AesCtr);
