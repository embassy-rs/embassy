//! Public Key Accelerator (PKA) - ECDSA Signature Verification
//!
//! This module provides hardware-accelerated ECDSA signature verification using the PKA
//! peripheral. The STM32WBA PKA is a limited-mode accelerator supporting only ECDSA
//! verification (MODE 0x26).
//!
//! # Supported Operations
//!
//! - **ECDSA Verify**: Verify signatures on elliptic curves (P-256, P-384, etc.)
//! - **NOT Supported**: ECDSA signing, RSA, Diffie-Hellman, other ECC operations
//!
//! # Example
//!
//! ```no_run
//! use embassy_stm32::pka::{Pka, EcdsaCurveParams, EcdsaPublicKey, EcdsaSignature};
//!
//! let mut pka = Pka::new_blocking(p.PKA, Irqs);
//!
//! // Use standard NIST P-256 curve
//! let params = EcdsaCurveParams::nist_p256();
//!
//! let public_key = EcdsaPublicKey {
//!     x: &pub_key_x_bytes,  // 32 bytes
//!     y: &pub_key_y_bytes,  // 32 bytes
//! };
//!
//! let signature = EcdsaSignature {
//!     r: &sig_r_bytes,      // 32 bytes
//!     s: &sig_s_bytes,      // 32 bytes
//! };
//!
//! // Verify signature on message hash
//! match pka.ecdsa_verify_blocking(&params, &public_key, &signature, &message_hash) {
//!     Ok(true) => {
//!         // Signature is valid
//!     }
//!     Ok(false) => {
//!         // Signature is invalid
//!     }
//!     Err(e) => {
//!         // Hardware error
//!     }
//! }
//! ```
//!
//! # Supported Curves
//!
//! The PKA hardware can verify signatures on any elliptic curve, but this module
//! currently provides built-in parameters for:
//! - **NIST P-256** (secp256r1) - via `EcdsaCurveParams::nist_p256()`
//!
//! Additional curves can be added by constructing `EcdsaCurveParams` manually with
//! the appropriate domain parameters (p, a, G, n).
//!
//! # Hardware Limitations
//!
//! **STM32WBA PKA v1a (Limited Mode)**:
//! - Only MODE 0x26 (ECDSA verification) is supported
//! - 1334 × 32-bit RAM words (5.2KB internal memory)
//! - No signing, RSA, DH, or other ECC operations
//! - Blocking operation only (no DMA)
//!
//! # Use Cases
//!
//! - Firmware signature verification
//! - Secure boot authentication
//! - Message authenticity verification
//! - Certificate validation
//! - IoT device authentication
//!
//! # Performance
//!
//! Hardware acceleration provides significant benefits:
//! - Faster than software ECC libraries
//! - Constant-time operation (side-channel resistant)
//! - Frees CPU for other tasks
//!
//! # Security Notes
//!
//! - Always hash the message before signing/verification (use SHA-256)
//! - Validate all inputs (curve parameters, public keys, signatures)
//! - Use trusted curve parameters (e.g., NIST P-256)
//! - Signature verification confirms message authenticity and integrity
//!
//! # See Also
//!
//! - [`aes`](crate::aes) - AES symmetric encryption
//! - [`saes`](crate::saes) - Secure AES with hardware keys
//! - [`hash`](crate::hash) - Hash functions for message digests

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac, peripherals, rcc};

static PKA_WAKER: AtomicWaker = AtomicWaker::new();

/// PKA interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let sr = T::regs().sr().read();

        if sr.procendf() {
            // Clear interrupt flags
            T::regs().clrfr().write(|w| w.set_procendfc(true));
            PKA_WAKER.wake();
        }

        if sr.ramerrf() || sr.addrerrf() || sr.operrf() {
            // Clear error flags
            T::regs().clrfr().write(|w| {
                w.set_ramerrfc(true);
                w.set_addrerrfc(true);
                w.set_operrfc(true);
            });
            PKA_WAKER.wake();
        }
    }
}

/// PKA error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// PKA RAM access error
    RamError,
    /// Invalid RAM address
    AddressError,
    /// Operation error
    OperationError,
    /// Invalid parameter size
    InvalidSize,
}

/// ECDSA curve parameters
pub struct EcdsaCurveParams {
    /// Prime field modulus p
    pub p_modulus: &'static [u8],
    /// Curve coefficient a
    pub a_coefficient: &'static [u8],
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
            a_coefficient: &P256_A,
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
const P256_A: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFC,
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

/// PKA driver
pub struct Pka<'d, T: Instance> {
    _peripheral: Peri<'d, T>,
}

impl<'d, T: Instance> Pka<'d, T> {
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

    /// Verify an ECDSA signature
    /// Returns Ok(true) if signature is valid, Ok(false) if invalid, Err on error
    pub fn ecdsa_verify_blocking(
        &mut self,
        curve_params: &EcdsaCurveParams,
        public_key: &EcdsaPublicKey,
        signature: &EcdsaSignature,
        message_hash: &[u8],
    ) -> Result<bool, Error> {
        let p = T::regs();

        // Validate input sizes (for P-256, all should be 32 bytes)
        let op_size = curve_params.p_modulus.len();
        if public_key.x.len() != op_size
            || public_key.y.len() != op_size
            || signature.r.len() != op_size
            || signature.s.len() != op_size
            || message_hash.len() != op_size
        {
            return Err(Error::InvalidSize);
        }

        // Clear any previous operation
        p.cr().modify(|w| w.set_en(false));

        // Enable PKA
        p.cr().modify(|w| w.set_en(true));

        // Set mode to ECDSA verification (0x26)
        // Note: The mode value is written directly to the register
        p.cr().modify(|w| w.set_mode(0x26));

        // Load operands into PKA RAM
        // For ECDSA verify, the RAM layout is defined by the PKA peripheral
        // Typical offsets (may vary by chip):
        const OFFSET_N: usize = 0x400; // Order
        const OFFSET_P: usize = 0x4B8; // Modulus
        const OFFSET_A: usize = 0x570; // Coefficient a
        const OFFSET_GX: usize = 0x628; // Generator x
        const OFFSET_GY: usize = 0x6E0; // Generator y
        const OFFSET_QX: usize = 0x798; // Public key x
        const OFFSET_QY: usize = 0x850; // Public key y
        const OFFSET_R: usize = 0x908; // Signature r
        const OFFSET_S: usize = 0x9C0; // Signature s
        const OFFSET_HASH: usize = 0xDE8; // Message hash

        // Write operands (big-endian format)
        self.write_operand(OFFSET_N, curve_params.order);
        self.write_operand(OFFSET_P, curve_params.p_modulus);
        self.write_operand(OFFSET_A, curve_params.a_coefficient);
        self.write_operand(OFFSET_GX, curve_params.generator_x);
        self.write_operand(OFFSET_GY, curve_params.generator_y);
        self.write_operand(OFFSET_QX, public_key.x);
        self.write_operand(OFFSET_QY, public_key.y);
        self.write_operand(OFFSET_R, signature.r);
        self.write_operand(OFFSET_S, signature.s);
        self.write_operand(OFFSET_HASH, message_hash);

        // Start operation
        p.cr().modify(|w| w.set_start(true));

        // Wait for completion
        while !p.sr().read().procendf() {
            // Check for errors
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
        }

        // Clear completion flag
        p.clrfr().write(|w| w.set_procendfc(true));

        // Check result - for ECDSA verify, result is at offset 0x5B0
        // Result = 0xD60D if signature is valid, otherwise invalid
        const RESULT_OFFSET: usize = 0x5B0;
        let result_word = self.read_ram_word(RESULT_OFFSET);

        // Disable PKA
        p.cr().modify(|w| w.set_en(false));

        // Check if signature is valid
        Ok(result_word == 0xD60D)
    }

    /// Write an operand to PKA RAM (big-endian)
    fn write_operand(&mut self, offset: usize, data: &[u8]) {
        // PKA expects data in little-endian word order but big-endian byte order within words
        // First write the length
        self.write_ram_word(offset, data.len() as u32);

        // Write data words (convert big-endian bytes to little-endian words)
        let mut word_offset = offset + 8; // Skip length field (2 words)
        for chunk in data.rchunks(4) {
            let mut word_bytes = [0u8; 4];
            word_bytes[4 - chunk.len()..].copy_from_slice(chunk);
            let word = u32::from_be_bytes(word_bytes);
            self.write_ram_word(word_offset, word);
            word_offset += 4;
        }
    }

    /// Write a 32-bit word to PKA RAM
    fn write_ram_word(&mut self, offset: usize, value: u32) {
        let p = T::regs();
        unsafe {
            let ram_base = p.ram(0).as_ptr() as *mut u32;
            let addr = ram_base.byte_add(offset);
            addr.write_volatile(value);
        }
    }

    /// Read a 32-bit word from PKA RAM
    fn read_ram_word(&mut self, offset: usize) -> u32 {
        let p = T::regs();
        unsafe {
            let ram_base = p.ram(0).as_ptr() as *const u32;
            let addr = ram_base.byte_add(offset);
            addr.read_volatile()
        }
    }
}

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
