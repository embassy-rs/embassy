//! CryptoCell PKA engine.
//!
//! The PKA is a big-integer coprocessor with a dedicated SRAM holding 32 registers. An
//! operation is one opcode word. It names the two operand registers, the result register,
//! and one of eight operand sizes from the sizes table. The engine runs it over the whole
//! register width.
//!
//! Modular operations use Barrett reduction. They need the modulus in register `N` and its
//! Barrett tag in register `NP`.
//!
//! The register file is virtual. A mapping table gives the SRAM address of each register.
//! The engine reserves registers 30 and 31 as its own temporaries.
//!
//! This follows the PKA layer of Arm's CryptoCell runtime library (`pka.c`, `pki.c`).

use core::sync::atomic::{AtomicBool, Ordering};

use crate::crypto::pka::Error;
use crate::pac;
use crate::pac::cc_pka::vals::{ConstA, ConstB, DiscardR, Opcode, PkaDoneStatus};

/// Width of a PKA word, the operand width of the hardware multiplier.
///
/// Both CryptoCell-310 and CryptoCell-312 have a 64x16 multiplier. Registers are a multiple
/// of this, and so are the Barrett tags.
const PKA_WORD_BITS: usize = 64;

/// A PKA word in 32-bit words.
const PKA_WORD_WORDS: usize = PKA_WORD_BITS / 32;
/// Extra bits every register carries, so unreduced intermediate results fit.
const EXTRA_BITS: usize = 8;
/// Usable size of the PKA SRAM, in 32-bit words.
///
/// This bounds how many registers of a given size an operation can use.
const SRAM_WORDS: usize = 4 * 1024 / 4;
/// Mapping table entry for a register that is not in use.
const ADDR_UNUSED: u32 = 0xFFC;

/// Register holding the modulus.
pub(super) const REG_N: u8 = 0;
/// Register holding the Barrett tag of the modulus.
pub(super) const REG_NP: u8 = 1;
/// Engine temporaries.
pub(super) const REG_T0: u8 = 30;
/// Engine temporaries.
pub(super) const REG_T1: u8 = 31;
/// Number of registers in the file.
pub(super) const REG_COUNT: usize = 32;

/// Sizes-table entry holding the exact modulus size.
pub(super) const LEN_N: u8 = 0;
/// Sizes-table entry holding the modulus size rounded up by one whole PKA word.
pub(super) const LEN_FULL: u8 = 1;
/// Sizes-table entry holding the size of the CRT primes.
pub(super) const LEN_PQ: u8 = 2;
/// Sizes-table entry holding the full register size.
pub(super) const LEN_MAX: u8 = 7;

fn wait_done() {
    while pac::CC_PKA.pka_done().read().status() != PkaDoneStatus::Completed {}
}

fn wait_pipe() {
    while !pac::CC_PKA.pka_pipe().read().status() {}
}

/// Number of 32-bit words in the operation size `bits`, rounded up by one whole PKA word.
const fn full_op_size_pka_words(bits: usize) -> usize {
    bits / PKA_WORD_BITS + (bits % PKA_WORD_BITS > 0) as usize + 1
}

/// Sets one entry of the sizes table.
pub(super) fn set_len(id: u8, bits: u32) {
    wait_done();
    pac::CC_PKA.pka_l(id as usize).write(|w| w.set_op_size(bits as u16));
}

/// Reads one entry of the sizes table.
pub(super) fn get_len(id: u8) -> u32 {
    wait_done();
    pac::CC_PKA.pka_l(id as usize).read().op_size() as u32
}

/// Sets the sizes-table entry `id` to the exact size and `id + 1` to the rounded-up size.
pub(super) fn set_len_pair(id: u8, bits: u32) {
    set_len(id, bits);
    set_len(id + 1, (PKA_WORD_BITS * full_op_size_pka_words(bits as usize)) as u32);
}

fn set_map(vreg: u8, addr: u32) {
    wait_done();
    pac::CC_PKA
        .memory_map(vreg as usize)
        .write_value(pac::cc_pka::regs::MemoryMap(addr));
}

fn get_map(vreg: u8) -> u32 {
    wait_done();
    pac::CC_PKA.memory_map(vreg as usize).read().0
}

/// Issues one operation.
#[allow(clippy::too_many_arguments)]
fn exec(op: Opcode, len: u8, a_const: bool, a: u8, b_const: bool, b: u8, discard: bool, res: u8, tag: u8) {
    let mut w = pac::cc_pka::regs::Opcode(0);
    w.set_opcode(op);
    w.set_len(len);
    w.set_const_a(if a_const { ConstA::Constant } else { ConstA::Register });
    w.set_reg_a(a);
    w.set_const_b(if b_const { ConstB::Constant } else { ConstB::Register });
    w.set_reg_b(b);
    w.set_discard_r(if discard { DiscardR::Discard } else { DiscardR::Register });
    w.set_reg_r(res);
    w.set_tag(tag);
    wait_pipe();
    pac::CC_PKA.opcode().write_value(w);
}

macro_rules! op_rr {
    ($(#[$m:meta])* $name:ident, $op:ident) => {
        $(#[$m])*
        pub(super) fn $name(len: u8, res: u8, a: u8, b: u8) {
            exec(Opcode::$op, len, false, a, false, b, false, res, 0);
        }
    };
}

macro_rules! op_ri {
    ($(#[$m:meta])* $name:ident, $op:ident) => {
        $(#[$m])*
        pub(super) fn $name(len: u8, res: u8, a: u8, imm: u8) {
            exec(Opcode::$op, len, false, a, true, imm, false, res, 0);
        }
    };
}

op_rr!(
    /// `res = a + b`
    add,
    AddInc
);
op_ri!(
    /// `res = a + imm`
    add_im,
    AddInc
);
op_rr!(
    /// `res = a - b`
    sub,
    SubDecNeg
);
op_ri!(
    /// `res = a - imm`
    sub_im,
    SubDecNeg
);
op_rr!(
    /// `res = (a + b) mod n`
    mod_add,
    ModAddInc
);
op_ri!(
    /// `res = (a + imm) mod n`
    mod_add_im,
    ModAddInc
);
op_rr!(
    /// `res = (a - b) mod n`
    mod_sub,
    ModSubDecNeg
);
op_ri!(
    /// `res = a & imm`
    and_im,
    Andtst0clr0
);
op_ri!(
    /// `res = a | imm`
    or_im,
    Orcopyset0
);
/// `res = a >> (shift + 1)`, shifting in zeros.
///
/// The shift count goes in the second operand field, without the immediate flag.
pub(super) fn shr0(len: u8, res: u8, a: u8, shift: u8) {
    exec(Opcode::Shr0, len, false, a, false, shift, false, res, 0);
}
op_rr!(
    /// `res = low half of a * b`
    mul_low,
    MulLow
);
op_rr!(
    /// `res = a * b mod n`
    mod_mul,
    ModMul
);
op_rr!(
    /// `res = a * b mod n`, leaving up to eight extra bits unreduced
    mod_mul_nfr,
    ModMulN
);
op_rr!(
    /// `res = a ^ b mod n`
    mod_exp,
    ModExp
);
op_rr!(
    /// `res = floor(a / b)`, replacing `a` with the remainder
    div,
    Division
);

/// `res = c + a * b mod n`, leaving up to eight extra bits unreduced.
pub(super) fn mod_mul_acc_nfr(len: u8, res: u8, a: u8, b: u8, c: u8) {
    exec(Opcode::ModMlacnr, len, false, a, false, b, false, res, c);
}

/// `res = a`
pub(super) fn copy(len: u8, res: u8, a: u8) {
    or_im(len, res, a, 0);
}

/// `res = a`, reducing it below the modulus.
pub(super) fn reduce(len: u8, res: u8, a: u8) {
    exec(Opcode::Reduction, len, false, a, false, 0, false, res, 0);
}

/// `res = 1 / b mod n`, for an odd modulus.
pub(super) fn mod_inv(len: u8, res: u8, b: u8) {
    exec(Opcode::ModInv, len, true, 1, false, b, false, res, 0);
}

/// Zeroes a register.
pub(super) fn clear(len: u8, reg: u8) {
    and_im(len, reg, reg, 0);
}

/// Zeroes a register including the bits above the operation size.
pub(super) fn clear2(len: u8, reg: u8) {
    clear(len, reg);
    clear(len, reg);
}

/// Sets a register to a small constant.
pub(super) fn set_value(reg: u8, value: u8) {
    and_im(LEN_FULL, reg, reg, 0);
    or_im(LEN_FULL, reg, reg, value);
}

/// Returns whether the two registers hold the same value.
pub(super) fn equal(len: u8, a: u8, b: u8) -> bool {
    exec(Opcode::Xorflp0invcmp, len, false, a, false, b, true, 0, 0);
    alu_out_zero()
}

/// Returns whether a register holds `imm`.
pub(super) fn equal_im(len: u8, a: u8, imm: u8) -> bool {
    exec(Opcode::Xorflp0invcmp, len, false, a, true, imm, true, 0, 0);
    alu_out_zero()
}

/// `res = 1 / a mod n` by exponentiation, for a prime modulus.
///
/// Unlike [`mod_inv`] this runs in a time that does not depend on `a`. `tmp` is overwritten.
pub(super) fn mod_inv_exp(res: u8, a: u8, tmp: u8) {
    sub_im(LEN_FULL, tmp, REG_N, 2);
    mod_exp(LEN_N, res, a, tmp);
}

fn alu_out_zero() -> bool {
    wait_done();
    pac::CC_PKA.pka_status().read().alu_out_zero()
}

/// Number of 32-bit words in one register.
pub(super) fn reg_words() -> usize {
    (get_len(LEN_MAX) as usize).div_ceil(32)
}

/// Writes a big-endian value into a register, zeroing the rest of it.
pub(super) fn write_be(vreg: u8, data: &[u8]) {
    let total = reg_words();
    let addr = get_map(vreg);
    wait_done();
    pac::CC_PKA.pka_sram_waddr().write_value(addr);
    let mut left = data.len();
    let mut written = 0;
    while left > 0 && written < total {
        let n = left.min(4);
        let mut w = [0u8; 4];
        w[4 - n..].copy_from_slice(&data[left - n..left]);
        pac::CC_PKA.pka_sram_wdata().write_value(u32::from_be_bytes(w));
        left -= n;
        written += 1;
    }
    for _ in written..total {
        pac::CC_PKA.pka_sram_wdata().write_value(0);
    }
}

/// Reads a register into a big-endian buffer.
pub(super) fn read_be(vreg: u8, out: &mut [u8]) {
    let addr = get_map(vreg);
    wait_done();
    pac::CC_PKA.pka_sram_raddr().write_value(addr);
    let mut left = out.len();
    for _ in 0..out.len().div_ceil(4) {
        let v = pac::CC_PKA.pka_sram_rdata().read().to_be_bytes();
        let n = left.min(4);
        out[left - n..left].copy_from_slice(&v[4 - n..]);
        left -= n;
    }
}

/// Reads one 32-bit word of a register.
pub(super) fn read_word(vreg: u8, index: usize) -> u32 {
    let addr = get_map(vreg);
    wait_done();
    pac::CC_PKA.pka_sram_raddr().write_value(addr + index as u32);
    pac::CC_PKA.pka_sram_rdata().read()
}

/// Writes one 32-bit word of a register. The other words of the same PKA word must be zero.
///
/// A write is only committed once the whole PKA word is written, so this rewrites the other
/// words too.
pub(super) fn write_word(vreg: u8, index: usize, value: u32) {
    let addr = get_map(vreg);
    let base = index - index % PKA_WORD_WORDS;
    wait_done();
    pac::CC_PKA.pka_sram_waddr().write_value(addr + base as u32);
    for i in 0..PKA_WORD_WORDS {
        pac::CC_PKA
            .pka_sram_wdata()
            .write_value(if base + i == index { value } else { 0 });
    }
}

/// Zeroes a block of registers, including the engine temporaries.
pub(super) fn clear_regs(first: u8, count: usize) {
    let words = reg_words();
    for i in 0..count as u8 {
        let addr = get_map(first + i);
        if addr == ADDR_UNUSED {
            continue;
        }
        wait_done();
        pac::CC_PKA.pka_sram_waddr().write_value(addr);
        for _ in 0..words {
            pac::CC_PKA.pka_sram_wdata().write_value(0);
        }
    }
}

/// Effective size in bits of the value in a register.
pub(super) fn effective_bits(vreg: u8) -> u32 {
    let words = reg_words();
    let addr = get_map(vreg);
    for i in (0..words).rev() {
        wait_done();
        pac::CC_PKA.pka_sram_raddr().write_value(addr + i as u32);
        let w = pac::CC_PKA.pka_sram_rdata().read();
        if w != 0 {
            return (i as u32) * 32 + (32 - w.leading_zeros());
        }
    }
    0
}

/// Prepares the engine for operations on values of `op_size_bits` bits.
///
/// Lays out the register mapping table and fills the sizes table. `regs_needed` counts the
/// registers the operation uses plus the two engine temporaries. Fails with `InvalidModulus`
/// if that many do not fit in the SRAM.
pub(super) fn init(op_size_bits: u32, regs_needed: usize) -> Result<(), Error> {
    // The engine holds all of the state of an operation, so two of them cannot be interleaved.
    // Every entry point of the driver goes through here, which turns what would otherwise be a
    // silently wrong result into a panic.
    if BUSY.swap(true, Ordering::Acquire) {
        panic!("the PKA engine is already in use");
    }
    match init_inner(op_size_bits, regs_needed) {
        Ok(()) => Ok(()),
        Err(e) => {
            BUSY.store(false, Ordering::Release);
            Err(e)
        }
    }
}

static BUSY: AtomicBool = AtomicBool::new(false);

fn init_inner(op_size_bits: u32, regs_needed: usize) -> Result<(), Error> {
    if op_size_bits < 32 {
        return Err(Error::InvalidModulus);
    }

    // Registers are one PKA word wider than the operation, so that unreduced results of the
    // modular operations fit. Small operations get a whole extra word on top.
    let op = op_size_bits as usize;
    let reg_words = if op < 2 * (PKA_WORD_BITS + EXTRA_BITS) {
        let bits = op + PKA_WORD_BITS + EXTRA_BITS - 1;
        bits.div_ceil(32) + (bits % 32 != 0) as usize
    } else {
        op.div_ceil(32)
    };
    let reg_pka_words = full_op_size_pka_words(reg_words * 32);
    let regs = (SRAM_WORDS / (reg_pka_words * PKA_WORD_WORDS)).min(REG_COUNT);
    if regs < regs_needed {
        return Err(Error::InvalidModulus);
    }

    // Polling the status registers is the only way this driver observes the engine.
    pac::CC_HOST_RGF
        .imr()
        .modify(|w| w.set_pka_mask(pac::cc_host_rgf::vals::PkaMask::IrqDisable));
    pac::CC_MISC.pka_clk().write(|w| w.set_enable(true));

    // Mapping table: registers laid out back to back, with the two engine temporaries last.
    let step = (reg_pka_words * PKA_WORD_WORDS) as u32;
    let mut addr = 0;
    for i in 0..(REG_COUNT as u8 - 2) {
        if (i as usize) < regs - 2 {
            set_map(i, addr);
            addr += step;
        } else {
            set_map(i, ADDR_UNUSED);
        }
    }
    set_map(REG_T0, addr);
    set_map(REG_T1, addr + step);
    set_n_np_t0_t1(REG_N, REG_NP, REG_T0, REG_T1);

    // Sizes table: the exact operation size, the size rounded up by a whole PKA word, then
    // the full register size in the remaining entries.
    set_len_pair(LEN_N, op_size_bits);
    for id in 2..8 {
        set_len(id, (reg_pka_words * PKA_WORD_BITS) as u32);
    }
    Ok(())
}

/// Points the engine at the registers holding the modulus, the Barrett tag and its two
/// temporaries.
pub(super) fn set_n_np_t0_t1(n: u8, np: u8, t0: u8, t1: u8) {
    wait_done();
    pac::CC_PKA.n_np_t0_t1_addr().write(|w| {
        w.set_n_virtual_addr(n);
        w.set_np_virtual_addr(np);
        w.set_t0_virtual_addr(t0);
        w.set_t1_virtual_addr(t1);
    });
}

/// Zeroes the registers an operation used and stops the engine.
pub(super) fn finish(regs_used: usize) {
    if regs_used > 0 {
        clear_regs(REG_N, regs_used.min(REG_COUNT - 2));
        clear_regs(REG_T0, 2);
    }
    wait_done();
    pac::CC_MISC.pka_clk().write(|w| w.set_enable(false));
    BUSY.store(false, Ordering::Release);
}

/// Computes the Barrett tag of the modulus in `reg_n` into `reg_np`.
///
/// The tag is `floor(2^(N + A + X - 1) / n)`, with `N` the modulus size, `A` the PKA word
/// size and `X` the extra bits. For a modulus larger than `2·(A + X)` bits the division runs
/// on a truncated modulus. That is what the engine expects.
pub(super) fn calc_np(size_bits: u32, reg_n: u8, reg_np: u8, tmp1: u8, tmp_n: u8) {
    const A: usize = PKA_WORD_BITS;
    const X: usize = EXTRA_BITS;
    let size = size_bits as usize;

    clear2(LEN_MAX, tmp1);
    clear2(LEN_MAX, tmp_n);
    clear2(LEN_MAX, reg_np);
    copy(LEN_MAX, tmp_n, reg_n);

    if size <= 2 * A + 2 * X {
        // Small modulus: divide the exact numerator by the whole modulus.
        let bits = size + A + X - 1;
        let (words, top) = if bits % 32 != 0 {
            (bits.div_ceil(32), 1u32 << (bits % 32))
        } else {
            (bits / 32 + 1, 1u32)
        };
        write_word(tmp1, words - 1, top);
        div(LEN_MAX, reg_np, tmp1, tmp_n);
    } else {
        let bits = 3 * A + 3 * X - 1;
        let (words, top) = if bits % 32 != 0 {
            (bits.div_ceil(32), 1u32 << (bits % 32))
        } else {
            (bits / 32 + 1, 1u32)
        };
        write_word(tmp1, words - 1, top);

        // Divide by the modulus truncated to its top 2·(A + X) bits, rounded up.
        let shift = size - 2 * A - 2 * X;
        sub_im(LEN_FULL, tmp_n, tmp_n, 1);
        for _ in 0..shift / 32 {
            shr0(LEN_FULL, tmp_n, tmp_n, 31);
        }
        if shift % 32 != 0 {
            shr0(LEN_FULL, tmp_n, tmp_n, (shift % 32 - 1) as u8);
        }
        add_im(LEN_FULL, tmp_n, tmp_n, 1);
        div(LEN_MAX, reg_np, tmp1, tmp_n);
    }

    clear2(LEN_MAX, tmp1);
    clear2(LEN_MAX, tmp_n);
}

/// Zeroes a register from word `from` up to its end.
pub(super) fn clear_from(vreg: u8, from: usize) {
    let words = reg_words();
    if from >= words {
        return;
    }
    let addr = get_map(vreg);
    wait_done();
    pac::CC_PKA.pka_sram_waddr().write_value(addr + from as u32);
    for _ in from..words {
        pac::CC_PKA.pka_sram_wdata().write_value(0);
    }
}
