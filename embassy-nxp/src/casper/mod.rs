//! Cryptographic Accelerator and Signaling Processing Engine (CASPER)
//!
//! This module provides hardware acceleration for asymmetric cryptography (RSA).

use crate::pac;
use embassy_hal_internal::{impl_peripheral, Peri};

/// Base address of CASPER SRAMX dedicated memory (non-secure mode)
pub const SRAMX_BASE: usize = 0x0400_0000;
/// Bit position used by the SRAMX interleaved addressing scheme to select
/// between the two RAMX banks.
const CASPER_RAM_OFFSET: usize = 14;
/// Total size of SRAMX memory (8 KB)
pub const SRAMX_SIZE: usize = 0x2000;

/// Standard SRAMX offsets for RSA/Modular operations operands
pub mod offset {
    pub const OPERAND_A: usize = 0x000;
    pub const OPERAND_B: usize = 0x200;
    pub const OPERAND_C: usize = 0x400;
    pub const OPERAND_D: usize = 0x600;
}

/// CASPER Hardware AHB Operations
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Mul64Nosum = 0x01,
    Mul64Sum = 0x02,
    Mul64Fullsum = 0x03,
    Mul64Reduce = 0x04,
    Add64 = 0x08,
    Sub64 = 0x09,
    Double64 = 0x0A,
    Xor64 = 0x0B,
    Rsub64 = 0x0C,
    Copy = 0x14,
    Fill = 0x16, // NOTE: Opcode::Fill is documented in the User Manual, but could not be exercised through the AHB interface. The official MCUX SDK also never uses this opcode. Support postponed until its semantics are experimentally determined.
    Zero = 0x17,
}

/// Custom peripheral marker struct for CASPER hardware block
pub struct CASPER;

impl_peripheral!(CASPER);

pub struct CasperDriver<'d> {
    _peri: Peri<'d, CASPER>,
}

impl<'d> CasperDriver<'d> {
    /// Create a new driver instance, enable clocks and apply hardware reset
    pub fn new(peri: impl Into<Peri<'d, CASPER>>) -> Self {
        let peri = peri.into();

        // Get access to SYSCON PAC instance
        let syscon = pac::SYSCON;

        // 1. Enable clock for CASPER in AHBCLKCTRL2 register (bit 24 LPC55S6xLPC55S2xLPC552x User manual)
        syscon.ahbclkctrl2().modify(|w| w.set_casper(true));

        // 2. Reset the CASPER block in PRESETCTRL2 register (bit 24 in LPC55S6xLPC55S2xLPC552x User manual)
        syscon.presetctrl2().modify(|w| w.set_casper_rst(pac::syscon::vals::CasperRst::ASSERTED));   // Activate reset
        syscon.presetctrl2().modify(|w| w.set_casper_rst(pac::syscon::vals::CasperRst::RELEASED));  // Release reset

        Self { _peri: peri }
    }

    /// Translate a CPU-visible SRAMX address into the interleaved address
    /// expected by the CASPER SRAM interface.
    #[inline(always)]
    fn interleave(addr: usize) -> usize {
        (((((addr >> 2) & 1) << CASPER_RAM_OFFSET) // Select the RAMX bank used for the interleaved address.
            + ((addr >> 3) << 2) // Calculate the word offset within the selected bank.
            + (addr & 3)) // Calculate the byte offset within the word.
            & 0xffff) // Leave only the lower 16 bits.
            | SRAMX_BASE // Restore the SRAMX base address.
    }

    #[inline(always)]
    pub fn regs(&self) -> pac::casper::Casper {
        pac::CASPER
    }
    
    /// Write a 32-bit value (word) into SRAMX memory at the specified offset.
    ///
    /// # Panics
    /// Panics if the write operation exceeds SRAMX boundaries (8 KB).
    pub fn write_word(&mut self, offset: usize, value: u32) {
        assert!(offset + 4 <= SRAMX_SIZE, "CASPER SRAMX write_word overflow, offset {} + 4B exceeds 8KB", offset);

        let ptr = Self::interleave(SRAMX_BASE + offset) as *mut u32;
        unsafe {
            core::ptr::write_volatile(ptr, value);
        }
    }
    
    /// Read a 32-bit value (word) from SRAMX memory at the specified offset.
    ///
    /// # Panics
    /// Panics if the read operation exceeds SRAMX boundaries (8 KB).
    pub fn read_word(&self, offset: usize) -> u32 {
        assert!(offset + 4 <= SRAMX_SIZE, "CASPER SRAMX read_word overflow, offset {} + 4B exceeds 8KB", offset);

        let ptr = Self::interleave(SRAMX_BASE + offset) as *const u32;
        unsafe {
            core::ptr::read_volatile(ptr)
        }
    }

    /// Write a 64-bit value (dword - double word) into SRAMX memory at the specified offset.
    ///
    /// # Panics
    /// Panics if the write operation exceeds SRAMX boundaries (8 KB).
    pub fn write_dword(&mut self, offset: usize, value: u64) {
        assert!(offset + 8 <= SRAMX_SIZE,"CASPER SRAMX write_dword overflow offset {} + 8B exceeds 8KB", offset);

        self.write_word(offset, value as u32);
        self.write_word(offset + 4, (value >> 32) as u32);
    }

    /// Read a 64-bit value (dword - double word) from SRAMX memory at the specified offset.
    ///
    /// # Panics
    /// Panics if the read operation exceeds SRAMX boundaries (8 KB).
    pub fn read_dword(&self, offset: usize) -> u64 {
        assert!(offset + 8 <= SRAMX_SIZE,"CASPER SRAMX read_dword overflow offset {} + 8B exceeds 8KB", offset);

        let low = self.read_word(offset) as u64;
        let high = self.read_word(offset + 4) as u64;

        low | (high << 32)
    }

    /// Zero-out a section of SRAMX memory.
    /// 
    /// # Panics
    /// Panics if the clear operation exceeds SRAMX boundaries (8 KB).
    pub fn clear(&mut self, offset: usize, len: usize) {
        assert!(offset + len <= SRAMX_SIZE, "CASPER SRAMX clear overflow, offset {} + len ({}B) exceeds 8KB", offset, len);
        assert!(len % 4 == 0,"clear length must be multiple of 4 bytes");

        for word_offset in (0..len).step_by(4) {
            self.write_word(offset + word_offset, 0);
        }
    }

    /// Check if CASPER hardware accelerator is currently busy.
    pub fn is_busy(&self) -> bool {
        pac::CASPER.status().read().busy() == pac::casper::vals::Busy::BUSY
    }

    /// Synchronous execution of a CASPER AHB operation
    pub fn execute_op_sync(&mut self, opcode: Opcode, iter: u8, a_offset: usize, c_offset: usize, res_offset: usize) {
        let casper = pac::CASPER;

        // CTRL0 stores AB/CD base offsets in 32-bit words.
        casper.ctrl0().write(|w| {
            w.set_aboff((a_offset / 4) as u16);
            w.set_cdoff((c_offset / 4) as u16);
        });

        // CTRL1 configures the operation mode, iteration count, and result offset.
        // Offsets are specified in 32-bit words.
        casper.ctrl1().write(|w| {
            w.set_mode(opcode as u8);
            w.set_iter(iter);
            w.set_resoff((res_offset / 4) as u16);
        });

        // Wait for the accelerator to complete the operation.
        while self.is_busy() {
            core::hint::spin_loop();
        }
    }
}