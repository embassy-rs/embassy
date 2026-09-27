//! Driver for the HASHCRYPT peripheral, mode switch skeleton
use embassy_hal_internal::Peri;
use nxp_pac::hashcrypt::vals::{Aeskeysz, Mode};
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySize {
    Bits128,
    Bits192,
    Bits256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AesError {
    /// Error triggers when `.set_key` is called without `set_key_size`
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
}

// Generic driver type
pub struct GenericHashcrypt<'d> {
    _peri: Peri<'d, HASHCRYPT>,
}

// mode switching implementation of generic driver
#[allow(dead_code)]
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
        Sha1 {
            _peri: self,
            buffer: [0u8; 64],
            buffer_len: 0usize,
            total_len: 0u64,
        }
    }

    pub fn sha256(&mut self) -> Sha256<'_, 'd> {
        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_mode(Mode::Sha2256);
            w.set_new_hash(true);
        });
        Sha256 {
            _peri: self,
            buffer: [0u8; 64],
            buffer_len: 0usize,
            total_len: 0u64,
        }
    }

    pub fn aes_ecb(&mut self) -> AesEcb<'_, 'd> {
        // AES-ECB config via register calls
        AesEcb {
            _peri: self,
            key_size: None,
            key: None,
        }
    }

    pub fn aes_cbc(&mut self) -> AesCbc<'_, 'd> {
        // AES-CBC config via register calls
        AesCbc {
            _peri: self,
            key_size: None,
            key: None,
        }
    }

    pub fn aes_ctr(&mut self) -> AesCtr<'_, 'd> {
        // AES-CTR config via register calls
        AesCtr {
            _peri: self,
            key_size: None,
            key: None,
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
#[allow(dead_code)]
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
}

pub trait Digest {
    type Output;
    // the update method is identical between sha1 and sha2 and so, it will be implemented
    // by a macro similar to the key related functions for AES

    /// Returns the digest of the message streamed via `update`
    fn finalise(&mut self) -> Self::Output;
}
//Helper methods fot the types that implement the Digest trait
fn drain_buffer(buffer: &[u8; 64]) {
    let sha256 = pac::HASHCRYPT.indata();
    for chunk in buffer.chunks_exact(4) {
        let word = u32::from_le_bytes(chunk.try_into().unwrap());
        sha256.write(|w| {
            w.set_data(word);
        });
    }
}

// TODO: add update with impl_sha! macro once it's introduced in the SHA PR

pub trait Aes {
    /// Encrypt `data` into `output`.
    ///
    /// `data` and `output` must have the same length. Except in CTR mode, the length must be
    /// a multiple of 16 bytes;
    fn encrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError>;

    /// Decrypt `data` into `output`.
    ///
    /// `data` and `output` must have the same length. Except in CTR mode, the length must be
    /// a multiple of 16 bytes;
    fn decrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError>;

    // Arbitrary-length messages: ECB and CBC will get an `AesPadded` trait using PKCS#7
    // padding, which pads every message (aligned ones get a full extra block) so the
    // padding can be removed on decryption. CTR is a streaming mode, so a short final
    // block is zero-padded internally and the extra output bytes are discarded.
}

// Specific driver types
pub struct Sha1<'a, 'd> {
    _peri: &'a mut GenericHashcrypt<'d>, // A mutable reference to the generic driver which holds the HASHCRYPT peripheral
    buffer: [u8; 64],                    // A 64 byte buffer in which we can dump incoming data streams
    buffer_len: usize, // The number of valid bytes currently buffered, resets to 0 after buffer is drained
    total_len: u64,    // The size of the complete message to be hashed
}

impl<'a, 'd> Digest for Sha1<'a, 'd> {
    type Output = [u8; 20];

    fn finalise(&mut self) -> Self::Output {
        todo!("Add finalise method for Sha1");
    }
}

pub struct Sha256<'a, 'd> {
    _peri: &'a mut GenericHashcrypt<'d>, // A mutable reference to the generic driver which holds the HASHCRYPT peripheral
    buffer: [u8; 64],                    // A 64 byte buffer in which we can dump incoming data streams
    buffer_len: usize, // The number of valid bytes currently buffered, resets to 0 after buffer is drained
    total_len: u64,    // The size of the complete message to be hashed
}

impl<'a, 'd> Digest for Sha256<'a, 'd> {
    type Output = [u8; 32];
    fn finalise(&mut self) -> Self::Output {
        todo!("Add finalise method for SHA-256");
    }
}

pub struct AesEcb<'a, 'd> {
    _peri: &'a mut GenericHashcrypt<'d>,
    key_size: Option<KeySize>,
    key: Option<Key>,
}

impl<'a, 'd> Aes for AesEcb<'a, 'd> {
    fn encrypt(&mut self, _data: &[u8], _output: &mut [u8]) -> Result<(), AesError> {
        todo!("Add encrypt method for ECB")
    }

    fn decrypt(&mut self, _data: &[u8], _output: &mut [u8]) -> Result<(), AesError> {
        todo!("Add decrypt method for ECB");
    }
}

impl<'a, 'd> AesEcb<'a, 'd> {
    // Does not require anything past the default aes methods
}
pub struct AesCbc<'a, 'd> {
    _peri: &'a mut GenericHashcrypt<'d>,
    key_size: Option<KeySize>,
    key: Option<Key>,
}
impl<'a, 'd> Aes for AesCbc<'a, 'd> {
    fn encrypt(&mut self, _data: &[u8], _output: &mut [u8]) -> Result<(), AesError> {
        todo!("Add encrypt method for CBC")
    }

    fn decrypt(&mut self, _data: &[u8], _output: &mut [u8]) -> Result<(), AesError> {
        todo!("Add decrypt method for CBC");
    }
}
impl<'a, 'd> AesCbc<'a, 'd> {
    pub fn set_iv(&mut self, _iv: &[u8; 16]) -> Result<(), AesError> {
        todo!("Add method body");
    }
}
pub struct AesCtr<'a, 'd> {
    _peri: &'a mut GenericHashcrypt<'d>,
    key_size: Option<KeySize>,
    key: Option<Key>,
}

impl<'a, 'd> Aes for AesCtr<'a, 'd> {
    fn encrypt(&mut self, _data: &[u8], _output: &mut [u8]) -> Result<(), AesError> {
        todo!("Add encrypt method for CTR")
    }

    fn decrypt(&mut self, _data: &[u8], _output: &mut [u8]) -> Result<(), AesError> {
        todo!("Add decrypt method for CTR");
    }
}

impl<'a, 'd> AesCtr<'a, 'd> {
    pub fn set_counter(&mut self, _counter: &[u8; 16]) -> Result<(), AesError> {
        todo!("Add method body");
    }
}

macro_rules! impl_aes {
    ($ty:ident) => {
        impl<'a, 'd> $ty<'a, 'd> {
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

            pub fn key_size(&self) -> Result<u32, AesError> {
                match pac::HASHCRYPT.cryptcfg().read().aeskeysz() {
                    // Convert from bits to bytes
                    Aeskeysz::Bits128 => return Ok(16),
                    Aeskeysz::Bits192 => return Ok(24),
                    Aeskeysz::Bits256 => return Ok(32),
                    Aeskeysz::_RESERVED_3 => return Err(AesError::DeviceError),
                }
            }

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
