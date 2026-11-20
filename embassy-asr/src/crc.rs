//! Hardware CRC unit.
//!
//! Register programming follows the vendor `tremo_crc` driver.

use crate::rcc::{self, Peripheral};
use crate::{Peri, pac, peripherals};

/// CRC data register address for narrow volatile feeds (the PAC models only
/// the whole-word accessor; sub-word writes need the raw address).
const CRC_DR_ADDR: usize = 0x4002_2004;

/// Polynomial width programmed into CRC_CR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
#[non_exhaustive]
pub enum PolySize {
    /// 32-bit polynomial.
    Bits32 = 0x00,
    /// 16-bit polynomial.
    Bits16 = 0x08,
    /// 8-bit polynomial.
    Bits8 = 0x10,
    /// 7-bit polynomial.
    Bits7 = 0x18,
}

/// Input bit-order reversal mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
#[non_exhaustive]
pub enum ReverseIn {
    /// Do not reverse input bits.
    None = 0x00,
    /// Reverse each input byte.
    Byte = 0x02,
    /// Reverse each input half-word.
    HalfWord = 0x04,
    /// Reverse each input word.
    Word = 0x06,
}

/// CRC configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// Initial CRC value written to INIT before calculation.
    pub init_value: u32,
    /// Polynomial width.
    pub poly_size: PolySize,
    /// Polynomial value.
    pub poly: u32,
    /// Input bit-order reversal.
    pub reverse_in: ReverseIn,
    /// Reverse the CRC result bits.
    pub reverse_out: bool,
}

impl Config {
    /// CRC-32/ISO-HDLC style defaults matching common SDK examples.
    pub const fn new() -> Self {
        Self {
            init_value: 0xffff_ffff,
            poly_size: PolySize::Bits32,
            poly: 0x04c1_1db7,
            reverse_in: ReverseIn::Word,
            reverse_out: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Owned CRC peripheral.
pub struct Crc<'d> {
    _peri: Peri<'d, peripherals::CRC>,
    config: Config,
}

impl<'d> Crc<'d> {
    /// Enable the CRC clock, release reset, and apply `config`.
    pub fn new(peri: Peri<'d, peripherals::CRC>, config: Config) -> Self {
        let _ = rcc::enable_peripheral(Peripheral::Crc);
        let _ = rcc::reset_peripheral(Peripheral::Crc);
        let mut this = Self { _peri: peri, config };
        this.configure(config);
        this
    }

    /// Reconfigure the CRC engine and latch the initial value.
    pub fn configure(&mut self, config: Config) {
        self.config = config;
        let regs = Self::regs();
        let poly_size = match config.poly_size {
            PolySize::Bits32 => pac::crc::cr::PolySize::Value32,
            PolySize::Bits16 => pac::crc::cr::PolySize::Value16,
            PolySize::Bits8 => pac::crc::cr::PolySize::Value8,
            PolySize::Bits7 => pac::crc::cr::PolySize::Value7,
        };
        let reverse_in = match config.reverse_in {
            ReverseIn::None => pac::crc::cr::ReverseIn::None,
            ReverseIn::Byte => pac::crc::cr::ReverseIn::Byte,
            ReverseIn::HalfWord => pac::crc::cr::ReverseIn::Hword,
            ReverseIn::Word => pac::crc::cr::ReverseIn::Word,
        };
        // INIT and POLY must be in place before CR (with CALC_INIT) is
        // written: that write is what latches the initial value.
        unsafe {
            regs.init().write_with_zero(|w| w.bits(config.init_value));
            regs.poly().write_with_zero(|w| w.bits(config.poly));
            regs.cr().write_with_zero(|w| {
                w.poly_size().variant(poly_size);
                w.reverse_in().variant(reverse_in);
                w.reverse_out_en().bit(config.reverse_out);
                w.calc_init().set_bit()
            });
        }
    }

    /// Restart the running CRC from the configured initial value.
    ///
    /// Re-latches `INIT` without touching the polynomial or reversal setup,
    /// so a second message can reuse the same configuration.
    pub fn reset(&mut self) {
        let regs = Self::regs();
        unsafe {
            regs.init().write_with_zero(|w| w.bits(self.config.init_value));
            regs.cr().modify(|_, w| w.calc_init().set_bit());
        }
    }

    /// Feed 32-bit words and return the CRC result.
    ///
    /// The engine consumes exactly what is written: there is no
    /// padding or length validation, so a trailing partial word must be
    /// padded by the caller (the vendor driver pads short units with
    /// `0xFF`).
    pub fn feed_u32(&mut self, data: &[u32]) -> u32 {
        let regs = Self::regs();
        for word in data {
            unsafe {
                regs.dr().write_with_zero(|w| w.bits(*word));
            }
        }
        Self::wait_done();
        regs.dr().read().bits()
    }

    /// Feed 16-bit half-words and return the CRC result.
    ///
    /// Same no-padding contract as [`Self::feed_u32`].
    pub fn feed_u16(&mut self, data: &[u16]) -> u32 {
        for half in data {
            unsafe {
                core::ptr::write_volatile(CRC_DR_ADDR as *mut u16, *half);
            }
        }
        Self::wait_done();
        Self::regs().dr().read().bits()
    }

    /// Feed bytes and return the CRC result.
    ///
    /// Same no-padding contract as [`Self::feed_u32`].
    pub fn feed_u8(&mut self, data: &[u8]) -> u32 {
        for byte in data {
            unsafe {
                core::ptr::write_volatile(CRC_DR_ADDR as *mut u8, *byte);
            }
        }
        Self::wait_done();
        Self::regs().dr().read().bits()
    }

    fn wait_done() {
        let regs = Self::regs();
        while regs.cr().read().calc_flag().bit_is_set() {}
    }

    #[inline]
    fn regs() -> pac::Crc {
        unsafe { pac::Crc::steal() }
    }
}

impl Drop for Crc<'_> {
    fn drop(&mut self) {
        let _ = rcc::disable_peripheral(Peripheral::Crc);
    }
}
