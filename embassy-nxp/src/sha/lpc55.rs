//! Blocking SHA-256 driver for the LPC55S69 HASHCRYPT peripheral.
//!
//! This module provides a blocking SHA-256 implementation using the hardware
//! HASHCRYPT accelerator available on the LPC55S69 microcontroller.
//!
//! The driver feeds message blocks to the peripheral through the INDATA register
//! and polls the peripheral status flags until processing is complete.
//!
//! # Features
//!
//! - SHA-256 hashing only
//! - CPU polling mode (no DMA)
//! - Streaming input through [`CpuSha256::update`]
//! - Software-managed SHA-256 padding
//!
//! # Limitations
//!
//! This implementation is intentionally blocking. The CPU remains active while
//! waiting for the HASHCRYPT peripheral. A future implementation could use
//! interrupts or DMA to avoid polling.

#![macro_use]

use embassy_hal_internal::{Peri, PeripheralType};

use crate::pac::hashcrypt::vals::Mode;
use crate::pac::syscon::vals::HashAesRst;
use crate::pac::{HASHCRYPT, SYSCON};

/// SHA-256 processes data in 512-bit message blocks.
const BLOCK_SIZE: usize = 64;

/// Number of bytes reserved for the SHA-256 message length field.
const LENGTH_FIELD_SIZE: usize = 8;

/// Offset where the 64-bit message length is stored in the final block.
const LENGTH_OFFSET: usize = BLOCK_SIZE - 8;

/// Size of one HASHCRYPT input word.
const WORD_SIZE: usize = 4;

/// Errors returned by the SHA-256 hardware driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The HASHCRYPT peripheral reported an internal processing error.
    HardwareFault,

    /// The input message length exceeded the maximum supported SHA-256 length.
    ///
    /// SHA-256 stores the message length in a 64-bit field, limiting the
    /// maximum message size to 2^64 - 1 bits.
    LengthOverflow,
}

/// Marker trait implemented by peripherals capable of providing
/// HASHCRYPT functionality.
///
/// This trait exists to allow the driver API to accept concrete peripheral
/// instances while keeping the implementation independent of the PAC type.

pub(crate) trait SealedInstance {}

/// Marker trait implemented by HASHCRYPT peripherals supported by this driver.
#[allow(private_bounds)]
pub trait Instance: PeripheralType + SealedInstance + 'static {}

impl SealedInstance for crate::peripherals::HASHCRYPT {}
impl Instance for crate::peripherals::HASHCRYPT {}

/// Blocking SHA-256 driver.
///
/// The driver owns the HASHCRYPT peripheral and maintains the software
/// state required to perform streaming SHA-256 hash computations.
#[must_use]
pub struct CpuSha256<'d, T: Instance> {
    _hashcrypt: Peri<'d, T>,

    /// Partial message block waiting to be sent to HASHCRYPT.
    buffer: [u8; BLOCK_SIZE],

    /// Number of valid bytes currently stored in `buffer`.
    buffer_len: usize,

    /// Total length of the message processed so far, measured in bits.
    total_bits: u64,
}

impl<'d, T: Instance> CpuSha256<'d, T> {
    /// Creates a new SHA-256 driver.
    ///
    /// The driver takes ownership of the HASHCRYPT peripheral and initializes
    /// both its software state and the underlying hardware.
    ///
    /// The returned instance is ready to receive message data through
    /// [`CpuSha256::update`].
    #[must_use]
    pub fn new(hashcrypt: Peri<'d, T>) -> Self {
        let mut driver = Self {
            _hashcrypt: hashcrypt,
            buffer: [0; BLOCK_SIZE],
            buffer_len: 0,
            total_bits: 0,
        };

        driver.reset();

        driver
    }

    /// Feeds additional message bytes into the SHA-256 calculation.
    ///
    /// Input may be provided in arbitrary sized chunks. Internally, the driver
    /// accumulates data until a complete 512-bit block is available and then sends
    /// it to the HASHCRYPT peripheral.
    ///
    /// This method does not finalize the hash operation. Call [`finish`] after all
    /// input data has been provided. To begin a new hash computation after
    /// finalization, call [`reset`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::LengthOverflow`] if the total message length exceeds the
    /// SHA-256 maximum representable size.
    pub fn update(&mut self, mut data: &[u8]) -> Result<(), Error> {
        if data.is_empty() {
            return Ok(());
        }

        self.total_bits = self
            .total_bits
            .checked_add((data.len() as u64) * 8)
            .ok_or(Error::LengthOverflow)?;

        if self.buffer_len > 0 {
            let available = BLOCK_SIZE - self.buffer_len;
            let to_copy = core::cmp::min(available, data.len());
            self.buffer[self.buffer_len..self.buffer_len + to_copy].copy_from_slice(&data[..to_copy]);
            self.buffer_len += to_copy;
            data = &data[to_copy..];

            if self.buffer_len == BLOCK_SIZE {
                self.write_block(&self.buffer);
                self.buffer_len = 0;
            }
        }

        let mut chunks = data.chunks_exact(BLOCK_SIZE);
        for block in &mut chunks {
            let block: &[u8; BLOCK_SIZE] = block.try_into().unwrap();
            self.write_block(block);
        }

        let remainder = chunks.remainder();
        if !remainder.is_empty() {
            self.buffer[..remainder.len()].copy_from_slice(remainder);
            self.buffer_len = remainder.len();
        }

        Ok(())
    }

    /// Resets the driver to begin a new SHA-256 computation.
    ///
    /// This clears the software state, reinitializes the HASHCRYPT hardware,
    /// and starts a new hash operation. Any previously computed digest is discarded.
    pub fn reset(&mut self) {
        self.buffer.fill(0);
        self.buffer_len = 0;
        self.total_bits = 0;

        Self::init_hardware();
    }

    /// Completes the SHA-256 operation and returns the final digest.
    ///
    /// This method performs the SHA-256 finalization sequence:
    ///
    /// - appends the mandatory `1` bit,
    /// - adds zero padding,
    /// - appends the original message length,
    /// - waits for HASHCRYPT completion,
    /// - reads the resulting digest.
    ///
    /// The driver remains usable after finalization. Call [`reset`] before
    /// beginning another independent hash operation using the same HASHCRYPT
    /// peripheral.
    pub fn finish(&mut self) -> Result<[u32; 8], Error> {
        self.buffer[self.buffer_len] = 0x80;
        self.buffer_len += 1;

        if self.buffer_len > LENGTH_OFFSET {
            self.buffer[self.buffer_len..BLOCK_SIZE].fill(0);
            self.write_block(&self.buffer);
            self.buffer_len = 0;
        }

        self.buffer[self.buffer_len..LENGTH_OFFSET].fill(0);

        self.buffer[LENGTH_OFFSET..BLOCK_SIZE].copy_from_slice(&self.total_bits.to_be_bytes());

        self.write_block(&self.buffer);

        Self::wait_digest()?;

        Ok(Self::read_digest())
    }

    /// Initializes the HASHCRYPT peripheral for a new SHA-256 operation.
    fn init_hardware() {
        Self::release_reset();
        Self::enable_clock();
        Self::configure();
    }

    /// Releases the HASHCRYPT peripheral from reset.
    fn release_reset() {
        SYSCON.presetctrl2().modify(|r| {
            r.set_hash_aes_rst(HashAesRst::Released);
        });
    }

    /// Enables the AHB clock for the HASHCRYPT peripheral.
    fn enable_clock() {
        SYSCON.ahbclkctrl2().modify(|r| {
            r.set_hash_aes(true);
        });
    }

    /// Configures the HASHCRYPT peripheral for SHA-256 and starts a new hardware hash session.
    fn configure() {
        HASHCRYPT.ctrl().modify(|r| {
            r.set_mode(Mode::Sha2256);
        });

        HASHCRYPT.ctrl().modify(|r| {
            r.set_new_hash(true);
        });
    }

    /// Waits until HASHCRYPT finishes processing the current message.
    fn wait_digest() -> Result<(), Error> {
        loop {
            let status = HASHCRYPT.status().read();

            if status.error() {
                return Err(Error::HardwareFault);
            }

            if status.digest() {
                return Ok(());
            }

            cortex_m::asm::nop();
        }
    }

    /// Transfers one complete 512-bit SHA-256 message block to the HASHCRYPT
    /// peripheral.
    ///
    /// The peripheral accepts data through the 32-bit INDATA register, so the
    /// block is written as sixteen consecutive 32-bit words. The driver waits
    /// until the hardware is ready before writing each word.
    #[inline(always)]
    fn write_block(&self, block: &[u8; BLOCK_SIZE]) {
        for chunk in block.chunks_exact(WORD_SIZE) {
            // HASHCRYPT expects 32-bit little-endian words through INDATA.
            let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);

            while !HASHCRYPT.status().read().waiting() {
                cortex_m::asm::nop();
            }

            HASHCRYPT.indata().write(|r| {
                r.set_data(word);
            });
        }
    }

    /// Reads the 256-bit digest from the HASHCRYPT digest registers.
    #[inline]
    fn read_digest() -> [u32; LENGTH_FIELD_SIZE] {
        let mut digest = [0u32; LENGTH_FIELD_SIZE];
        for (i, word) in digest.iter_mut().enumerate() {
            *word = HASHCRYPT.digest0(i).read().digest();
        }

        digest
    }
}
