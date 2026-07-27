//! Loader for ST Edge AI *Epoch Controller binaries*.
//!
//! When a network is compiled with `--enable-epoch-controller`, ST Edge AI
//! emits one or more *EC binaries* (the `_ec_blob_<network>_<n>` arrays in
//! `<network>_ecblobs.h`). An EC binary is a container holding:
//!
//! - the **blob**: the command stream executed by the NPU's epoch controller,
//! - an optional **relocation table**: named lists of blob words to which a
//!   base address must be *added* (used e.g. to point the blob at the user's
//!   input/output buffers),
//! - an optional **patch table**: named lists of blob words in which a
//!   masked/shifted *value* must be substituted.
//!
//! This module is a `no_std` port of ST's `ecloader.c`. Typical use mirrors
//! the generated `LL_ATON_EC_Network_Init_*` / `LL_ATON_EC_Inference_Init_*`
//! functions:
//!
//! ```rust,ignore
//! // The EC binary as emitted by ST Edge AI (u64 words, 8-byte aligned).
//! static EC_BIN_1: &[u64] = &[ /* ..._ec_blob_network_1 contents... */ ];
//!
//! // Writable copy of the blob, in NPU-visible RAM.
//! static BLOB_1: StaticCell<[u64; 3800]> = StaticCell::new();
//!
//! let bin = EcBinary::new(EC_BIN_1)?;
//! let blob = BLOB_1.init([0; 3800]);
//! bin.load_blob(blob)?;                       // once, at network init
//! let mut prev = 0;
//! bin.reloc(blob, 0, input.as_ptr() as u32, &mut prev)?; // before inference
//! ```
//!
//! Blobs that need no relocation (fully linked at generation time) can be
//! executed straight from flash without copying.

/// Magic number at the start of an Epoch Controller binary container.
pub const BINARY_MAGIC: u32 = 0xECBF_0050;
/// Magic number at the start of an Epoch Controller blob.
pub const BLOB_MAGIC: u32 = 0xCA05_7A7A;

/// Errors returned by [`EcBinary`] operations.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum EcError {
    /// The container does not start with [`BINARY_MAGIC`].
    BadBinaryMagic,
    /// The blob section is missing or does not start with [`BLOB_MAGIC`].
    BadBlobMagic,
    /// A section offset in the container header is out of bounds or
    /// misaligned.
    Malformed,
    /// The destination buffer is too small for the blob.
    BufferTooSmall,
    /// The requested relocation/patch index or identifier does not exist.
    NotFound,
}

/// A parsed, read-only view of an Epoch Controller binary container.
#[derive(Copy, Clone)]
pub struct EcBinary<'a> {
    words: &'a [u32],
    reloc_off: usize, // word offsets into `words`, 0 = absent
    patch_off: usize,
    blob_off: usize,
}

impl<'a> EcBinary<'a> {
    /// Parse an EC binary stored as `u64` words (the layout ST Edge AI
    /// generates in `<network>_ecblobs.h`).
    pub fn new(binary: &'a [u64]) -> Result<Self, EcError> {
        // Safety: reinterpreting u64 words as u32 words; the target is
        // little-endian, matching the on-disk layout consumed by ecloader.c.
        let words = unsafe { core::slice::from_raw_parts(binary.as_ptr() as *const u32, binary.len() * 2) };
        Self::from_words(words)
    }

    /// Parse an EC binary from raw `u32` words.
    pub fn from_words(words: &'a [u32]) -> Result<Self, EcError> {
        if words.len() < 5 || words[0] != BINARY_MAGIC {
            return Err(EcError::BadBinaryMagic);
        }
        // Container header: [magic, reloc_off, patch_off, debug_off, blob_off]
        // with byte offsets relative to the start of the container.
        let byte_to_word = |off: u32| -> Result<usize, EcError> {
            if off % 4 != 0 || (off / 4) as usize > words.len() {
                return Err(EcError::Malformed);
            }
            Ok((off / 4) as usize)
        };
        Ok(Self {
            words,
            reloc_off: byte_to_word(words[1])?,
            patch_off: byte_to_word(words[2])?,
            blob_off: byte_to_word(words[4])?,
        })
    }

    // ── blob ────────────────────────────────────────────────────────────────

    fn blob_section(&self) -> Result<&'a [u32], EcError> {
        if self.blob_off == 0 || self.blob_off + 2 > self.words.len() {
            return Err(EcError::BadBlobMagic);
        }
        let s = &self.words[self.blob_off..];
        if s[0] != BLOB_MAGIC {
            return Err(EcError::BadBlobMagic);
        }
        Ok(s)
    }

    /// Size of the blob in `u64` words, including its magic/length header —
    /// i.e. the minimum length of the buffer passed to [`Self::load_blob`].
    pub fn blob_len(&self) -> Result<usize, EcError> {
        let s = self.blob_section()?;
        let instr_words = s[1] as usize; // length in u32 instructions
        Ok((instr_words + 2).div_ceil(2))
    }

    /// Copy the blob (magic + length + instructions) into `dest`, returning
    /// the number of `u64` words used. `dest` may then be relocated/patched
    /// and handed to [`Npu::run_epoch_blob`](super::Npu::run_epoch_blob).
    pub fn load_blob(&self, dest: &mut [u64]) -> Result<usize, EcError> {
        let s = self.blob_section()?;
        let instr_words = s[1] as usize;
        let total_u32 = instr_words + 2;
        if total_u32 > s.len() {
            return Err(EcError::Malformed);
        }
        let needed = total_u32.div_ceil(2);
        if dest.len() < needed {
            return Err(EcError::BufferTooSmall);
        }
        let dest_u32 = unsafe { core::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut u32, dest.len() * 2) };
        dest_u32[..total_u32].copy_from_slice(&s[..total_u32]);
        Ok(needed)
    }

    // ── generic table walking (relocation & patch tables share the shape) ──

    /// Table layout: `[count, {id_off, ...entry words...} x count, ...]` where
    /// all offsets are byte offsets relative to the table start.
    fn table(&self, off: usize) -> Option<&'a [u32]> {
        if off == 0 || off >= self.words.len() {
            None
        } else {
            Some(&self.words[off..])
        }
    }

    fn table_str(table: &'a [u32], byte_off: u32) -> Option<&'a str> {
        let bytes: &[u8] = unsafe { core::slice::from_raw_parts(table.as_ptr() as *const u8, table.len() * 4) };
        let start = byte_off as usize;
        if start >= bytes.len() {
            return None;
        }
        let s = &bytes[start..];
        let end = s.iter().position(|&b| b == 0)?;
        core::str::from_utf8(&s[..end]).ok()
    }

    // ── relocations ─────────────────────────────────────────────────────────

    /// Number of relocations described by this binary.
    pub fn num_relocs(&self) -> usize {
        self.table(self.reloc_off).map_or(0, |t| t[0] as usize)
    }

    /// Identifier of relocation `idx` (e.g. `"_user_io_input_0"`).
    pub fn reloc_id(&self, idx: usize) -> Option<&'a str> {
        let t = self.table(self.reloc_off)?;
        if idx >= t[0] as usize {
            return None;
        }
        Self::table_str(t, t[3 * idx + 1])
    }

    /// Apply relocation `idx` to a loaded blob: adds `base - *prev_base` to
    /// every blob word listed in the relocation, then stores `base` in
    /// `prev_base`. Call with `*prev_base == 0` the first time; subsequent
    /// calls with an unchanged `base` are no-ops, matching ST's `ec_reloc`.
    pub fn reloc(&self, blob: &mut [u64], idx: usize, base: u32, prev_base: &mut u32) -> Result<(), EcError> {
        let t = self.table(self.reloc_off).ok_or(EcError::NotFound)?;
        if idx >= t[0] as usize {
            return Err(EcError::NotFound);
        }
        if base == *prev_base {
            return Ok(());
        }
        let num = t[3 * idx + 2] as usize;
        let list_off = t[3 * idx + 3];
        if list_off % 4 != 0 {
            return Err(EcError::Malformed);
        }
        let list = &t[(list_off / 4) as usize..];
        if list.len() < num {
            return Err(EcError::Malformed);
        }
        let blob_u32 = unsafe { core::slice::from_raw_parts_mut(blob.as_mut_ptr() as *mut u32, blob.len() * 2) };
        let delta = base.wrapping_sub(*prev_base);
        for &word_off in &list[..num] {
            // Offsets are relative to the first instruction, i.e. skip the
            // blob's magic + length words.
            let i = word_off as usize + 2;
            if i >= blob_u32.len() {
                return Err(EcError::Malformed);
            }
            blob_u32[i] = blob_u32[i].wrapping_add(delta);
        }
        *prev_base = base;
        Ok(())
    }

    /// Apply the relocation with identifier `id`. See [`Self::reloc`].
    pub fn reloc_by_id(&self, blob: &mut [u64], id: &str, base: u32, prev_base: &mut u32) -> Result<(), EcError> {
        let idx = self.find_by_id(self.reloc_off, 3, id).ok_or(EcError::NotFound)?;
        self.reloc(blob, idx, base, prev_base)
    }

    // ── patches ─────────────────────────────────────────────────────────────

    /// Number of patches described by this binary.
    pub fn num_patches(&self) -> usize {
        self.table(self.patch_off).map_or(0, |t| t[0] as usize)
    }

    /// Identifier of patch `idx`.
    pub fn patch_id(&self, idx: usize) -> Option<&'a str> {
        let t = self.table(self.patch_off)?;
        if idx >= t[0] as usize {
            return None;
        }
        Self::table_str(t, t[5 * idx + 1])
    }

    /// Apply patch `idx` to a loaded blob: replaces the masked bit-field of
    /// every listed blob word with `value` (shifted/masked as encoded in the
    /// patch table).
    pub fn patch(&self, blob: &mut [u64], idx: usize, value: u64) -> Result<(), EcError> {
        let t = self.table(self.patch_off).ok_or(EcError::NotFound)?;
        if idx >= t[0] as usize {
            return Err(EcError::NotFound);
        }
        let shr = t[5 * idx + 2] as i32;
        let mut mask = t[5 * idx + 3];
        let num = t[5 * idx + 4] as usize;
        let list_off = t[5 * idx + 5];
        if list_off % 4 != 0 {
            return Err(EcError::Malformed);
        }
        let list = &t[(list_off / 4) as usize..];
        if list.len() < num {
            return Err(EcError::Malformed);
        }
        let mut value = value;
        if shr >= 0 {
            value >>= shr;
        } else {
            mask <<= -shr;
            value <<= -shr;
        }
        let value = (value as u32) & mask;
        let blob_u32 = unsafe { core::slice::from_raw_parts_mut(blob.as_mut_ptr() as *mut u32, blob.len() * 2) };
        for &word_off in &list[..num] {
            let i = word_off as usize + 2;
            if i >= blob_u32.len() {
                return Err(EcError::Malformed);
            }
            blob_u32[i] = (blob_u32[i] & !mask) | value;
        }
        Ok(())
    }

    /// Apply the patch with identifier `id`. See [`Self::patch`].
    pub fn patch_by_id(&self, blob: &mut [u64], id: &str, value: u64) -> Result<(), EcError> {
        let idx = self.find_by_id(self.patch_off, 5, id).ok_or(EcError::NotFound)?;
        self.patch(blob, idx, value)
    }

    fn find_by_id(&self, table_off: usize, stride: usize, id: &str) -> Option<usize> {
        let t = self.table(table_off)?;
        let count = t[0] as usize;
        (0..count).find(|&n| Self::table_str(t, t[stride * n + 1]) == Some(id))
    }
}
