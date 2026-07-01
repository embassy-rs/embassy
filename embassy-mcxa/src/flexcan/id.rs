//! Module for CAN IDs.
//! 
//! The types in this module are just newtypes for the ID types exposed
//! in the generic `embedded-can` crate, with the addition of `defmt` impls.
//! Aside from the `defmt` impls, these types are identical to those from `embedded-can`
//! and can be converted to and from those types (zero cost) as needed.
//! 
//! <!-- 
//! Note for anyone working on the source code in here:
//! 
//! The only reason this module exists (instead of just re-exporting the `embedded-can` types
//! directly) is so `defmt` could be implemented. As a general rule, every type publicly exposed by
//! this FlexCAN HAL should probably support `defmt`, so directly exposing the `embedded-can` types wouldn't
//! really have been an option.
//! 
//! With that said, this module is supposed to be 1:1 with the `embedded-can` ID types. This includes
//! the rustdoc comments for these types/functions (they should literally be copy-pasted from `embedded-can`).
//! --> 

/// Standard 11-bit CAN Identifier (0..=0x7FF).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StandardId(embedded_can::StandardId);

impl StandardId {
    /// CAN ID 0, the highest priority.
    pub const ZERO: Self = Self(embedded_can::StandardId::ZERO);

    /// CAN ID 0x7FF, the lowest priority.
    pub const MAX: Self = Self(embedded_can::StandardId::MAX);

    /// Tries to create a StandardId from a raw 16-bit integer.
    /// 
    /// This will return None if raw is out of range of an 11-bit integer (> 0x7FF).
    pub const fn new(raw: u16) -> Option<Self> {
        match embedded_can::StandardId::new(raw) {
            Some(id) => Some(Self(id)),
            None => None,
        }
    }

    /// Creates a new StandardId without checking if it is inside the valid range.
    ///
    /// # Safety
    ///
    /// Using this method can create an invalid ID and is thus marked as unsafe.
    pub const unsafe fn new_unchecked(raw: u16) -> Self {
        // SAFETY: forwarded to the caller's contract that `raw <= 0x7FF`.
        Self(unsafe { embedded_can::StandardId::new_unchecked(raw) })
    }

    /// Returns this CAN Identifier as a raw 16-bit integer.
    pub fn as_raw(self) -> u16 {
        self.0.as_raw()
    }
}

/// Extended 29-bit CAN Identifier (0..=1FFF_FFFF).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtendedId(embedded_can::ExtendedId);

impl ExtendedId {
    /// CAN ID 0, the highest priority.
    pub const ZERO: Self = Self(embedded_can::ExtendedId::ZERO);

    /// CAN ID 0x1FFFFFFF, the lowest priority.
    pub const MAX: Self = Self(embedded_can::ExtendedId::MAX);

    /// Tries to create a ExtendedId from a raw 32-bit integer.
    ///
    /// This will return None if raw is out of range of an 29-bit integer (> 0x1FFF_FFFF).
    pub const fn new(raw: u32) -> Option<Self> {
        match embedded_can::ExtendedId::new(raw) {
            Some(id) => Some(Self(id)),
            None => None,
        }
    }

    /// Creates a new ExtendedId without checking if it is inside the valid range.
    ///
    /// # Safety
    ///
    /// Using this method can create an invalid ID and is thus marked as unsafe.
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        // SAFETY: forwarded to the caller's contract that `raw <= 0x1FFF_FFFF`.
        Self(unsafe { embedded_can::ExtendedId::new_unchecked(raw) })
    }

    /// Returns the Base ID part of this extended identifier.
    pub fn standard_id(self) -> StandardId {
        StandardId(self.0.standard_id())
    }

    /// Returns this CAN Identifier as a raw 32-bit integer.
    pub fn as_raw(self) -> u32 {
        self.0.as_raw()
    }
}

/// A CAN Identifier (standard or extended).
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Id {
    /// Standard 11-bit Identifier (0..=0x7FF).
    Standard(StandardId),

    /// Extended 29-bit Identifier (0..=0x1FFF_FFFF).
    Extended(ExtendedId),
}

impl From<StandardId> for Id {
    fn from(id: StandardId) -> Self { Id::Standard(id) }
}

impl From<ExtendedId> for Id {
    fn from(id: ExtendedId) -> Self { Id::Extended(id) }
}


//
// CONVERSIONS TO AND FROM THE `embedded_can` TYPES
//

impl From<embedded_can::StandardId> for StandardId {
    fn from(id: embedded_can::StandardId) -> Self { Self(id) }
}

impl From<StandardId> for embedded_can::StandardId {
    fn from(id: StandardId) -> Self { id.0 }
}

impl From<embedded_can::ExtendedId> for ExtendedId {
    fn from(id: embedded_can::ExtendedId) -> Self { Self(id) }
}

impl From<ExtendedId> for embedded_can::ExtendedId {
    fn from(id: ExtendedId) -> Self { id.0 }
}

impl From<embedded_can::Id> for Id {
    fn from(id: embedded_can::Id) -> Self {
        match id {
            embedded_can::Id::Standard(s) => Id::Standard(StandardId(s)),
            embedded_can::Id::Extended(e) => Id::Extended(ExtendedId(e)),
        }
    }
}

impl From<Id> for embedded_can::Id {
    fn from(id: Id) -> Self {
        match id {
            Id::Standard(s) => embedded_can::Id::Standard(s.0),
            Id::Extended(e) => embedded_can::Id::Extended(e.0),
        }
    }
}

//
// DEFMT IMPLS
//

#[cfg(feature = "defmt")]
impl defmt::Format for StandardId {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "StandardId(0x{=u16:03X})", self.as_raw());
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ExtendedId {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ExtendedId(0x{=u32:08X})", self.as_raw());
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Id {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Id::Standard(s) => defmt::write!(fmt, "Standard({})", s),
            Id::Extended(e) => defmt::write!(fmt, "Extended({})", e),
        }
    }
}
