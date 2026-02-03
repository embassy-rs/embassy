//! Advanced Encryption Standard (AES) hardware accelerator
//!
//! This module provides support for the AES v3b hardware accelerator peripheral
//! found on STM32WBA series microcontrollers.
//!
//! # Supported Cipher Modes
//!
//! | Mode | Padding | Auth | Use Case |
//! |------|---------|------|----------|
//! | ECB  | Required | No  | Keys only (not recommended for data) |
//! | CBC  | Required | No  | File/disk encryption |
//! | CTR  | No | No  | Streaming data, random access |
//! | GCM  | No | Yes | **Recommended** - Modern applications |
//! | CCM  | No | Yes | Resource-constrained devices |
//!
//! # Key Sizes
//!
//! - 128-bit (16 bytes)
//! - 256-bit (32 bytes)
//! - Note: 192-bit keys are NOT supported on this hardware
//!
//! # Examples
//!
//! ## Basic ECB Mode (Block Cipher)
//!
//! ```no_run
//! use embassy_stm32::aes::{Aes, AesEcb, Direction};
//! use embassy_stm32::{bind_interrupts, peripherals};
//!
//! bind_interrupts!(struct Irqs {
//!     AES => embassy_stm32::aes::InterruptHandler<peripherals::AES>;
//! });
//!
//! let key = [0u8; 16];  // 128-bit key
//! let cipher = AesEcb::new(&key);
//!
//! let mut aes = Aes::new_blocking(p.AES, Irqs);
//! let mut ctx = aes.start(&cipher, Direction::Encrypt);
//!
//! let plaintext = [0u8; 16];
//! let mut ciphertext = [0u8; 16];
//! aes.payload_blocking(&mut ctx, &plaintext, &mut ciphertext, true);
//! aes.finish_blocking(ctx);
//! ```
//!
//! ## CBC Mode (With IV)
//!
//! ```no_run
//! use embassy_stm32::aes::{Aes, AesCbc, Direction};
//!
//! let key = [0u8; 16];
//! let iv = [0u8; 16];  // Random IV, unique per message
//! let cipher = AesCbc::new(&key, &iv);
//!
//! let mut ctx = aes.start(&cipher, Direction::Encrypt);
//! // Process multiple blocks
//! aes.payload_blocking(&mut ctx, &plaintext, &mut ciphertext, true);
//! aes.finish_blocking(ctx);
//! ```
//!
//! ## CTR Mode (Stream Cipher - No Padding)
//!
//! ```no_run
//! use embassy_stm32::aes::{Aes, AesCtr, Direction};
//!
//! let key = [0u8; 16];
//! let counter = [0u8; 16];  // Nonce + counter
//! let cipher = AesCtr::new(&key, &counter);
//!
//! let mut ctx = aes.start(&cipher, Direction::Encrypt);
//! // Can process any length data (no padding needed)
//! let partial_data = [0u8; 13]; // Not block-aligned - OK for CTR!
//! let mut output = [0u8; 13];
//! aes.payload_blocking(&mut ctx, &partial_data, &mut output, true);
//! aes.finish_blocking(ctx);
//! ```
//!
//! ## GCM Mode (Authenticated Encryption - Recommended)
//!
//! ```no_run
//! use embassy_stm32::aes::{Aes, AesGcm, Direction};
//!
//! let key = [0u8; 16];
//! let iv = [0u8; 12];  // 96-bit nonce (12 bytes)
//! let cipher = AesGcm::new(&key, &iv);
//!
//! let mut ctx = aes.start(&cipher, Direction::Encrypt);
//!
//! // Process Additional Authenticated Data (AAD) - optional
//! let aad = b"metadata that will be authenticated but not encrypted";
//! aes.aad_blocking(&mut ctx, aad, true);
//!
//! // Encrypt payload
//! aes.payload_blocking(&mut ctx, &plaintext, &mut ciphertext, true);
//!
//! // Get authentication tag
//! if let Ok(Some(tag)) = aes.finish_blocking(ctx) {
//!     // Send tag with ciphertext for verification
//! }
//! ```
//!
//! # Security Best Practices
//!
//! ## Key Management
//! - Use hardware RNG for key generation
//! - Never hardcode keys in production
//! - Consider using SAES hardware key derivation
//! - Never reuse keys inappropriately
//!
//! ## IV/Nonce Requirements
//! - **CBC**: Random, unique per message
//! - **CTR**: Must NEVER repeat with same key (use counter)
//! - **GCM**: 96-bit (12 bytes), unique per message
//! - **CRITICAL**: IV reuse is catastrophic in CTR/GCM modes
//!
//! ## Mode Selection
//! - **Use GCM** for new applications (provides authentication)
//! - **Use CTR** for streaming or arbitrary-length data
//! - **Avoid ECB** for anything except encrypting random keys
//! - **CBC/CTR alone** don't provide authentication - consider GCM or add HMAC
//!
//! # Hardware Capabilities
//!
//! **AES v3b (STM32WBA)**:
//! - Block size: 16 bytes (128 bits)
//! - Key sizes: 128-bit, 256-bit
//! - DMA support: Yes (async mode)
//! - Interrupt support: Yes
//! - Suspend/resume: Yes (for GCM/CCM)
//!
//! # Performance
//!
//! Hardware acceleration provides significant speed improvements over software:
//! - ~10-20× faster than pure software implementation
//! - Constant-time operation (side-channel resistant)
//! - Low CPU overhead
//!
//! # See Also
//!
//! - [`saes`](crate::saes) - Secure AES with hardware key derivation
//! - [`pka`](crate::pka) - Public Key Accelerator (ECDSA verification)
//! - Examples: `examples/stm32wba/src/bin/aes_*.rs`

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::ChannelAndRequest;
use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::{interrupt, pac, peripherals, rcc};

const AES_BLOCK_SIZE: usize = 16; // 128 bits

static AES_WAKER: AtomicWaker = AtomicWaker::new();

/// AES interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let sr = T::regs().sr().read();

        // Wake on completion flag
        if sr.ccf() {
            // Clear all interrupt flags by writing 1s to ICR
            T::regs().icr().write(|w| w.0 = 0xFFFF_FFFF);
            AES_WAKER.wake();
        }

        // Clear error flags if any
        if sr.rderr() || sr.wrerr() {
            T::regs().icr().write(|w| w.0 = 0xFFFF_FFFF);
        }
    }
}

/// AES error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Invalid key size
    KeyError,
    /// Read error - unexpected output read during computation
    ReadError,
    /// Write error - unexpected input write during output phase
    WriteError,
    /// Invalid configuration
    ConfigError,
}

/// AES cipher direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Encryption mode
    Encrypt = 0,
    /// Decryption mode (MODE = 2)
    Decrypt = 2,
}

/// AES key size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum KeySize {
    /// 128-bit key
    Bits128 = 0,
    /// 256-bit key
    Bits256 = 1,
}

/// This trait encapsulates all cipher-specific behavior.
pub trait Cipher<'c> {
    /// Processing block size (always 16 bytes for AES).
    const BLOCK_SIZE: usize = AES_BLOCK_SIZE;

    /// Indicates whether the cipher requires the application to provide padding.
    const REQUIRES_PADDING: bool = false;

    /// Returns the symmetric key.
    fn key(&self) -> &[u8];

    /// Returns the initialization vector.
    fn iv(&self) -> &[u8];

    /// Returns the key size.
    fn key_size(&self) -> KeySize {
        match self.key().len() {
            16 => KeySize::Bits128,
            32 => KeySize::Bits256,
            _ => panic!("Invalid key size"),
        }
    }

    /// Sets the cipher mode (CHMOD field).
    fn set_mode(&self, p: pac::aes::Aes);

    /// Returns the data type setting for this cipher mode.
    /// - 0 = NO_SWAP (32-bit words, no swapping) - Default for all modes
    ///
    /// Note: The ST HAL uses different DATATYPE values (BYTE_SWAP for CBC, BIT_SWAP for CTR)
    /// with pre-swapped test vectors. This driver uses NO_SWAP consistently with big-endian
    /// byte conversion (from_be_bytes/to_be_bytes) for direct NIST test vector compatibility.
    fn datatype(&self) -> u8 {
        0 // NO_SWAP for all modes - handles NIST vectors correctly with from_be_bytes/to_be_bytes
    }

    /// Performs any key preparation within the processor, if necessary.
    fn prepare_key(&self, _p: pac::aes::Aes, _dir: Direction) {}

    /// Performs any cipher-specific initialization.
    fn init_phase_blocking<T: Instance, M: Mode>(&self, _p: pac::aes::Aes, _aes: &Aes<T, M>) {}

    /// Performs any cipher-specific initialization (async).
    async fn init_phase<T: Instance>(&self, _p: pac::aes::Aes, _aes: &mut Aes<'_, T, Async>) {}

    /// Called prior to processing the last data block for cipher-specific operations.
    fn pre_final(&self, _p: pac::aes::Aes, _dir: Direction, _padding_len: usize) -> [u32; 4] {
        [0; 4]
    }

    /// Called after processing the last data block for cipher-specific operations.
    fn post_final_blocking<T: Instance, M: Mode>(
        &self,
        _p: pac::aes::Aes,
        _aes: &Aes<T, M>,
        _dir: Direction,
        _int_data: &mut [u8; AES_BLOCK_SIZE],
        _temp1: [u32; 4],
        _padding_mask: [u8; 16],
    ) {
    }

    /// Called after processing the last data block for cipher-specific operations (async).
    async fn post_final<T: Instance>(
        &self,
        _p: pac::aes::Aes,
        _aes: &mut Aes<'_, T, Async>,
        _dir: Direction,
        _int_data: &mut [u8; AES_BLOCK_SIZE],
        _temp1: [u32; 4],
        _padding_mask: [u8; 16],
    ) {
    }

    /// Returns the AAD header block as required by the cipher (for authenticated modes).
    fn get_header_block(&self) -> &[u8] {
        [0; 0].as_slice()
    }

    /// Indicates whether this cipher mode uses GCM/CCM phases (init, header, payload, final).
    fn uses_gcm_phases(&self) -> bool {
        false
    }

    /// Indicates whether this is CCM mode (which has different final phase handling).
    /// CCM doesn't use a length block in the final phase, unlike GCM.
    fn is_ccm_mode(&self) -> bool {
        false
    }

    /// Returns the pre-computed B0 block for CCM mode (None for other modes).
    fn ccm_b0(&self) -> Option<&[u8; 16]> {
        None
    }

    /// Returns the formatted AAD length prefix for CCM mode.
    /// CCM requires AAD to be prefixed with its length encoding.
    fn ccm_format_aad_header(&self, aad_len: usize) -> ([u8; 10], usize) {
        // Default implementation - returns empty for non-CCM modes
        // Format: if aad_len < 2^16-2^8: 2 bytes
        //         if aad_len < 2^32: 0xFFFE + 4 bytes
        //         else: 0xFFFF + 8 bytes
        let mut header = [0u8; 10];
        let len = if aad_len == 0 {
            0
        } else if aad_len < (1 << 16) - (1 << 8) {
            header[0] = (aad_len >> 8) as u8;
            header[1] = aad_len as u8;
            2
        } else if aad_len < (1u64 << 32) as usize {
            header[0] = 0xFF;
            header[1] = 0xFE;
            header[2..6].copy_from_slice(&(aad_len as u32).to_be_bytes());
            6
        } else {
            header[0] = 0xFF;
            header[1] = 0xFF;
            header[2..10].copy_from_slice(&(aad_len as u64).to_be_bytes());
            10
        };
        (header, len)
    }
}

/// This trait enables restriction of ciphers to specific key sizes.
pub trait CipherSized {}

/// This trait enables restriction of initialization vectors to sizes compatible with a cipher mode.
pub trait IVSized {}

/// This trait enables restriction of a header phase to authenticated ciphers only.
pub trait CipherAuthenticated<const TAG_SIZE: usize> {
    /// Defines the authentication tag size.
    const TAG_SIZE: usize = TAG_SIZE;
}

/// AES-ECB Cipher Mode
pub struct AesEcb<'c, const KEY_SIZE: usize> {
    iv: &'c [u8; 0],
    key: &'c [u8; KEY_SIZE],
}

impl<'c, const KEY_SIZE: usize> AesEcb<'c, KEY_SIZE> {
    /// Constructs a new AES-ECB cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE]) -> Self {
        Self { key, iv: &[0; 0] }
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for AesEcb<'c, KEY_SIZE> {
    const REQUIRES_PADDING: bool = true;

    fn key(&self) -> &[u8] {
        self.key
    }

    fn iv(&self) -> &[u8] {
        self.iv
    }

    fn set_mode(&self, p: pac::aes::Aes) {
        // ECB mode: CHMOD = 0
        p.cr().modify(|w| {
            w.set_chmod(pac::aes::vals::Chmod::from_bits(0));
        });
    }

    fn prepare_key(&self, p: pac::aes::Aes, dir: Direction) {
        // For ECB decryption, need to prepare key (RM Section 26.4.6 steps 2-7)
        if dir == Direction::Decrypt {
            // Step 2: Set MODE to key derivation (MODE[1:0] = 0x1)
            p.cr().modify(|w| w.set_mode(pac::aes::vals::Mode::from_bits(1)));
            // Step 5: Enable AES - peripheral starts key preparation
            p.cr().modify(|w| w.set_en(true));
            // Step 6: Wait for CCF flag
            while !p.sr().read().ccf() {}
            // Step 7: Clear CCF flag - AES disables automatically
            p.icr().write(|w| w.0 = 0xFFFF_FFFF);
            // Note: Peripheral is now disabled, caller will reconfigure and re-enable
        }
    }
}

impl<'c> CipherSized for AesEcb<'c, { 128 / 8 }> {}
impl<'c> CipherSized for AesEcb<'c, { 256 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for AesEcb<'c, KEY_SIZE> {}

/// AES-CBC Cipher Mode
pub struct AesCbc<'c, const KEY_SIZE: usize> {
    iv: &'c [u8; 16],
    key: &'c [u8; KEY_SIZE],
}

impl<'c, const KEY_SIZE: usize> AesCbc<'c, KEY_SIZE> {
    /// Constructs a new AES-CBC cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; 16]) -> Self {
        Self { key, iv }
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for AesCbc<'c, KEY_SIZE> {
    const REQUIRES_PADDING: bool = true;

    fn key(&self) -> &[u8] {
        self.key
    }

    fn iv(&self) -> &[u8] {
        self.iv
    }

    fn set_mode(&self, p: pac::aes::Aes) {
        // CBC mode: CHMOD = 1
        p.cr().modify(|w| {
            w.set_chmod(pac::aes::vals::Chmod::from_bits(1));
        });
    }

    // Uses default datatype() = 0 (NO_SWAP) for NIST vector compatibility

    fn prepare_key(&self, p: pac::aes::Aes, dir: Direction) {
        // For CBC/ECB decryption, need to prepare key (RM Section 26.4.6 steps 2-7)
        if dir == Direction::Decrypt {
            // Step 2: Set MODE to key derivation (MODE[1:0] = 0x1)
            p.cr().modify(|w| w.set_mode(pac::aes::vals::Mode::from_bits(1)));
            // Step 5: Enable AES - peripheral starts key preparation
            p.cr().modify(|w| w.set_en(true));
            // Step 6: Wait for CCF flag
            while !p.sr().read().ccf() {}
            // Step 7: Clear CCF flag - AES disables automatically
            p.icr().write(|w| w.0 = 0xFFFF_FFFF);
            // Note: Peripheral is now disabled, caller will reconfigure and re-enable
        }
    }
}

impl<'c> CipherSized for AesCbc<'c, { 128 / 8 }> {}
impl<'c> CipherSized for AesCbc<'c, { 256 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for AesCbc<'c, KEY_SIZE> {}

/// AES-CTR Cipher Mode
pub struct AesCtr<'c, const KEY_SIZE: usize> {
    iv: &'c [u8; 16],
    key: &'c [u8; KEY_SIZE],
}

impl<'c, const KEY_SIZE: usize> AesCtr<'c, KEY_SIZE> {
    /// Constructs a new AES-CTR cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; 16]) -> Self {
        Self { key, iv }
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for AesCtr<'c, KEY_SIZE> {
    const REQUIRES_PADDING: bool = false; // Stream cipher, no padding needed

    fn key(&self) -> &[u8] {
        self.key
    }

    fn iv(&self) -> &[u8] {
        self.iv
    }

    fn set_mode(&self, p: pac::aes::Aes) {
        // CTR mode: CHMOD = 2
        p.cr().modify(|w| {
            w.set_chmod(pac::aes::vals::Chmod::from_bits(2));
        });
    }

    // Uses default datatype() = 0 (NO_SWAP) for NIST vector compatibility
}

impl<'c> CipherSized for AesCtr<'c, { 128 / 8 }> {}
impl<'c> CipherSized for AesCtr<'c, { 256 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for AesCtr<'c, KEY_SIZE> {}

/// AES-GCM Cipher Mode
pub struct AesGcm<'c, const KEY_SIZE: usize> {
    key: &'c [u8; KEY_SIZE],
    iv: [u8; 16],
}

impl<'c, const KEY_SIZE: usize> AesGcm<'c, KEY_SIZE> {
    /// Constructs a new AES-GCM cipher for a cryptographic operation.
    /// The IV should be 12 bytes long (96 bits).
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; 12]) -> Self {
        let mut iv_full = [0u8; 16];
        iv_full[..12].copy_from_slice(iv);
        iv_full[15] = 2; // Initial counter value
        Self { key, iv: iv_full }
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for AesGcm<'c, KEY_SIZE> {
    const REQUIRES_PADDING: bool = false;

    fn key(&self) -> &[u8] {
        self.key
    }

    fn iv(&self) -> &[u8] {
        &self.iv
    }

    fn set_mode(&self, p: pac::aes::Aes) {
        // GCM mode: CHMOD = 3
        p.cr().modify(|w| {
            w.set_chmod(pac::aes::vals::Chmod::from_bits(3));
        });
    }

    fn init_phase_blocking<T: Instance, M: Mode>(&self, p: pac::aes::Aes, _aes: &Aes<T, M>) {
        // GCMPH was already set to 0 in start() before key loading
        // Enable EN to start hash key (H) calculation (RM step 6)
        p.cr().modify(|w| w.set_en(true));

        // Wait for CCF (hash key calculation complete - RM step 7)
        while !p.sr().read().ccf() {}

        // Clear completion flag (RM step 8)
        p.icr().write(|w| w.0 = 0xFFFF_FFFF);

        // ST HAL does NOT disable EN after init phase
        // Leave peripheral enabled for next phase transition
    }

    async fn init_phase<T: Instance>(&self, p: pac::aes::Aes, _aes: &mut Aes<'_, T, Async>) {
        // Set GCM phase to init (GCMPH = 0)
        p.cr().modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::from_bits(0)));
        p.cr().modify(|w| w.set_en(true));
        // Wait for completion
        poll_fn(|cx| {
            if p.sr().read().ccf() {
                Poll::Ready(())
            } else {
                AES_WAKER.register(cx.waker());
                // Re-check after registering waker
                if p.sr().read().ccf() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        })
        .await;
    }

    fn pre_final(&self, p: pac::aes::Aes, _dir: Direction, padding_len: usize) -> [u32; 4] {
        // Set number of padding bytes for partial block
        if padding_len > 0 {
            p.cr().modify(|w| w.set_npblb(padding_len as u8));
        }
        [0; 4]
    }

    fn uses_gcm_phases(&self) -> bool {
        true
    }
}

impl<'c> CipherSized for AesGcm<'c, { 128 / 8 }> {}
impl<'c> CipherSized for AesGcm<'c, { 256 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for AesGcm<'c, KEY_SIZE> {}
impl<'c, const KEY_SIZE: usize> CipherAuthenticated<16> for AesGcm<'c, KEY_SIZE> {}

/// AES-GMAC Cipher Mode (Galois Message Authentication Code)
///
/// GMAC provides message authentication without encryption. Use this when you need
/// to authenticate data integrity without confidentiality - the data remains in
/// plaintext but any tampering will be detected.
///
/// # Use Cases
/// - Authenticating packet headers in network protocols
/// - Verifying integrity of publicly-readable metadata
/// - Any scenario requiring authentication without encryption
///
/// # Example
/// ```no_run
/// let key = [0u8; 16];
/// let iv = [0u8; 12];  // 96-bit nonce
/// let cipher = AesGmac::new(&key, &iv);
///
/// let mut ctx = aes.start(&cipher, Direction::Encrypt);
/// aes.aad_blocking(&mut ctx, &header_data, true);
/// if let Ok(Some(tag)) = aes.finish_blocking(ctx) {
///     // Use tag to verify integrity
/// }
/// ```
pub struct AesGmac<'c, const KEY_SIZE: usize> {
    key: &'c [u8; KEY_SIZE],
    iv: [u8; 16],
}

impl<'c, const KEY_SIZE: usize> AesGmac<'c, KEY_SIZE> {
    /// Constructs a new AES-GMAC cipher for message authentication.
    /// The IV should be 12 bytes long (96 bits) and unique per message.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; 12]) -> Self {
        let mut iv_full = [0u8; 16];
        iv_full[..12].copy_from_slice(iv);
        iv_full[15] = 2; // Initial counter value (same as GCM)
        Self { key, iv: iv_full }
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for AesGmac<'c, KEY_SIZE> {
    const REQUIRES_PADDING: bool = false;

    fn key(&self) -> &[u8] {
        self.key
    }

    fn iv(&self) -> &[u8] {
        &self.iv
    }

    fn set_mode(&self, p: pac::aes::Aes) {
        // GMAC uses the same hardware mode as GCM: CHMOD = 3
        p.cr().modify(|w| {
            w.set_chmod(pac::aes::vals::Chmod::from_bits(3));
        });
    }

    fn init_phase_blocking<T: Instance, M: Mode>(&self, p: pac::aes::Aes, _aes: &Aes<T, M>) {
        // Same init phase as GCM - compute hash key H
        p.cr().modify(|w| w.set_en(true));
        while !p.sr().read().ccf() {}
        p.icr().write(|w| w.0 = 0xFFFF_FFFF);
    }

    async fn init_phase<T: Instance>(&self, p: pac::aes::Aes, _aes: &mut Aes<'_, T, Async>) {
        p.cr().modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::from_bits(0)));
        p.cr().modify(|w| w.set_en(true));
        poll_fn(|cx| {
            if p.sr().read().ccf() {
                Poll::Ready(())
            } else {
                AES_WAKER.register(cx.waker());
                if p.sr().read().ccf() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        })
        .await;
    }

    fn uses_gcm_phases(&self) -> bool {
        true
    }
}

impl<'c> CipherSized for AesGmac<'c, { 128 / 8 }> {}
impl<'c> CipherSized for AesGmac<'c, { 256 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for AesGmac<'c, KEY_SIZE> {}
impl<'c, const KEY_SIZE: usize> CipherAuthenticated<16> for AesGmac<'c, KEY_SIZE> {}

/// AES-CCM Cipher Mode (Counter with CBC-MAC)
pub struct AesCcm<'c, const KEY_SIZE: usize, const IV_SIZE: usize, const TAG_SIZE: usize> {
    key: &'c [u8; KEY_SIZE],
    iv: [u8; 16],
    #[allow(dead_code)] // Stored for potential future use in verification
    aad_len: usize,
    #[allow(dead_code)] // Stored for potential future use in verification
    payload_len: usize,
}

impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize, const TAG_SIZE: usize> AesCcm<'c, KEY_SIZE, IV_SIZE, TAG_SIZE> {
    /// Constructs a new AES-CCM cipher for a cryptographic operation.
    /// - `key`: The encryption key (16 or 32 bytes)
    /// - `iv`: The nonce/IV (7-13 bytes recommended, typically 12 bytes)
    /// - `aad_len`: Length of additional authenticated data (known in advance)
    /// - `payload_len`: Length of payload data (known in advance)
    ///
    /// Note: CCM requires knowing the data lengths in advance.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; IV_SIZE], aad_len: usize, payload_len: usize) -> Self {
        // Validate IV size (7-13 bytes per NIST SP 800-38C)
        assert!(IV_SIZE >= 7 && IV_SIZE <= 13, "CCM IV must be 7-13 bytes");
        // Validate tag size (4, 6, 8, 10, 12, 14, or 16 bytes)
        assert!(
            TAG_SIZE >= 4 && TAG_SIZE <= 16 && TAG_SIZE % 2 == 0,
            "CCM tag must be 4-16 bytes and even"
        );

        let mut iv_full = [0u8; 16];
        // Format B0 block for CCM
        let l = 15 - IV_SIZE; // l = size of length field
        iv_full[0] = ((l - 1) as u8) | ((((TAG_SIZE - 2) / 2) as u8) << 3);
        if aad_len > 0 {
            iv_full[0] |= 0x40; // Set Adata flag
        }
        iv_full[1..1 + IV_SIZE].copy_from_slice(iv);

        // Encode payload length in the last l bytes (as big-endian)
        // Use u64 to ensure consistent handling on 32-bit and 64-bit platforms
        let payload_bytes = (payload_len as u64).to_be_bytes();
        let offset = 16 - l;
        iv_full[offset..].copy_from_slice(&payload_bytes[8 - l..]);

        Self {
            key,
            iv: iv_full,
            aad_len,
            payload_len,
        }
    }
}

impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize, const TAG_SIZE: usize> Cipher<'c>
    for AesCcm<'c, KEY_SIZE, IV_SIZE, TAG_SIZE>
{
    const REQUIRES_PADDING: bool = false;

    fn key(&self) -> &[u8] {
        self.key
    }

    fn iv(&self) -> &[u8] {
        &self.iv
    }

    fn set_mode(&self, p: pac::aes::Aes) {
        // CCM mode: CHMOD = 4 (0b100) per RM0493
        p.cr().modify(|w| {
            w.set_chmod(pac::aes::vals::Chmod::from_bits(4));
        });
    }

    fn init_phase_blocking<T: Instance, M: Mode>(&self, p: pac::aes::Aes, _aes: &Aes<T, M>) {
        // GCMPH was already set to 0 in start() before key loading
        // Enable EN to start initialization
        p.cr().modify(|w| w.set_en(true));
        // Wait for CCF
        while !p.sr().read().ccf() {}
        // Clear completion flag
        p.icr().write(|w| w.0 = 0xFFFF_FFFF);
        // ST HAL does NOT disable EN after init phase
        // Leave peripheral enabled for next phase transition
    }

    async fn init_phase<T: Instance>(&self, p: pac::aes::Aes, _aes: &mut Aes<'_, T, Async>) {
        // Set CCM phase to init (GCMPH = 0)
        p.cr().modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::from_bits(0)));
        p.cr().modify(|w| w.set_en(true));
        // Wait for completion
        poll_fn(|cx| {
            if p.sr().read().ccf() {
                Poll::Ready(())
            } else {
                AES_WAKER.register(cx.waker());
                // Re-check after registering waker
                if p.sr().read().ccf() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        })
        .await;
    }

    fn pre_final(&self, p: pac::aes::Aes, _dir: Direction, padding_len: usize) -> [u32; 4] {
        // Set number of padding bytes for partial block
        if padding_len > 0 {
            p.cr().modify(|w| w.set_npblb(padding_len as u8));
        }
        [0; 4]
    }

    fn uses_gcm_phases(&self) -> bool {
        true
    }

    fn is_ccm_mode(&self) -> bool {
        true
    }

    fn ccm_b0(&self) -> Option<&[u8; 16]> {
        Some(&self.iv)
    }
}

impl<'c, const IV_SIZE: usize, const TAG_SIZE: usize> CipherSized for AesCcm<'c, { 128 / 8 }, IV_SIZE, TAG_SIZE> {}
impl<'c, const IV_SIZE: usize, const TAG_SIZE: usize> CipherSized for AesCcm<'c, { 256 / 8 }, IV_SIZE, TAG_SIZE> {}
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize, const TAG_SIZE: usize> IVSized
    for AesCcm<'c, KEY_SIZE, IV_SIZE, TAG_SIZE>
{
}
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize, const TAG_SIZE: usize> CipherAuthenticated<TAG_SIZE>
    for AesCcm<'c, KEY_SIZE, IV_SIZE, TAG_SIZE>
{
}

/// Stores the state of the AES peripheral for suspending/resuming operations.
#[derive(Clone)]
pub struct Context<'c, C: Cipher<'c>> {
    /// The cipher configuration
    pub cipher: &'c C,
    /// Encryption or decryption direction
    pub dir: Direction,
    /// Whether the last block has been processed
    pub last_block_processed: bool,
    /// Whether this is a GCM/CCM authenticated mode
    pub is_gcm_ccm: bool,
    // For GCM/CCM authenticated modes
    /// Whether the header (AAD) has been processed
    pub header_processed: bool,
    /// Total length of additional authenticated data
    pub header_len: u64,
    /// Total length of payload data
    pub payload_len: u64,
    /// Buffer for partial AAD blocks
    pub aad_buffer: [u8; 16],
    /// Number of bytes in AAD buffer
    pub aad_buffer_len: usize,
    // Hardware state
    /// Control register state
    pub cr: u32,
    /// Initialization vector state
    pub iv: [u32; 4],
    /// Suspend registers for GCM/CCM
    pub suspr: [u32; 8],
}

/// AES driver.
pub struct Aes<'d, T: Instance, M: Mode> {
    _peripheral: Peri<'d, T>,
    _phantom: PhantomData<M>,
    #[allow(dead_code)] // Reserved for future async/DMA implementation
    dma_in: Option<ChannelAndRequest<'d>>,
    #[allow(dead_code)] // Reserved for future async/DMA implementation
    dma_out: Option<ChannelAndRequest<'d>>,
}

impl<'d, T: Instance> Aes<'d, T, Blocking> {
    /// Instantiates, resets, and enables the AES peripheral.
    pub fn new_blocking(
        peripheral: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        let instance = Self {
            _peripheral: peripheral,
            _phantom: PhantomData,
            dma_in: None,
            dma_out: None,
        };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }
}

impl<'d, T: Instance> Aes<'d, T, Async> {
    /// Instantiates, resets, and enables the AES peripheral with DMA support.
    pub fn new<D1: DmaIn<T>, D2: DmaOut<T>>(
        peripheral: Peri<'d, T>,
        dma_in: Peri<'d, D1>,
        dma_out: Peri<'d, D2>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        let instance = Self {
            _peripheral: peripheral,
            _phantom: PhantomData,
            dma_in: new_dma!(dma_in, _irq),
            dma_out: new_dma!(dma_out, _irq),
        };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }
}

impl<'d, T: Instance, M: Mode> Aes<'d, T, M> {
    /// Starts a new cipher operation and returns the context.
    pub fn start<'c, C>(&mut self, cipher: &'c C, dir: Direction) -> Context<'c, C>
    where
        C: Cipher<'c> + CipherSized + IVSized,
    {
        let p = T::regs();

        // Disable the peripheral
        p.cr().modify(|w| w.set_en(false));

        // Configure data type based on cipher mode (NO_SWAP, BYTE_SWAP, or BIT_SWAP)
        p.cr()
            .modify(|w| w.set_datatype(pac::aes::vals::Datatype::from_bits(cipher.datatype())));

        // Configure key size (false = 128-bit, true = 256-bit)
        let keysize = cipher.key_size();
        p.cr().modify(|w| w.set_keysize(keysize == KeySize::Bits256));

        // Set direction
        p.cr()
            .modify(|w| w.set_mode(pac::aes::vals::Mode::from_bits(dir as u8)));

        // Set cipher mode
        cipher.set_mode(p);

        // Check if this is an authenticated mode that uses GCM/CCM phases
        let is_gcm_ccm = cipher.uses_gcm_phases();

        // For GCM/CCM, set GCMPH to init BEFORE loading key (per RM step 2)
        if is_gcm_ccm {
            p.cr().modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::from_bits(0)));
        }

        // For ECB/CBC decryption, need key preparation first
        let needs_key_prep = dir == Direction::Decrypt && !is_gcm_ccm && cipher.key().len() > 0;

        if is_gcm_ccm {
            // GCM/CCM mode (RM 26.4.8): Load key first, then IV
            self.load_key(cipher.key());
            if !cipher.iv().is_empty() {
                self.load_iv(cipher.iv());
            }
        } else if needs_key_prep {
            // ECB/CBC decryption (RM 26.4.6): Key prep, then IV
            self.load_key(cipher.key());
            cipher.prepare_key(p, dir);

            // Step 8: Select cipher mode and decryption mode (keep other params)
            p.cr()
                .modify(|w| w.set_mode(pac::aes::vals::Mode::from_bits(dir as u8)));
            cipher.set_mode(p); // Set CHMOD

            // Step 9: Write IV (for CBC decryption, AFTER key preparation)
            if !cipher.iv().is_empty() {
                self.load_iv(cipher.iv());
            }
        } else {
            // ECB/CBC/CTR encryption (RM 26.4.5): IV first, then key
            if !cipher.iv().is_empty() {
                self.load_iv(cipher.iv());
            }
            self.load_key(cipher.key());
        }

        // Perform init phase for GCM/CCM modes (RM step 6-8)
        // This calculates the hash key H needed for GCM authentication
        if is_gcm_ccm {
            cipher.init_phase_blocking(p, self);
        } else {
            // For non-GCM/CCM modes, just enable the peripheral
            p.cr().modify(|w| w.set_en(true));
        }

        // Create context (peripheral is now enabled, safe to read registers)
        Context {
            cipher,
            dir,
            last_block_processed: false,
            is_gcm_ccm,
            header_processed: false,
            header_len: 0,
            payload_len: 0,
            aad_buffer: [0; 16],
            aad_buffer_len: 0,
            cr: p.cr().read().0,
            iv: [p.ivr(0).read(), p.ivr(1).read(), p.ivr(2).read(), p.ivr(3).read()],
            suspr: [0; 8],
        }
    }

    /// Process authenticated additional data (AAD) for GCM/CCM modes.
    /// Must be called after `start` and before `payload_blocking`.
    /// Set `last` to true for the final AAD block.
    pub fn aad_blocking<'c, C>(&mut self, ctx: &mut Context<'c, C>, aad: &[u8], last: bool) -> Result<(), Error>
    where
        C: Cipher<'c> + CipherAuthenticated<16>,
    {
        let p = T::regs();

        if ctx.header_processed && last {
            return Ok(());
        }

        // Set GCM phase to header (GCMPH = 1)
        p.cr().modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::from_bits(1)));
        // Enable the peripheral for header phase
        p.cr().modify(|w| w.set_en(true));

        let mut aad_remaining = aad.len();
        let mut aad_index = 0;

        // Process buffered AAD first if any
        if ctx.aad_buffer_len > 0 {
            let space_available = 16 - ctx.aad_buffer_len;
            let to_copy = core::cmp::min(space_available, aad_remaining);
            ctx.aad_buffer[ctx.aad_buffer_len..ctx.aad_buffer_len + to_copy].copy_from_slice(&aad[..to_copy]);
            ctx.aad_buffer_len += to_copy;
            aad_index += to_copy;
            aad_remaining -= to_copy;

            if ctx.aad_buffer_len == 16 {
                self.write_block_blocking(&ctx.aad_buffer)?;
                // Wait for CCF (block processed) - no read in header phase
                while !p.sr().read().ccf() {}
                p.icr().write(|w| w.0 = 0xFFFF_FFFF);
                ctx.header_len += 16;
                ctx.aad_buffer_len = 0;
            }
        }

        // Process complete blocks
        while aad_remaining >= 16 {
            self.write_block_blocking(&aad[aad_index..aad_index + 16])?;
            // Wait for CCF (block processed) - no read in header phase
            while !p.sr().read().ccf() {}
            p.icr().write(|w| w.0 = 0xFFFF_FFFF);
            ctx.header_len += 16;
            aad_index += 16;
            aad_remaining -= 16;
        }

        // Buffer any remaining partial block
        if aad_remaining > 0 {
            ctx.aad_buffer[..aad_remaining].copy_from_slice(&aad[aad_index..aad_index + aad_remaining]);
            ctx.aad_buffer_len = aad_remaining;
        }

        // If this is the last AAD block, pad and process
        if last {
            if ctx.aad_buffer_len > 0 {
                // Pad with zeros (per GCM spec, AAD is zero-padded to 16-byte boundary)
                for i in ctx.aad_buffer_len..16 {
                    ctx.aad_buffer[i] = 0;
                }
                // Note: Do NOT set NPBLB for header phase - NPBLB is only for payload phase
                self.write_block_blocking(&ctx.aad_buffer)?;
                // Wait for CCF (block processed)
                while !p.sr().read().ccf() {}
                p.icr().write(|w| w.0 = 0xFFFF_FFFF);
                ctx.header_len += ctx.aad_buffer_len as u64;
                ctx.aad_buffer_len = 0;
            }
            ctx.header_processed = true;
        }

        Ok(())
    }

    /// Process payload data in blocking mode.
    ///
    /// Set `last` to true for the final block. Intermediate chunks (`last=false`)
    /// must be block-aligned (16 bytes). Only the final chunk can be a partial block.
    pub fn payload_blocking<'c, C>(
        &mut self,
        ctx: &mut Context<'c, C>,
        input: &[u8],
        output: &mut [u8],
        last: bool,
    ) -> Result<(), Error>
    where
        C: Cipher<'c>,
    {
        let p = T::regs();

        if output.len() < input.len() {
            return Err(Error::ConfigError);
        }

        // For GCM/CCM, switch to payload phase
        if ctx.is_gcm_ccm {
            let header_was_skipped = !ctx.header_processed;
            if header_was_skipped {
                // No AAD provided, mark header as done
                ctx.header_processed = true;
            }
            // Set GCM phase to payload (per RM step 11a)
            // ST HAL shows: just change GCMPH, DON'T disable EN between phases
            p.cr().modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::from_bits(2)));
            // Reset NPBLB to 0 (per ST HAL)
            p.cr().modify(|w| w.set_npblb(0));
            // Only enable if header was skipped (per RM step 11b)
            if header_was_skipped {
                p.cr().modify(|w| w.set_en(true));
            }
            // If header was processed, EN is already enabled - don't touch it
        }

        let block_size = C::BLOCK_SIZE;
        let mut processed = 0;

        // Ensure proper block alignment for intermediate chunks (all modes).
        // Only the final chunk (last=true) can be partial. This applies to all modes
        // including CTR, as keystream buffering is not implemented.
        if !last && input.len() % block_size != 0 {
            return Err(Error::ConfigError);
        }

        // Process complete blocks
        let complete_blocks = if last {
            input.len() / block_size
        } else {
            input.len() / block_size
        };

        for _ in 0..complete_blocks {
            let block = &input[processed..processed + block_size];
            let out_block = &mut output[processed..processed + block_size];
            self.write_block_blocking(block)?;
            self.read_block_blocking(out_block)?;
            processed += block_size;
            ctx.payload_len += block_size as u64;
        }

        // Handle partial block if last
        if last && processed < input.len() {
            if C::REQUIRES_PADDING {
                return Err(Error::ConfigError); // Padding modes don't support partial blocks
            }

            let remaining = input.len() - processed;
            let mut partial_block = [0u8; 16];
            partial_block[..remaining].copy_from_slice(&input[processed..]);

            // Set NPBLB (Number of Padding Bytes in Last Block)
            // Per ST HAL:
            // - GCM: NPBLB is set for both encryption and decryption
            // - CCM: NPBLB is ONLY set for decryption (NOT for encryption)
            // NPBLB = 16 - valid_bytes, e.g., for 13 valid bytes, NPBLB = 3
            let is_ccm = ctx.cipher.is_ccm_mode();
            let should_set_npblb = if is_ccm {
                ctx.dir == Direction::Decrypt // CCM: only for decryption
            } else {
                true // GCM: always set NPBLB
            };
            if should_set_npblb {
                let npblb = (16 - remaining) as u8;
                p.cr().modify(|w| w.set_npblb(npblb));
            }

            self.write_block_blocking(&partial_block)?;
            self.read_block_blocking(&mut partial_block)?;

            output[processed..processed + remaining].copy_from_slice(&partial_block[..remaining]);
            ctx.payload_len += remaining as u64;
        }

        if last {
            ctx.last_block_processed = true;
        }

        Ok(())
    }

    /// Finishes the cipher operation and returns the authentication tag (for GCM/CCM).
    pub fn finish_blocking<'c, C>(&mut self, ctx: Context<'c, C>) -> Result<Option<[u8; 16]>, Error>
    where
        C: Cipher<'c>,
    {
        let p = T::regs();

        // For GCM/CCM, perform final phase to get tag
        if ctx.is_gcm_ccm {
            // Wait for BUSY flag to clear before modifying CR (per ST HAL)
            while p.sr().read().busy() {}

            // Set GCM/CCM phase to final (GCMPH = 3)
            p.cr().modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::from_bits(3)));

            // GCM and CCM have different final phase handling:
            // - GCM: Write length block (64-bit header len || 64-bit payload len in bits)
            // - CCM: Enable peripheral to trigger final tag computation
            if ctx.cipher.is_ccm_mode() {
                // CCM: Just enable the peripheral to trigger final computation
                // Per ST HAL HAL_CRYPEx_AESCCM_GenerateAuthTAG
                p.cr().modify(|w| w.set_en(true));
            } else {
                // GCM: Write lengths (in bits) as final block per GCM spec
                // ST HAL writes: DINR=0, DINR=header_bits, DINR=0, DINR=payload_bits
                let header_bits = (ctx.header_len * 8) as u32;
                let payload_bits = (ctx.payload_len * 8) as u32;

                p.dinr().write_value(0);
                p.dinr().write_value(header_bits);
                p.dinr().write_value(0);
                p.dinr().write_value(payload_bits);
            }

            // Wait for CCF flag
            while !p.sr().read().ccf() {}

            // Read the authentication tag
            // With NO_SWAP mode, use big-endian byte order for consistency
            let mut tag = [0u8; 16];
            for i in 0..4 {
                let word = p.doutr().read();
                tag[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
            }

            // Clear CCF flag
            p.icr().write(|w| w.0 = 0xFFFF_FFFF);

            // Disable peripheral
            p.cr().modify(|w| w.set_en(false));

            Ok(Some(tag))
        } else {
            // For non-authenticated modes, just disable
            p.cr().modify(|w| w.set_en(false));
            Ok(None)
        }
    }

    /// Load key into AES peripheral.
    fn load_key(&mut self, key: &[u8]) {
        let p = T::regs();

        // Keys are loaded as 32-bit words
        // NIST vectors are in big-endian byte order, form words accordingly
        let key_words = key.len() / 4;
        for i in 0..key_words {
            let word = u32::from_be_bytes([key[i * 4], key[i * 4 + 1], key[i * 4 + 2], key[i * 4 + 3]]);
            p.keyr(key_words - 1 - i).write_value(word); // Reverse order
        }
    }

    /// Load IV into AES peripheral.
    fn load_iv(&mut self, iv: &[u8]) {
        if iv.is_empty() {
            return;
        }

        let p = T::regs();

        // IV is loaded as 32-bit words in reverse register order (per ST HAL):
        // IVR3 = first word (bytes 0-3), IVR2 = second word, IVR1 = third, IVR0 = fourth
        let iv_words = core::cmp::min(iv.len(), 16) / 4;
        for i in 0..iv_words {
            let word = u32::from_be_bytes([iv[i * 4], iv[i * 4 + 1], iv[i * 4 + 2], iv[i * 4 + 3]]);
            p.ivr(iv_words - 1 - i).write_value(word); // Reverse order like ST HAL
        }
    }

    /// Write a block to the AES peripheral (blocking).
    /// Uses big-endian byte order for NIST test vector compatibility with NO_SWAP mode.
    fn write_block_blocking(&mut self, block: &[u8]) -> Result<(), Error> {
        let p = T::regs();

        // Write all 4 words of the block
        // Use big-endian byte order with NO_SWAP datatype for NIST vector compatibility
        for i in 0..4 {
            if p.sr().read().wrerr() {
                p.icr().write(|w| w.0 = 0xFFFF_FFFF);
                return Err(Error::WriteError);
            }

            let word = u32::from_be_bytes([block[i * 4], block[i * 4 + 1], block[i * 4 + 2], block[i * 4 + 3]]);
            p.dinr().write_value(word);
        }

        Ok(())
    }

    /// Read a block from the AES peripheral (blocking).
    /// Uses big-endian byte order for NIST test vector compatibility with NO_SWAP mode.
    fn read_block_blocking(&mut self, block: &mut [u8]) -> Result<(), Error> {
        let p = T::regs();

        // Wait for computation complete
        while !p.sr().read().ccf() {}

        // Check for errors before reading
        let sr = p.sr().read();
        if sr.rderr() {
            p.icr().write(|w| w.0 = 0xFFFF_FFFF);
            return Err(Error::ReadError);
        }

        // Read as 32-bit words and convert to big-endian byte arrays
        // With NO_SWAP datatype, use big-endian for NIST vector compatibility
        for i in 0..4 {
            let word = p.doutr().read();
            let bytes = word.to_be_bytes();
            block[i * 4..i * 4 + 4].copy_from_slice(&bytes);
        }

        // Clear flags after successful read
        p.icr().write(|w| w.0 = 0xFFFF_FFFF);

        Ok(())
    }
}

trait SealedInstance {
    fn regs() -> pac::aes::Aes;
}

/// AES instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this AES instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, aes, AES, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::aes::Aes {
                crate::pac::$inst
            }
        }
    };
);

dma_trait!(DmaIn, Instance);
dma_trait!(DmaOut, Instance);
