//! GATT types and constants

/// Service handle (returned by add_service)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(transparent)]
pub struct ServiceHandle(pub u16);

/// Characteristic handle (returned by add_characteristic)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(transparent)]
pub struct CharacteristicHandle(pub u16);

/// UUID type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UuidType {
    /// 16-bit UUID
    Uuid16 = 0x01,
    /// 128-bit UUID
    Uuid128 = 0x02,
}

/// Service type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ServiceType {
    /// Primary service
    Primary = 0x01,
    /// Secondary service
    Secondary = 0x02,
}

/// Characteristic properties (as per Bluetooth spec)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CharProperties(pub u8);

impl CharProperties {
    pub const BROADCAST: Self = Self(0x01);
    pub const READ: Self = Self(0x02);
    pub const WRITE_WITHOUT_RESPONSE: Self = Self(0x04);
    pub const WRITE: Self = Self(0x08);
    pub const NOTIFY: Self = Self(0x10);
    pub const INDICATE: Self = Self(0x20);
    pub const AUTHENTICATED_SIGNED_WRITES: Self = Self(0x40);
    pub const EXTENDED_PROPERTIES: Self = Self(0x80);

    /// Create empty properties
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Combine properties using bitwise OR
    pub const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Check if a property is set
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl core::ops::BitOr for CharProperties {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for CharProperties {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Security permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SecurityPermissions(pub u8);

impl SecurityPermissions {
    pub const NONE: Self = Self(0x00);
    pub const AUTHEN_READ: Self = Self(0x01);
    pub const AUTHOR_READ: Self = Self(0x02);
    pub const ENCRY_READ: Self = Self(0x04);
    pub const AUTHEN_WRITE: Self = Self(0x08);
    pub const AUTHOR_WRITE: Self = Self(0x10);
    pub const ENCRY_WRITE: Self = Self(0x20);

    /// Create empty permissions
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Combine permissions using bitwise OR
    pub const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl core::ops::BitOr for SecurityPermissions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for SecurityPermissions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// GATT event mask
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GattEventMask(pub u8);

impl GattEventMask {
    pub const NONE: Self = Self(0x00);
    pub const ATTRIBUTE_MODIFIED: Self = Self(0x01);

    /// Create empty event mask
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Combine event masks using bitwise OR
    pub const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl core::ops::BitOr for GattEventMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// UUID wrapper
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Uuid {
    /// 16-bit UUID
    Uuid16(u16),
    /// 128-bit UUID (stored in little-endian)
    Uuid128([u8; 16]),
}

impl Uuid {
    /// Create a 16-bit UUID
    pub const fn from_u16(uuid: u16) -> Self {
        Self::Uuid16(uuid)
    }

    /// Create a 128-bit UUID from bytes (little-endian)
    pub const fn from_u128_le(uuid: [u8; 16]) -> Self {
        Self::Uuid128(uuid)
    }

    /// Get the UUID type
    pub const fn uuid_type(&self) -> UuidType {
        match self {
            Uuid::Uuid16(_) => UuidType::Uuid16,
            Uuid::Uuid128(_) => UuidType::Uuid128,
        }
    }

    /// Get pointer and length for passing to C functions
    pub fn as_ptr_and_type(&self) -> (*const u8, UuidType) {
        match self {
            Uuid::Uuid16(uuid) => {
                let bytes = uuid.to_le_bytes();
                (bytes.as_ptr(), UuidType::Uuid16)
            }
            Uuid::Uuid128(uuid) => (uuid.as_ptr(), UuidType::Uuid128),
        }
    }
}

// Common 16-bit UUIDs
impl Uuid {
    /// Generic Access service
    pub const GAP_SERVICE: Self = Self::Uuid16(0x1800);
    /// Generic Attribute service
    pub const GATT_SERVICE: Self = Self::Uuid16(0x1801);
    /// Device Name characteristic
    pub const DEVICE_NAME: Self = Self::Uuid16(0x2A00);
    /// Appearance characteristic
    pub const APPEARANCE: Self = Self::Uuid16(0x2A01);
    /// Service Changed characteristic
    pub const SERVICE_CHANGED: Self = Self::Uuid16(0x2A05);
}
