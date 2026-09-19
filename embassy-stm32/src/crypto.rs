//! Cipher-mode types shared by [`crate::aes`] and [`crate::saes`].
//!
//! Everything here is register-agnostic: the [`Cipher`] trait only describes a
//! mode (key, IV, `CHMOD` value, whether it has GCM/CCM phases, ...), and the
//! drivers turn that description into register writes.

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

/// AES block size in bytes (128 bits).
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
    ///
    /// The drivers use NO_SWAP (0) consistently with big-endian byte
    /// conversion (`from_be_bytes`/`to_be_bytes`) for direct NIST test-vector
    /// compatibility.
    fn datatype(&self) -> u8 {
        0
    }

    /// Returns the raw `CHMOD` field value for this cipher mode.
    fn chmod_bits(&self) -> u8 {
        0 // ECB default
    }

    /// Indicates whether this cipher mode uses GCM/CCM phases (init, header, payload, final).
    fn uses_gcm_phases(&self) -> bool {
        false
    }

    /// Indicates whether this is CCM mode (which has different final phase handling).
    fn is_ccm_mode(&self) -> bool {
        false
    }

    /// CCM only: the encoded associated-data length that precedes the
    /// associated data in the first header block (NIST SP 800-38C A.2.2), and
    /// its size. Empty for other modes and when there is no associated data.
    fn ccm_aad_header(&self) -> ([u8; 10], usize) {
        ([0; 10], 0)
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
#[cfg(not(aes_v1))]
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
#[cfg(not(aes_v1))]
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
    const REQUIRES_PADDING: bool = false;

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
#[cfg(not(aes_v1))]
impl<'c> CipherSized for AesCtr<'c, { 256 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for AesCtr<'c, KEY_SIZE> {}

// The authenticated modes need the GCM/CCM phases, which aes_v1 lacks.
#[cfg(not(aes_v1))]
mod authenticated {
    use super::*;

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
    /// GMAC provides message authentication without encryption. The data remains
    /// in plaintext but any tampering is detected via the authentication tag.
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
            // GMAC uses the same hardware mode as GCM.
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
        aad_len: usize,
    }

    impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize, const TAG_SIZE: usize> AesCcm<'c, KEY_SIZE, IV_SIZE, TAG_SIZE> {
        /// Constructs a new AES-CCM cipher for a cryptographic operation.
        /// - `key`: The encryption key (16 or 32 bytes)
        /// - `iv`: The nonce/IV (7-13 bytes)
        /// - `aad_len`: Length of additional authenticated data (known in advance)
        /// - `payload_len`: Length of payload data (known in advance)
        pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; IV_SIZE], aad_len: usize, payload_len: usize) -> Self {
            Self {
                key,
                iv: build_ccm_iv(iv, TAG_SIZE, aad_len, payload_len),
                aad_len,
            }
        }

        /// View the precomputed state as a type-erased [`CcmOp`].
        fn op(&self) -> CcmOp<'c> {
            CcmOp {
                key: self.key,
                iv: self.iv,
                aad_len: self.aad_len,
            }
        }
    }

    /// Format the B0 block for CCM, shared by [`AesCcm::new`] and the
    /// type-erased [`CcmOp`] constructor.
    fn build_ccm_iv(iv: &[u8], tag_size: usize, aad_len: usize, payload_len: usize) -> [u8; 16] {
        let mut iv_full = [0u8; 16];
        let l = 15 - iv.len(); // size of the length field
        iv_full[0] = ((l - 1) as u8) | ((((tag_size - 2) / 2) as u8) << 3);
        if aad_len > 0 {
            iv_full[0] |= 0x40; // Adata flag
        }
        iv_full[1..1 + iv.len()].copy_from_slice(iv);

        let payload_bytes = (payload_len as u64).to_be_bytes();
        let offset = 16 - l;
        iv_full[offset..].copy_from_slice(&payload_bytes[8 - l..]);

        iv_full
    }

    /// Type-erased AES-CCM operation.
    pub(crate) struct CcmOp<'c> {
        key: &'c [u8],
        iv: [u8; 16],
        aad_len: usize,
    }

    impl<'c> CcmOp<'c> {
        /// Constructs a type-erased CCM operation, validating the nonce and tag
        /// sizes at runtime (the const-generic [`AesCcm`] validates them at
        /// compile time instead).
        #[allow(dead_code)] // Only used by the optional `embassy-crypto` driver (`aes/driver.rs`).
        pub(crate) fn new(key: &'c [u8], iv: &[u8], tag_size: usize, aad_len: usize, payload_len: usize) -> Self {
            assert!((7..=13).contains(&iv.len()), "CCM IV must be 7-13 bytes");
            assert!(
                tag_size >= 4 && tag_size <= 16 && tag_size % 2 == 0,
                "CCM tag must be 4-16 bytes and even"
            );
            Self {
                key,
                iv: build_ccm_iv(iv, tag_size, aad_len, payload_len),
                aad_len,
            }
        }
    }

    impl<'c> Cipher<'c> for CcmOp<'c> {
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

        fn ccm_aad_header(&self) -> ([u8; 10], usize) {
            let mut header = [0u8; 10];
            let len = if self.aad_len == 0 {
                0
            } else if self.aad_len < (1 << 16) - (1 << 8) {
                header[..2].copy_from_slice(&(self.aad_len as u16).to_be_bytes());
                2
            } else if (self.aad_len as u64) < (1u64 << 32) {
                header[..2].copy_from_slice(&[0xff, 0xfe]);
                header[2..6].copy_from_slice(&(self.aad_len as u32).to_be_bytes());
                6
            } else {
                header[..2].copy_from_slice(&[0xff, 0xff]);
                header[2..10].copy_from_slice(&(self.aad_len as u64).to_be_bytes());
                10
            };
            (header, len)
        }
    }

    impl<'c> CipherSized for CcmOp<'c> {}
    impl<'c> IVSized for CcmOp<'c> {}
    // `finish` always returns the full 16-byte block; the const parameter only
    // exists so the type system can size the return value and the driver
    // truncates it to the runtime tag length.
    impl<'c> CipherAuthenticated<16> for CcmOp<'c> {}

    /// `Cipher` implementation for [`AesCcm`], forwarding to the shared
    /// type-erased [`CcmOp`] implementation so the hardware logic exists only
    /// once.
    impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize, const TAG_SIZE: usize> Cipher<'c>
        for AesCcm<'c, KEY_SIZE, IV_SIZE, TAG_SIZE>
    {
        const REQUIRES_PADDING: bool = false;

        fn key(&self) -> &[u8] {
            // Direct field access: the returned slice borrows `self`, so this
            // cannot forward through a temporary `CcmOp`.
            self.key
        }

        fn iv(&self) -> &[u8] {
            &self.iv
        }

        fn chmod_bits(&self) -> u8 {
            self.op().chmod_bits()
        }

        fn uses_gcm_phases(&self) -> bool {
            self.op().uses_gcm_phases()
        }

        fn is_ccm_mode(&self) -> bool {
            self.op().is_ccm_mode()
        }

        fn ccm_aad_header(&self) -> ([u8; 10], usize) {
            self.op().ccm_aad_header()
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
}
#[cfg(not(aes_v1))]
pub use authenticated::*;

/// Stores the state of the AES peripheral for a cipher operation.
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
    /// Whether the header (AAD) has been processed
    pub header_processed: bool,
    /// Total length of additional authenticated data
    pub header_len: u64,
    /// Total length of payload data
    pub payload_len: u64,
    /// Buffer for partial AAD blocks
    pub aad_buffer: [u8; 16],
    /// Number of bytes in the AAD buffer
    pub aad_buffer_len: usize,
    /// Control register state
    pub cr: u32,
    /// Initialization vector state
    pub iv: [u32; 4],
    /// Suspend registers for GCM/CCM
    pub suspr: [u32; 8],
}
