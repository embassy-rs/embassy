//! Cryptographic Accelerator and Signaling Processing Engine (CASPER)
//!
//! This module provides hardware acceleration for asymmetric cryptography (RSA).

use crate::pac;
use embassy_hal_internal::{impl_peripheral, Peri};

/// Base address of CASPER SRAMX dedicated memory
pub const SRAMX_BASE: usize = 0x1400_0000;

/// Total size of SRAMX memory (8 KB)
pub const SRAMX_SIZE: usize = 0x2000;

/// Standard SRAMX offsets for RSA/Modular operations operands
pub mod offset {
    pub const OPERAND_A: usize = 0x000;
    pub const OPERAND_B: usize = 0x200;
    pub const OPERAND_C: usize = 0x400;
    pub const OPERAND_D: usize = 0x600;
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

        // Test reading the status register to verify the block is responding
        let _status = pac::CASPER.status().read();

        Self { _peri: peri }
    }

    /// Write a byte slice into SRAMX memory at the specified offset.
    ///
    /// # Panics
    /// Panics if the write operation exceeds SRAMX boundaries (8 KB).
    pub fn write_sramx(&mut self, offset: usize, data: &[u8]) {
        assert!(offset + data.len() <= SRAMX_SIZE, "CASPER SRAMX write overflow: offset {} + len {} exceeds 8KB", offset, data.len());

        let dest_ptr = (SRAMX_BASE + offset) as *mut u8;
        unsafe {
            core::ptr::copy_nonoverlapping(data.as_ptr(), dest_ptr, data.len());
        }
    }

    /// Read data from SRAMX memory at the specified offset into a buffer.
    ///
    /// # Panics
    /// Panics if the read operation exceeds SRAMX boundaries (8 KB).
    pub fn read_sramx(&self, offset: usize, buffer: &mut [u8]) {
        assert!(offset + buffer.len() <= SRAMX_SIZE, "CASPER SRAMX read overflow: offset {} + len {} exceeds 8KB", offset, buffer.len());

        let src_ptr = (SRAMX_BASE + offset) as *const u8;
        unsafe {
            core::ptr::copy_nonoverlapping(src_ptr, buffer.as_mut_ptr(), buffer.len());
        }
    }

    /// Zero-out a section of SRAMX memory.
    /// 
    /// # Panics
    /// Panics if the clear operation exceeds SRAMX boundaries (8 KB).
    pub fn clear_sramx(&mut self, offset: usize, len: usize) {
        assert!(offset + len <= SRAMX_SIZE, "CASPER SRAMX clear overflow: offset {} + len {} exceeds 8KB", offset, len);

        let dest_ptr = (SRAMX_BASE + offset) as *mut u8;
        unsafe {
            core::ptr::write_bytes(dest_ptr, 0, len);
        }
    }
}