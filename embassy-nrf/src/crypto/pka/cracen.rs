//! PKA driver for the CRACEN BA414EP public key engine.
//!
//! Unlike the CryptoCell PKA, this engine runs whole public key operations in hardware.
//! Software writes the operands into a dedicated crypto RAM, names an operation, and reads
//! the result back.
//!
//! The crypto RAM is divided into fixed-size slots. Every operation has a fixed assignment of
//! operands to slots. Everything here runs in big-endian mode, where an operand is
//! right-aligned at the end of its slot.
//!
//! Operation codes, slot assignments and status codes follow the `silexpk` layer of Nordic's
//! `nrf_security`.

use core::ptr;

use super::{Curve, Error, Predefined};
use crate::pac;

/// Offset of the crypto RAM within the CRACEN core.
const CRYPTORAM_OFFSET: usize = 0x8000;
/// Offset of the microcode RAM within the CRACEN core.
const MICROCODE_OFFSET: usize = 0xC000;
/// Size of the microcode RAM.
pub(super) const MICROCODE_WORDS: usize = 5120 / 4;

// Operation codes.
const OP_MODMULT: u8 = 0x03;
const OP_MODINV: u8 = 0x06;
const OP_MODEXP: u8 = 0x10;
const OP_RSA_CRT: u8 = 0x13;
const OP_ECC_PTMUL: u8 = 0x22;
const OP_ECC_PTADD: u8 = 0x21;
const OP_ECC_PTONCURVE: u8 = 0x26;
const OP_ECDSA_SIGN: u8 = 0x30;
const OP_ECDSA_VERIFY: u8 = 0x31;

// Slots. The first six hold the curve parameters of an elliptic curve operation.
const SLOT_CURVE_P: u8 = 0;
const SLOT_PTR_A: u8 = 6;
const SLOT_PTR_B: u8 = 8;
const SLOT_PTR_C: u8 = 10;
const SLOT_PTR_AA: u8 = 12;
/// Holds the random factor the engine blinds an operation with.
const SLOT_BLIND: u8 = 15;
/// Width of the blinding factor.
const BLIND_LEN: usize = 8;

fn core() -> pac::cracencore::Cracencore {
    pac::CRACENCORE
}

fn pk() -> pac::cracencore::Pk {
    core().pk()
}

fn cryptoram() -> usize {
    core().as_ptr() as usize + CRYPTORAM_OFFSET
}

/// Size of a crypto RAM slot, which the hardware reports through its maximum operand size.
fn slot_size() -> usize {
    if pk().hwconfig().read().maxopsize() > 0x200 {
        0x400
    } else {
        0x200
    }
}

/// Largest operand the engine accepts.
fn max_op_size() -> usize {
    pk().hwconfig().read().maxopsize() as usize
}

/// Keeps CRACEN powered while the driver exists.
pub(super) struct Handle {
    _activation: crate::crypto::ActivationHandle,
}

impl Handle {
    /// Powers CRACEN up and loads the microcode into the engine.
    ///
    /// The microcode RAM is lost whenever CRACEN powers down. This handle keeps it up.
    pub(super) fn new(microcode: &[u32]) -> Self {
        let activation = crate::crypto::activate();
        let base = core().as_ptr() as usize + MICROCODE_OFFSET;
        for (i, &word) in microcode.iter().enumerate() {
            unsafe { ptr::write_volatile((base + i * 4) as *mut u32, word) };
        }
        Self {
            _activation: activation,
        }
    }
}

/// Writes a big-endian value right-aligned in the operand window of a slot. The rest of the
/// window is zeroed.
///
/// The crypto RAM only accepts whole aligned words. Keeping the writes aligned may zero a
/// few bytes below the operand window, which belong to no operand.
fn write_slot(slot: u8, op_size: usize, value: &[u8]) {
    let end = cryptoram() + (slot as usize + 1) * slot_size();
    let value_start = end - value.len();
    let mut addr = (end - op_size) & !3;
    while addr < end {
        let mut w = [0u8; 4];
        for (i, b) in w.iter_mut().enumerate() {
            let a = addr + i;
            if a >= value_start {
                *b = value[a - value_start];
            }
        }
        unsafe { ptr::write_volatile(addr as *mut u32, u32::from_le_bytes(w)) };
        addr += 4;
    }
}

/// Reads a big-endian value from the end of a slot.
fn read_slot(slot: u8, out: &mut [u8]) {
    let end = cryptoram() + (slot as usize + 1) * slot_size();
    let start = end - out.len();
    let mut addr = start & !3;
    while addr < end {
        let w = unsafe { ptr::read_volatile(addr as *const u32) }.to_le_bytes();
        for (i, &b) in w.iter().enumerate() {
            let a = addr + i;
            if a >= start {
                out[a - start] = b;
            }
        }
        addr += 4;
    }
}

/// An operation to run on the engine.
struct Op {
    code: u8,
    /// Size of every operand in bytes.
    op_size: usize,
    /// Recompute the Montgomery constant, needed whenever the modulus changes.
    calc_r2: bool,
    /// Curve whose parameters the hardware holds itself.
    predefined: Option<Predefined>,
    /// Slot the first operand pointer must be aimed at.
    ptr_a: u8,
    /// Countermeasures against power analysis. They need a random factor in [`SLOT_BLIND`].
    blind: Blind,
}

/// How the engine should randomize an operation.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Blind {
    /// No randomization, for operations that only touch public values.
    None,
    /// Randomize the scalar and the projective coordinates of a curve operation.
    Ecc,
    /// Randomize the modulus of a modular exponentiation.
    Modulus,
}

impl Op {
    fn new(code: u8, op_size: usize) -> Self {
        Self {
            code,
            op_size,
            calc_r2: false,
            predefined: None,
            ptr_a: SLOT_PTR_A,
            blind: Blind::None,
        }
    }

    /// Selects the operation. Operands are written after this and before [`Self::run`].
    fn start(&self) -> Result<(), Error> {
        if self.op_size == 0 || self.op_size > max_op_size() {
            return Err(Error::InvalidLength);
        }
        wait_idle();
        pk().command().write(|w| {
            w.set_opeaddr(self.code);
            w.set_opbytesm1(self.op_size as u16 - 1);
            // Every operation here takes its operands big-endian.
            w.set_swapbytes(pac::cracencore::vals::Swapbytes::Swapped);
            if self.calc_r2 {
                w.set_calcr2(pac::cracencore::vals::Calcr2::Recalculate);
            }
            match self.blind {
                Blind::None => {}
                Blind::Ecc => {
                    w.set_randke(true);
                    w.set_randproj(true);
                }
                Blind::Modulus => w.set_randmod(true),
            }
            if let Some(curve) = self.predefined {
                use pac::cracencore::vals::Selcurve;
                w.set_selcurve(match curve {
                    Predefined::P192 => Selcurve::P192,
                    Predefined::P256 => Selcurve::P256,
                    Predefined::P384 => Selcurve::P384,
                    Predefined::P521 => Selcurve::P521,
                });
            }
        });
        wait_idle();
        if self.blind != Blind::None {
            self.write_blinding_factor();
        }
        Ok(())
    }

    /// Draws the random factor for the countermeasures.
    ///
    /// The forced bits make the factor odd and of fixed width, so the engine's running time
    /// does not depend on it.
    fn write_blinding_factor(&self) {
        let mut words = [0u32; BLIND_LEN / 4];
        crate::crypto::cracen::random_words(&mut words);
        let mut factor = [0u8; BLIND_LEN];
        for (chunk, word) in factor.chunks_exact_mut(4).zip(words) {
            chunk.copy_from_slice(&word.to_be_bytes());
        }
        factor[0] = (factor[0] & 0x3f) | 0x20;
        factor[BLIND_LEN - 1] |= 1;
        write_slot(SLOT_BLIND, self.op_size, &factor);
    }
    fn run(&self) -> Result<(), Error> {
        pk().pointers().write(|w| {
            w.set_opptra(self.ptr_a);
            w.set_opptrb(SLOT_PTR_B);
            w.set_opptrc(SLOT_PTR_C);
        });
        pk().control().write(|w| {
            w.set_start(true);
            w.set_clearirq(true);
        });
        wait_idle();
        status()
    }
}

fn wait_idle() {
    while pk().status().read().pkbusy() {}
}

/// Translates the error flags of the engine.
fn status() -> Result<(), Error> {
    let flags = pk().status().read().errorflags();
    if flags == 0 {
        return Ok(());
    }
    Err(match flags.trailing_zeros() {
        // Point not on the curve, or not a quadratic residue while decompressing one.
        0 | 9 => Error::InvalidPoint,
        // Operand out of range, or a point of the wrong order.
        2 | 10 => Error::InvalidScalar,
        3 => Error::InvalidModulus,
        5 => Error::InvalidSignature,
        7 => Error::NotInvertible,
        _ => Error::Hardware,
    })
}

/// Writes the curve parameters into the slots the engine reads them from, unless the hardware
/// holds them itself.
fn write_curve(curve: &Curve) {
    if curve.predefined.is_some() {
        return;
    }
    let n = curve.size();
    for (i, param) in [curve.p, curve.n, curve.gx, curve.gy, curve.a, curve.b]
        .iter()
        .enumerate()
    {
        write_slot(SLOT_CURVE_P + i as u8, n, param);
    }
}

fn ecc_op(code: u8, curve: &Curve, ptr_a: u8, blind: Blind) -> Op {
    Op {
        code,
        op_size: curve.size(),
        calc_r2: false,
        predefined: curve.predefined,
        ptr_a,
        blind,
    }
}

/// Multiplies a point by a scalar.
///
/// `blind` enables the engine's countermeasures against power analysis, for a secret scalar.
/// Always returns `Ok(true)`, since the engine handles every scalar.
pub(super) fn ecc_mul(
    curve: &Curve,
    scalar: &[u8],
    px: &[u8],
    py: &[u8],
    rx: &mut [u8],
    ry: &mut [u8],
    blind: bool,
) -> Result<bool, Error> {
    const SLOT_K: u8 = SLOT_PTR_B;
    const SLOT_PX: u8 = SLOT_PTR_AA;
    const SLOT_RX: u8 = SLOT_PTR_C;

    let blind = if blind { Blind::Ecc } else { Blind::None };
    let op = ecc_op(OP_ECC_PTMUL, curve, SLOT_PX, blind);
    op.start()?;
    write_curve(curve);
    write_slot(SLOT_K, op.op_size, scalar);
    write_slot(SLOT_PX, op.op_size, px);
    write_slot(SLOT_PX + 1, op.op_size, py);
    op.run()?;
    read_slot(SLOT_RX, rx);
    read_slot(SLOT_RX + 1, ry);
    Ok(true)
}

/// Adds two affine points that are neither equal nor opposite.
pub(super) fn ecc_add(
    curve: &Curve,
    px: &[u8],
    py: &[u8],
    qx: &[u8],
    qy: &[u8],
    rx: &mut [u8],
    ry: &mut [u8],
) -> Result<(), Error> {
    const SLOT_PX: u8 = SLOT_PTR_A;
    const SLOT_QX: u8 = SLOT_PTR_B;
    const SLOT_RX: u8 = SLOT_PTR_C;

    let op = ecc_op(OP_ECC_PTADD, curve, SLOT_PX, Blind::None);
    op.start()?;
    write_curve(curve);
    write_slot(SLOT_PX, op.op_size, px);
    write_slot(SLOT_PX + 1, op.op_size, py);
    write_slot(SLOT_QX, op.op_size, qx);
    write_slot(SLOT_QX + 1, op.op_size, qy);
    op.run()?;
    read_slot(SLOT_RX, rx);
    read_slot(SLOT_RX + 1, ry);
    Ok(())
}

/// Doubles an affine point, as a blinded multiplication by two.
pub(super) fn ecc_double(curve: &Curve, px: &[u8], py: &[u8], rx: &mut [u8], ry: &mut [u8]) -> Result<(), Error> {
    let mut two = [0u8; super::MAX_CURVE_LEN];
    let two = &mut two[..curve.size()];
    two[curve.size() - 1] = 2;
    ecc_mul(curve, two, px, py, rx, ry, true).map(|_| ())
}

/// Checks that a point is on the curve.
pub(super) fn point_check(curve: &Curve, x: &[u8], y: &[u8]) -> Result<(), Error> {
    const SLOT_PX: u8 = SLOT_PTR_AA;

    let op = ecc_op(OP_ECC_PTONCURVE, curve, SLOT_PX, Blind::None);
    op.start()?;
    write_curve(curve);
    write_slot(SLOT_PX, op.op_size, x);
    write_slot(SLOT_PX + 1, op.op_size, y);
    op.run()
}

/// Signs a hash. `hash` is already reduced to the size of the curve.
pub(super) fn ecdsa_sign(
    curve: &Curve,
    private_key: &[u8],
    k: &[u8],
    hash: &[u8],
    out_r: &mut [u8],
    out_s: &mut [u8],
) -> Result<(), Error> {
    const SLOT_D: u8 = 6;
    const SLOT_K: u8 = 7;
    const SLOT_R: u8 = 10;
    const SLOT_S: u8 = 11;
    const SLOT_H: u8 = 12;

    let op = ecc_op(OP_ECDSA_SIGN, curve, 0, Blind::Ecc);
    op.start()?;
    write_curve(curve);
    write_slot(SLOT_D, op.op_size, private_key);
    write_slot(SLOT_K, op.op_size, k);
    write_slot(SLOT_H, op.op_size, hash);
    match op.run() {
        Ok(()) => {}
        // The ephemeral key gave a signature component of zero.
        Err(Error::NotInvertible) => return Err(Error::RetryWithNewK),
        Err(e) => return Err(e),
    }
    read_slot(SLOT_R, out_r);
    read_slot(SLOT_S, out_s);
    Ok(())
}

/// Verifies a signature. `hash` is already reduced to the size of the curve.
///
/// The engine fails with "not invertible" when an intermediate sum of `u1*G + u2*Q` is the
/// point at infinity, which a public key equal to the generator or a crafted signature can
/// cause. `u1` and `u2` are computed then and `Ok(false)` returned, for the caller to finish
/// the verification another way.
#[allow(clippy::too_many_arguments)]
pub(super) fn ecdsa_verify(
    curve: &Curve,
    qx: &[u8],
    qy: &[u8],
    r: &[u8],
    s: &[u8],
    hash: &[u8],
    u1: &mut [u8],
    u2: &mut [u8],
) -> Result<bool, Error> {
    const SLOT_QX: u8 = 8;
    const SLOT_QY: u8 = 9;
    const SLOT_R: u8 = 10;
    const SLOT_S: u8 = 11;
    const SLOT_H: u8 = 12;

    let op = ecc_op(OP_ECDSA_VERIFY, curve, 0, Blind::None);
    op.start()?;
    write_curve(curve);
    write_slot(SLOT_QX, op.op_size, qx);
    write_slot(SLOT_QY, op.op_size, qy);
    write_slot(SLOT_R, op.op_size, r);
    write_slot(SLOT_S, op.op_size, s);
    write_slot(SLOT_H, op.op_size, hash);
    match op.run() {
        Ok(()) => return Ok(true),
        Err(Error::NotInvertible) => {}
        Err(e) => return Err(e),
    }

    // u1 = hash / s, u2 = r / s (mod n). The hash is below 2^bits(n) but may exceed n.
    let n = curve.size();
    let mut h = [0u8; super::MAX_CURVE_LEN];
    let h = &mut h[..n];
    if super::less_than(hash, curve.n) {
        h.copy_from_slice(hash);
    } else {
        super::sub_be(hash, curve.n, h);
    }
    let mut s_inv = [0u8; super::MAX_CURVE_LEN];
    let s_inv = &mut s_inv[..n];
    mod_op(OP_MODINV, curve.n, &[], s, s_inv)?;
    mod_op(OP_MODMULT, curve.n, h, s_inv, u1)?;
    mod_op(OP_MODMULT, curve.n, r, s_inv, u2)?;
    Ok(false)
}

/// Runs a primitive modular operation: `out = f(a, b) mod modulus`, with `a` unused by the
/// operations of one operand.
fn mod_op(code: u8, modulus: &[u8], a: &[u8], b: &[u8], out: &mut [u8]) -> Result<(), Error> {
    const SLOT_M: u8 = 0;

    let mut op = Op::new(code, modulus.len());
    op.calc_r2 = true;
    op.start()?;
    write_slot(SLOT_M, op.op_size, modulus);
    if !a.is_empty() {
        write_slot(SLOT_PTR_A, op.op_size, a);
    }
    write_slot(SLOT_PTR_B, op.op_size, b);
    op.run()?;
    read_slot(SLOT_PTR_C, out);
    Ok(())
}

/// Computes `base ^ exponent mod modulus`.
pub(super) fn mod_exp(base: &[u8], exponent: &[u8], modulus: &[u8], out: &mut [u8]) -> Result<(), Error> {
    const SLOT_M: u8 = 0;
    const SLOT_IN: u8 = SLOT_PTR_A;
    const SLOT_EXP: u8 = SLOT_PTR_B;
    const SLOT_OUT: u8 = SLOT_PTR_C;

    let mut op = Op::new(OP_MODEXP, modulus.len());
    op.calc_r2 = true;
    op.blind = Blind::Modulus;
    op.start()?;
    write_slot(SLOT_M, op.op_size, modulus);
    write_slot(SLOT_IN, op.op_size, base);
    write_slot(SLOT_EXP, op.op_size, exponent);
    op.run()?;
    read_slot(SLOT_OUT, out);
    Ok(())
}

/// Computes `input ^ d mod p·q` from the Chinese remainder theorem parameters.
#[allow(clippy::too_many_arguments)]
pub(super) fn rsa_crt(
    input: &[u8],
    p: &[u8],
    q: &[u8],
    dp: &[u8],
    dq: &[u8],
    qinv: &[u8],
    out: &mut [u8],
) -> Result<(), Error> {
    const SLOT_P: u8 = 2;
    const SLOT_Q: u8 = 3;
    const SLOT_IN: u8 = 4;
    const SLOT_OUT: u8 = 5;
    const SLOT_DP: u8 = 10;
    const SLOT_DQ: u8 = 11;
    const SLOT_QINV: u8 = 12;

    // The operation runs at the size of the modulus, with the half-size parameters
    // right-aligned in their slots.
    let op = Op::new(OP_RSA_CRT, input.len());
    op.start()?;
    write_slot(SLOT_IN, op.op_size, input);
    write_slot(SLOT_P, op.op_size, p);
    write_slot(SLOT_Q, op.op_size, q);
    write_slot(SLOT_DP, op.op_size, dp);
    write_slot(SLOT_DQ, op.op_size, dq);
    write_slot(SLOT_QINV, op.op_size, qinv);
    op.run()?;
    read_slot(SLOT_OUT, out);
    Ok(())
}
