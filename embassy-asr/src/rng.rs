//! True random number generator.
//!
//! The vendor RNG implementation lives in the closed-source `libcrypto.a`.
//! This driver reconstructs the documented register programming from
//! `algorithm.h` / `rng.h` and the SDK example, and exposes [`rand_core::RngCore`].

use rand_core::RngCore;

use crate::rcc::{self, Peripheral};
use crate::{Peri, pac, peripherals};

const RNGCLKEN: u32 = 1 << 7;
const RNGEN: u32 = 1 << 7;
const DATARDY: u32 = 1 << 0;
const DATACKERR: u32 = 1 << 1;

/// RNG configuration reconstructed from the vendor crypto headers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Clock divider field written to `RNGCLK` bits `[6:0]`.
    pub divider: u8,
    /// Mode field written to `RNGCR` bits `[6:0]` (entropy/preprocess/postprocess).
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
pub enum Error {
    /// Vendor `DATACKERR` status bit was set.
    ClockError,
}

/// Owned RNG peripheral.
pub struct Rng<'d> {
    _peri: Peri<'d, peripherals::RNG>,
}

impl<'d> Rng<'d> {
    /// Enable the RNG clock domain and start the generator.
    pub fn new(peri: Peri<'d, peripherals::RNG>, config: Config) -> Self {
        let _ = rcc::enable_peripheral(Peripheral::Rng);
        let this = Self { _peri: peri };
        this.configure(config);
        this
    }

    /// Apply configuration and enable generation.
    pub fn configure(&self, config: Config) {
        let regs = Self::regs();
        unsafe {
            regs.rngclk()
                .write_with_zero(|w| w.bits((config.divider as u32 & 0x7f) | RNGCLKEN));
            regs.rngcr()
                .write_with_zero(|w| w.bits((config.mode as u32 & 0x7f) | RNGEN));
        }
    }

    /// Stop the generator.
    pub fn close(&self) {
        let regs = Self::regs();
        unsafe {
            regs.rngcr().write_with_zero(|w| w.bits(0));
            regs.rngclk().write_with_zero(|w| w.bits(0));
        }
    }

    /// Fill `buf` with random bytes.
    pub fn try_fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let regs = Self::regs();
        let mut i = 0;
        while i < buf.len() {
            while regs.rngsr().read().bits() & DATARDY == 0 {
                if regs.rngsr().read().bits() & DATACKERR != 0 {
                    return Err(Error::ClockError);
                }
            }
            let word = regs.rngdata().read().bits();
            for byte in word.to_le_bytes() {
                if i >= buf.len() {
                    break;
                }
                buf[i] = byte;
                i += 1;
            }
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
    fn regs() -> pac::Rng {
        unsafe { pac::Rng::steal() }
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
        self.try_next_u32().unwrap_or(0)
    }

    fn next_u64(&mut self) -> u64 {
        let lo = RngCore::next_u32(self) as u64;
        let hi = RngCore::next_u32(self) as u64;
        lo | (hi << 32)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let _ = self.try_fill_bytes(dest);
    }
}
