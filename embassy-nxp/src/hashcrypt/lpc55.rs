//! Driver for the HASHCRYPT peripheral, mode switch sckeleton
use embassy_hal_internal::Peri;
use nxp_pac::hashcrypt::vals::{Aeskeysz, Mode};

use crate::hashcrypt::inner::Key::{Key128, Key192, Key256};
use crate::pac;
use crate::peripherals::HASHCRYPT;

enum Key {
    Key128([u8; 16]),
    Key192([u8; 24]),
    Key256([u8; 32]),
}
#[derive(Clone, Copy)]
pub enum KeySize {
    Bits128,
    Bits192,
    Bits256,
}

pub enum AesError {
    KeySizeNeeded, //triggeres when .set_key is called without set_key_size
    KeyNeeded,     // trigers when .encryp)/.decrypt or set_iv/set_counter are called before set_key()
    IvNeeded,      // triggers when .encrypt/.decrypt for cbc are called without set_iv
    CounterNeeded, // triggers when .encrypt/.decrypt for ctr are called without set_counter
    WrongKeySize, // triggered set_key is called with a parameter that does not respect the size astablished by set_key_size
    DeviceError,  // Reserved functiones were ussed
}

fn wait_data() {
    let mut trys = 0;
    while !pac::HASHCRYPT.status().read().waiting() {
        cortex_m::asm::nop();
        trys += 1;
        if trys > 25 {
            break;
        }
    }
}

fn wait_key() {
    let mut tries = 0;
    while !pac::HASHCRYPT.status().read().needkey() {
        cortex_m::asm::nop();
        tries += 1;
        if tries > 25 {
            break;
        }
    }
}

fn feed_word(word: u32) {
    pac::HASHCRYPT.indata().write(|w| {
        w.set_data(word);
    });
}

fn feed_key(key: &Key) {
    wait_data();
    wait_key();

    match key {
        Key128(bytes) => {
            for chunk in bytes.chunks_exact(4) {
                feed_word(u32::from_le_bytes(chunk.try_into().unwrap()));
            }
        }
        Key192(bytes) => {
            for chunk in bytes.chunks_exact(4) {
                feed_word(u32::from_le_bytes(chunk.try_into().unwrap()));
            }
        }
        Key256(bytes) => {
            for chunk in bytes.chunks_exact(4) {
                feed_word(u32::from_le_bytes(chunk.try_into().unwrap()));
            }
        }
    }
}

// Generic driver type
pub struct GenericDriver<'d> {
    _peri: Peri<'d, HASHCRYPT>,
}

// mode switching implementation of generic driver
impl<'d> GenericDriver<'d> {
    pub fn new(peri: Peri<'d, HASHCRYPT>) -> Self {
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
    // rename to generic driver or _driver
}

pub trait Digest {
    type Output;
    #[doc = "Accepts an arbitrary-length slice of bytes at the time, to be buffered until a full block is built, then drained in to the FIFO"]
    fn update(&mut self, data: &[u8]);

    #[doc = "Returns the digest of the message streamed via `update`"]
    fn finalise(self) -> Self::Output;
}

pub trait Aes {
    fn encrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError>;
    // Universal encrypt confuguration, meaning
    // MSW1ST = true, MSW1ST_OUT = true, SWAPKEY = true, SWAPDAT = true, AESDECRYPT = Encrypt
    // Chop user provided data into words, feed 4 words at the time to indata()
    // Every 4 words, poll digest and apend it to ouptut
    // If the final part of the message is less than 4 words, padd with 0s
    // Before feeding last 4 words, flip STREAMEDLAST to true
    // Check that data.len = output.len
    // Flip STREAMEDLAST back to false in case the user wants to decrypt another message using the same key

    fn decrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError>;
    // Universal decrypt confuguration, meaning
    // MSW1ST = true, MSW1ST_OUT = true, SWAPKEY = true, SWAPDAT = false, AESDECRYPT = Decrypt
    // Chop user provided data into words, feed 4 words at the time to indata()
    // Every 4 words, poll digest and apend it to ouptut
    // If the final part of the message is less than 4 words, padd with 0s
    // Before feeding last 4 words, flip STREAMEDLAST to true
    // Check that data.len = output.len
    // Flip STREAMEDLAST back to false in case the user wants to decrypt another message using the same key
}

// Specific driver types
pub struct Sha1<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}

impl<'a, 'd> Digest for Sha1<'a, 'd> {
    type Output = [u8; 20];

    fn update(&mut self, _data: &[u8]) {
        todo!("Add update function for Sha1");
    }

    fn finalise(self) -> Self::Output {
        todo!("Add finalise method for Sha1");
    }
}

pub struct Sha256<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
}

impl<'a, 'd> Digest for Sha256<'a, 'd> {
    type Output = [u8; 32];

    fn update(&mut self, _data: &[u8]) {
        todo!("Add update method for Sha 256");
    }

    fn finalise(self) -> Self::Output {
        todo!("Add finalise method for Sha 256");
    }
}

pub struct AesEcb<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
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
    // Does not require anything passed the default aes methods
}
pub struct AesCbc<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
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
        todo!("Add method boady");
    }
}
pub struct AesCtr<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>,
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
        todo!("Add method boady");
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

            pub fn key_size(&mut self) -> Result<u32, AesError> {
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

                    match &self.key {
                        Some(key) => feed_key(key),
                        None => {}
                    }
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
