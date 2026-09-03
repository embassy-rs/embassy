//! Poly1305 on the CryptoCell PKA.
//!
//! Poly1305 evaluates a polynomial over the prime field of `2¹³⁰ - 5`, which the PKA does
//! directly: every 16-byte block becomes a field element with an extra bit set above it, and
//! the accumulator is `acc = (acc + block) · r mod p`.
//!
//! This follows `poly.c` of Arm's CryptoCell runtime library.

use super::engine::{self as pka, LEN_FULL, LEN_N, REG_N, REG_NP};

/// `2¹³⁰ - 5`, big-endian.
const PRIME: [u8; 17] = [
    0x03, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfb,
];
const PRIME_BITS: u32 = 130;

/// Accumulator.
const R_ACC: u8 = 2;
/// The `r` half of the key.
const R_R: u8 = 3;
/// The block being absorbed.
const R_DATA: u8 = 4;
/// Registers 0 to 4, plus the two the engine reserves for itself.
const REGS: usize = 7;

/// Length of a Poly1305 block, and of the tag.
pub(crate) const BLOCK_LEN: usize = 16;

/// A Poly1305 computation in progress.
///
/// The accumulator lives in this struct rather than in the engine, so that other operations
/// can use the PKA between calls.
#[derive(Clone)]
pub(crate) struct Poly1305 {
    /// Clamped `r`, big-endian.
    r: [u8; BLOCK_LEN],
    /// `s`, little-endian as it comes out of the key.
    s: [u8; BLOCK_LEN],
    /// Accumulator, big-endian, wide enough for the whole field.
    acc: [u8; 20],
}

impl Poly1305 {
    /// Starts a computation with a one-time key.
    pub(crate) fn new(key: &[u8; 32]) -> Self {
        let mut r = [0u8; BLOCK_LEN];
        r.copy_from_slice(&key[..BLOCK_LEN]);
        // Clamp: clear the top four bits of every fourth byte and the bottom two bits of the
        // bytes that start a limb.
        for i in [3, 7, 11, 15] {
            r[i] &= 15;
        }
        for i in [4, 8, 12] {
            r[i] &= 252;
        }
        r.reverse();
        let mut s = [0u8; BLOCK_LEN];
        s.copy_from_slice(&key[BLOCK_LEN..]);
        Self { r, s, acc: [0; 20] }
    }

    /// Absorbs whole blocks. The caller pads the last block of a message with zeros.
    pub(crate) fn update(&mut self, data: &[u8]) {
        debug_assert!(data.len() % BLOCK_LEN == 0);
        if data.is_empty() {
            return;
        }
        // The engine is only ever busy inside this function, so a failure to lay out the
        // registers cannot happen for this fixed, tiny modulus.
        if pka::init(PRIME_BITS, REGS).is_err() {
            return;
        }
        pka::write_be(REG_N, &PRIME);
        pka::calc_np(PRIME_BITS, REG_N, REG_NP, R_ACC, R_DATA);
        pka::write_be(R_R, &self.r);
        pka::write_be(R_ACC, &self.acc);

        let mut block = [0u8; 20];
        // Every block carries an extra bit above its 128, which is what makes the encoding
        // of a block injective.
        block[3] = 1;
        for chunk in data.chunks_exact(BLOCK_LEN) {
            for (i, &b) in chunk.iter().enumerate() {
                block[19 - i] = b;
            }
            pka::write_be(R_DATA, &block);
            pka::mod_add(LEN_FULL, R_ACC, R_ACC, R_DATA);
            pka::mod_mul(LEN_N, R_ACC, R_ACC, R_R);
        }

        pka::read_be(R_ACC, &mut self.acc);
        pka::finish(REGS);
    }

    /// Returns the tag.
    pub(crate) fn finish(self) -> [u8; BLOCK_LEN] {
        // tag = (acc + s) mod 2¹²⁸, little-endian.
        let mut tag = [0u8; BLOCK_LEN];
        let mut carry = 0u16;
        for i in 0..BLOCK_LEN {
            let v = self.acc[19 - i] as u16 + self.s[i] as u16 + carry;
            tag[i] = v as u8;
            carry = v >> 8;
        }
        tag
    }
}

