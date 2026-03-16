//! Secure Advanced Encryption Standard (SAES) hardware accelerator
//!
//! SAES provides the same cipher modes as AES but with enhanced security features
//! for key management and protection. It's particularly useful in secure boot scenarios
//! and applications requiring hardware root of trust.
//!
//! # Key Differences from AES
//!
//! | Feature | AES | SAES |
//! |---------|-----|------|
//! | Key Sources | Software only | Software + Hardware (DHUK, BHK) |
//! | Key Protection | Basic | KEYPROT + isolation |
//! | Key Sharing | No | Yes (with AES, other peripherals) |
//! | Key Wrapping | No | Yes (wrapped/encrypted keys) |
//! | Security Context | Standard | Enhanced/Secure |
//!
//! # Hardware Key Sources
//!
//! - **DHUK** (Derived Hardware Unique Key): Device-unique key derived from UID
//! - **BHK** (Boot Hardware Key): Key loaded during secure boot
//! - **XOR**: XOR combination of DHUK and BHK
//!
//! These keys are never exposed to software and remain in secure hardware.
//!
//! # Examples
//!
//! ## Using Software Keys (Same as AES)
//!
//! ```no_run
//! use embassy_stm32::saes::{Saes, AesGcm, Direction};
//!
//! let key = [0u8; 16];
//! let iv = [0u8; 12];
//! let cipher = AesGcm::new(&key, &iv);
//!
//! let mut saes = Saes::new_blocking(p.SAES, Irqs);
//! let mut ctx = saes.start(&cipher, Direction::Encrypt);
//! // ... same as AES
//! ```
//!
//! ## Using Hardware-Derived Keys
//!
//! ```no_run
//! use embassy_stm32::saes::{Saes, AesGcm, Direction, HardwareKeySource};
//!
//! let iv = [0u8; 12];
//! let cipher = AesGcm::new(&[], &iv); // No software key needed
//!
//! let mut saes = Saes::new_blocking(p.SAES, Irqs);
//!
//! // Use device-unique hardware key
//! let mut ctx = saes.start_with_hw_key(
//!     HardwareKeySource::DHUK,
//!     &cipher,
//!     Direction::Encrypt
//! );
//!
//! // Hardware key is used automatically - never exposed to software
//! saes.payload_blocking(&mut ctx, &plaintext, &mut ciphertext, true);
//! saes.finish_blocking(ctx);
//! ```
//!
//! ## Key Sharing Between Peripherals
//!
//! ```no_run
//! use embassy_stm32::saes::{Saes, KeyShareTarget};
//!
//! // After unwrapping a key with SAES, share it with AES peripheral
//! saes.share_key_with(KeyShareTarget::AES);
//! // Now AES peripheral can use the unwrapped key
//! ```
//!
//! # Security Features
//!
//! - **Key Protection**: KEYPROT flag prevents key readback
//! - **Hardware Keys**: Never exposed to software, immune to memory dumps
//! - **Key Wrapping**: Import encrypted keys securely
//! - **Peripheral Isolation**: Keys can be shared without software access
//!
//! # Availability
//!
//! **Important**: SAES is only available on:
//! - STM32WBA52 and higher
//! - STM32WBA55
//! - STM32WBA6x
//! - NOT available on STM32WBA50
//!
//! # Use Cases
//!
//! - Secure boot key management
//! - Device-unique encryption (uses DHUK based on chip UID)
//! - Key provisioning and wrapping
//! - Multi-peripheral cryptographic workflows
//! - High-security applications requiring hardware root of trust
//!
//! # See Also
//!
//! - [`aes`](crate::aes) - Standard AES implementation (all WBA chips)
//! - [`pka`](crate::pka) - Public Key Accelerator

// Re-export most types from AES since they're identical
use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

pub use crate::aes::{
    AesCbc, AesCcm, AesCtr, AesEcb, AesGcm, Cipher, CipherAuthenticated, CipherSized, Context, Direction, Error,
    IVSized, KeySize,
};
use crate::dma::ChannelAndRequest;
use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::{interrupt, pac, peripherals, rcc};

static SAES_WAKER: AtomicWaker = AtomicWaker::new();

/// SAES interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // Wake on computation complete flag (CCF) from ISR, not on BUSY clearing.
        // BUSY also clears during init/RNG-fetch/key-transfer which must not wake tasks.
        // Note: CCF is in SAES_ISR, not SAES_SR (unlike AES which has CCF in SR).
        let isr = T::regs().isr().read();
        if isr.ccf() {
            // Clear all interrupt flags
            T::regs().icr().write(|w| w.0 = 0xFFFF_FFFF);
            SAES_WAKER.wake();
        }

        // Clear error flags
        if isr.rweif() {
            T::regs().icr().write(|w| w.0 = 0xFFFF_FFFF);
        }
    }
}

/// Hardware key source for SAES
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HardwareKeySource {
    /// Derived Hardware Unique Key
    DHUK = 1,
    /// Boot Hardware Key
    BHK = 2,
    /// XOR of DHUK and BHK
    XorDhukBhk = 3,
}

/// Key mode for SAES
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum KeyMode {
    /// Normal software key mode
    Normal = 0,
    /// Wrapped key mode (encrypted key)
    WrappedKey = 1,
    /// Shared key mode (key shared between peripherals)
    SharedKey = 2,
}

/// Peripheral to share key with
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum KeyShareTarget {
    /// Share with AES peripheral
    AES = 0,
}

/// SAES driver.
pub struct Saes<'d, T: Instance, M: Mode> {
    _peripheral: Peri<'d, T>,
    _phantom: PhantomData<M>,
    #[allow(dead_code)] // Reserved for future async/DMA implementation
    dma_in: Option<ChannelAndRequest<'d>>,
    #[allow(dead_code)] // Reserved for future async/DMA implementation
    dma_out: Option<ChannelAndRequest<'d>>,
}

impl<'d, T: Instance> Saes<'d, T, Blocking> {
    /// Instantiates, resets, and enables the SAES peripheral.
    pub fn new_blocking(
        peripheral: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        // On WBA6, SAES fetches a random seed from the RNG on every reset/enable.
        // If the RNG is not already running, start it with HSI as the clock source.
        // This mirrors the same workaround used by the PKA driver on WBA6.
        #[cfg(rng_wba6)]
        {
            let rcc = pac::RCC;
            if !rcc.ahb2enr().read().rngen() {
                rcc.ccipr2().modify(|w| w.set_rngsel(pac::rcc::vals::Rngsel::HSI));
                rcc.ahb2enr().modify(|w| w.set_rngen(true));
                pac::RNG.cr().modify(|w| w.set_rngen(true));
                // Brief settle delay (~100 µs at 96 MHz) before SAES tries to read from RNG
                cortex_m::asm::delay(10_000);
            }
        }

        rcc::enable_and_reset::<T>();

        let p = T::regs();
        // After reset, SAES sets BUSY while it fetches a random number from the internal RNG.
        // Writing CR before BUSY clears is forbidden (HAL: CRYP_FLAG_BUSY check at init).
        while p.sr().read().busy() {}
        // Panic on RNG error - the peripheral is unusable without a working RNG.
        assert!(!p.isr().read().rngeif(), "SAES: RNG error during initialization");

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

impl<'d, T: Instance> Saes<'d, T, Async> {
    /// Instantiates, resets, and enables the SAES peripheral with DMA support.
    pub fn new<D1: DmaIn<T>, D2: DmaOut<T>>(
        peripheral: Peri<'d, T>,
        dma_in: Peri<'d, D1>,
        dma_out: Peri<'d, D2>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + 'd,
    ) -> Self {
        // On WBA6, SAES fetches a random seed from the RNG on every reset/enable.
        // If the RNG is not already running, start it with HSI as the clock source.
        // This mirrors the same workaround used by the PKA driver on WBA6.
        #[cfg(rng_wba6)]
        {
            let rcc = pac::RCC;
            if !rcc.ahb2enr().read().rngen() {
                rcc.ccipr2().modify(|w| w.set_rngsel(pac::rcc::vals::Rngsel::HSI));
                rcc.ahb2enr().modify(|w| w.set_rngen(true));
                pac::RNG.cr().modify(|w| w.set_rngen(true));
                cortex_m::asm::delay(10_000);
            }
        }

        rcc::enable_and_reset::<T>();

        let p = T::regs();
        // After reset, SAES sets BUSY while it fetches a random number from the internal RNG.
        // Writing CR before BUSY clears is forbidden (HAL: CRYP_FLAG_BUSY check at init).
        while p.sr().read().busy() {}
        // Panic on RNG error - the peripheral is unusable without a working RNG.
        assert!(!p.isr().read().rngeif(), "SAES: RNG error during initialization");

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

impl<'d, T: Instance, M: Mode> Saes<'d, T, M> {
    /// Starts a new cipher operation with a software key.
    pub fn start<'c, C>(&mut self, cipher: &'c C, dir: Direction) -> Context<'c, C>
    where
        C: Cipher<'c> + CipherSized + IVSized,
    {
        self.start_with_key_mode(cipher, dir, KeyMode::Normal, None)
    }

    /// Starts a new cipher operation with a hardware-derived key.
    pub fn start_with_hw_key<'c, C>(
        &mut self,
        key_source: HardwareKeySource,
        cipher: &'c C,
        dir: Direction,
    ) -> Context<'c, C>
    where
        C: Cipher<'c> + CipherSized + IVSized,
    {
        self.start_with_key_mode(cipher, dir, KeyMode::Normal, Some(key_source))
    }

    /// Internal start method with full control over key mode.
    fn start_with_key_mode<'c, C>(
        &mut self,
        cipher: &'c C,
        dir: Direction,
        key_mode: KeyMode,
        hw_key: Option<HardwareKeySource>,
    ) -> Context<'c, C>
    where
        C: Cipher<'c> + CipherSized + IVSized,
    {
        let p = T::regs();

        // Disable the peripheral, then wait for BUSY to clear before touching CR.
        // The HAL checks BUSY before every CR write (SetConfig, Encrypt, Decrypt).
        p.cr().modify(|w| w.set_en(false));
        while p.sr().read().busy() {}

        // Software reset (IPRST): clears KEYVALID and all internal key state.
        // Without this, KEYVALID from a previous 128-bit operation stays set when
        // switching to 256-bit, causing the KEYVALID wait to exit before all 8 key
        // registers are written (producing silent wrong-key encryption).
        // Mirrors HAL_CRYP_DeInit(): SET_BIT(CR, IPRST) then CLEAR_BIT(CR, IPRST).
        p.cr().modify(|w| w.set_iprst(true));
        p.cr().modify(|w| w.set_iprst(false));

        // Clear all pending flags — in particular KEIF, which if left set will permanently
        // block KEYVALID from being asserted, making the key load silently fail.
        p.icr().write(|w| w.0 = 0xFFFF_FFFF);

        // Configure data type based on cipher mode (NO_SWAP, BYTE_SWAP, or BIT_SWAP)
        p.cr()
            .modify(|w| w.set_datatype(pac::saes::vals::Datatype::from_bits(cipher.datatype())));

        // Configure key size
        let keysize = cipher.key_size();
        let keysize_val = match keysize {
            KeySize::Bits128 => pac::saes::vals::Keysize::BITS128,
            KeySize::Bits256 => pac::saes::vals::Keysize::BITS256,
        };
        p.cr().modify(|w| w.set_keysize(keysize_val));
        // Changing KEYSIZE may trigger a new RNG mask fetch (BUSY=1) inside SAES,
        // particularly when switching from 128-bit to 256-bit, which needs a larger mask.
        while p.sr().read().busy() {}

        // Set cipher mode using SAES-compatible method
        self.set_cipher_mode(p, cipher);
        let is_gcm_ccm = cipher.uses_gcm_phases();

        // Set direction
        let mode_val = match dir {
            Direction::Encrypt => pac::saes::vals::Mode::ENCRYPTION,
            Direction::Decrypt => pac::saes::vals::Mode::DECRYPTION,
        };
        p.cr().modify(|w| w.set_mode(mode_val));

        // Set key mode
        let kmod_val = pac::saes::vals::Kmod::from_bits(key_mode as u8);
        p.cr().modify(|w| w.set_kmod(kmod_val));

        // For GCM/CCM (authenticated) modes, set GCMPH=0 (init phase) BEFORE loading the key.
        if is_gcm_ccm {
            p.cr().modify(|w| w.set_gcmph(pac::saes::vals::Gcmph::from_bits(0)));
        }

        // Configure and load the key (after GCMPH is set).
        if let Some(hw_key_src) = hw_key {
            let keysel_val = pac::saes::vals::Keysel::from_bits(hw_key_src as u8);
            p.cr().modify(|w| w.set_keysel(keysel_val));
            p.cr().modify(|w| w.set_keyprot(true));
            // For hardware keys (non-SW), SAES fetches the key autonomously: wait for KEYVALID.
            while !p.sr().read().keyvalid() {}
        } else {
            // Load software key, then wait for KEYVALID.
            // Unlike plain AES, SAES validates the key register write sequence; KEYVALID must
            // be set before EN can be asserted (RM: "EN cannot be set as long as KEYVALID = 0").
            self.load_key(cipher.key());
            while !p.sr().read().keyvalid() {}
        }

        // For ECB/CBC decryption, perform key derivation (MODE=1) before the actual operation.
        // CTR, GCM, and CCM use the encryption key schedule in both directions - no derivation.
        let needs_key_derivation = dir == Direction::Decrypt && matches!(cipher.chmod_bits(), 0 | 1);
        if needs_key_derivation {
            p.cr().modify(|w| w.set_mode(pac::saes::vals::Mode::KEY_DERIVATION));
            p.cr().modify(|w| w.set_en(true));
            // Wait for CCF (computation complete), not BUSY
            while !p.isr().read().ccf() {}
            // Clear CCF via ICR
            p.icr().write(|w| w.0 = 0xFFFF_FFFF);
            // Restore decrypt mode for the actual operation
            p.cr().modify(|w| w.set_mode(mode_val));
        }

        // Load IV
        self.load_iv(cipher.iv());

        // Perform init phase for GCM/CCM (hash-key H calculation, phase 0).
        // MODE is already ENCRYPTION here (set above) — correct for H = AES_ENCRYPT(K, 0).
        if is_gcm_ccm {
            p.cr().modify(|w| w.set_en(true));
            // Wait for CCF (init phase complete)
            while !p.isr().read().ccf() {}
            // Clear flags
            p.icr().write(|w| w.0 = 0xFFFF_FFFF);
        } else {
            // For non-GCM/CCM modes, enable the peripheral then wait for BUSY to clear.
            // SAES may assert BUSY after EN=1 to apply the per-key-size RNG mask
            // (observed with 256-bit keys: the upper-half mask is applied here).
            p.cr().modify(|w| w.set_en(true));
            while p.sr().read().busy() {}
        }

        // Create context (peripheral is now enabled)
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

    /// Share the current unwrapped key with another peripheral.
    /// This must be called after a decryption operation that unwrapped a key.
    pub fn share_key_with(&mut self, target: KeyShareTarget) {
        let p = T::regs();
        let kshareid_val = match target {
            KeyShareTarget::AES => pac::saes::vals::Kshareid::AES,
        };
        p.cr().modify(|w| w.set_kshareid(kshareid_val));
    }

    /// Set cipher mode for SAES peripheral using the cipher's CHMOD bits.
    fn set_cipher_mode<'c, C>(&mut self, p: pac::saes::Saes, cipher: &C)
    where
        C: Cipher<'c>,
    {
        // Use the cipher's canonical CHMOD value (0=ECB, 1=CBC, 2=CTR, 3=GCM/GMAC, 4=CCM).
        // Inferring mode from IV length is unreliable: GCM, CBC, and CTR all use 16-byte IVs
        // after AesGcm::new() pads the 12-byte nonce to 16 bytes.
        p.cr()
            .modify(|w| w.set_chmod(pac::saes::vals::Chmod::from_bits(cipher.chmod_bits())));
    }

    /// Process authenticated additional data (AAD) for GCM/CCM modes.
    pub fn aad_blocking<'c, C>(&mut self, ctx: &mut Context<'c, C>, aad: &[u8], last: bool) -> Result<(), Error>
    where
        C: Cipher<'c> + CipherAuthenticated<16>,
    {
        // Reuse AES implementation logic
        let p = T::regs();

        if ctx.header_processed && last {
            return Ok(());
        }

        // Set GCM phase to header (phase 1), then re-enable.
        // After the init phase SAES auto-clears EN, so we must set EN=1 here.
        p.cr().modify(|w| w.set_gcmph(pac::saes::vals::Gcmph::from_bits(1)));
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
                // Wait for CCF (header block processed) — no output read in header phase.
                // SAES CCF is in ISR, not SR (unlike plain AES).
                while !p.isr().read().ccf() {}
                p.icr().write(|w| w.0 = 0xFFFF_FFFF);
                ctx.header_len += 16;
                ctx.aad_buffer_len = 0;
            }
        }

        // Process complete blocks
        while aad_remaining >= 16 {
            self.write_block_blocking(&aad[aad_index..aad_index + 16])?;
            // Wait for CCF after each header block
            while !p.isr().read().ccf() {}
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
                // Pad partial block with zeros (NPBLB is payload-phase-only, not set here)
                for i in ctx.aad_buffer_len..16 {
                    ctx.aad_buffer[i] = 0;
                }
                self.write_block_blocking(&ctx.aad_buffer)?;
                // Wait for CCF after last header block
                while !p.isr().read().ccf() {}
                p.icr().write(|w| w.0 = 0xFFFF_FFFF);
                ctx.header_len += ctx.aad_buffer_len as u64;
                ctx.aad_buffer_len = 0;
            }
            ctx.header_processed = true;
        }

        Ok(())
    }

    /// Process payload data in blocking mode.
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

        // For GCM/CCM, switch to payload phase.
        // SAES requires EN=0→1 at every GCMPH transition; without re-enabling here,
        // the GHASH state from the header phase is not properly transferred and the
        // encrypt/decrypt tags diverge (observed on SAES v1a / WBA65RI with AAD).
        if ctx.is_gcm_ccm {
            if !ctx.header_processed {
                ctx.header_processed = true;
            }
            p.cr().modify(|w| w.set_gcmph(pac::saes::vals::Gcmph::from_bits(2)));
            p.cr().modify(|w| w.set_npblb(0));
            p.cr().modify(|w| w.set_en(true));
        }

        let block_size = C::BLOCK_SIZE;
        let mut processed = 0;

        // Ensure proper block alignment for modes that require padding
        if C::REQUIRES_PADDING && !last && input.len() % block_size != 0 {
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
                return Err(Error::ConfigError);
            }

            let remaining = input.len() - processed;
            let mut partial_block = [0u8; 16];
            partial_block[..remaining].copy_from_slice(&input[processed..]);

            // NPBLB = Number of Padding Bytes in Last Block.
            // For `remaining` valid bytes, there are (16 - remaining) padding bytes.
            let padding_bytes = (16 - remaining) as u8;
            p.cr().modify(|w| w.set_npblb(padding_bytes));

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

        // For GCM, perform final phase to get tag
        if ctx.is_gcm_ccm {
            // SAES may set BUSY during GCM payload encryption. The PAC document states:
            // "When GCM encryption is selected, the flag must be at zero before selecting
            // the GCM final phase." The HAL (HAL_CRYPEx_AESGCM_GenerateAuthTAG) also
            // checks BUSY before setting the final phase.
            while p.sr().read().busy() {}

            // Set GCM phase to final (phase 3)
            p.cr().modify(|w| w.set_gcmph(pac::saes::vals::Gcmph::from_bits(3)));

            // Write lengths (in bits) as final block
            let header_bits = (ctx.header_len * 8) as u64;
            let payload_bits = (ctx.payload_len * 8) as u64;

            let mut length_block = [0u8; 16];
            length_block[0..8].copy_from_slice(&header_bits.to_be_bytes());
            length_block[8..16].copy_from_slice(&payload_bits.to_be_bytes());

            self.write_block_blocking(&length_block)?;

            // Read the authentication tag
            let mut tag = [0u8; 16];
            self.read_block_blocking(&mut tag)?;

            // Disable peripheral
            p.cr().modify(|w| w.set_en(false));

            Ok(Some(tag))
        } else {
            // For non-authenticated modes, just disable
            p.cr().modify(|w| w.set_en(false));
            Ok(None)
        }
    }

    /// Load key into SAES peripheral.
    fn load_key(&mut self, key: &[u8]) {
        let p = T::regs();

        // Keys are loaded as 32-bit words (big-endian byte order), high register first.
        // KEYR7 = key MSB (key[0..3]), KEYR0 = key LSB (key[28..31]) for 256-bit.
        // This order matches the STM32Cube HAL's CRYP_SetKey() function.
        let key_words = key.len() / 4;
        for i in 0..key_words {
            let word = u32::from_be_bytes([key[i * 4], key[i * 4 + 1], key[i * 4 + 2], key[i * 4 + 3]]);
            p.keyr(key_words - 1 - i).write_value(word); // KEYR(N-1-i): descending from MSB
        }
    }

    /// Load IV into SAES peripheral.
    fn load_iv(&mut self, iv: &[u8]) {
        if iv.is_empty() {
            return;
        }

        let p = T::regs();

        // IV is loaded as 32-bit words (big-endian byte order)
        let iv_words = core::cmp::min(iv.len(), 16) / 4;
        for i in 0..iv_words {
            let word = u32::from_be_bytes([iv[i * 4], iv[i * 4 + 1], iv[i * 4 + 2], iv[i * 4 + 3]]);
            p.ivr(i).write_value(word);
        }

        // Handle partial IV words
        let remaining = core::cmp::min(iv.len(), 16) % 4;
        if remaining > 0 {
            let i = iv_words * 4;
            let mut bytes = [0u8; 4];
            bytes[..remaining].copy_from_slice(&iv[i..i + remaining]);
            let word = u32::from_be_bytes(bytes);
            p.ivr(iv_words).write_value(word);
        }
    }

    /// Write a block to the SAES peripheral (blocking).
    fn write_block_blocking(&mut self, block: &[u8]) -> Result<(), Error> {
        let p = T::regs();

        // Check for write error before starting
        if p.sr().read().wrerr() {
            return Err(Error::WriteError);
        }

        // Write all 4 words of the block (big-endian byte order)
        for i in 0..4 {
            let word = u32::from_be_bytes([block[i * 4], block[i * 4 + 1], block[i * 4 + 2], block[i * 4 + 3]]);
            p.dinr().write_value(word);
        }

        Ok(())
    }

    /// Read a block from the SAES peripheral (blocking).
    fn read_block_blocking(&mut self, block: &mut [u8]) -> Result<(), Error> {
        let p = T::regs();

        // Wait for CCF (Computation Complete Flag) in ISR — BUSY is not a reliable completion
        // signal as it also clears during init/RNG-fetch/key-transfer.
        while !p.isr().read().ccf() {}

        // Check for read/write error flag in ISR before reading output
        if p.isr().read().rweif() {
            p.icr().write(|w| w.0 = 0xFFFF_FFFF);
            return Err(Error::ReadError);
        }

        // Read as 32-bit words and convert to big-endian byte arrays
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
    fn regs() -> pac::saes::Saes;
}

/// SAES instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this SAES instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, saes, SAES, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::saes::Saes {
                crate::pac::$inst
            }
        }
    };
);

dma_trait!(DmaIn, Instance);
dma_trait!(DmaOut, Instance);
