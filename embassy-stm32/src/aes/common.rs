//! Register-agnostic AES layer shared by the `aes_v2` and `aes_v3b` drivers.
//!
//! This module holds the cipher-mode types, the [`Cipher`] trait and its cipher
//! implementations, the operation [`Context`], and the GCM/CCM state machine
//! ([`op_start`], [`op_aad`], [`op_payload`], [`op_finish`]). The two hardware
//! revisions are identical at this level; they differ only in a handful of
//! register primitives, which are selected with `#[cfg(aes_v2)]` /
//! `#[cfg(aes_v3b)]` below. The version modules (`v2`, `v3b`) provide the
//! `Aes` driver shell, instance wiring, and constructors, and delegate the
//! algorithm here. The types `saes` also needs (`Error`, `Direction`,
//! `KeySize`, and the marker traits) come from [`crate::crypto`] and are
//! re-exported.

pub use crate::crypto::{CipherAuthenticated, CipherSized, Direction, Error, IVSized, KeySize};
#[cfg(aes_v2)]
use crate::pac::aes::regs::{Dinr, Ivr, Keyr};
use crate::pac::aes::vals::{Datatype, Gcmph, Mode};
use crate::{pac, peripherals};

/// AES block size in bytes (128 bits).
pub(crate) const AES_BLOCK_SIZE: usize = 16;

// Register primitives — the only points where aes_v2 and aes_v3b diverge.

/// Clear the computation-complete flag (CCF).
#[cfg(aes_v2)]
#[inline]
pub(crate) fn clear_ccf(p: pac::aes::Aes) {
    // aes_v2 has no ICR; CCF is cleared through the CCFC bit of CR.
    p.cr().modify(|w| w.set_ccfc(true));
}
#[cfg(aes_v3b)]
#[inline]
pub(crate) fn clear_ccf(p: pac::aes::Aes) {
    p.icr().write(|w| w.0 = 0xFFFF_FFFF);
}

/// Clear the computation-complete and read/write error flags together.
#[cfg(aes_v2)]
#[inline]
pub(crate) fn clear_flags(p: pac::aes::Aes) {
    p.cr().modify(|w| {
        w.set_ccfc(true);
        w.set_errc(true);
    });
}
#[cfg(aes_v3b)]
#[inline]
pub(crate) fn clear_flags(p: pac::aes::Aes) {
    p.icr().write(|w| w.0 = 0xFFFF_FFFF);
}

/// Write a 32-bit word to the data input register.
#[cfg(aes_v2)]
#[inline]
pub(crate) fn write_din(p: pac::aes::Aes, word: u32) {
    p.dinr().write_value(Dinr(word));
}
#[cfg(aes_v3b)]
#[inline]
pub(crate) fn write_din(p: pac::aes::Aes, word: u32) {
    p.dinr().write_value(word);
}

/// Read a 32-bit word from the data output register.
#[cfg(aes_v2)]
#[inline]
pub(crate) fn read_dout(p: pac::aes::Aes) -> u32 {
    p.doutr().read().0
}
#[cfg(aes_v3b)]
#[inline]
pub(crate) fn read_dout(p: pac::aes::Aes) -> u32 {
    p.doutr().read()
}

/// Read a 32-bit word from an initialization-vector register.
#[cfg(aes_v2)]
#[inline]
fn read_ivr(p: pac::aes::Aes, i: usize) -> u32 {
    p.ivr(i).read().0
}
#[cfg(aes_v3b)]
#[inline]
fn read_ivr(p: pac::aes::Aes, i: usize) -> u32 {
    p.ivr(i).read()
}

/// Write a 32-bit word to a key register.
#[cfg(aes_v2)]
#[inline]
fn write_keyr(p: pac::aes::Aes, i: usize, word: u32) {
    p.keyr(i).write_value(Keyr(word));
}
#[cfg(aes_v3b)]
#[inline]
fn write_keyr(p: pac::aes::Aes, i: usize, word: u32) {
    p.keyr(i).write_value(word);
}

/// Write a 32-bit word to an initialization-vector register.
#[cfg(aes_v2)]
#[inline]
fn write_ivr(p: pac::aes::Aes, i: usize, word: u32) {
    p.ivr(i).write_value(Ivr(word));
}
#[cfg(aes_v3b)]
#[inline]
fn write_ivr(p: pac::aes::Aes, i: usize, word: u32) {
    p.ivr(i).write_value(word);
}

/// Set the cipher mode (`CHMOD`).
#[cfg(aes_v2)]
#[inline]
fn set_chmod(p: pac::aes::Aes, bits: u8) {
    // aes_v2 splits CHMOD into CHMOD[1:0] and CHMOD[2].
    p.cr().modify(|w| {
        w.set_chmod10(bits & 0b11);
        w.set_chmod2((bits & 0b100) != 0);
    });
}
#[cfg(aes_v3b)]
#[inline]
fn set_chmod(p: pac::aes::Aes, bits: u8) {
    p.cr().modify(|w| w.set_chmod(pac::aes::vals::Chmod::from_bits(bits)));
}

// Public types

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
    /// This driver uses NO_SWAP (0) consistently with big-endian byte
    /// conversion (`from_be_bytes`/`to_be_bytes`) for direct NIST test-vector
    /// compatibility.
    fn datatype(&self) -> u8 {
        0
    }

    /// Returns the raw `CHMOD` field value for this cipher mode.
    fn chmod_bits(&self) -> u8 {
        0 // ECB default
    }

    /// Sets the cipher mode (`CHMOD` field).
    fn set_mode(&self, p: pac::aes::Aes) {
        set_chmod(p, self.chmod_bits());
    }

    /// Performs any key preparation within the processor, if necessary.
    fn prepare_key(&self, _p: pac::aes::Aes, _dir: Direction) {}

    /// Performs any cipher-specific initialization (blocking).
    fn init_phase_blocking(&self, _p: pac::aes::Aes) {}

    /// Indicates whether this cipher mode uses GCM/CCM phases (init, header, payload, final).
    fn uses_gcm_phases(&self) -> bool {
        false
    }

    /// Indicates whether this is CCM mode (which has different final phase handling).
    fn is_ccm_mode(&self) -> bool {
        false
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

    fn prepare_key(&self, p: pac::aes::Aes, dir: Direction) {
        // For ECB decryption, derive the decryption key first (RM key-derivation sequence).
        if dir == Direction::Decrypt {
            p.cr().modify(|w| w.set_mode(Mode::from_bits(1)));
            p.cr().modify(|w| w.set_en(true));
            while !p.sr().read().ccf() {}
            clear_ccf(p);
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

    fn chmod_bits(&self) -> u8 {
        1
    }

    fn prepare_key(&self, p: pac::aes::Aes, dir: Direction) {
        if dir == Direction::Decrypt {
            p.cr().modify(|w| w.set_mode(Mode::from_bits(1)));
            p.cr().modify(|w| w.set_en(true));
            while !p.sr().read().ccf() {}
            clear_ccf(p);
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

    fn init_phase_blocking(&self, p: pac::aes::Aes) {
        // GCMPH was set to init in op_start() before key loading. Enable EN to
        // start the hash-key (H) calculation, then wait and clear.
        p.cr().modify(|w| w.set_en(true));
        while !p.sr().read().ccf() {}
        clear_ccf(p);
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

    fn init_phase_blocking(&self, p: pac::aes::Aes) {
        p.cr().modify(|w| w.set_en(true));
        while !p.sr().read().ccf() {}
        clear_ccf(p);
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
}

impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize, const TAG_SIZE: usize> AesCcm<'c, KEY_SIZE, IV_SIZE, TAG_SIZE> {
    /// Constructs a new AES-CCM cipher for a cryptographic operation.
    /// - `key`: The encryption key (16 or 32 bytes)
    /// - `iv`: The nonce/IV (7-13 bytes)
    /// - `aad_len`: Length of additional authenticated data (known in advance)
    /// - `payload_len`: Length of payload data (known in advance)
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; IV_SIZE], aad_len: usize, payload_len: usize) -> Self {
        assert!(IV_SIZE >= 7 && IV_SIZE <= 13, "CCM IV must be 7-13 bytes");
        assert!(
            TAG_SIZE >= 4 && TAG_SIZE <= 16 && TAG_SIZE % 2 == 0,
            "CCM tag must be 4-16 bytes and even"
        );

        // Format the B0 block for CCM.
        let mut iv_full = [0u8; 16];
        let l = 15 - IV_SIZE; // size of the length field
        iv_full[0] = ((l - 1) as u8) | ((((TAG_SIZE - 2) / 2) as u8) << 3);
        if aad_len > 0 {
            iv_full[0] |= 0x40; // Adata flag
        }
        iv_full[1..1 + IV_SIZE].copy_from_slice(iv);

        let payload_bytes = (payload_len as u64).to_be_bytes();
        let offset = 16 - l;
        iv_full[offset..].copy_from_slice(&payload_bytes[8 - l..]);

        Self { key, iv: iv_full }
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

    fn init_phase_blocking(&self, p: pac::aes::Aes) {
        p.cr().modify(|w| w.set_en(true));
        while !p.sr().read().ccf() {}
        clear_ccf(p);
    }

    fn uses_gcm_phases(&self) -> bool {
        true
    }

    fn is_ccm_mode(&self) -> bool {
        true
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

// Instance plumbing (register access, shared by all AES peripherals)

pub(crate) trait SealedInstance {
    fn regs() -> pac::aes::Aes;
}

foreach_peripheral!(
    (aes, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {
            fn regs() -> pac::aes::Aes {
                pac::$inst
            }
        }
    };
);

// Low-level data movement (shared)

/// Load a key into the AES peripheral (big-endian words, reverse register order).
pub(crate) fn load_key(p: pac::aes::Aes, key: &[u8]) {
    let key_words = key.len() / 4;
    for i in 0..key_words {
        let word = u32::from_be_bytes([key[i * 4], key[i * 4 + 1], key[i * 4 + 2], key[i * 4 + 3]]);
        write_keyr(p, key_words - 1 - i, word);
    }
}

/// Load an IV into the AES peripheral (big-endian words, reverse register order).
pub(crate) fn load_iv(p: pac::aes::Aes, iv: &[u8]) {
    if iv.is_empty() {
        return;
    }
    let iv_words = core::cmp::min(iv.len(), 16) / 4;
    for i in 0..iv_words {
        let word = u32::from_be_bytes([iv[i * 4], iv[i * 4 + 1], iv[i * 4 + 2], iv[i * 4 + 3]]);
        write_ivr(p, iv_words - 1 - i, word);
    }
}

/// Write a 16-byte block to the AES peripheral (no wait).
pub(crate) fn write_block(p: pac::aes::Aes, block: &[u8]) -> Result<(), Error> {
    for i in 0..4 {
        if p.sr().read().wrerr() {
            clear_flags(p);
            return Err(Error::WriteError);
        }
        let word = u32::from_be_bytes([block[i * 4], block[i * 4 + 1], block[i * 4 + 2], block[i * 4 + 3]]);
        write_din(p, word);
    }
    Ok(())
}

/// Read a 16-byte block from the AES peripheral, blocking on completion.
pub(crate) fn read_block_blocking(p: pac::aes::Aes, block: &mut [u8]) -> Result<(), Error> {
    while !p.sr().read().ccf() {}
    if p.sr().read().rderr() {
        clear_flags(p);
        return Err(Error::ReadError);
    }
    read_out_block(p, block);
    clear_ccf(p);
    Ok(())
}

/// Read the four output words into a 16-byte block (assumes CCF already set).
pub(crate) fn read_out_block(p: pac::aes::Aes, block: &mut [u8]) {
    for i in 0..4 {
        let word = read_dout(p);
        block[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
}

// State machine (shared). The blocking init-phase wait is the only hardware
// wait these entry points perform; async drivers reuse `setup`/`make_context`
// and the low-level helpers above with their own wait strategy.

/// Configure the peripheral and load key/IV. Returns whether GCM/CCM phases are used.
pub(crate) fn setup<'c, C>(p: pac::aes::Aes, cipher: &'c C, dir: Direction) -> bool
where
    C: Cipher<'c> + CipherSized + IVSized,
{
    // Disable the peripheral.
    p.cr().modify(|w| w.set_en(false));

    // Clear the padding length. NPBLB is only written when a final partial
    // block needs it, but the hardware keeps the last value, so a leftover
    // count from an earlier operation would corrupt this one's tag.
    p.cr().modify(|w| w.set_npblb(0));

    // Data type (NO_SWAP) and key size.
    p.cr()
        .modify(|w| w.set_datatype(Datatype::from_bits(cipher.datatype())));
    let keysize = cipher.key_size();
    p.cr().modify(|w| w.set_keysize(keysize == KeySize::Bits256));

    // Direction.
    p.cr().modify(|w| w.set_mode(Mode::from_bits(dir as u8)));

    // Cipher mode (CHMOD).
    cipher.set_mode(p);

    let is_gcm_ccm = cipher.uses_gcm_phases();

    // For GCM/CCM, select the init phase BEFORE loading the key.
    if is_gcm_ccm {
        p.cr().modify(|w| w.set_gcmph(Gcmph::from_bits(0)));
    }

    let needs_key_prep = dir == Direction::Decrypt && !is_gcm_ccm && !cipher.key().is_empty();

    if is_gcm_ccm {
        // GCM/CCM: load key first, then IV.
        load_key(p, cipher.key());
        if !cipher.iv().is_empty() {
            load_iv(p, cipher.iv());
        }
    } else if needs_key_prep {
        // ECB/CBC decryption: key preparation, then IV. Key derivation is a
        // one-shot hardware step; it is polled even on the async path.
        load_key(p, cipher.key());
        cipher.prepare_key(p, dir);

        p.cr().modify(|w| w.set_mode(Mode::from_bits(dir as u8)));
        cipher.set_mode(p);

        if !cipher.iv().is_empty() {
            load_iv(p, cipher.iv());
        }
    } else {
        // ECB/CBC/CTR encryption: IV first, then key.
        if !cipher.iv().is_empty() {
            load_iv(p, cipher.iv());
        }
        load_key(p, cipher.key());
    }

    is_gcm_ccm
}

/// Build the operation context by snapshotting the enabled peripheral.
pub(crate) fn make_context<'c, C>(p: pac::aes::Aes, cipher: &'c C, dir: Direction, is_gcm_ccm: bool) -> Context<'c, C>
where
    C: Cipher<'c>,
{
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
        iv: [read_ivr(p, 0), read_ivr(p, 1), read_ivr(p, 2), read_ivr(p, 3)],
        suspr: [0; 8],
    }
}

/// Start a new cipher operation (blocking) and return the context.
pub(crate) fn op_start<'c, C>(p: pac::aes::Aes, cipher: &'c C, dir: Direction) -> Context<'c, C>
where
    C: Cipher<'c> + CipherSized + IVSized,
{
    let is_gcm_ccm = setup(p, cipher, dir);

    // Init phase for GCM/CCM (computes the hash key H); otherwise just enable.
    if is_gcm_ccm {
        cipher.init_phase_blocking(p);
    } else {
        p.cr().modify(|w| w.set_en(true));
    }

    make_context(p, cipher, dir, is_gcm_ccm)
}

/// Process authenticated additional data (AAD) for GCM/CCM modes (blocking).
pub(crate) fn op_aad<'c, C>(p: pac::aes::Aes, ctx: &mut Context<'c, C>, aad: &[u8], last: bool) -> Result<(), Error>
where
    C: Cipher<'c> + CipherAuthenticated<16>,
{
    if ctx.header_processed && last {
        return Ok(());
    }

    // Header phase (GCMPH = 1).
    p.cr().modify(|w| w.set_gcmph(Gcmph::from_bits(1)));
    p.cr().modify(|w| w.set_en(true));

    let mut aad_remaining = aad.len();
    let mut aad_index = 0;

    // Process buffered AAD first, if any.
    if ctx.aad_buffer_len > 0 {
        let space_available = 16 - ctx.aad_buffer_len;
        let to_copy = core::cmp::min(space_available, aad_remaining);
        ctx.aad_buffer[ctx.aad_buffer_len..ctx.aad_buffer_len + to_copy].copy_from_slice(&aad[..to_copy]);
        ctx.aad_buffer_len += to_copy;
        aad_index += to_copy;
        aad_remaining -= to_copy;

        if ctx.aad_buffer_len == 16 {
            write_block(p, &ctx.aad_buffer)?;
            while !p.sr().read().ccf() {}
            clear_ccf(p);
            ctx.header_len += 16;
            ctx.aad_buffer_len = 0;
        }
    }

    // Process complete blocks.
    while aad_remaining >= 16 {
        write_block(p, &aad[aad_index..aad_index + 16])?;
        while !p.sr().read().ccf() {}
        clear_ccf(p);
        ctx.header_len += 16;
        aad_index += 16;
        aad_remaining -= 16;
    }

    // Buffer any remaining partial block.
    if aad_remaining > 0 {
        ctx.aad_buffer[..aad_remaining].copy_from_slice(&aad[aad_index..aad_index + aad_remaining]);
        ctx.aad_buffer_len = aad_remaining;
    }

    // On the last call, zero-pad and process the final partial block.
    if last {
        if ctx.aad_buffer_len > 0 {
            for i in ctx.aad_buffer_len..16 {
                ctx.aad_buffer[i] = 0;
            }
            write_block(p, &ctx.aad_buffer)?;
            while !p.sr().read().ccf() {}
            clear_ccf(p);
            ctx.header_len += ctx.aad_buffer_len as u64;
            ctx.aad_buffer_len = 0;
        }
        ctx.header_processed = true;
    }

    Ok(())
}

/// Switch to the payload phase for GCM/CCM (shared by blocking and async).
pub(crate) fn begin_payload<'c, C>(p: pac::aes::Aes, ctx: &mut Context<'c, C>)
where
    C: Cipher<'c>,
{
    if ctx.is_gcm_ccm {
        let header_was_skipped = !ctx.header_processed;
        if header_was_skipped {
            ctx.header_processed = true;
        }
        p.cr().modify(|w| w.set_gcmph(Gcmph::from_bits(2)));
        p.cr().modify(|w| w.set_npblb(0));
        if header_was_skipped {
            p.cr().modify(|w| w.set_en(true));
        }
    }
}

/// Set `NPBLB` for a final partial payload block, per GCM/CCM rules.
pub(crate) fn set_final_npblb<'c, C>(p: pac::aes::Aes, ctx: &Context<'c, C>, remaining: usize)
where
    C: Cipher<'c>,
{
    // GCM sets NPBLB for both directions; CCM only for decryption.
    let should_set_npblb = if ctx.cipher.is_ccm_mode() {
        ctx.dir == Direction::Decrypt
    } else {
        true
    };
    if should_set_npblb {
        p.cr().modify(|w| w.set_npblb((16 - remaining) as u8));
    }
}

/// Process payload data (blocking).
pub(crate) fn op_payload<'c, C>(
    p: pac::aes::Aes,
    ctx: &mut Context<'c, C>,
    input: &[u8],
    output: &mut [u8],
    last: bool,
) -> Result<(), Error>
where
    C: Cipher<'c>,
{
    if output.len() < input.len() {
        return Err(Error::ConfigError);
    }

    begin_payload(p, ctx);

    let block_size = C::BLOCK_SIZE;
    let mut processed = 0;

    // Intermediate chunks must be block-aligned (all modes).
    if !last && input.len() % block_size != 0 {
        return Err(Error::ConfigError);
    }

    let complete_blocks = input.len() / block_size;

    for _ in 0..complete_blocks {
        let block = &input[processed..processed + block_size];
        let out_block = &mut output[processed..processed + block_size];
        write_block(p, block)?;
        read_block_blocking(p, out_block)?;
        processed += block_size;
        ctx.payload_len += block_size as u64;
    }

    // Final partial block.
    if last && processed < input.len() {
        if C::REQUIRES_PADDING {
            return Err(Error::ConfigError);
        }

        let remaining = input.len() - processed;
        let mut partial_block = [0u8; 16];
        partial_block[..remaining].copy_from_slice(&input[processed..]);

        set_final_npblb(p, ctx, remaining);

        write_block(p, &partial_block)?;
        read_block_blocking(p, &mut partial_block)?;

        output[processed..processed + remaining].copy_from_slice(&partial_block[..remaining]);
        ctx.payload_len += remaining as u64;
    }

    if last {
        ctx.last_block_processed = true;
    }

    Ok(())
}

/// Write the GCM length block (AAD bits || payload bits, big-endian) or, for
/// CCM, enable the peripheral to trigger the final tag computation.
pub(crate) fn begin_final<'c, C>(p: pac::aes::Aes, ctx: &Context<'c, C>)
where
    C: Cipher<'c>,
{
    // Draining the pipeline is short; poll BUSY directly.
    while p.sr().read().busy() {}

    p.cr().modify(|w| w.set_gcmph(Gcmph::from_bits(3)));

    if ctx.cipher.is_ccm_mode() {
        p.cr().modify(|w| w.set_en(true));
    } else {
        let header_bits = ctx.header_len * 8;
        let payload_bits = ctx.payload_len * 8;

        write_din(p, (header_bits >> 32) as u32);
        write_din(p, header_bits as u32);
        write_din(p, (payload_bits >> 32) as u32);
        write_din(p, payload_bits as u32);
    }
}

/// Finish the operation and return the authentication tag for GCM/CCM (blocking).
pub(crate) fn op_finish<'c, C>(p: pac::aes::Aes, ctx: Context<'c, C>) -> Result<Option<[u8; 16]>, Error>
where
    C: Cipher<'c>,
{
    if ctx.is_gcm_ccm {
        begin_final(p, &ctx);

        while !p.sr().read().ccf() {}

        let mut tag = [0u8; 16];
        read_out_block(p, &mut tag);

        clear_ccf(p);
        p.cr().modify(|w| w.set_en(false));

        Ok(Some(tag))
    } else {
        p.cr().modify(|w| w.set_en(false));
        Ok(None)
    }
}
