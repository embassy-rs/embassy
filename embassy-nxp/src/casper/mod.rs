//! Cryptographic Accelerator and Signaling Processing Engine (CASPER)
//!
//! This module provides hardware acceleration for asymmetric cryptography (RSA).

use embassy_hal_internal::{Peri, impl_peripheral};

use crate::pac;

/// Base address of CASPER SRAMX dedicated memory (non-secure mode)
pub const SRAMX_BASE: usize = 0x0400_0000;
/// Bit position used by the SRAMX interleaved addressing scheme to select
/// between the two RAMX banks.
const CASPER_RAM_OFFSET: usize = 14;
/// Total size of SRAMX memory (8 KB)
pub const SRAMX_SIZE: usize = 0x2000;

/// Internal SRAMX layout used for high-level CASPER operations
pub(crate) mod offset {
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
        syscon
            .presetctrl2()
            .modify(|w| w.set_casper_rst(pac::syscon::vals::CasperRst::ASSERTED)); // Activate reset
        syscon
            .presetctrl2()
            .modify(|w| w.set_casper_rst(pac::syscon::vals::CasperRst::RELEASED)); // Release reset

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
    /// # Panics
    /// Panics if the write operation exceeds SRAMX boundaries (8 KB).
    /// Also, panics if the offset is not aligned to 4 bytes.
    pub fn write_word(&mut self, offset: usize, value: u32) {
        assert!(
            offset <= SRAMX_SIZE - 4,
            "CASPER SRAMX write_word overflow, offset {:#x} + 4B exceeds {:#x} (8KB)",
            offset,
            SRAMX_SIZE
        );
        assert!(
            offset % 4 == 0,
            "CASPER SRAMX write_word offset must be aligned to 4 bytes"
        );

        let ptr = Self::interleave(SRAMX_BASE + offset) as *mut u32;
        unsafe {
            core::ptr::write_volatile(ptr, value);
        }
    }

    /// Read a 32-bit value (word) from SRAMX memory at the specified offset.
    ///
    /// # Panics
    /// Panics if the read operation exceeds SRAMX boundaries (8 KB).
    /// Also, panics if the offset is not aligned to 4 bytes.
    pub fn read_word(&self, offset: usize) -> u32 {
        assert!(
            offset <= SRAMX_SIZE - 4,
            "CASPER SRAMX read_word overflow, offset {:#x} + 4B exceeds {:#x} (8KB)",
            offset,
            SRAMX_SIZE
        );
        assert!(
            offset % 4 == 0,
            "CASPER SRAMX read_word offset must be aligned to 4 bytes"
        );

        let ptr = Self::interleave(SRAMX_BASE + offset) as *const u32;
        unsafe { core::ptr::read_volatile(ptr) }
    }

    /// Write a 64-bit value (dword - double word) into SRAMX memory at the specified offset.
    ///
    /// # Panics
    /// Panics if the write operation exceeds SRAMX boundaries (8 KB).
    /// Also, panics if the offset is not aligned to 8 bytes.
    pub fn write_dword(&mut self, offset: usize, value: u64) {
        assert!(
            offset <= SRAMX_SIZE - 8,
            "CASPER SRAMX write_dword overflow, offset {:#x} + 8B exceeds {:#x} (8KB)",
            offset,
            SRAMX_SIZE
        );
        assert!(
            offset % 8 == 0,
            "CASPER SRAMX write_dword offset must be aligned to 8 bytes"
        );

        self.write_word(offset, value as u32);
        self.write_word(offset + 4, (value >> 32) as u32);
    }

    /// Read a 64-bit value (dword - double word) from SRAMX memory at the specified offset.
    ///
    /// # Panics
    /// Panics if the read operation exceeds SRAMX boundaries (8 KB).
    /// Also, panics if the offset is not aligned to 8 bytes.
    pub fn read_dword(&self, offset: usize) -> u64 {
        assert!(
            offset <= SRAMX_SIZE - 8,
            "CASPER SRAMX read_dword overflow, offset {:#x} + 8B exceeds {:#x} (8KB)",
            offset,
            SRAMX_SIZE
        );
        assert!(
            offset % 8 == 0,
            "CASPER SRAMX read_dword offset must be aligned to 8 bytes"
        );

        let low = self.read_word(offset) as u64;
        let high = self.read_word(offset + 4) as u64;

        low | (high << 32)
    }

    /// Zero-out a section of SRAMX memory.
    ///
    /// # Panics
    /// Panics if the clear operation exceeds SRAMX boundaries (8 KB).
    /// Also, panics if the offset or length is not aligned to 4 bytes.
    pub fn clear(&mut self, offset: usize, len: usize) {
        assert!(
            offset <= SRAMX_SIZE,
            "CASPER SRAMX clear overflow, offset {:#x} exceeds {:#x} (8KB)",
            offset,
            SRAMX_SIZE
        );
        assert!(
            len <= SRAMX_SIZE - offset,
            "CASPER SRAMX clear overflow, offset {:#x} + len ({}B) exceeds {:#x} (8KB)",
            offset,
            len,
            SRAMX_SIZE
        );
        assert!(len % 4 == 0, "clear length must be multiple of 4 bytes");
        assert!(offset % 4 == 0, "CASPER SRAMX clear offset must be aligned to 4 bytes");

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
        assert!(
            a_offset <= SRAMX_SIZE,
            "CASPER SRAMX execute_op_sync overflow: a_offset {:#x} exceeds {:#x} (8KB)",
            a_offset,
            SRAMX_SIZE
        );
        assert!(
            c_offset <= SRAMX_SIZE,
            "CASPER SRAMX execute_op_sync overflow: c_offset {:#x} exceeds {:#x} (8KB)",
            c_offset,
            SRAMX_SIZE
        );
        assert!(
            res_offset <= SRAMX_SIZE,
            "CASPER SRAMX execute_op_sync overflow: res_offset {:#x} exceeds {:#x} (8KB)",
            res_offset,
            SRAMX_SIZE
        );
        assert!(
            a_offset % 4 == 0,
            "CASPER execute_op_sync a_offset must be aligned to 4 bytes"
        );
        assert!(
            c_offset % 4 == 0,
            "CASPER execute_op_sync c_offset must be aligned to 4 bytes"
        );
        assert!(
            res_offset % 4 == 0,
            "CASPER execute_op_sync res_offset must be aligned to 4 bytes"
        );

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

    /// Returns the carry flag from the last CASPER operation.
    pub fn carry(&self) -> bool {
        pac::CASPER.status().read().carry()
    }

    /// Copy a sequence of 64-bit values from one SRAMX location to another.
    /// The slice `values` uses litte-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Panics
    /// Panics if the source or destination offsets plus the total size of values exceed SRAMX boundaries (8 KB), or if the source or destination offsets are not aligned to 8 bytes.
    /// Also, panics if the values slice is empty or contains more than 256 elements as CASPER supports a maximum of 255 iterations (256 values).
    pub fn copy_values(&mut self, src_offset: usize, dst_offset: usize, values: &[u64]) {
        assert!(
            !values.is_empty(),
            "CASPER COPY operation requires at least one value to be copied"
        );
        assert!(
            values.len() <= 256,
            "CASPER copy operation supports a maximum of 256 values"
        );

        let n: usize = values.len();
        assert!(
            src_offset <= SRAMX_SIZE,
            "CASPER SRAMX copy overflow, src_offset {:#x} exceeds {:#x} (8KB)",
            src_offset,
            SRAMX_SIZE
        );
        assert!(
            dst_offset <= SRAMX_SIZE,
            "CASPER SRAMX copy overflow, dst_offset {:#x} exceeds {:#x} (8KB)",
            dst_offset,
            SRAMX_SIZE
        );
        assert!(
            n * 8 <= SRAMX_SIZE - src_offset,
            "CASPER copy overflow, src_offset {:#x} + {} values ({}B) exceeds {:#x} (8KB)",
            src_offset,
            n,
            n * 8,
            SRAMX_SIZE
        );
        assert!(
            n * 8 <= SRAMX_SIZE - dst_offset,
            "CASPER copy overflow, dst_offset {:#x} + {} values ({}B) exceeds {:#x} (8KB)",
            dst_offset,
            n,
            n * 8,
            SRAMX_SIZE
        );
        assert!(
            src_offset % 8 == 0,
            "CASPER copy source offset must be aligned to 8 bytes"
        );
        assert!(
            dst_offset % 8 == 0,
            "CASPER copy destination offset must be aligned to 8 bytes"
        );

        for i in 0..n {
            self.write_dword(src_offset + i * 8, values[i]);
        }
        self.execute_op_sync(Opcode::Copy, (n - 1) as u8, src_offset, 0, dst_offset);
    }

    /// Zero-out a sequence of 64-bit values (dwords - double words) in SRAMX memory.
    ///
    /// # Panics
    /// Panics if the destination offset plus the total size of values to be zeroed exceeds SRAMX boundaries (8 KB), or if the destination offset is not aligned to 8 bytes.
    /// Also, panics if the number of `dwords` to be zeroed is zero or exceeds 256, as CASPER supports a maximum of 255 iterations (256 values).
    pub fn zero(&mut self, res_offset: usize, dwords: usize) {
        assert!(
            res_offset <= SRAMX_SIZE,
            "CASPER SRAMX zero overflow, res_offset {:#x} exceeds {:#x} (8KB)",
            res_offset,
            SRAMX_SIZE
        );
        assert!(
            dwords > 0,
            "CASPER ZERO operation requires at least one value to be zeroed"
        );
        assert!(dwords <= 256, "CASPER ZERO operation supports a maximum of 256 values");
        assert!(
            dwords * 8 <= SRAMX_SIZE - res_offset,
            "CASPER ZERO destination overflow, res_offset {:#x} + {} values ({}B) exceeds {:#x} (8KB)",
            res_offset,
            dwords,
            dwords * 8,
            SRAMX_SIZE
        );
        assert!(
            res_offset % 8 == 0,
            "CASPER ZERO destination offset must be aligned to 8 bytes"
        );

        self.execute_op_sync(Opcode::Zero, (dwords - 1) as u8, 0, 0, res_offset);
    }

    /// Perform a bitwise XOR operation on pairs of 64-bit values (dwords - double words) in SRAMX memory.
    /// The slices `operands` and `result` use litte-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Panics
    /// Panics if the result buffer is not large enough to hold the results of the XOR operation.
    /// Also, panics if the operands slice is empty or contains more than 256 pairs of values, as CASPER supports a maximum of 255 iterations (256 pairs).
    pub fn xor(&mut self, operands: &[(u64, u64)], result: &mut [u64]) {
        let n = operands.len();
        assert!(
            !operands.is_empty(),
            "CASPER XOR operation requires at least one pair of operands"
        );
        assert!(
            n <= 256,
            "CASPER XOR operation supports a maximum of 256 pairs of operands"
        );
        assert!(
            result.len() >= n,
            "CASPER XOR operation requires a result buffer of at least {} elements",
            n
        );

        for i in 0..n {
            let (r, a) = operands[i];
            self.write_dword(offset::AB + i * 8, a);
            self.write_dword(offset::RES + i * 8, r);
        }
        self.execute_op_sync(Opcode::Xor64, (n - 1) as u8, offset::AB, 0, offset::RES);

        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8);
        }
    }

    /// Perform a doubling operation on a sequence of 64-bit values (dwords - double words) in SRAMX memory.
    /// The slices `values` and `result` use litte-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Panics
    /// Panics if the result buffer is not large enough to hold the results of the doubling operation.
    /// Also, panics if the values slice is empty or contains more than 256 values, as CASPER supports a maximum of 255 iterations (256 values).
    pub fn double(&mut self, values: &[u64], result: &mut [u64]) -> bool {
        let n = values.len();
        assert!(
            !values.is_empty(),
            "CASPER DOUBLE operation requires at least one value"
        );
        assert!(n <= 256, "CASPER DOUBLE operation supports a maximum of 256 values");
        assert!(
            result.len() >= n,
            "CASPER DOUBLE operation requires a result buffer of at least {} elements",
            n
        );

        for i in 0..n {
            self.write_dword(offset::RES + i * 8, values[i]);
        }
        self.execute_op_sync(Opcode::Double64, (n - 1) as u8, 0, 0, offset::RES);

        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8);
        }
        self.carry()
    }

    /// Perform an addition operation on pairs of 64-bit values (dwords - double words) in SRAMX memory.
    /// The slices `operands` and `result` use litte-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Panics
    /// Panics if the result buffer is not large enough to hold the results of the addition operation.
    /// Also, panics if the operands slice is empty or contains more than 256 pairs of values, as CASPER supports a maximum of 255 iterations (256 pairs).
    pub fn add(&mut self, operands: &[(u64, u64)], result: &mut [u64]) -> bool {
        let n = operands.len();
        assert!(
            !operands.is_empty(),
            "CASPER ADD operation requires at least one pair of operands"
        );
        assert!(
            n <= 256,
            "CASPER ADD operation supports a maximum of 256 pairs of operands"
        );
        assert!(
            result.len() >= n,
            "CASPER ADD operation requires a result buffer of at least {} elements",
            n
        );

        for i in 0..n {
            let (r, a) = operands[i];
            self.write_dword(offset::AB + i * 8, a);
            self.write_dword(offset::RES + i * 8, r);
        }
        self.execute_op_sync(Opcode::Add64, (n - 1) as u8, offset::AB, 0, offset::RES);

        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8);
        }
        self.carry()
    }

    /// Perform a subtraction operation on pairs of 64-bit values (dwords - double words) in SRAMX memory.
    /// CASPER supports subtraction with borrow, and the carry flag indicates whether a borrow occurred during the operation.
    /// Uses forward subtraction (R - A) where R is the minuend and A is the subtrahend. R is the first operand and A is the second operand in each pair.
    /// The slices `operands` and `result` use litte-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Panics
    /// Panics if the result buffer is not large enough to hold the results of the subtraction operation.
    /// Also, panics if the operands slice is empty or contains more than 256 pairs of values, as CASPER supports a maximum of 255 iterations (256 pairs).
    pub fn sub(&mut self, operands: &[(u64, u64)], result: &mut [u64]) -> bool {
        let n = operands.len();
        assert!(
            !operands.is_empty(),
            "CASPER SUB operation requires at least one pair of operands"
        );
        assert!(
            n <= 256,
            "CASPER SUB operation supports a maximum of 256 pairs of operands"
        );
        assert!(
            result.len() >= n,
            "CASPER SUB operation requires a result buffer of at least {} elements",
            n
        );

        for i in 0..n {
            let (r, a) = operands[i];
            self.write_dword(offset::AB + i * 8, a);
            self.write_dword(offset::RES + i * 8, r);
        }
        self.execute_op_sync(Opcode::Sub64, (n - 1) as u8, offset::AB, 0, offset::RES);

        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8);
        }
        self.carry()
    }

    /// Perform a reverse subtraction operation on pairs of 64-bit values (dwords - double words) in SRAMX memory.
    /// CASPER supports subtraction with borrow, and the carry flag indicates whether a borrow occurred during the operation.
    /// Uses reverse subtraction (A - R) where A is the minuend and R is the subtrahend. A is the first operand and R is the second operand in each pair.
    /// The slices `operands` and `result` use litte-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Panics
    /// Panics if the result buffer is not large enough to hold the results of the reverse subtraction operation.
    /// Also, panics if the operands slice is empty or contains more than 256 pairs of values, as CASPER supports a maximum of 255 iterations (256 pairs).
    pub fn rsub(&mut self, operands: &[(u64, u64)], result: &mut [u64]) -> bool {
        let n = operands.len();
        assert!(
            !operands.is_empty(),
            "CASPER RSUB operation requires at least one pair of operands"
        );
        assert!(
            n <= 256,
            "CASPER RSUB operation supports a maximum of 256 pairs of operands"
        );
        assert!(
            result.len() >= n,
            "CASPER RSUB operation requires a result buffer of at least {} elements",
            n
        );

        for i in 0..n {
            let (a, r) = operands[i];
            self.write_dword(offset::AB + i * 8, a);
            self.write_dword(offset::RES + i * 8, r);
        }
        self.execute_op_sync(Opcode::Rsub64, (n - 1) as u8, offset::AB, 0, offset::RES);

        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8);
        }
        self.carry()
    }

    /// Perform a 64-bit multiplication without accumulating the result into the existing RES contents.
    /// The 64-bit value `ab` is multiplied by the sequence of 64-bit values in `cd`.
    /// The resulting multi-word value is written to RES and returned through `result`.
    /// The slices `cd` and `result` use litte-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Panics
    /// Panics if the `cd` slice is empty or contains more than 256 elements, or if the result buffer is not large enough to hold the `cd.len() + 1` output values.
    pub fn mul_nosum(&mut self, ab: u64, cd: &[u64], result: &mut [u64]) {
        let n = cd.len();
        assert!(
            !cd.is_empty(),
            "CASPER MUL_NOSUM operation requires at least one CD operand"
        );
        assert!(
            n <= 256,
            "CASPER MUL_NOSUM operation supports a maximum of 256 CD operands"
        );
        assert!(
            result.len() > n,
            "CASPER MUL_NOSUM operation requires a result buffer of at least {} elements",
            n + 1
        );

        self.write_dword(offset::AB, ab);
        for i in 0..n {
            self.write_dword(offset::CD + i * 8, cd[i]);
        }
        self.execute_op_sync(Opcode::Mul64Nosum, (n - 1) as u8, offset::AB, offset::CD, offset::RES);
        for i in 0..=n {
            result[i] = self.read_dword(offset::RES + i * 8);
        }
    }

    /// Multiply a 64-bit value by a sequence of 64-bit values and accumulate each product into the existing RES values.
    /// The operation performs the CASPER MUL64_SUM operation, which reads the existing RES contents,
    /// adds the corresponding product, and writes the accumulated result back to RES.
    /// The `w` slice provides the initial RES values. It may contain more elements than `cd`; only the RES words reached by the CASPER operation are modified.
    /// The slices `cd`, `w` and `result` use litte-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Panics
    /// Panics if `cd` or `w` is empty, if `cd` contains more elements than `w`,
    /// if either slice contains more than 256 elements, or if the result is not large enough to hold the output.
    pub fn mul_sum(&mut self, ab: u64, cd: &[u64], w: &[u64], result: &mut [u64]) {
        assert!(
            !cd.is_empty(),
            "CASPER MUL_SUM operation requires at least one CD operand"
        );
        assert!(
            !w.is_empty(),
            "CASPER MUL_SUM operation requires at least one W operand"
        );
        assert!(
            cd.len() <= w.len(),
            "CASPER MUL_SUM operation requires CD to have no more elements than W"
        );
        assert!(
            cd.len() <= 256 && w.len() <= 256,
            "CASPER MUL_SUM operation supports a maximum of 256 CD and 256 W operands"
        );
        let n = cd.len();
        assert!(
            result.len() > n,
            "CASPER MUL_SUM operation requires a result buffer of at least {} elements",
            n + 1
        );

        self.write_dword(offset::AB, ab);
        for i in 0..n {
            self.write_dword(offset::CD + i * 8, cd[i]);
        }
        for i in 0..w.len() {
            self.write_dword(offset::RES + i * 8, w[i]);
        }
        self.execute_op_sync(Opcode::Mul64Sum, (n - 1) as u8, offset::AB, offset::CD, offset::RES);
        for i in 0..=n {
            result[i] = self.read_dword(offset::RES + i * 8);
        }
    }

    /// Multiply a 64-bit value by a sequence of 64-bit values and accumulate the products into the existing RES values, including the most significant RES words.
    /// This operation performs the CASPER MUL64_FULLSUM operation, which reads the existing RES contents,
    /// adds the corresponding products, and propagates the carry through the full result.
    /// The `w` slice provides the initial RES values.
    /// The slices `cd`, `w` and `result` use litte-endian word order: the least significant 64-bit word is at index 0.
    /// Returns the carry flag reported by the last CASPER operation.
    ///
    /// # Panics
    /// Panics if `cd` or `w` is empty, if `cd` contains more elements than `w`,
    /// if either slice contains more than 256 elements, or if the result buffer is not large enough to hold the output.
    pub fn mul_fullsum(&mut self, ab: u64, cd: &[u64], w: &[u64], result: &mut [u64]) -> bool {
        assert!(
            !cd.is_empty(),
            "CASPER MUL_FULLSUM operation requires at least one CD operand"
        );
        assert!(
            !w.is_empty(),
            "CASPER MUL_FULLSUM operation requires at least one W operand"
        );
        assert!(
            cd.len() <= w.len(),
            "CASPER MUL_FULLSUM operation requires CD to have no more elements than W"
        );
        assert!(
            cd.len() <= 256 && w.len() <= 256,
            "CASPER MUL_FULLSUM operation supports a maximum of 256 CD and 256 W operands"
        );
        let n = cd.len();
        assert!(
            result.len() > n,
            "CASPER MUL_FULLSUM operation requires a result buffer of at least {} elements",
            n + 1
        );

        self.write_dword(offset::AB, ab);
        for i in 0..n {
            self.write_dword(offset::CD + i * 8, cd[i]);
        }
        for i in 0..w.len() {
            self.write_dword(offset::RES + i * 8, w[i]);
        }
        self.execute_op_sync(Opcode::Mul64Fullsum, (n - 1) as u8, offset::AB, offset::CD, offset::RES);
        for i in 0..=n {
            result[i] = self.read_dword(offset::RES + i * 8);
        }
        self.carry()
    }

    /// Perform the CASPER MUL64_REDUCE operation, which is used as a step in Montgomery reduction algorithms.
    /// The 64-bit value `m` is multiplied by the sequence of 64-bit values in `cd` and accumulated into the existing RES values.
    /// The first RES write is skipped and the resulting value is shifted by one 64-bit word, as required by the CASPER reduction operation.
    /// The `w` slice provides the initial RES values and must contain the same number of elements as `cd`.
    /// The `m` value is expected to be the precomputed Montgomery reduction factor; CASPER does not calculate this value itself.
    /// The RES workspace is cleared before the operation to prevent stale SRAMX contents from affecting the reduction.
    /// The slices `cd`, `w` and `result` use litte-endian word order: the least significant 64-bit word is at index 0.
    ///
    /// # Panics
    /// Panics if `cd` or `w` is empty, if `cd` and `w` have different lengths,
    /// if either slice contains more than 256 elements, or if the result buffer does not have exactly the same length as `cd`.
    pub fn mul_reduce(&mut self, m: u64, cd: &[u64], w: &[u64], result: &mut [u64]) {
        assert!(
            !cd.is_empty(),
            "CASPER MUL_REDUCE operation requires at least one CD operand"
        );
        assert!(
            !w.is_empty(),
            "CASPER MUL_REDUCE operation requires at least one W operand"
        );
        assert!(
            cd.len() <= 256 && w.len() <= 256,
            "CASPER MUL_REDUCE operation supports a maximum of 256 CD and 256 W operands"
        );
        assert!(
            cd.len() == w.len(),
            "CASPER MUL_REDUCE operation requires CD and W to have the same length"
        );
        let n = cd.len();
        assert!(
            result.len() == n,
            "CASPER REDUCE operation requires a result buffer of exactly {} elements",
            n
        );

        self.write_dword(offset::AB, m);
        for i in 0..n {
            self.write_dword(offset::CD + i * 8, cd[i]);
        }

        // Clear the RES workspace required by MUL64_REDUCE. The operation may access additional RES 64-bit words beyond the n explicitly initialized W 64-bit words.
        self.clear(offset::RES, 2 * n * 8);

        for i in 0..n {
            self.write_dword(offset::RES + i * 8, w[i]);
        }
        self.execute_op_sync(Opcode::Mul64Reduce, (n - 1) as u8, offset::AB, offset::CD, offset::RES);
        for i in 0..n {
            result[i] = self.read_dword(offset::RES + i * 8);
        }
    }
}
