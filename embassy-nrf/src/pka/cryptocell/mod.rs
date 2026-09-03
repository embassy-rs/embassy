//! PKA driver for the CryptoCell subsystem.

mod ecc;
mod engine;
pub(crate) mod poly;

use engine::{LEN_FULL, LEN_MAX, LEN_N, LEN_PQ, REG_N, REG_NP, REG_T0, REG_T1};
pub(super) use ecc::{ecc_mul, ecdsa_sign, ecdsa_verify, point_check};

use crate::pka::{Error, bit_len};

/// Keeps the CryptoCell powered while the driver exists.
pub(super) struct Handle {
    _activation: crate::cryptocell::CryptoCellActivationHandle,
}

impl Handle {
    pub(super) fn new() -> Self {
        Self {
            _activation: crate::cryptocell::activate(),
        }
    }
}

/// Computes `base ^ exponent mod modulus`.
pub(super) fn mod_exp(base: &[u8], exponent: &[u8], modulus: &[u8], out: &mut [u8]) -> Result<(), Error> {
    const R_IN: u8 = 2;
    const R_EXP: u8 = 3;
    const R_OUT: u8 = 4;
    /// Registers 0 to 4, plus the two the engine reserves for itself.
    const REGS: usize = 7;

    let bits = bit_len(modulus) as u32;
    engine::init(bits, REGS)?;
    engine::write_be(REG_N, modulus);
    engine::calc_np(bits, REG_N, REG_NP, R_IN, R_OUT);
    engine::write_be(R_IN, base);
    engine::write_be(R_EXP, exponent);
    engine::mod_exp(LEN_N, R_OUT, R_IN, R_EXP);
    engine::read_be(R_OUT, out);
    engine::finish(REGS);
    Ok(())
}

/// Computes `input ^ d mod p·q` from the Chinese remainder theorem parameters.
///
/// Follows `RsaExecPrivKeyExpCrt` of Arm's CryptoCell runtime library: the two half-size
/// exponentiations run with the prime in the modulus register, which means moving the second
/// prime into a spare register and pointing the engine at it.
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
    /// Holds the second prime once the modulus register has been switched over.
    const R_Q: u8 = 0;
    const R_D: u8 = 2;
    const R_T: u8 = 3;
    const R_T1: u8 = 4;
    const R_MQ: u8 = 5;
    /// Holds the first prime, and is the modulus register for the second exponentiation.
    const R_P: u8 = 6;
    const R_QINV: u8 = 7;
    const R_TMP1: u8 = 8;
    const R_TMP2: u8 = 9;
    /// Registers 0 to 9, plus the two the engine reserves for itself.
    const REGS: usize = 12;

    let p_bits = bit_len(p) as u32;
    let q_bits = bit_len(q) as u32;
    let n_bits = (input.len() * 8) as u32;
    let pq_words = p.len().div_ceil(4);

    engine::init(n_bits, REGS)?;

    // Mq = input ^ dq mod q
    engine::set_len_pair(LEN_PQ, q_bits);
    engine::write_be(REG_N, q);
    engine::calc_np(q_bits, REG_N, REG_NP, R_TMP1, R_TMP2);
    engine::write_be(R_D, dq);
    engine::write_be(R_T, input);
    engine::copy(LEN_MAX, R_T1, R_T);
    engine::div(LEN_FULL, R_P, R_T, REG_N);
    engine::clear_from(R_T, pq_words);
    engine::mod_exp(LEN_PQ, R_MQ, R_T, R_D);

    // Mp = input ^ dp mod p, with p as the modulus in its own register.
    engine::write_be(R_P, p);
    engine::set_n_np_t0_t1(R_P, REG_NP, REG_T0, REG_T1);
    engine::set_len_pair(LEN_PQ, p_bits);
    engine::calc_np(p_bits, R_P, REG_NP, R_TMP1, R_TMP2);
    engine::write_be(R_D, dp);
    engine::write_be(R_QINV, qinv);
    engine::div(LEN_FULL, R_T, R_T1, R_P);
    engine::clear_from(R_T1, pq_words);
    engine::mod_exp(LEN_PQ, R_T, R_T1, R_D);

    // h = (Mp - Mq) · qinv mod p
    engine::mod_add_im(LEN_PQ, R_T1, R_MQ, 0);
    engine::mod_sub(LEN_PQ, R_T, R_T, R_T1);
    engine::mod_mul(LEN_PQ, R_T1, R_T, R_QINV);

    // The half-size operations leave the upper half of their registers untouched, so clear it
    // before the full-size multiplication.
    engine::clear_from(R_T1, pq_words);
    engine::clear_from(R_T, pq_words);
    engine::clear_from(R_Q, pq_words);
    engine::clear_from(R_MQ, pq_words);

    // M = Mq + q·h
    engine::copy(LEN_MAX, R_T, R_T1);
    engine::copy(LEN_MAX, R_T1, R_MQ);
    engine::mul_low(LEN_FULL, R_T, R_T, R_Q);
    engine::add(LEN_N, R_T, R_T1, R_T);
    engine::read_be(R_T, out);

    engine::finish(REGS);
    Ok(())
}

