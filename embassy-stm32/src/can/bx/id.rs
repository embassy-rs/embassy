//! CAN Identifiers.

/// Standard 11-bit CAN Identifier (`0..=0x7FF`).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StandardId(u16);

impl StandardId {
    /// CAN ID `0`, the highest priority.
    pub const ZERO: Self = Self(0);

    /// CAN ID `0x7FF`, the lowest priority.
    pub const MAX: Self = Self(0x7FF);

    /// Tries to create a `StandardId` from a raw 16-bit integer.
    ///
    /// This will return `None` if `raw` is out of range of an 11-bit integer (`> 0x7FF`).
    #[inline]
    pub const fn new(raw: u16) -> Option<Self> {
        if raw <= 0x7FF {
            Some(Self(raw))
        } else {
            None
        }
    }

    /// Creates a new `StandardId` without checking if it is inside the valid range.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `raw` is in the valid range, otherwise the behavior is
    /// undefined.
    #[inline]
    pub const unsafe fn new_unchecked(raw: u16) -> Self {
        Self(raw)
    }

    /// Returns this CAN Identifier as a raw 16-bit integer.
    #[inline]
    pub fn as_raw(&self) -> u16 {
        self.0
    }
}

/// Extended 29-bit CAN Identifier (`0..=1FFF_FFFF`).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExtendedId(u32);

impl ExtendedId {
    /// CAN ID `0`, the highest priority.
    pub const ZERO: Self = Self(0);

    /// CAN ID `0x1FFFFFFF`, the lowest priority.
    pub const MAX: Self = Self(0x1FFF_FFFF);

    /// Tries to create a `ExtendedId` from a raw 32-bit integer.
    ///
    /// This will return `None` if `raw` is out of range of an 29-bit integer (`> 0x1FFF_FFFF`).
    #[inline]
    pub const fn new(raw: u32) -> Option<Self> {
        if raw <= 0x1FFF_FFFF {
            Some(Self(raw))
        } else {
            None
        }
    }

    /// Creates a new `ExtendedId` without checking if it is inside the valid range.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `raw` is in the valid range, otherwise the behavior is
    /// undefined.
    #[inline]
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns this CAN Identifier as a raw 32-bit integer.
    #[inline]
    pub fn as_raw(&self) -> u32 {
        self.0
    }

    /// Returns the Base ID part of this extended identifier.
    pub fn standard_id(&self) -> StandardId {
        // ID-28 to ID-18
        StandardId((self.0 >> 18) as u16)
    }
}

/// A CAN Identifier (standard or extended).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Id {
    /// Standard 11-bit Identifier (`0..=0x7FF`).
    Standard(StandardId),

    /// Extended 29-bit Identifier (`0..=0x1FFF_FFFF`).
    Extended(ExtendedId),
}

impl From<StandardId> for Id {
    #[inline]
    fn from(id: StandardId) -> Self {
        Id::Standard(id)
    }
}

impl From<ExtendedId> for Id {
    #[inline]
    fn from(id: ExtendedId) -> Self {
        Id::Extended(id)
    }
}
