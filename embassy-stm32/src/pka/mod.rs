//! Public Key Accelerator (PKA)
//!
//! This module provides hardware-accelerated public key cryptographic operations using the PKA
//! peripheral. The PKA can accelerate:
//!
//! - **ECDSA**: Signature generation and verification
//! - **ECDH**: Elliptic Curve Diffie-Hellman key agreement (via scalar multiplication)
//! - **RSA**: Encryption, decryption, and signing (via modular exponentiation)
//! - **Arithmetic**: Modular operations, Montgomery multiplication
//!
//! # Supported Operations
//!
//! | Operation | Mode | Description |
//! |-----------|------|-------------|
//! | Modular Exponentiation | 0x00 | RSA encryption/decryption |
//! | Montgomery Parameter | 0x01 | Compute Montgomery parameter for RSA |
//! | RSA CRT Exponentiation | 0x07 | Fast RSA with Chinese Remainder Theorem |
//! | Modular Inversion | 0x08 | Compute modular inverse |
//! | ECC Scalar Multiplication | 0x20 | ECDH key agreement, point multiplication |
//! | ECDSA Sign | 0x24 | Generate ECDSA signatures |
//! | ECDSA Verify | 0x26 | Verify ECDSA signatures |
//! | Point Check | 0x28 | Validate point is on curve |
//!
//! # Example - ECDSA Signature Verification
//!
//! ```no_run
//! use embassy_stm32::pka::{Pka, EcdsaCurveParams, EcdsaPublicKey, EcdsaSignature};
//!
//! let mut pka = Pka::new_blocking(p.PKA, Irqs);
//! let params = EcdsaCurveParams::nist_p256();
//!
//! let public_key = EcdsaPublicKey {
//!     x: &pub_key_x,
//!     y: &pub_key_y,
//! };
//!
//! let signature = EcdsaSignature {
//!     r: &sig_r,
//!     s: &sig_s,
//! };
//!
//! let valid = pka.ecdsa_verify(&params, &public_key, &signature, &hash)?;
//! ```
//!
//! # Example - ECDH Key Agreement
//!
//! ```no_run
//! use embassy_stm32::pka::{Pka, EccMulParams, EccPoint};
//!
//! let mut pka = Pka::new_blocking(p.PKA, Irqs);
//! let params = EccMulParams::nist_p256();
//!
//! // Compute shared_secret = private_key * peer_public_key
//! let peer_public = EccPoint { x: &peer_x, y: &peer_y };
//! let shared_point = pka.ecc_mul(&params, &private_key, &peer_public)?;
//! ```
//!
//! # Security Notes
//!
//! - Always use cryptographically secure random numbers for ECDSA k values
//! - Validate all public keys before use (use `point_check`)
//! - Use constant-time operations when possible (hardware provides this)
//! - Clear sensitive data from memory after use

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac, peripherals, rcc};

static PKA_WAKER: AtomicWaker = AtomicWaker::new();

// ============================================================================
// PKA Modes
// ============================================================================

/// PKA operation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PkaMode {
    /// Modular exponentiation (RSA)
    ModularExp = 0x00,
    /// Montgomery parameter computation
    MontgomeryParam = 0x01,
    /// Modular exponentiation fast mode
    ModularExpFast = 0x02,
    /// Modular exponentiation with protection
    ModularExpProtect = 0x03,
    /// RSA CRT exponentiation
    RsaCrtExp = 0x07,
    /// Modular inversion
    ModularInv = 0x08,
    /// Arithmetic addition
    ArithmeticAdd = 0x09,
    /// Arithmetic subtraction
    ArithmeticSub = 0x0A,
    /// Arithmetic multiplication
    ArithmeticMul = 0x0B,
    /// Comparison
    Comparison = 0x0C,
    /// Modular reduction
    ModularRed = 0x0D,
    /// Modular addition
    ModularAdd = 0x0E,
    /// Modular subtraction
    ModularSub = 0x0F,
    /// Montgomery multiplication
    MontgomeryMul = 0x10,
    /// ECC scalar multiplication
    EccMul = 0x20,
    /// ECC complete addition
    EccCompleteAdd = 0x23,
    /// ECDSA signature generation
    EcdsaSign = 0x24,
    /// ECDSA signature verification
    EcdsaVerify = 0x26,
    /// Double base ladder
    DoubleBaseLadder = 0x27,
    /// Point check (validate point on curve)
    PointCheck = 0x28,
    /// ECC projective to affine
    EccProjectiveToAffine = 0x2F,
}

// ============================================================================
// RAM Offsets for each operation (byte offsets from PKA RAM base)
// Derived from CMSIS headers: offset = raw_address - 0x0400
// ============================================================================

#[allow(dead_code)] // Offsets for future PKA operations
mod offsets {
    // Montgomery parameter computation
    pub mod montgomery_param {
        pub const IN_MOD_NB_BITS: usize = 0x08;
        pub const IN_MODULUS: usize = 0xC88;
        pub const OUT_PARAMETER: usize = 0x220;
    }

    // Modular exponentiation (RSA)
    pub mod modular_exp {
        pub const IN_EXP_NB_BITS: usize = 0x00;
        pub const IN_OP_NB_BITS: usize = 0x08;
        pub const IN_MONTGOMERY_PARAM: usize = 0x220;
        pub const IN_EXPONENT_BASE: usize = 0x868;
        pub const IN_EXPONENT: usize = 0xA78;
        pub const IN_MODULUS: usize = 0xC88;
        pub const OUT_RESULT: usize = 0x438;
        pub const OUT_ERROR: usize = 0xE98;
    }

    // RSA CRT exponentiation
    pub mod rsa_crt {
        pub const IN_MOD_NB_BITS: usize = 0x08;
        pub const IN_DP_CRT: usize = 0x330;
        pub const IN_DQ_CRT: usize = 0xA78;
        pub const IN_QINV_CRT: usize = 0x548;
        pub const IN_PRIME_P: usize = 0x760;
        pub const IN_PRIME_Q: usize = 0xC88;
        pub const IN_EXPONENT_BASE: usize = 0xEA0;
        pub const OUT_RESULT: usize = 0x438;
    }

    // ECC scalar multiplication
    pub mod ecc_mul {
        pub const IN_EXP_NB_BITS: usize = 0x00;
        pub const IN_OP_NB_BITS: usize = 0x08;
        pub const IN_A_COEFF_SIGN: usize = 0x10;
        pub const IN_A_COEFF: usize = 0x18;
        pub const IN_B_COEFF: usize = 0x120;
        pub const IN_MOD_GF: usize = 0xC88;
        pub const IN_K: usize = 0xEA0;
        pub const IN_INITIAL_POINT_X: usize = 0x178;
        pub const IN_INITIAL_POINT_Y: usize = 0x70;
        pub const IN_N_PRIME_ORDER: usize = 0xB88;
        pub const OUT_RESULT_X: usize = 0x178;
        pub const OUT_RESULT_Y: usize = 0x1D0;
        pub const OUT_ERROR: usize = 0x280;
    }

    // ECDSA signature generation
    pub mod ecdsa_sign {
        pub const IN_ORDER_NB_BITS: usize = 0x00;
        pub const IN_MOD_NB_BITS: usize = 0x08;
        pub const IN_A_COEFF_SIGN: usize = 0x10;
        pub const IN_A_COEFF: usize = 0x18;
        pub const IN_B_COEFF: usize = 0x120;
        pub const IN_MOD_GF: usize = 0xC88;
        pub const IN_K: usize = 0xEA0;
        pub const IN_INITIAL_POINT_X: usize = 0x178;
        pub const IN_INITIAL_POINT_Y: usize = 0x70;
        pub const IN_HASH_E: usize = 0xBE8;
        pub const IN_PRIVATE_KEY_D: usize = 0xB28;
        pub const IN_ORDER_N: usize = 0xB88;
        pub const OUT_ERROR: usize = 0xBE0;
        pub const OUT_SIGNATURE_R: usize = 0x330;
        pub const OUT_SIGNATURE_S: usize = 0x388;
        pub const OUT_FINAL_POINT_X: usize = 0x1000;
        pub const OUT_FINAL_POINT_Y: usize = 0x1058;
    }

    // ECDSA signature verification
    pub mod ecdsa_verif {
        pub const IN_ORDER_NB_BITS: usize = 0x08;
        pub const IN_MOD_NB_BITS: usize = 0xC8;
        pub const IN_A_COEFF_SIGN: usize = 0x68;
        pub const IN_A_COEFF: usize = 0x70;
        pub const IN_MOD_GF: usize = 0xD0;
        pub const IN_INITIAL_POINT_X: usize = 0x278;
        pub const IN_INITIAL_POINT_Y: usize = 0x2D0;
        pub const IN_PUBLIC_KEY_POINT_X: usize = 0xEF8;
        pub const IN_PUBLIC_KEY_POINT_Y: usize = 0xF50;
        pub const IN_SIGNATURE_R: usize = 0xCE0;
        pub const IN_SIGNATURE_S: usize = 0x868;
        pub const IN_HASH_E: usize = 0xFA8;
        pub const IN_ORDER_N: usize = 0xC88;
        pub const OUT_RESULT: usize = 0x1D0;
    }

    // Point check
    pub mod point_check {
        pub const IN_MOD_NB_BITS: usize = 0x08;
        pub const IN_A_COEFF_SIGN: usize = 0x10;
        pub const IN_A_COEFF: usize = 0x18;
        pub const IN_B_COEFF: usize = 0x120;
        pub const IN_MOD_GF: usize = 0x70;
        pub const IN_INITIAL_POINT_X: usize = 0x178;
        pub const IN_INITIAL_POINT_Y: usize = 0x1D0;
        pub const IN_MONTGOMERY_PARAM: usize = 0xC8;
        pub const OUT_ERROR: usize = 0x280;
    }

    // Modular inversion
    pub mod modular_inv {
        pub const IN_NB_BITS: usize = 0x08;
        pub const IN_OP1: usize = 0x650;
        pub const IN_OP2_MOD: usize = 0x868;
        pub const OUT_RESULT: usize = 0xA78;
    }

    // Generic arithmetic operations
    pub mod arithmetic {
        pub const IN_NB_BITS: usize = 0x08;
        pub const IN_OP1: usize = 0x650;
        pub const IN_OP2: usize = 0x868;
        pub const IN_OP3_MOD: usize = 0xC88;
        pub const OUT_RESULT: usize = 0xA78;
    }
}

// ============================================================================
// Interrupt Handler
// ============================================================================

/// PKA interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let sr = T::regs().sr().read();

        if sr.procendf() {
            T::regs().clrfr().write(|w| w.set_procendfc(true));
            PKA_WAKER.wake();
        }

        if sr.ramerrf() || sr.addrerrf() || sr.operrf() {
            T::regs().clrfr().write(|w| {
                w.set_ramerrfc(true);
                w.set_addrerrfc(true);
                w.set_operrfc(true);
            });
            PKA_WAKER.wake();
        }
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// PKA error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// PKA RAM access error
    RamError,
    /// Invalid RAM address
    AddressError,
    /// Operation error (invalid inputs or computation failed)
    OperationError,
    /// Invalid parameter size
    InvalidSize,
    /// Initialization timeout
    Timeout,
    /// Point is not on the curve
    PointNotOnCurve,
}

// ============================================================================
// Data Structures
// ============================================================================

/// ECDSA/ECC curve parameters
#[derive(Clone)]
pub struct EcdsaCurveParams {
    /// Prime field modulus p
    pub p_modulus: &'static [u8],
    /// Curve coefficient |a| (absolute value)
    pub a_coefficient: &'static [u8],
    /// Sign of curve coefficient a (0 = positive, 1 = negative)
    pub a_coefficient_sign: u32,
    /// Curve coefficient b
    pub b_coefficient: &'static [u8],
    /// Base point x-coordinate
    pub generator_x: &'static [u8],
    /// Base point y-coordinate
    pub generator_y: &'static [u8],
    /// Curve order n
    pub order: &'static [u8],
}

impl EcdsaCurveParams {
    /// NIST P-256 (secp256r1) curve parameters
    pub const fn nist_p256() -> Self {
        Self {
            p_modulus: &P256_P,
            // For P-256, a = -3 (mod p), so we use |a| = 3 with sign = 1 (negative)
            a_coefficient: &P256_A,
            a_coefficient_sign: 1, // negative
            b_coefficient: &P256_B,
            generator_x: &P256_GX,
            generator_y: &P256_GY,
            order: &P256_N,
        }
    }
}

// NIST P-256 curve parameters (big-endian)
const P256_P: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
];
// |a| = 3 (absolute value of -3)
const P256_A: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
];
const P256_B: [u8; 32] = [
    0x5A, 0xC6, 0x35, 0xD8, 0xAA, 0x3A, 0x93, 0xE7, 0xB3, 0xEB, 0xBD, 0x55, 0x76, 0x98, 0x86, 0xBC, 0x65, 0x1D, 0x06,
    0xB0, 0xCC, 0x53, 0xB0, 0xF6, 0x3B, 0xCE, 0x3C, 0x3E, 0x27, 0xD2, 0x60, 0x4B,
];
const P256_GX: [u8; 32] = [
    0x6B, 0x17, 0xD1, 0xF2, 0xE1, 0x2C, 0x42, 0x47, 0xF8, 0xBC, 0xE6, 0xE5, 0x63, 0xA4, 0x40, 0xF2, 0x77, 0x03, 0x7D,
    0x81, 0x2D, 0xEB, 0x33, 0xA0, 0xF4, 0xA1, 0x39, 0x45, 0xD8, 0x98, 0xC2, 0x96,
];
const P256_GY: [u8; 32] = [
    0x4F, 0xE3, 0x42, 0xE2, 0xFE, 0x1A, 0x7F, 0x9B, 0x8E, 0xE7, 0xEB, 0x4A, 0x7C, 0x0F, 0x9E, 0x16, 0x2B, 0xCE, 0x33,
    0x57, 0x6B, 0x31, 0x5E, 0xCE, 0xCB, 0xB6, 0x40, 0x68, 0x37, 0xBF, 0x51, 0xF5,
];
const P256_N: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xBC, 0xE6, 0xFA,
    0xAD, 0xA7, 0x17, 0x9E, 0x84, 0xF3, 0xB9, 0xCA, 0xC2, 0xFC, 0x63, 0x25, 0x51,
];

/// ECDSA public key
pub struct EcdsaPublicKey<'a> {
    /// Public key x-coordinate
    pub x: &'a [u8],
    /// Public key y-coordinate
    pub y: &'a [u8],
}

/// ECDSA signature
pub struct EcdsaSignature<'a> {
    /// Signature r component
    pub r: &'a [u8],
    /// Signature s component
    pub s: &'a [u8],
}

/// ECC point (for scalar multiplication results)
pub struct EccPoint {
    /// X coordinate
    pub x: [u8; 66], // Max size for P-521
    /// Y coordinate
    pub y: [u8; 66],
    /// Actual size of coordinates in bytes
    pub size: usize,
}

impl EccPoint {
    /// Create a new point with given size
    pub fn new(size: usize) -> Self {
        Self {
            x: [0u8; 66],
            y: [0u8; 66],
            size,
        }
    }
}

/// RSA operation parameters
pub struct RsaParams<'a> {
    /// Modulus n
    pub modulus: &'a [u8],
    /// Exponent (public or private)
    pub exponent: &'a [u8],
}

/// RSA CRT parameters for fast decryption
pub struct RsaCrtParams<'a> {
    /// Prime p
    pub prime_p: &'a [u8],
    /// Prime q
    pub prime_q: &'a [u8],
    /// d mod (p-1)
    pub dp: &'a [u8],
    /// d mod (q-1)
    pub dq: &'a [u8],
    /// q^(-1) mod p
    pub qinv: &'a [u8],
}

// ============================================================================
// PKA Driver
// ============================================================================

/// PKA driver
pub struct Pka<'d, T: Instance> {
    _peripheral: Peri<'d, T>,
}

impl<'d, T: Instance> Pka<'d, T> {
    const RAM_ERASE_TIMEOUT: u32 = 100_000;

    /// Create a new PKA driver
    pub fn new_blocking(
        peripheral: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self {
            _peripheral: peripheral,
        }
    }

    // ========================================================================
    // ECDSA Operations
    // ========================================================================

    /// Verify an ECDSA signature
    ///
    /// Returns `Ok(true)` if signature is valid, `Ok(false)` if invalid.
    pub fn ecdsa_verify(
        &mut self,
        curve: &EcdsaCurveParams,
        public_key: &EcdsaPublicKey,
        signature: &EcdsaSignature,
        message_hash: &[u8],
    ) -> Result<bool, Error> {
        let modulus_size = curve.p_modulus.len();
        let order_size = curve.order.len();

        // Validate sizes
        if curve.a_coefficient.len() != modulus_size
            || curve.generator_x.len() != modulus_size
            || curve.generator_y.len() != modulus_size
            || public_key.x.len() != modulus_size
            || public_key.y.len() != modulus_size
            || signature.r.len() != order_size
            || signature.s.len() != order_size
            || message_hash.len() > order_size
        {
            return Err(Error::InvalidSize);
        }

        self.init_pka()?;

        // Write bit counts
        let order_nb_bits = Self::get_opt_bit_size(order_size, curve.order[0]);
        let mod_nb_bits = Self::get_opt_bit_size(modulus_size, curve.p_modulus[0]);

        self.write_ram_word(offsets::ecdsa_verif::IN_ORDER_NB_BITS, order_nb_bits);
        self.write_ram_word(offsets::ecdsa_verif::IN_MOD_NB_BITS, mod_nb_bits);
        self.write_ram_word(offsets::ecdsa_verif::IN_A_COEFF_SIGN, curve.a_coefficient_sign);

        // Write curve parameters (matching ST-HAL order)
        self.write_operand(offsets::ecdsa_verif::IN_A_COEFF, curve.a_coefficient);
        self.write_operand(offsets::ecdsa_verif::IN_MOD_GF, curve.p_modulus);
        self.write_operand(offsets::ecdsa_verif::IN_INITIAL_POINT_X, curve.generator_x);
        self.write_operand(offsets::ecdsa_verif::IN_INITIAL_POINT_Y, curve.generator_y);

        // Write public key
        self.write_operand(offsets::ecdsa_verif::IN_PUBLIC_KEY_POINT_X, public_key.x);
        self.write_operand(offsets::ecdsa_verif::IN_PUBLIC_KEY_POINT_Y, public_key.y);

        // Write signature
        self.write_operand(offsets::ecdsa_verif::IN_SIGNATURE_R, signature.r);
        self.write_operand(offsets::ecdsa_verif::IN_SIGNATURE_S, signature.s);

        // Write hash and order (ST-HAL writes these last)
        self.write_operand(offsets::ecdsa_verif::IN_HASH_E, message_hash);
        self.write_operand(offsets::ecdsa_verif::IN_ORDER_N, curve.order);

        // Set mode and start (matching ST-HAL: mode is set AFTER writing parameters)
        self.set_mode(PkaMode::EcdsaVerify);
        self.start_and_wait()?;

        let result = self.read_ram_word(offsets::ecdsa_verif::OUT_RESULT);
        self.disable_pka();

        Ok(result == 0xD60D)
    }

    /// Generate an ECDSA signature
    ///
    /// # Arguments
    /// * `curve` - Curve parameters
    /// * `private_key` - Private key d
    /// * `k` - Random nonce (MUST be cryptographically random and unique per signature!)
    /// * `message_hash` - Hash of the message to sign
    ///
    /// # Returns
    /// Signature (r, s) as byte arrays
    ///
    /// # Security Warning
    /// The `k` value MUST be:
    /// - Cryptographically random
    /// - Unique for every signature
    /// - Never reused or predictable
    /// Failure to ensure this will compromise the private key!
    pub fn ecdsa_sign(
        &mut self,
        curve: &EcdsaCurveParams,
        private_key: &[u8],
        k: &[u8],
        message_hash: &[u8],
        signature_r: &mut [u8],
        signature_s: &mut [u8],
    ) -> Result<(), Error> {
        let modulus_size = curve.p_modulus.len();
        let order_size = curve.order.len();

        // Validate sizes
        if private_key.len() != order_size
            || k.len() != order_size
            || message_hash.len() > order_size
            || signature_r.len() < order_size
            || signature_s.len() < order_size
        {
            return Err(Error::InvalidSize);
        }

        self.init_pka()?;
        self.set_mode(PkaMode::EcdsaSign);

        // Write bit counts
        let order_nb_bits = Self::get_opt_bit_size(order_size, curve.order[0]);
        let mod_nb_bits = Self::get_opt_bit_size(modulus_size, curve.p_modulus[0]);

        self.write_ram_word(offsets::ecdsa_sign::IN_ORDER_NB_BITS, order_nb_bits);
        self.write_ram_word(offsets::ecdsa_sign::IN_MOD_NB_BITS, mod_nb_bits);
        self.write_ram_word(offsets::ecdsa_sign::IN_A_COEFF_SIGN, curve.a_coefficient_sign);

        // Write curve parameters
        self.write_operand(offsets::ecdsa_sign::IN_A_COEFF, curve.a_coefficient);
        self.write_operand(offsets::ecdsa_sign::IN_B_COEFF, curve.b_coefficient);
        self.write_operand(offsets::ecdsa_sign::IN_MOD_GF, curve.p_modulus);
        self.write_operand(offsets::ecdsa_sign::IN_INITIAL_POINT_X, curve.generator_x);
        self.write_operand(offsets::ecdsa_sign::IN_INITIAL_POINT_Y, curve.generator_y);
        self.write_operand(offsets::ecdsa_sign::IN_ORDER_N, curve.order);

        // Write private key and random k
        self.write_operand(offsets::ecdsa_sign::IN_PRIVATE_KEY_D, private_key);
        self.write_operand(offsets::ecdsa_sign::IN_K, k);
        self.write_operand(offsets::ecdsa_sign::IN_HASH_E, message_hash);

        self.start_and_wait()?;

        // Check for errors - 0xD60D indicates success
        let result = self.read_ram_word(offsets::ecdsa_sign::OUT_ERROR);
        if result != 0xD60D {
            self.disable_pka();
            return Err(Error::OperationError);
        }

        // Read signature
        self.read_operand(offsets::ecdsa_sign::OUT_SIGNATURE_R, &mut signature_r[..order_size]);
        self.read_operand(offsets::ecdsa_sign::OUT_SIGNATURE_S, &mut signature_s[..order_size]);

        self.disable_pka();
        Ok(())
    }

    // ========================================================================
    // ECC Scalar Multiplication (for ECDH)
    // ========================================================================

    /// Perform ECC scalar multiplication: result = k * P
    ///
    /// This is the core operation for ECDH key agreement:
    /// - To generate public key: public = private_key * G (generator point)
    /// - To compute shared secret: shared = my_private * peer_public
    ///
    /// # Arguments
    /// * `curve` - Curve parameters
    /// * `k` - Scalar multiplier
    /// * `point_x` - Input point X coordinate
    /// * `point_y` - Input point Y coordinate
    /// * `result` - Output point (must be initialized with correct size)
    pub fn ecc_mul(
        &mut self,
        curve: &EcdsaCurveParams,
        k: &[u8],
        point_x: &[u8],
        point_y: &[u8],
        result: &mut EccPoint,
    ) -> Result<(), Error> {
        let modulus_size = curve.p_modulus.len();
        let order_size = curve.order.len();

        if k.len() != order_size
            || point_x.len() != modulus_size
            || point_y.len() != modulus_size
            || result.size != modulus_size
        {
            return Err(Error::InvalidSize);
        }

        self.init_pka()?;

        // Write bit counts
        // ST HAL uses scalar size with MSB of prime order (not scalar MSB)
        let exp_nb_bits = Self::get_opt_bit_size(k.len(), curve.order[0]);
        let mod_nb_bits = Self::get_opt_bit_size(modulus_size, curve.p_modulus[0]);

        self.write_ram_word(offsets::ecc_mul::IN_EXP_NB_BITS, exp_nb_bits);
        self.write_ram_word(offsets::ecc_mul::IN_OP_NB_BITS, mod_nb_bits);
        self.write_ram_word(offsets::ecc_mul::IN_A_COEFF_SIGN, curve.a_coefficient_sign);

        // Write curve parameters
        self.write_operand(offsets::ecc_mul::IN_A_COEFF, curve.a_coefficient);
        self.write_operand(offsets::ecc_mul::IN_B_COEFF, curve.b_coefficient);
        self.write_operand(offsets::ecc_mul::IN_MOD_GF, curve.p_modulus);
        self.write_operand(offsets::ecc_mul::IN_N_PRIME_ORDER, curve.order);

        // Write scalar and point
        self.write_operand(offsets::ecc_mul::IN_K, k);
        self.write_operand(offsets::ecc_mul::IN_INITIAL_POINT_X, point_x);
        self.write_operand(offsets::ecc_mul::IN_INITIAL_POINT_Y, point_y);

        // Set mode right before start (matching ST HAL order)
        self.set_mode(PkaMode::EccMul);
        self.start_and_wait()?;

        // Check for errors - 0xD60D indicates success
        let status = self.read_ram_word(offsets::ecc_mul::OUT_ERROR);
        if status != 0xD60D {
            self.disable_pka();
            return Err(Error::OperationError);
        }

        // Read result
        self.read_operand(offsets::ecc_mul::OUT_RESULT_X, &mut result.x[..modulus_size]);
        self.read_operand(offsets::ecc_mul::OUT_RESULT_Y, &mut result.y[..modulus_size]);

        self.disable_pka();
        Ok(())
    }

    /// Check if a point is on the curve
    ///
    /// This should be called to validate any externally-provided public key
    /// before using it in cryptographic operations.
    pub fn point_check(&mut self, curve: &EcdsaCurveParams, point_x: &[u8], point_y: &[u8]) -> Result<bool, Error> {
        let modulus_size = curve.p_modulus.len();

        if point_x.len() != modulus_size || point_y.len() != modulus_size {
            return Err(Error::InvalidSize);
        }

        self.init_pka()?;

        let mod_nb_bits = Self::get_opt_bit_size(modulus_size, curve.p_modulus[0]);

        self.write_ram_word(offsets::point_check::IN_MOD_NB_BITS, mod_nb_bits);
        self.write_ram_word(offsets::point_check::IN_A_COEFF_SIGN, curve.a_coefficient_sign);

        self.write_operand(offsets::point_check::IN_A_COEFF, curve.a_coefficient);
        self.write_operand(offsets::point_check::IN_B_COEFF, curve.b_coefficient);
        self.write_operand(offsets::point_check::IN_MOD_GF, curve.p_modulus);
        self.write_operand(offsets::point_check::IN_INITIAL_POINT_X, point_x);
        self.write_operand(offsets::point_check::IN_INITIAL_POINT_Y, point_y);

        // Set mode right before start (matching ST HAL order)
        self.set_mode(PkaMode::PointCheck);
        self.start_and_wait()?;

        let result = self.read_ram_word(offsets::point_check::OUT_ERROR);
        self.disable_pka();

        // 0xD60D means point is on curve
        Ok(result == 0xD60D)
    }

    // ========================================================================
    // RSA Operations
    // ========================================================================

    /// Perform modular exponentiation: result = base^exp mod n
    ///
    /// This is the core RSA operation:
    /// - Encryption: ciphertext = plaintext^e mod n
    /// - Decryption: plaintext = ciphertext^d mod n
    /// - Signing: signature = hash^d mod n
    /// - Verification: hash = signature^e mod n
    ///
    /// # Arguments
    /// * `base` - Base value (plaintext/ciphertext)
    /// * `exponent` - Exponent (e for encrypt/verify, d for decrypt/sign)
    /// * `modulus` - RSA modulus n
    /// * `result` - Output buffer (must be same size as modulus)
    pub fn modular_exp(
        &mut self,
        base: &[u8],
        exponent: &[u8],
        modulus: &[u8],
        result: &mut [u8],
    ) -> Result<(), Error> {
        let mod_size = modulus.len();
        let exp_size = exponent.len();

        if base.len() > mod_size || result.len() < mod_size {
            return Err(Error::InvalidSize);
        }

        self.init_pka()?;
        self.set_mode(PkaMode::ModularExp);

        let exp_nb_bits = Self::get_opt_bit_size(exp_size, exponent[0]);
        let mod_nb_bits = Self::get_opt_bit_size(mod_size, modulus[0]);

        self.write_ram_word(offsets::modular_exp::IN_EXP_NB_BITS, exp_nb_bits);
        self.write_ram_word(offsets::modular_exp::IN_OP_NB_BITS, mod_nb_bits);

        self.write_operand(offsets::modular_exp::IN_EXPONENT_BASE, base);
        self.write_operand(offsets::modular_exp::IN_EXPONENT, exponent);
        self.write_operand(offsets::modular_exp::IN_MODULUS, modulus);

        self.start_and_wait()?;

        // Check for errors - 0xD60D indicates success
        let status = self.read_ram_word(offsets::modular_exp::OUT_ERROR);
        if status != 0xD60D {
            self.disable_pka();
            return Err(Error::OperationError);
        }

        self.read_operand(offsets::modular_exp::OUT_RESULT, &mut result[..mod_size]);

        self.disable_pka();
        Ok(())
    }

    /// Perform RSA CRT exponentiation for fast decryption
    ///
    /// Uses Chinese Remainder Theorem for ~4x faster RSA private key operations.
    ///
    /// # Arguments
    /// * `ciphertext` - Encrypted data
    /// * `params` - CRT parameters (p, q, dp, dq, qinv)
    /// * `result` - Output buffer
    pub fn rsa_crt_exp(&mut self, ciphertext: &[u8], params: &RsaCrtParams, result: &mut [u8]) -> Result<(), Error> {
        let p_size = params.prime_p.len();
        let q_size = params.prime_q.len();
        let mod_size = p_size + q_size; // n = p * q

        if ciphertext.len() > mod_size
            || params.dp.len() != p_size
            || params.dq.len() != q_size
            || params.qinv.len() != p_size
            || result.len() < mod_size
        {
            return Err(Error::InvalidSize);
        }

        self.init_pka()?;
        self.set_mode(PkaMode::RsaCrtExp);

        // For CRT, we use the size of the larger prime
        let mod_nb_bits = Self::get_opt_bit_size(mod_size, ciphertext[0]);

        self.write_ram_word(offsets::rsa_crt::IN_MOD_NB_BITS, mod_nb_bits);

        self.write_operand(offsets::rsa_crt::IN_PRIME_P, params.prime_p);
        self.write_operand(offsets::rsa_crt::IN_PRIME_Q, params.prime_q);
        self.write_operand(offsets::rsa_crt::IN_DP_CRT, params.dp);
        self.write_operand(offsets::rsa_crt::IN_DQ_CRT, params.dq);
        self.write_operand(offsets::rsa_crt::IN_QINV_CRT, params.qinv);
        self.write_operand(offsets::rsa_crt::IN_EXPONENT_BASE, ciphertext);

        self.start_and_wait()?;

        self.read_operand(offsets::rsa_crt::OUT_RESULT, &mut result[..mod_size]);

        self.disable_pka();
        Ok(())
    }

    // ========================================================================
    // Modular Arithmetic Operations
    // ========================================================================

    /// Compute modular inverse: result = a^(-1) mod n
    pub fn modular_inv(&mut self, a: &[u8], modulus: &[u8], result: &mut [u8]) -> Result<(), Error> {
        let size = modulus.len();

        if a.len() != size || result.len() < size {
            return Err(Error::InvalidSize);
        }

        self.init_pka()?;
        self.set_mode(PkaMode::ModularInv);

        let nb_bits = Self::get_opt_bit_size(size, modulus[0]);
        self.write_ram_word(offsets::modular_inv::IN_NB_BITS, nb_bits);

        self.write_operand(offsets::modular_inv::IN_OP1, a);
        self.write_operand(offsets::modular_inv::IN_OP2_MOD, modulus);

        self.start_and_wait()?;

        self.read_operand(offsets::modular_inv::OUT_RESULT, &mut result[..size]);

        self.disable_pka();
        Ok(())
    }

    /// Compute modular addition: result = (a + b) mod n
    pub fn modular_add(&mut self, a: &[u8], b: &[u8], modulus: &[u8], result: &mut [u8]) -> Result<(), Error> {
        self.arithmetic_op(PkaMode::ModularAdd, a, b, Some(modulus), result)
    }

    /// Compute modular subtraction: result = (a - b) mod n
    pub fn modular_sub(&mut self, a: &[u8], b: &[u8], modulus: &[u8], result: &mut [u8]) -> Result<(), Error> {
        self.arithmetic_op(PkaMode::ModularSub, a, b, Some(modulus), result)
    }

    /// Compute arithmetic multiplication: result = a * b
    pub fn arithmetic_mul(&mut self, a: &[u8], b: &[u8], result: &mut [u8]) -> Result<(), Error> {
        self.arithmetic_op(PkaMode::ArithmeticMul, a, b, None, result)
    }

    // Generic arithmetic operation helper
    fn arithmetic_op(
        &mut self,
        mode: PkaMode,
        a: &[u8],
        b: &[u8],
        modulus: Option<&[u8]>,
        result: &mut [u8],
    ) -> Result<(), Error> {
        let size = a.len();

        if b.len() != size {
            return Err(Error::InvalidSize);
        }

        self.init_pka()?;
        self.set_mode(mode);

        let nb_bits = Self::get_opt_bit_size(size, a[0].max(b[0]));
        self.write_ram_word(offsets::arithmetic::IN_NB_BITS, nb_bits);

        self.write_operand(offsets::arithmetic::IN_OP1, a);
        self.write_operand(offsets::arithmetic::IN_OP2, b);

        if let Some(m) = modulus {
            self.write_operand(offsets::arithmetic::IN_OP3_MOD, m);
        }

        self.start_and_wait()?;

        let result_size = if mode == PkaMode::ArithmeticMul { size * 2 } else { size };
        self.read_operand(offsets::arithmetic::OUT_RESULT, &mut result[..result_size]);

        self.disable_pka();
        Ok(())
    }

    // ========================================================================
    // Internal Helper Functions
    // ========================================================================

    fn init_pka(&mut self) -> Result<(), Error> {
        let p = T::regs();
        let sr_ptr = p.sr().as_ptr() as *const u32;

        // Check if PKA is already enabled and initialized
        let sr_raw = unsafe { sr_ptr.read_volatile() };
        let cr_raw = p.cr().read().0;

        // If already enabled and INITOK is set, skip re-initialization
        if (cr_raw & 0x01) != 0 && (sr_raw & 0x01) != 0 {
            return Ok(());
        }

        // If not enabled, enable it
        if (cr_raw & 0x01) == 0 {
            p.cr().write(|w| w.set_en(true));

            // Wait for EN bit to be set
            let mut timeout: u32 = 0;
            while !p.cr().read().en() {
                timeout += 1;
                if timeout > Self::RAM_ERASE_TIMEOUT {
                    return Err(Error::Timeout);
                }
            }
        }

        // Wait for INITOK (bit 0 of SR) - indicates RAM initialization complete
        let mut timeout: u32 = 0;
        loop {
            let sr_raw = unsafe { sr_ptr.read_volatile() };
            if sr_raw & 0x01 != 0 {
                break;
            }
            timeout += 1;
            if timeout > 1_000_000 {
                return Err(Error::Timeout);
            }
        }

        // Clear any pending flags
        p.clrfr().write(|w| {
            w.set_procendfc(true);
            w.set_ramerrfc(true);
            w.set_addrerrfc(true);
            w.set_operrfc(true);
        });

        Ok(())
    }

    fn set_mode(&mut self, mode: PkaMode) {
        let p = T::regs();
        p.cr().modify(|w| {
            w.set_mode(mode as u8);
            w.set_procendie(false);
            w.set_ramerrie(false);
            w.set_addrerrie(false);
            w.set_operrie(false);
        });
    }

    fn start_and_wait(&mut self) -> Result<(), Error> {
        let p = T::regs();

        p.cr().modify(|w| w.set_start(true));

        let mut timeout: u32 = 0;
        loop {
            let sr = p.sr().read();

            if sr.ramerrf() {
                p.clrfr().write(|w| w.set_ramerrfc(true));
                return Err(Error::RamError);
            }
            if sr.addrerrf() {
                p.clrfr().write(|w| w.set_addrerrfc(true));
                return Err(Error::AddressError);
            }
            if sr.operrf() {
                p.clrfr().write(|w| w.set_operrfc(true));
                return Err(Error::OperationError);
            }
            if sr.procendf() {
                p.clrfr().write(|w| w.set_procendfc(true));
                break;
            }

            timeout += 1;
            if timeout > 10_000_000 {
                return Err(Error::Timeout);
            }
        }

        Ok(())
    }

    fn disable_pka(&mut self) {
        T::regs().cr().modify(|w| w.set_en(false));
    }

    fn get_opt_bit_size(byte_count: usize, msb: u8) -> u32 {
        let position = if msb == 0 { 0 } else { 8 - msb.leading_zeros() };
        ((byte_count as u32 - 1) * 8) + position
    }

    fn write_operand(&mut self, offset: usize, data: &[u8]) {
        let n = data.len();
        let word_count = (n + 3) / 4;

        for index in 0..(n / 4) {
            let i = n - (index * 4);
            let word = (data[i - 1] as u32)
                | ((data[i - 2] as u32) << 8)
                | ((data[i - 3] as u32) << 16)
                | ((data[i - 4] as u32) << 24);
            self.write_ram_word(offset + index * 4, word);
        }

        let remainder = n % 4;
        if remainder > 0 {
            let index = n / 4;
            let word = match remainder {
                1 => data[0] as u32,
                2 => (data[1] as u32) | ((data[0] as u32) << 8),
                3 => (data[2] as u32) | ((data[1] as u32) << 8) | ((data[0] as u32) << 16),
                _ => 0,
            };
            self.write_ram_word(offset + index * 4, word);
        }

        // Terminate with two zero words (matches ST-HAL __PKA_RAM_PARAM_END macro)
        self.write_ram_word(offset + word_count * 4, 0);
        self.write_ram_word(offset + (word_count + 1) * 4, 0);
    }

    fn read_operand(&self, offset: usize, data: &mut [u8]) {
        let n = data.len();

        for index in 0..(n / 4) {
            let word = self.read_ram_word(offset + index * 4);
            let i = n - (index * 4);
            data[i - 1] = (word & 0xFF) as u8;
            data[i - 2] = ((word >> 8) & 0xFF) as u8;
            data[i - 3] = ((word >> 16) & 0xFF) as u8;
            data[i - 4] = ((word >> 24) & 0xFF) as u8;
        }

        let remainder = n % 4;
        if remainder > 0 {
            let index = n / 4;
            let word = self.read_ram_word(offset + index * 4);
            match remainder {
                1 => data[0] = (word & 0xFF) as u8,
                2 => {
                    data[1] = (word & 0xFF) as u8;
                    data[0] = ((word >> 8) & 0xFF) as u8;
                }
                3 => {
                    data[2] = (word & 0xFF) as u8;
                    data[1] = ((word >> 8) & 0xFF) as u8;
                    data[0] = ((word >> 16) & 0xFF) as u8;
                }
                _ => {}
            }
        }
    }

    fn write_ram_word(&mut self, offset: usize, value: u32) {
        let p = T::regs();
        let word_index = offset / 4;
        unsafe {
            let ram_ptr = p.ram(word_index).as_ptr() as *mut u32;
            ram_ptr.write_volatile(value);
        }
    }

    fn read_ram_word(&self, offset: usize) -> u32 {
        let p = T::regs();
        let word_index = offset / 4;
        unsafe {
            let ram_ptr = p.ram(word_index).as_ptr() as *const u32;
            ram_ptr.read_volatile()
        }
    }
}

// ============================================================================
// Instance Traits
// ============================================================================

trait SealedInstance {
    fn regs() -> pac::pka::Pka;
}

/// PKA instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this PKA instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, pka, PKA, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::pka::Pka {
                crate::pac::$inst
            }
        }
    };
);
