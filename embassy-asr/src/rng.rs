//! True random number generator.
//!
//! The vendor RNG implementation lives in the closed-source `libcrypto.a`.
//! This driver reconstructs the documented register programming from
//! `algorithm.h` / `rng.h` and the SDK example, and exposes fallible inherent
//! `try_*` methods plus the [`rand_core::RngCore`] wrapper (which panics on a
//! clock failure rather than returning predictable zeros).

use rand_core::RngCore;

use crate::rcc::{self, Peripheral};
use crate::{Peri, pac, peripherals};

/// RNG configuration reconstructed from the vendor crypto headers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// Clock divider field written to `RNGCLK` bits `[6:0]`.
    ///
    /// Masked to 7 bits on write: bit 7 is the clock-enable bit, set
    /// separately (the vendor example passes `0xff`, i.e. all value bits).
    pub divider: u8,
    /// Mode field written to `RNGCR` bits `[6:0]` (entropy/preprocess/postprocess).
    ///
    /// Masked to 7 bits on write, like `divider`.
    pub mode: u8,
}

impl Config {
    /// Values used by the vendor `crypto/rng` example (`rng_init(0x0f, 0xff)`).
    pub const fn new() -> Self {
        Self {
            divider: 0x0f,
            mode: 0x7f,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// RNG error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Vendor `DATACKERR` status bit was set.
    ClockError,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ClockError => f.write_str("RNG clock error"),
        }
    }
}

impl core::error::Error for Error {}

/// Owned RNG peripheral.
pub struct Rng<'d> {
    _peri: Peri<'d, peripherals::RNG>,
}

impl<'d> Rng<'d> {
    /// Enable the RNG clock domain and start the generator.
    pub fn new(peri: Peri<'d, peripherals::RNG>, config: Config) -> Self {
        let _ = rcc::enable_peripheral(Peripheral::Rng);
        // Reset so a second construction never inherits a stale divider,
        // mode, or error latch from a previous owner.
        let _ = rcc::reset_peripheral(Peripheral::Rng);
        let mut this = Self { _peri: peri };
        this.configure(config);
        this
    }

    /// Apply configuration and enable generation.
    pub fn configure(&mut self, config: Config) {
        let regs = Self::regs();
        unsafe {
            regs.rngclk().write_with_zero(|w| {
                w.div().bits(config.divider & 0x7f);
                w.clock_enable().set_bit()
            });
            regs.rngcr().write_with_zero(|w| {
                w.postprosel().bits(config.mode & 0x03);
                w.precodesel().bits((config.mode >> 2) & 0x03);
                w.entropysel().bits((config.mode >> 4) & 0x03);
                w.true_random_mode().bit(config.mode & 0x40 != 0);
                w.enable().set_bit()
            });
        }
    }

    /// Stop the generator.
    pub fn close(&mut self) {
        let regs = Self::regs();
        unsafe {
            regs.rngcr().write_with_zero(|w| w.enable().clear_bit());
            regs.rngclk().write_with_zero(|w| w.clock_enable().clear_bit());
        }
    }

    /// Fill `buf` with random bytes.
    pub fn try_fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let regs = Self::regs();
        let mut i = 0;
        while i < buf.len() {
            let mut sr = regs.rngsr().read();
            while !sr.data_ready().bit_is_set() {
                if regs.rngsr().read().data_clock_error().bit_is_set() {
                    return Err(Error::ClockError);
                }
                sr = regs.rngsr().read();
            }
            // The data register delivers one random byte per data-ready (the
            // upper 24 bits always read zero on silicon, verified over 256
            // samples); using them would pad every word with zeros.
            buf[i] = regs.rngdata().read().data().bits();
            i += 1;
        }
        Ok(())
    }

    /// Return one random `u32`.
    pub fn try_next_u32(&mut self) -> Result<u32, Error> {
        let mut bytes = [0u8; 4];
        self.try_fill_bytes(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    #[inline]
    fn regs() -> pac::Rngc {
        unsafe { pac::Rngc::steal() }
    }
}

impl Drop for Rng<'_> {
    fn drop(&mut self) {
        self.close();
        let _ = rcc::disable_peripheral(Peripheral::Rng);
    }
}

impl RngCore for Rng<'_> {
    fn next_u32(&mut self) -> u32 {
        Rng::try_next_u32(self).expect("RNG clock failure")
    }

    fn next_u64(&mut self) -> u64 {
        let lo = u64::from(Rng::try_next_u32(self).expect("RNG clock failure"));
        let hi = u64::from(Rng::try_next_u32(self).expect("RNG clock failure"));
        lo | (hi << 32)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        Rng::try_fill_bytes(self, dest).expect("RNG clock failure");
    }
}
