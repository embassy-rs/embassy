//! Validated, relocated images in permanently owned ordinary SRAM.

use super::program::{Isa, Program, RelocationKind};

/// A program could not be installed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LoadError {
    /// Unknown instruction architecture.
    UnsupportedIsa,
    /// Empty image or insufficient caller-provided capacity.
    Capacity,
    /// Storage is outside the ordinary, nonsecure SRAM aperture.
    StorageRegion,
    /// Address arithmetic overflowed or crossed the supported branch region.
    AddressRange,
    /// Empty/duplicate entry name or entry outside the image.
    InvalidEntry,
    /// Invalid, duplicate or unsupported relocation, or wrong instruction kind.
    InvalidRelocation,
}

/// An image installed in exclusively owned, permanently borrowed storage.
///
/// Loading does not start SmartDMA or change memory permissions. Only ordinary
/// SRAM in the `0x20000000..0x20080000` aperture is accepted; this is **not** a
/// claim that every chip has 512 KiB. The valid Rust reference and application
/// linker layout must already describe implemented, initialized, noncacheable
/// memory. Instruction-fetch accessibility is additionally required at launch.
///
/// This initial API deliberately has no unload or mutable-storage return: its
/// static borrow remains consumed even if the handle is forgotten or dropped.
/// Multiple images may reside in distinct storage; there is still only one engine.
pub struct LoadedProgram {
    image: Program,
    words: &'static [u32],
    address: u32,
}

impl LoadedProgram {
    /// Validate metadata, copy words and apply address relocations.
    ///
    /// Every metadata check completes before the first destination write.
    /// No SRAM is reserved implicitly, and no security/execute-permission
    /// registers are changed. This does not establish safe execution.
    pub fn load(image: &Program, storage: &'static mut [u32]) -> Result<Self, LoadError> {
        if !matches!(image.isa(), Isa::EzhB) {
            return Err(LoadError::UnsupportedIsa);
        }
        let length = image.words().len();
        if length == 0 || length > storage.len() {
            return Err(LoadError::Capacity);
        }
        let address = u32::try_from(storage.as_ptr() as usize).map_err(|_| LoadError::StorageRegion)?;
        let bytes = u32::try_from(length)
            .ok()
            .and_then(|words| words.checked_mul(4))
            .ok_or(LoadError::AddressRange)?;
        let end = address.checked_add(bytes).ok_or(LoadError::AddressRange)?;
        if address < 0x2000_0000 || end > 0x2008_0000 || address & 3 != 0 {
            return Err(LoadError::StorageRegion);
        }
        // GOTO retains the runtime PC's top nine bits. Keep all source PCs,
        // including a conservative pipeline margin, in the same 8 MiB region.
        let pipeline_end = end.checked_add(12).ok_or(LoadError::AddressRange)?;
        if address & 0xff80_0000 != pipeline_end & 0xff80_0000 {
            return Err(LoadError::AddressRange);
        }

        for (index, entry) in image.entries().iter().enumerate() {
            if entry.name().is_empty()
                || entry.word_offset() as usize >= length
                || image.entries()[..index]
                    .iter()
                    .any(|other| other.name() == entry.name())
            {
                return Err(LoadError::InvalidEntry);
            }
        }
        for (index, relocation) in image.relocations().iter().enumerate() {
            let offset = relocation.word_offset() as usize;
            let target = relocation.target_word_offset() as usize;
            if offset >= length
                || target > length
                || image.relocations()[..index]
                    .iter()
                    .any(|other| other.word_offset() == relocation.word_offset())
            {
                return Err(LoadError::InvalidRelocation);
            }
            match relocation.kind() {
                RelocationKind::Absolute32 => {}
                RelocationKind::Goto => {
                    if target == length || image.words()[offset] & 0x21f != 0x215 {
                        return Err(LoadError::InvalidRelocation);
                    }
                }
            }
        }

        let words = &mut storage[..length];
        words.copy_from_slice(image.words());
        for relocation in image.relocations() {
            // Bounds and checked image-end arithmetic above include a permitted
            // one-past-end Absolute32 target. Multiplication/addition cannot wrap.
            let target = address + relocation.target_word_offset() * 4;
            let word = &mut words[relocation.word_offset() as usize];
            *word = match relocation.kind() {
                RelocationKind::Absolute32 => target,
                RelocationKind::Goto => (*word & 0x7ff) | ((target & 0x007f_fffc) << 9),
            };
        }
        Ok(Self {
            image: *image,
            words,
            address,
        })
    }

    /// Loaded byte address for diagnostics, not permission to execute it.
    pub fn address(&self) -> u32 {
        self.address
    }

    /// Immutable, relocated instruction/data words.
    pub fn words(&self) -> &[u32] {
        self.words
    }

    pub(super) fn entry_address(&self, name: &str) -> Option<u32> {
        self.image
            .entry(name)
            .map(|entry| self.address + entry.word_offset() * 4)
    }
}
