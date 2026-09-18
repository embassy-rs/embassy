//! Hardware CRC unit.
//!
//! Register programming follows the vendor `tremo_crc` driver.

use crate::rcc::{self, Peripheral};
use crate::{Peri, pac, peripherals};

const CRC_DR_ADDR: usize = 0x4002_2004;

const CR_CALC_FLAG: u32 = 1 << 6;
const CR_CALC_INIT: u32 = 1 << 5;
const CR_REVERSE_OUT: u32 = 1 << 0;

/// Polynomial width programmed into CRC_CR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
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
}

impl<'d> Crc<'d> {
    /// Enable the CRC clock, release reset, and apply `config`.
    pub fn new(peri: Peri<'d, peripherals::CRC>, config: Config) -> Self {
        let _ = rcc::enable_peripheral(Peripheral::Crc);
        let _ = rcc::reset_peripheral(Peripheral::Crc);
        let this = Self { _peri: peri };
        this.configure(config);
        this
    }

    /// Reconfigure the CRC engine and latch the initial value.
    pub fn configure(&self, config: Config) {
        let regs = Self::regs();
        let mut cr = config.poly_size as u32 | config.reverse_in as u32;
        if config.reverse_out {
            cr |= CR_REVERSE_OUT;
        }
        cr |= CR_CALC_INIT;

        unsafe {
            regs.init().write_with_zero(|w| w.bits(config.init_value));
            regs.poly().write_with_zero(|w| w.bits(config.poly));
            regs.cr().write_with_zero(|w| w.bits(cr));
        }
    }

    /// Feed 32-bit words and return the CRC result.
    pub fn feed_u32(&self, data: &[u32]) -> u32 {
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
    pub fn feed_u16(&self, data: &[u16]) -> u32 {
        for half in data {
            unsafe {
                core::ptr::write_volatile(CRC_DR_ADDR as *mut u16, *half);
            }
        }
        Self::wait_done();
        Self::regs().dr().read().bits()
    }

    /// Feed bytes and return the CRC result.
    pub fn feed_u8(&self, data: &[u8]) -> u32 {
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
        while regs.cr().read().bits() & CR_CALC_FLAG != 0 {}
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
