//! Driver for the HASHCRYPT peripheral, mode switch skeleton
use embassy_hal_internal::Peri;
use nxp_pac::hashcrypt::vals::{Aeskeysz, Mode};
use nxp_pac::syscon::vals::HashAesRst::Released;

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
#[allow(dead_code)] // Used by the SHA and AES modes; remove once a caller exists.
fn read_digest(count: usize, out: &mut [u8]) {
    loop {
        let status = pac::HASHCRYPT.status().read().digest();
        // Block until the DIGEST status flag signals the output registers hold a
        // complete result, then read `count` words out of DIGEST0..n.
        if status {
            for i in 0..count {
                let word = pac::HASHCRYPT.digest0(i).read().digest();
                out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
            }
            break;
        }
        // If status is false, then that means there is no digest ready to be read, and since there is
        // no more incoming data, then that means we just need to keep waiting and so there is no need
        // to explicitly handle that case
    }
}

// Generic driver type
pub struct GenericDriver<'d> {
    _peri: Peri<'d, HASHCRYPT>,
}

// mode switching implementation of generic driver
impl<'d> GenericDriver<'d> {
    /// Creates an instance of a generic hashcrypt driver, which can be used to create an instance of a specific driver for one of the cryptographic operations that the chip is capable of
    pub fn new(peri: Peri<'d, HASHCRYPT>) -> Self {
        pac::SYSCON.ahbclkctrl2().modify(|w| {
            w.set_hash_aes(true);
        });

        pac::SYSCON.presetctrl2().modify(|w| {
            w.set_hash_aes_rst(Released);
        });
        Self { _peri: peri }
    }

    /// Creates an instance of a SHA-1 driver.
    ///
    /// # Warning
    /// SHA-1 has been cryptographically broken since 2017
    /// It's provided for legacy/protocol compatibility only, avoid it for anything security-sensitive.
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

    /// Creates an instance of a SHA-256 driver
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
    // rename to generic driver or _driver
}

pub trait Digest {
    type Output;
    /// Returns the digest of the message streamed via `update`
    fn finalise(&mut self) -> Self::Output;
}
// Helper methods for the types that implement the Digest trait
fn drain_buffer(buffer: &[u8; 64]) {
    let sha = pac::HASHCRYPT.indata();
    for chunk in buffer.chunks_exact(4) {
        let word = u32::from_le_bytes(chunk.try_into().unwrap());
        sha.write(|w| {
            w.set_data(word);
        });
    }
}

fn pad_drain_final(buffer: &mut [u8; 64], buffer_len: usize, total_len: u64) {
    // This method is meant to be called inside the body of a finalise method
    // Now that we know that there is no more incoming data from this message
    // we can start padding the message padding according to FIPS 180-4 §5.1.1, pg 13

    // Separate the message from the padding with a single 1
    buffer[buffer_len] = 0x80;
    // Is there room for the size of the message in the current block ?
    if buffer_len < 56 {
        // Add the padding until the last 2 words
        for i in (buffer_len + 1)..56 {
            buffer[i] = 0;
        }
        // Append the size of the entire message
        buffer[56..64].copy_from_slice(&(total_len * 8).to_be_bytes());
        drain_buffer(buffer);
    } else {
        // Pad until the block is completely filled
        for i in (buffer_len + 1)..64 {
            buffer[i] = 0;
        }
        // Drain the buffer in to the FIFO
        drain_buffer(buffer);

        // Reset
        *buffer = [0u8; 64];
        // Append the size of the entire message
        buffer[56..64].copy_from_slice(&(total_len * 8).to_be_bytes());

        // Now we can hash the final padded block
        // Hashing begins automatically once the 16 words (512 bits) of the FIFO are full
        drain_buffer(buffer);
    }
}

pub trait Aes {
    fn encrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError>;
    // Universal encrypt configuration via register calls
    // Chop user provided data into words, feed 4 words at the time to indata()
    // Every 4 words, poll digest and append it to output
    // If the final part of the message is less than 4 words, padd with 0s
    // Before feeding last 4 words, flip STREAMEDLAST to true
    // Check that data.len = output.len
    // Flip STREAMEDLAST back to false in case the user wants to decrypt another message using the same key

    fn decrypt(&mut self, data: &[u8], output: &mut [u8]) -> Result<(), AesError>;
    // Universal decrypt configuration using register calls
    // Chop user provided data into words, feed 4 words at the time to indata()
    // Every 4 words, poll digest and append it to output
    // If the final part of the message is less than 4 words, padd with 0s
    // Before feeding last 4 words, flip STREAMEDLAST to true
    // Check that data.len = output.len
    // Flip STREAMEDLAST back to false in case the user wants to decrypt another message using the same key

    // to handle messages that are not divisible in 4 blocks, add pading with 0, keep track of the size of
    // the paddind, all padding will produce garbage output which will need to be trimmed from the last block
    // of the digest
}

// Specific driver types
pub struct Sha1<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>, // A mutable reference to the generic driver which holds the HASHCRYPT peripheral
    buffer: [u8; 64],                 // A 64 byte buffer in which we can dump incoming data streams
    buffer_len: usize, // The number of valid bytes currently buffered, resets to 0 after buffer is drained
    total_len: u64,    // The size of the complete message to be hashed
}

impl<'a, 'd> Digest for Sha1<'a, 'd> {
    type Output = [u8; 20];

    fn finalise(&mut self) -> Self::Output {
        pad_drain_final(&mut self.buffer, self.buffer_len, self.total_len);
        // Now we can prepare the 8 word digest

        let mut digest: [u8; 20] = [0u8; 20];
        read_digest(5, &mut digest);

        // Reset the HASHCRYPT peripheral, so it's ready for a new hash. When finalize() is called again,
        // all the registers including the length of the message that was previously hashed are reset, so that
        // the next digest is correct and free of residual values from previous hash operations.
        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        // Clean the buffer and total message length to get rid of any leftovers from previous messages.
        self.buffer = [0u8; 64];
        self.buffer_len = 0;
        self.total_len = 0;

        digest
    }
}

pub struct Sha256<'a, 'd> {
    _peri: &'a mut GenericDriver<'d>, // A mutable reference to the generic driver which holds the HASHCRYPT peripheral
    buffer: [u8; 64],                 // A 64 byte buffer in which we can dump incoming data streams
    buffer_len: usize, // The number of valid bytes currently buffered, resets to 0 after buffer is drained
    total_len: u64,    // The size of the complete message to be hashed
}

impl<'a, 'd> Digest for Sha256<'a, 'd> {
    type Output = [u8; 32];
    fn finalise(&mut self) -> Self::Output {
        pad_drain_final(&mut self.buffer, self.buffer_len, self.total_len);
        // Now we can prepare the 8 word digest

        let mut digest: [u8; 32] = [0u8; 32];
        read_digest(8, &mut digest);

        // Reset the HASHCRYPT peripheral, so it's ready for a new hash. When finalize() is called again,
        // all the registers including the length of the message that was previously hashed are reset, so that
        // the next digest is correct and free of residual values from previous hash operations.
        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        // Clean the buffer and total message length to get rid of any leftovers from previous messages.
        self.buffer = [0u8; 64];
        self.buffer_len = 0;
        self.total_len = 0;

        digest
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
    // Does not require anything past the default AES methods
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
        todo!("Add method body");
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
        todo!("Add method body");
    }
}

macro_rules! impl_sha {
    ($ty:ident) => {
        impl<'a, 'd> $ty<'a, 'd> {
            /// Accepts and buffers an arbitrary length SHA message to be hashed
            pub fn update(&mut self, data: &[u8]) {
                let data_len = data.len() as u32; // Length of the incoming data
                let mut offset = 0; // tracks how many bytes of `data` have been consumed so far

                self.total_len += data.len() as u64;
                while offset < data_len {
                    // how much room is there in the buffer ?
                    let space = 64 - self.buffer_len;

                    // how much can i take from the incoming data ?
                    let take = space.min((data_len - offset) as usize);

                    // move "take" bytes from the data to the buffer, occupying whatever space is left in the buffer
                    self.buffer[self.buffer_len..(self.buffer_len + take)]
                        .copy_from_slice(&data[(offset as usize)..((offset) as usize) + take]);

                    self.buffer_len += take;
                    offset += take as u32;

                    // Once the buffer is full, we drain it in to the FIFO via .indata().set_data()
                    if self.buffer_len == 64 {
                        // buffer is full, so we drain the message streamed so far into the FIFO
                        drain_buffer(&self.buffer);
                        // Once the 16 word FIFO is full (see [drain_buffer]), hashing begins automatically, and we are free to start
                        // overwriting the buffer so we can fill it once more with the incoming data

                        // Reset the buffer
                        self.buffer_len = 0;
                        self.buffer = [0u8; 64];
                    }

                    // Even though a digest might be ready to read at this point,
                    // polling HASHCRYPT.status().read().digest() here would yield a useless, incomplete hash,
                    // caused by an incomplete message. Instead, we will poll it in [finalize()] when we can
                    // be sure that no more data will be streamed.
                }
            }
        }
    };
}

impl_sha!(Sha1);
impl_sha!(Sha256);

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
