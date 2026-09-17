#![cfg(lpc55)]
//! Cryptographic Accelerator and Signaling Processing Engine with RAM-sharing (CASPER) driver.
//!
//! This module provides hardware acceleration for big-integer arithmetic operations
//! (addition, subtraction, multiplication, and Montgomery reduction steps)
//! typically used in asymmetric cryptography (such as RSA and ECC).

use embassy_hal_internal::Peri;

use crate::pac;
use crate::peripherals::CASPER;

/// Base address of CASPER-dedicated SRAMX memory (non-secure mode)
const SRAMX_BASE: usize = 0x0400_0000;
/// Bit position used by the SRAMX interleaved addressing scheme to select between the two RAMX banks.
const CASPER_RAM_OFFSET: usize = 14;
/// Total size of CASPER-dedicated SRAMX memory (8 KB)
const SRAMX_SIZE: usize = 0x2000;

/// Internal SRAMX layout used for high-level CASPER operations
mod offset {
    pub const AB: usize = 0x0000;
    pub const CD: usize = 0x0800;
    pub const RES: usize = 0x1000;
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
    Fill = 0x16, // Documented by the User Manual, but not used by the MCUX SDK and could not be exercised through the AHB interface. Support postponed.
    Zero = 0x17,
}

/// Error variants for CASPER driver operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Memory offset or operation span exceeds SRAMX boundary (8 KB)
    OutOfBounds,
    /// Offset or length is not properly aligned (e.g. 4-byte or 8-byte aligned)
    MisalignedOffset,
    /// Invalid length or iteration count (e.g. 0 or exceeds 256)
    InvalidLength,
    /// The destination slice/buffer is too small to receive the operation result
    BufferTooSmall,
    /// Operands length mismatch (e.g. CD and W length differ)
    LengthMismatch,
}

pub struct CasperDriver<'d> {
    _peri: Peri<'d, CASPER>,
}

impl<'d> CasperDriver<'d> {
    /// Create a new driver instance, enable clocks and apply hardware reset
    pub fn new(peri: Peri<'d, CASPER>) -> Self {
        // Get access to SYSCON PAC instance
        let syscon = pac::SYSCON;

        // 1. Enable clock for CASPER in AHBCLKCTRL2 register (bit 24 LPC55S6xLPC55S2xLPC552x User manual)
        syscon.ahbclkctrl2().modify(|w| w.set_casper(true));

        // 2. Reset the CASPER block in PRESETCTRL2 register (bit 24 in LPC55S6xLPC55S2xLPC552x User manual)
        syscon
            .presetctrl2()
            .modify(|w| w.set_casper_rst(pac::syscon::vals::CasperRst::Asserted)); // Activate reset
        syscon
            .presetctrl2()
            .modify(|w| w.set_casper_rst(pac::syscon::vals::CasperRst::Released)); // Release reset

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

    /// Write a 32-bit value (word) into SRAMX memory at the specified offset.
    ///
    /// # Errors
    ///
    /// * [`Error::OutOfBounds`] - if the write operation exceeds SRAMX boundaries (8 KB).
    /// * [`Error::MisalignedOffset`] - if `offset` is not aligned to 4 bytes.
    pub fn write_word(&mut self, offset: usize, value: u32) -> Result<(), Error> {
        if offset > SRAMX_SIZE - 4 {
            return Err(Error::OutOfBounds);
        }
        if offset % 4 != 0 {
            return Err(Error::MisalignedOffset);
        }

        let ptr = Self::interleave(SRAMX_BASE + offset) as *mut u32;
        unsafe {
            core::ptr::write_volatile(ptr, value);
        }
        Ok(())
    }

    /// Read a 32-bit value (word) from SRAMX memory at the specified offset.
    ///
    /// Returns the read `u32` value on success.
    ///
    /// # Errors
    ///
    /// * [`Error::OutOfBounds`] - if the read operation exceeds SRAMX boundaries (8 KB).
    /// * [`Error::MisalignedOffset`] - if `offset` is not aligned to 4 bytes.
    pub fn read_word(&self, offset: usize) -> Result<u32, Error> {
        if offset > SRAMX_SIZE - 4 {
            return Err(Error::OutOfBounds);
        }
        if offset % 4 != 0 {
            return Err(Error::MisalignedOffset);
        }

        let ptr = Self::interleave(SRAMX_BASE + offset) as *const u32;
        Ok(unsafe { core::ptr::read_volatile(ptr) })
    }

    /// Write a 64-bit value (dword - double word) into SRAMX memory at the specified offset.
    ///
    /// # Errors
    ///
    /// * [`Error::OutOfBounds`] - if the write operation exceeds SRAMX boundaries (8 KB).
    /// * [`Error::MisalignedOffset`] - if `offset` is not aligned to 8 bytes.
    pub fn write_dword(&mut self, offset: usize, value: u64) -> Result<(), Error> {
        if offset > SRAMX_SIZE - 8 {
            return Err(Error::OutOfBounds);
        }
        if offset % 8 != 0 {
            return Err(Error::MisalignedOffset);
        }

        self.write_word(offset, value as u32)?;
        self.write_word(offset + 4, (value >> 32) as u32)?;
        Ok(())
    }

    /// Read a 64-bit value (dword - double word) from SRAMX memory at the specified offset.
    ///
    /// Returns the reconstructed 64-bit value from two consecutive 32-bit words on success.
    ///
    /// # Errors
    ///
    /// * [`Error::OutOfBounds`] - if the read operation exceeds SRAMX boundaries (8 KB).
    /// * [`Error::MisalignedOffset`] - if `offset` is not aligned to 8 bytes.
    pub fn read_dword(&self, offset: usize) -> Result<u64, Error> {
        if offset > SRAMX_SIZE - 8 {
            return Err(Error::OutOfBounds);
        }
        if offset % 8 != 0 {
            return Err(Error::MisalignedOffset);
        }

        let low = self.read_word(offset)? as u64;
        let high = self.read_word(offset + 4)? as u64;

        Ok(low | (high << 32))
    }

    /// Zero-out a section of SRAMX memory.
    ///
    /// # Errors
    ///
    /// * [`Error::OutOfBounds`] - if the clear operation exceeds SRAMX boundaries (8 KB).
    /// * [`Error::MisalignedOffset`] - if `offset` or `len` is not aligned to 4 bytes.
    pub fn clear(&mut self, offset: usize, len: usize) -> Result<(), Error> {
        if offset > SRAMX_SIZE || len > SRAMX_SIZE - offset {
            return Err(Error::OutOfBounds);
        }
        if len % 4 != 0 || offset % 4 != 0 {
            return Err(Error::MisalignedOffset);
        }

        for word_offset in (0..len).step_by(4) {
            self.write_word(offset + word_offset, 0)?;
        }
        Ok(())
    }

    /// Check if CASPER hardware accelerator is currently busy.
    pub fn is_busy(&self) -> bool {
        pac::CASPER.status().read().busy() == pac::casper::vals::Busy::Busy
    }

    /// Synchronous execution of a CASPER AHB operation
    ///
    /// # Errors
    ///
    /// * [`Error::OutOfBounds`] - if `a_offset`, `c_offset`, or `res_offset` exceeds SRAMX boundaries (8 KB).
    /// * [`Error::MisalignedOffset`] - if `a_offset`, `c_offset`, or `res_offset` is not aligned to 4 bytes.
    pub fn execute_op_sync(
        &mut self,
        opcode: Opcode,
        iter: u8,
        a_offset: usize,
        c_offset: usize,
        res_offset: usize,
    ) -> Result<(), Error> {
        if a_offset > SRAMX_SIZE || c_offset > SRAMX_SIZE || res_offset > SRAMX_SIZE {
            return Err(Error::OutOfBounds);
        }
        if a_offset % 4 != 0 || c_offset % 4 != 0 || res_offset % 4 != 0 {
            return Err(Error::MisalignedOffset);
        }

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
        Ok(())
    }

    /// Returns the carry flag from the last CASPER operation.
    pub fn carry(&self) -> bool {
        pac::CASPER.status().read().carry()
    }

    /// Copy a sequence of 64-bit values from one SRAMX location to another.
    ///
    /// The slice `values` uses little-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidLength`] - if `values` is empty or contains more than 256 elements.
    /// * [`Error::OutOfBounds`] - if the source or destination range exceeds SRAMX boundaries (8 KB).
    /// * [`Error::MisalignedOffset`] - if `src_offset` or `dst_offset` is not aligned to 8 bytes.
    pub fn copy_values(&mut self, src_offset: usize, dst_offset: usize, values: &[u64]) -> Result<(), Error> {
        if values.is_empty() || values.len() > 256 {
            return Err(Error::InvalidLength);
        }

        let n: usize = values.len();
        if src_offset > SRAMX_SIZE
            || dst_offset > SRAMX_SIZE
            || n * 8 > SRAMX_SIZE - src_offset
            || n * 8 > SRAMX_SIZE - dst_offset
        {
            return Err(Error::OutOfBounds);
        }
        if src_offset % 8 != 0 || dst_offset % 8 != 0 {
            return Err(Error::MisalignedOffset);
        }

        for i in 0..n {
            self.write_dword(src_offset + i * 8, values[i])?;
        }
        self.execute_op_sync(Opcode::Copy, (n - 1) as u8, src_offset, 0, dst_offset)?;
        Ok(())
    }

    /// Zero-out a sequence of 64-bit values (dwords - double words) in SRAMX memory.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidLength`] - if `dwords` is zero or exceeds 256.
    /// * [`Error::OutOfBounds`] - if `res_offset` plus the size of values to be zeroed exceeds SRAMX boundaries (8 KB).
    /// * [`Error::MisalignedOffset`] - if `res_offset` is not aligned to 8 bytes.
    pub fn zero(&mut self, res_offset: usize, dwords: usize) -> Result<(), Error> {
        if dwords == 0 || dwords > 256 {
            return Err(Error::InvalidLength);
        }
        if res_offset > SRAMX_SIZE || dwords * 8 > SRAMX_SIZE - res_offset {
            return Err(Error::OutOfBounds);
        }
        if res_offset % 8 != 0 {
            return Err(Error::MisalignedOffset);
        }

        self.execute_op_sync(Opcode::Zero, (dwords - 1) as u8, 0, 0, res_offset)?;
        Ok(())
    }

    /// Perform a bitwise XOR operation on pairs of 64-bit values (dwords - double words) in SRAMX memory.
    ///
    /// The slices `operands` and `result` use little-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidLength`] - if `operands` is empty or contains more than 256 pairs of values.
    /// * [`Error::BufferTooSmall`] - if `result` buffer is not large enough to hold the results of the XOR operation.
    pub fn xor(&mut self, operands: &[(u64, u64)], result: &mut [u64]) -> Result<(), Error> {
        if operands.is_empty() || operands.len() > 256 {
            return Err(Error::InvalidLength);
        }
        if result.len() < operands.len() {
            return Err(Error::BufferTooSmall);
        }
        let n = operands.len();

        for i in 0..n {
            let (r, a) = operands[i];
            self.write_dword(offset::AB + i * 8, a)?;
            self.write_dword(offset::RES + i * 8, r)?;
        }
        self.execute_op_sync(Opcode::Xor64, (n - 1) as u8, offset::AB, 0, offset::RES)?;

        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8)?;
        }
        Ok(())
    }

    /// Perform a doubling operation on a sequence of 64-bit values (dwords - double words) in SRAMX memory.
    ///
    /// The slices `values` and `result` use little-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// Returns the carry flag reported by the CASPER operation.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidLength`] - if `values` is empty or contains more than 256 values.
    /// * [`Error::BufferTooSmall`] - if `result` buffer is not large enough to hold the results of the DOUBLE operation.
    pub fn double(&mut self, values: &[u64], result: &mut [u64]) -> Result<bool, Error> {
        if values.is_empty() || values.len() > 256 {
            return Err(Error::InvalidLength);
        }
        if result.len() < values.len() {
            return Err(Error::BufferTooSmall);
        }
        let n = values.len();

        for i in 0..n {
            self.write_dword(offset::RES + i * 8, values[i])?;
        }
        self.execute_op_sync(Opcode::Double64, (n - 1) as u8, 0, 0, offset::RES)?;

        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8)?;
        }
        Ok(self.carry())
    }

    /// Perform an addition operation on pairs of 64-bit values (dwords - double words) in SRAMX memory.
    ///
    /// The slices `operands` and `result` use little-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// Returns the carry flag reported by the CASPER operation.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidLength`] - if `operands` is empty or contains more than 256 pairs of values.
    /// * [`Error::BufferTooSmall`] - if `result` buffer is not large enough to hold the results of the ADD operation.
    pub fn add(&mut self, operands: &[(u64, u64)], result: &mut [u64]) -> Result<bool, Error> {
        if operands.is_empty() || operands.len() > 256 {
            return Err(Error::InvalidLength);
        }
        if result.len() < operands.len() {
            return Err(Error::BufferTooSmall);
        }
        let n = operands.len();

        for i in 0..n {
            let (r, a) = operands[i];
            self.write_dword(offset::AB + i * 8, a)?;
            self.write_dword(offset::RES + i * 8, r)?;
        }
        self.execute_op_sync(Opcode::Add64, (n - 1) as u8, offset::AB, 0, offset::RES)?;

        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8)?;
        }
        Ok(self.carry())
    }

    /// Perform a subtraction operation on pairs of 64-bit values (dwords - double words) in SRAMX memory.
    ///
    /// CASPER supports subtraction with borrow, and the carry flag indicates whether a borrow occurred during the operation.
    /// Uses forward subtraction (R - A) where R is the minuend and A is the subtrahend. R is the first operand and A is the second operand in each pair.
    ///
    /// The slices `operands` and `result` use little-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// Returns the carry flag reported by the CASPER operation.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidLength`] - if `operands` is empty or contains more than 256 pairs of values.
    /// * [`Error::BufferTooSmall`] - if `result` buffer is not large enough to hold the results of the SUB operation.
    pub fn sub(&mut self, operands: &[(u64, u64)], result: &mut [u64]) -> Result<bool, Error> {
        if operands.is_empty() || operands.len() > 256 {
            return Err(Error::InvalidLength);
        }
        if result.len() < operands.len() {
            return Err(Error::BufferTooSmall);
        }
        let n = operands.len();

        for i in 0..n {
            let (r, a) = operands[i];
            self.write_dword(offset::AB + i * 8, a)?;
            self.write_dword(offset::RES + i * 8, r)?;
        }
        self.execute_op_sync(Opcode::Sub64, (n - 1) as u8, offset::AB, 0, offset::RES)?;

        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8)?;
        }
        Ok(self.carry())
    }

    /// Perform a reverse subtraction operation on pairs of 64-bit values (dwords - double words) in SRAMX memory.
    ///
    /// CASPER supports subtraction with borrow, and the carry flag indicates whether a borrow occurred during the operation.
    /// Uses reverse subtraction (A - R) where A is the minuend and R is the subtrahend. A is the first operand and R is the second operand in each pair.
    ///
    /// The slices `operands` and `result` use little-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// Returns the carry flag reported by the CASPER operation.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidLength`] - if `operands` is empty or contains more than 256 pairs of values.
    /// * [`Error::BufferTooSmall`] - if `result` buffer is not large enough to hold the results of the RSUB operation.
    pub fn rsub(&mut self, operands: &[(u64, u64)], result: &mut [u64]) -> Result<bool, Error> {
        if operands.is_empty() || operands.len() > 256 {
            return Err(Error::InvalidLength);
        }
        if result.len() < operands.len() {
            return Err(Error::BufferTooSmall);
        }
        let n = operands.len();

        for i in 0..n {
            let (a, r) = operands[i];
            self.write_dword(offset::AB + i * 8, a)?;
            self.write_dword(offset::RES + i * 8, r)?;
        }
        self.execute_op_sync(Opcode::Rsub64, (n - 1) as u8, offset::AB, 0, offset::RES)?;

        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8)?;
        }
        Ok(self.carry())
    }

    /// Perform a 64-bit multiplication without accumulating the result into the existing RES contents.
    ///
    /// The 64-bit value `ab` is multiplied by the sequence of 64-bit values in `cd`.
    /// The resulting multi-word value is written to RES and returned through `result`.
    ///
    /// The slices `cd` and `result` use little-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidLength`] - if `cd` is empty or contains more than 256 elements.
    /// * [`Error::BufferTooSmall`] - if `result` buffer is not large enough to hold the `cd.len() + 1` output values.
    pub fn mul_nosum(&mut self, ab: u64, cd: &[u64], result: &mut [u64]) -> Result<(), Error> {
        if cd.is_empty() || cd.len() > 256 {
            return Err(Error::InvalidLength);
        }
        if result.len() <= cd.len() {
            return Err(Error::BufferTooSmall);
        }
        let n = cd.len();

        self.write_dword(offset::AB, ab)?;
        for i in 0..n {
            self.write_dword(offset::CD + i * 8, cd[i])?;
        }
        self.execute_op_sync(Opcode::Mul64Nosum, (n - 1) as u8, offset::AB, offset::CD, offset::RES)?;
        for i in 0..=n {
            result[i] = self.read_dword(offset::RES + i * 8)?;
        }
        Ok(())
    }

    /// Multiply a 64-bit value by a sequence of 64-bit values and accumulate each product into the existing RES values.
    ///
    /// The operation performs the CASPER MUL64_SUM operation, which reads the existing RES contents,
    /// adds the corresponding product, and writes the accumulated result back to RES.
    /// The `w` slice provides the initial RES values. It may contain more elements than `cd`; only the RES words reached by the CASPER operation are modified.
    ///
    /// The slices `cd`, `w` and `result` use little-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidLength`] - if `cd` or `w` is empty, or if either slice contains more than 256 elements.
    /// * [`Error::LengthMismatch`] - if `cd` contains more elements than `w`.
    /// * [`Error::BufferTooSmall`] - if `result` buffer is not large enough to hold the `cd.len() + 1` output values.
    pub fn mul_sum(&mut self, ab: u64, cd: &[u64], w: &[u64], result: &mut [u64]) -> Result<(), Error> {
        if cd.is_empty() || w.is_empty() || cd.len() > 256 || w.len() > 256 {
            return Err(Error::InvalidLength);
        }
        if cd.len() > w.len() {
            return Err(Error::LengthMismatch);
        }
        if result.len() <= cd.len() {
            return Err(Error::BufferTooSmall);
        }
        let n = cd.len();

        self.write_dword(offset::AB, ab)?;
        for i in 0..n {
            self.write_dword(offset::CD + i * 8, cd[i])?;
        }
        for i in 0..w.len() {
            self.write_dword(offset::RES + i * 8, w[i])?;
        }
        self.execute_op_sync(Opcode::Mul64Sum, (n - 1) as u8, offset::AB, offset::CD, offset::RES)?;
        for i in 0..=n {
            result[i] = self.read_dword(offset::RES + i * 8)?;
        }
        Ok(())
    }

    /// Multiply a 64-bit value by a sequence of 64-bit values and accumulate the products into the existing RES values, including the most significant RES words.
    ///
    /// This operation performs the CASPER MUL64_FULLSUM operation, which reads the existing RES contents,
    /// adds the corresponding products, and propagates the carry through the full result.
    /// The `w` slice provides the initial RES values.
    ///
    /// The slices `cd`, `w` and `result` use little-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// Returns the carry flag reported by the CASPER operation.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidLength`] - if `cd` or `w` is empty, or if either slice contains more than 256 elements.
    /// * [`Error::LengthMismatch`] - if `cd` contains more elements than `w`.
    /// * [`Error::BufferTooSmall`] - if `result` buffer is not large enough to hold the `cd.len() + 1` output values.
    pub fn mul_fullsum(&mut self, ab: u64, cd: &[u64], w: &[u64], result: &mut [u64]) -> Result<bool, Error> {
        if cd.is_empty() || w.is_empty() || cd.len() > 256 || w.len() > 256 {
            return Err(Error::InvalidLength);
        }
        if cd.len() > w.len() {
            return Err(Error::LengthMismatch);
        }
        if result.len() <= cd.len() {
            return Err(Error::BufferTooSmall);
        }
        let n = cd.len();

        self.write_dword(offset::AB, ab)?;
        for i in 0..n {
            self.write_dword(offset::CD + i * 8, cd[i])?;
        }
        for i in 0..w.len() {
            self.write_dword(offset::RES + i * 8, w[i])?;
        }
        self.execute_op_sync(Opcode::Mul64Fullsum, (n - 1) as u8, offset::AB, offset::CD, offset::RES)?;
        for i in 0..=n {
            result[i] = self.read_dword(offset::RES + i * 8)?;
        }
        Ok(self.carry())
    }

    /// Perform the CASPER MUL64_REDUCE operation, which is used as a step in Montgomery reduction algorithms.
    ///
    /// The 64-bit value `m` is multiplied by the sequence of 64-bit values in `cd` and accumulated into the existing RES values.
    /// The first RES write is skipped and the resulting value is shifted by one 64-bit word, as required by the CASPER reduction operation.
    /// The `w` slice provides the initial RES values and must contain the same number of elements as `cd`.
    /// The `m` value is expected to be the precomputed Montgomery reduction factor; CASPER does not calculate this value itself.
    /// The RES workspace is cleared before the operation to prevent stale SRAMX contents from affecting the reduction.
    ///
    /// The slices `cd`, `w` and `result` use little-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidLength`] - if `cd` or `w` is empty, or if either slice contains more than 256 elements.
    /// * [`Error::LengthMismatch`] - if `cd` and `w` have different lengths.
    /// * [`Error::BufferTooSmall`] - if `result` buffer does not have exactly the same length as `cd`.
    pub fn mul_reduce(&mut self, m: u64, cd: &[u64], w: &[u64], result: &mut [u64]) -> Result<(), Error> {
        if cd.is_empty() || w.is_empty() || cd.len() > 256 || w.len() > 256 {
            return Err(Error::InvalidLength);
        }
        if cd.len() != w.len() {
            return Err(Error::LengthMismatch);
        }
        if result.len() != cd.len() {
            return Err(Error::BufferTooSmall);
        }
        let n = cd.len();

        self.write_dword(offset::AB, m)?;
        for i in 0..n {
            self.write_dword(offset::CD + i * 8, cd[i])?;
        }

        // Clear the RES workspace required by MUL64_REDUCE. The operation may access additional RES 64-bit words beyond the n explicitly initialized W 64-bit words.
        self.clear(offset::RES, 2 * n * 8)?;

        for i in 0..n {
            self.write_dword(offset::RES + i * 8, w[i])?;
        }
        self.execute_op_sync(Opcode::Mul64Reduce, (n - 1) as u8, offset::AB, offset::CD, offset::RES)?;
        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8)?;
        }
        Ok(())
    }
}
