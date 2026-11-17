//! Immutable SmartDMA assembly images and relocation metadata.
//!
//! All offsets are measured in 32-bit words from the beginning of the image,
//! not bytes. These types describe an image; they do not reserve executable
//! storage or grant access to the SmartDMA engine.

/// Instruction architecture required by an image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Isa {
    /// EZH Architecture B. Individual chip integrations may have further limits.
    EzhB,
}

/// An exported, not-yet-loaded program entry.
///
/// An entry offset is not an executable address. A loader must bind it to a
/// particular loaded image before it can be used to select an execution entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Entry {
    name: &'static str,
    word_offset: u32,
}

impl Entry {
    /// Describe an entry. This does not validate its offset against an image.
    pub const fn new(name: &'static str, word_offset: u32) -> Self {
        Self { name, word_offset }
    }

    /// Exported symbol name.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Entry offset in 32-bit words from the image start.
    pub const fn word_offset(&self) -> u32 {
        self.word_offset
    }
}

/// How a reference to a program-local address is encoded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RelocationKind {
    /// Replace the entire data word with the target's absolute byte address.
    Absolute32,
    /// Encode the target in an immediate GOTO instruction, preserving its
    /// condition and link fields. A loader must validate the address alignment
    /// and reachable address region before patching the instruction.
    Goto,
}

/// A reference to a program-local address that must be resolved at load time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Relocation {
    word_offset: u32,
    target_word_offset: u32,
    kind: RelocationKind,
}

impl Relocation {
    /// Describe a relocation. This does not validate offsets or instruction bits.
    pub const fn new(word_offset: u32, target_word_offset: u32, kind: RelocationKind) -> Self {
        Self {
            word_offset,
            target_word_offset,
            kind,
        }
    }

    /// Word to patch, measured in words from the image start.
    pub const fn word_offset(&self) -> u32 {
        self.word_offset
    }

    /// Referenced target, measured in words from the image start.
    pub const fn target_word_offset(&self) -> u32 {
        self.target_word_offset
    }

    /// Encoding to apply after validating the final load address.
    pub const fn kind(&self) -> RelocationKind {
        self.kind
    }
}

/// An immutable, unrelocated SmartDMA image.
///
/// The assembly macros produce this type without a runtime parser or allocator.
/// It can also describe application-provided raw words via [`Self::from_parts`].
///
/// Constructing a description is safe because it neither loads nor executes
/// code. A loader must validate all metadata, including raw descriptions, before
/// using it. Valid assembly is not a memory-safety or waveform-timing proof.
#[derive(Clone, Copy, Debug)]
pub struct Program {
    isa: Isa,
    words: &'static [u32],
    entries: &'static [Entry],
    relocations: &'static [Relocation],
}

impl Program {
    /// Describe an image without executing it or validating its metadata.
    ///
    /// Raw instruction words do not receive inferred relocations. Every
    /// load-address-dependent reference must be described explicitly.
    pub const fn from_parts(
        isa: Isa,
        words: &'static [u32],
        entries: &'static [Entry],
        relocations: &'static [Relocation],
    ) -> Self {
        Self {
            isa,
            words,
            entries,
            relocations,
        }
    }

    /// Instruction architecture required by this image.
    pub const fn isa(&self) -> Isa {
        self.isa
    }

    /// Encoded words before load-time relocation.
    pub const fn words(&self) -> &'static [u32] {
        self.words
    }

    /// Exported entry names and offsets.
    pub const fn entries(&self) -> &'static [Entry] {
        self.entries
    }

    /// Explicit load-time relocation records.
    pub const fn relocations(&self) -> &'static [Relocation] {
        self.relocations
    }

    /// Find a case-sensitive exported symbol. This does not produce an
    /// executable pointer or a loaded-entry handle.
    pub fn entry(&self, name: &str) -> Option<Entry> {
        self.entries.iter().find(|entry| entry.name == name).copied()
    }
}
