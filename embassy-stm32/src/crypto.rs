//! Shared AES cipher mode types used by [`crate::aes`] and [`crate::saes`].

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

/// This trait enables restriction of ciphers to specific key sizes.
pub trait CipherSized {}

/// This trait enables restriction of initialization vectors to sizes compatible with a cipher mode.
pub trait IVSized {}

/// This trait enables restriction of a header phase to authenticated ciphers only.
pub trait CipherAuthenticated<const TAG_SIZE: usize> {
    /// Defines the authentication tag size.
    const TAG_SIZE: usize = TAG_SIZE;
}

// The cipher-mode types below are only reachable through `crate::saes`'s N6
// re-export (this module is private, and `aes_v3b` chips use `crate::aes`'s
// copy instead), so they only need to exist for that configuration.
#[cfg(all(saes_n6, not(aes_v3b)))]
mod ciphers {
    use super::{CipherAuthenticated, CipherSized, Direction, IVSized, KeySize};

    const AES_BLOCK_SIZE: usize = 16;

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

        /// Returns the data type setting for this cipher mode.
        fn datatype(&self) -> u8 {
            0
        }

        /// Returns the raw CHMOD field value for this cipher mode.
        fn chmod_bits(&self) -> u8 {
            0
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
        fn is_ccm_mode(&self) -> bool {
            false
        }

        /// Returns the pre-computed B0 block for CCM mode (None for other modes).
        fn ccm_b0(&self) -> Option<&[u8; 16]> {
            None
        }

        /// Returns the formatted AAD length prefix for CCM mode.
        fn ccm_format_aad_header(&self, aad_len: usize) -> ([u8; 10], usize) {
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

        fn chmod_bits(&self) -> u8 {
            0
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

        fn chmod_bits(&self) -> u8 {
            1
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

        fn chmod_bits(&self) -> u8 {
            2
        }
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

        fn chmod_bits(&self) -> u8 {
            3
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

        fn chmod_bits(&self) -> u8 {
            3
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

        fn chmod_bits(&self) -> u8 {
            4
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
}

#[cfg(all(saes_n6, not(aes_v3b)))]
pub use ciphers::{AesCbc, AesCcm, AesCtr, AesEcb, AesGcm, AesGmac, Cipher, Context};
