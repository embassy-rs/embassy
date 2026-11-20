//! GATT types and constants
//!
//! The flag and value types come from `stm32wb-hci`; only the handles and
//! the [`Uuid`] wrapper are defined here.

pub use stm32wb_hci::aci::flags::{AccessPermissions, CharProperties, GattEventMask, SecurityPermissions};
pub use stm32wb_hci::aci::values::ServiceType;

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

/// Characteristic descriptor handle (returned by add_descriptor)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(transparent)]
pub struct DescriptorHandle(pub u16);

/// A 16-bit or 128-bit UUID, declared by `stm32wb-hci`.
pub use stm32wb_hci::wire::Uuid;
